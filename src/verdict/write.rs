// Write side: `add` creates a question, `observe` appends one observation,
// `retire` withdraws a question, `amend` corrects an answer or an
// observation's prose keeping the history, `cite` records a manual passage
// supporting the answer, `void` withdraws an observation or a citation.
// Each takes the records directory's lock,
// strictly loads it (never write into a store that is already invalid),
// validates the new state with the same rules the read side enforces, and
// only then touches disk. po computes fixture hashes itself from the bytes it
// stores, so a recorded hash cannot disagree with its file. `add` and
// `observe` take an optional capture key that makes a rerun a no-op.

use anyhow::{Context as _, Result, bail};
use std::collections::BTreeSet;
use std::path::{Component, Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use toml::value::Datetime;

use super::validate::{self, Context, Report, fixture_path, question_path, sha256_hex};
use super::{
    AmendField, Amendment, Candidate, CandidateState, Citation, Diag, Kind, Observation, Outcome,
    Question, Relation, Retired, Source, Store, Void, citation, load,
};

/// A question to create.
#[derive(Debug, Clone)]
pub struct NewQuestion {
    pub kind: Kind,
    /// Capture key: `add` under a key the directory already holds returns
    /// that question if the payload matches, and fails if it differs.
    pub key: Option<String>,
    pub question: String,
    pub answer: String,
    pub identifiers: Vec<String>,
    pub follow_up_to: Vec<String>,
    pub basis: Vec<String>,
}

/// The id `add` assigned plus any non-blocking warnings about the question.
#[derive(Debug, Clone)]
pub struct Added {
    pub id: String,
    /// The key was already held by an identical question, whose id this is;
    /// nothing was written.
    pub existing: bool,
    pub warnings: Vec<String>,
}

/// Where `observe` put the observation: its 1-based position in file order
/// (the `#N` `show` prints).
#[derive(Debug, Clone)]
pub struct Observed {
    pub number: usize,
    /// The key was already held by an identical observation, whose number
    /// this is; nothing was written.
    pub existing: bool,
    pub warnings: Vec<String>,
}

/// An observation to append. Paths (`fixture`, `evidence`) are as the caller
/// named them; `observe` copies the fixture into the store and rewrites
/// evidence relative to the records directory.
#[derive(Debug, Clone)]
pub struct NewObservation {
    pub source: Source,
    /// Capture key: re-observing under a key the question already holds is
    /// a no-op if the payload matches, and fails if it differs.
    pub key: Option<String>,
    /// `YYYY-MM-DD`. Exactly one of `date` and `date_before` must be set.
    pub date: Option<String>,
    /// `YYYY-MM-DD` upper bound, for a capture whose date was never recorded.
    pub date_before: Option<String>,
    /// `None` states that no byte-exact fixture was measured.
    pub fixture: Option<PathBuf>,
    pub outcome: Option<Outcome>,
    pub crash: Option<String>,
    pub errors: Vec<Diag>,
    pub warnings: Vec<Diag>,
    pub result: Option<String>,
    pub settings: Option<String>,
    /// Declared candidates (state ignored; set from `selected` / `refuted`).
    pub candidates: Vec<Candidate>,
    pub selected: Vec<String>,
    pub refuted: Vec<String>,
    pub evidence: Vec<PathBuf>,
    pub environment: Option<String>,
    pub note: Option<String>,
    pub inconclusive: Option<String>,
}

/// Create a question under `root` and return its generated id.
pub fn add(root: &Path, new: &NewQuestion) -> Result<Added> {
    let _lock = lock(root)?;
    let store = load(&[root.to_path_buf()])?;
    let mut q = Question {
        id: String::new(),
        root: root.to_path_buf(),
        kind: new.kind,
        // Keys are exact: stored and matched as given, never normalized.
        key: new.key.clone(),
        question: new.question.trim().to_string(),
        answer: new.answer.trim().to_string(),
        identifiers: new.identifiers.clone(),
        follow_up_to: ids(&new.follow_up_to),
        basis: ids(&new.basis),
        legacy_derived_from: None,
        retired: None,
        amendments: Vec::new(),
        citations: Vec::new(),
        observations: Vec::new(),
    };
    if let Some(key) = &q.key
        && let Some(existing) = store
            .questions()
            .iter()
            .find(|x| x.key.as_ref() == Some(key))
    {
        if let Some(r) = &existing.retired {
            bail!(
                "key `{key}` belongs to question {}, which is retired{}",
                existing.id,
                replaced_hint(r)
            );
        }
        // Only the current values of the fields `add` sets are compared:
        // observations, citations, amendments and retirement come later, not
        // part of the add payload. A script still emitting an answer since
        // amended fails here, which is the point.
        let stored = Question {
            observations: Vec::new(),
            amendments: Vec::new(),
            citations: Vec::new(),
            ..existing.clone()
        };
        let diff = field_diff(&stored, &q)?;
        if !diff.is_empty() {
            bail!(
                "key `{key}` belongs to question {}, which differs:\n  {}",
                existing.id,
                diff.join("\n  ")
            );
        }
        return Ok(Added {
            id: existing.id.clone(),
            existing: true,
            warnings: Vec::new(),
        });
    }
    let taken: BTreeSet<String> = store.questions().iter().map(|x| x.id.clone()).collect();
    q.id = fresh_id(&q.question, &taken, root);
    let warnings = check(&store, &q, None)?;
    let text = toml::to_string(&q).context("serializing question")?;
    write_atomic(&question_path(root, &q.id), text.as_bytes())?;
    Ok(Added {
        id: q.id,
        existing: false,
        warnings,
    })
}

/// ` (replaced by a, b)`, or nothing when a retired question names no
/// replacement.
fn replaced_hint(r: &Retired) -> String {
    if r.replaced_by.is_empty() {
        String::new()
    } else {
        format!(" (replaced by {})", r.replaced_by.join(", "))
    }
}

/// The top-level fields on which two records differ, as `field: stored X,
/// given Y` lines. Compared through their TOML form, so the diff speaks the
/// on-disk field names.
fn field_diff<T: serde::Serialize>(stored: &T, given: &T) -> Result<Vec<String>> {
    let table = |v: &T| -> Result<toml::Table> {
        toml::Table::try_from(v).context("serializing record for comparison")
    };
    let (a, b) = (table(stored)?, table(given)?);
    let fields: BTreeSet<&String> = a.keys().chain(b.keys()).collect();
    let show =
        |v: Option<&toml::Value>| v.map_or_else(|| "(absent)".to_string(), ToString::to_string);
    Ok(fields
        .into_iter()
        .filter(|f| a.get(*f) != b.get(*f))
        .map(|f| format!("`{f}`: stored {}, given {}", show(a.get(f)), show(b.get(f))))
        .collect())
}

/// Hold the records directory's write lock until the returned file drops.
/// Every write verb holds it from its strict load through its last write, so
/// a check (key absent, question not retired) cannot go stale before the
/// write that relies on it. The lock file is a persistent dot-file (the
/// loader ignores dot-files); it is never deleted, since recreating it would
/// let two processes lock different files.
fn lock(root: &Path) -> Result<std::fs::File> {
    if !root.is_dir() {
        bail!("records directory {} does not exist", root.display());
    }
    let path = root.join(".lock");
    let file = std::fs::OpenOptions::new()
        .create(true)
        .truncate(false)
        .write(true)
        .open(&path)
        .with_context(|| format!("opening {}", path.display()))?;
    file.lock()
        .with_context(|| format!("locking {}", path.display()))?;
    Ok(file)
}

/// Retire question `id` under `root`: withdraw it for `reason` (ill-posed,
/// mis-filed, reworded, abandoned), naming where the investigation
/// continued. Refuses an already-retired question, and one an active question
/// still uses as a premise (retiring it must first force a review of that
/// inference).
pub fn retire(root: &Path, id: &str, reason: &str, replaced_by: &[String]) -> Result<()> {
    let _lock = lock(root)?;
    let store = load(&[root.to_path_buf()])?;
    let Some(existing) = store.get(id) else {
        bail!("no question `{id}` in {}", root.display());
    };
    if existing.is_retired() {
        bail!("question {} is already retired", existing.id);
    }
    let users: Vec<&str> = store
        .referrers(&existing.id, Relation::Basis)
        .into_iter()
        .filter(|q| !q.is_retired())
        .map(|q| q.id.as_str())
        .collect();
    if !users.is_empty() {
        bail!(
            "question {} is a premise of active question(s) {}: rework their `basis` first",
            existing.id,
            users.join(", ")
        );
    }
    let mut q = existing.clone();
    q.retired = Some(Retired {
        reason: reason.trim().to_string(),
        replaced_by: ids(replaced_by),
    });
    check(&store, &q, None)?;
    let text = toml::to_string(&q).context("serializing question")?;
    write_atomic(&question_path(root, &q.id), text.as_bytes())
}

/// What `amend` corrects: the question's answer, or one prose field of the
/// observation numbered `number` (1-based file order, the `#N` `show` prints).
#[derive(Debug, Clone, Copy)]
pub enum AmendTarget {
    Answer,
    Observation { number: usize, field: AmendField },
}

/// Correct question `id`'s answer or an observation's prose field to `value`
/// (`None` clears an optional field), recording the old and new value, the
/// time and `reason` beside the field. Allowed on retired questions too: it
/// is a correction of the record, not a measurement. Validation then runs as
/// for any write, so e.g. marking a run inconclusive recomputes its status
/// and clearing a required runtime `result` is refused. For an answer
/// amendment, returns what the answer now rests on (see `answer_support`).
pub fn amend(
    root: &Path,
    id: &str,
    target: AmendTarget,
    value: Option<&str>,
    reason: &str,
) -> Result<Option<String>> {
    let _lock = lock(root)?;
    let store = load(&[root.to_path_buf()])?;
    let Some(existing) = store.get(id) else {
        bail!("no question `{id}` in {}", root.display());
    };
    let mut q = existing.clone();
    let now = value.map(|v| v.trim().to_string());
    let at = now_utc()?;
    let reason = reason.trim().to_string();
    match target {
        AmendTarget::Answer => {
            let Some(answer) = &now else {
                bail!("the answer cannot be cleared, only replaced");
            };
            if *answer == q.answer {
                bail!("the answer already reads that");
            }
            let was = std::mem::replace(&mut q.answer, answer.clone());
            q.amendments.push(Amendment {
                at,
                field: AmendField::Answer,
                was: Some(was),
                now,
                reason,
            });
        }
        AmendTarget::Observation { number, field } => {
            let o = observation_mut(&mut q, number)?;
            let Some(current) = o.field_mut(field) else {
                bail!(
                    "`{}` is a question field, not an observation field",
                    field.as_str()
                );
            };
            if *current == now {
                bail!("`{}` already reads that", field.as_str());
            }
            let was = std::mem::replace(current, now.clone());
            o.amendments.push(Amendment {
                at,
                field,
                was,
                now,
                reason,
            });
        }
    }
    check(&store, &q, None)?;
    let text = toml::to_string(&q).context("serializing question")?;
    write_atomic(&question_path(root, &q.id), text.as_bytes())?;
    // Computed from the state just written, under the lock, so it describes
    // what was written and not what another writer made of it since.
    Ok(matches!(target, AmendTarget::Answer).then(|| answer_support(&store.with(&q), &q)))
}

/// What an amended answer now rests on, for the person asserting it: po
/// never checks that support implies an answer, so it shows the support
/// instead. Measured: the observations that decide status, weaker ones that
/// count, and active citations (stale ones marked); inferred: the premises.
/// A retired question's answer is history and rests on nothing.
fn answer_support(store: &Store, q: &Question) -> String {
    if q.is_retired() {
        return format!(
            "question {} is retired: the amended former answer has no active disposition",
            q.id
        );
    }
    let resolved = store.resolve(q);
    let support = if q.is_inferred() {
        let premises: Vec<String> = q
            .basis
            .iter()
            .map(|id| match store.get(id) {
                Some(p) => format!("{id} ({})", store.resolve(p)),
                None => id.clone(),
            })
            .collect();
        format!("premises {}", premises.join(", "))
    } else {
        let numbered = |keep: fn(&Observation) -> bool| -> Vec<String> {
            q.observations
                .iter()
                .enumerate()
                .filter(|(_, o)| keep(o))
                .map(|(i, o)| format!("#{} {}", i + 1, o.source.as_str()))
                .collect()
        };
        let deciding = numbered(Observation::decides);
        let weaker = numbered(|o| o.counts() && !o.decides());
        let mut parts = Vec::new();
        if deciding.is_empty() {
            parts.push("no deciding editor or chart observation".to_string());
        } else {
            parts.push(format!("deciding observations {}", deciding.join(", ")));
        }
        if !weaker.is_empty() {
            parts.push(format!(
                "weaker counting observations {}",
                weaker.join(", ")
            ));
        }
        let citations: Vec<String> = q
            .citations
            .iter()
            .enumerate()
            .filter(|(_, c)| c.void.is_none())
            .map(|(i, c)| match citation::staleness(c) {
                None => format!("#{} {}", i + 1, c.section),
                Some(_) => format!("#{} {} (stale)", i + 1, c.section),
            })
            .collect();
        if !citations.is_empty() {
            parts.push(format!("citations {}", citations.join(", ")));
        }
        parts.join("; ")
    };
    format!("the amended answer rests on {support}; status {resolved}")
}

/// What `void` withdraws, by 1-based file-order number (the `#N` `show`
/// prints for observations and citations).
#[derive(Debug, Clone, Copy)]
pub enum VoidTarget {
    Observation(usize),
    Citation(usize),
}

/// Withdraw an observation or a citation of question `id`: it stays visible
/// and never counts again. For a mis-transcribed outcome, code or candidate,
/// which `amend` cannot touch: void it, then observe the corrected
/// measurement. For a stale citation: void it after review, then cite
/// again; for one applied in error: void it. Not reversible by po.
pub fn void(root: &Path, id: &str, target: VoidTarget, reason: &str) -> Result<()> {
    let _lock = lock(root)?;
    let store = load(&[root.to_path_buf()])?;
    let Some(existing) = store.get(id) else {
        bail!("no question `{id}` in {}", root.display());
    };
    let mut q = existing.clone();
    let slot = match target {
        VoidTarget::Observation(number) => &mut observation_mut(&mut q, number)?.void,
        VoidTarget::Citation(number) => {
            let count = q.citations.len();
            match number.checked_sub(1).and_then(|i| q.citations.get_mut(i)) {
                Some(c) => &mut c.void,
                None => bail!("question {id} has no citation #{number} (it has {count})"),
            }
        }
    };
    if slot.is_some() {
        bail!("that record of question {id} is already void");
    }
    *slot = Some(Void {
        at: now_utc()?,
        reason: reason.trim().to_string(),
    });
    check(&store, &q, None)?;
    let text = toml::to_string(&q).context("serializing question")?;
    write_atomic(&question_path(root, &q.id), text.as_bytes())
}

/// Where `cite` put the citation: its 1-based position in file order.
#[derive(Debug, Clone)]
pub struct Cited {
    pub number: usize,
    /// An identical active citation already existed, whose number this is;
    /// nothing was written.
    pub existing: bool,
    /// The question's disposition after the write, then one line per active
    /// citation that is stale (computed under the write lock).
    pub outcome: Vec<String>,
}

/// Cite the manual section `section` (the 8-hex id `po search` prints, or
/// `page#anchor`) as support for question `id`'s answer, quoting `quotes`
/// from it. Each quote must lie inside one block of the section subtree in
/// the manual baked into this binary; the subtree's markdown digest is kept
/// so later drift makes the citation stale. Citing the same section with
/// the same quotes again is a no-op while the earlier citation is active
/// and current (a citation is a claim, not a measurement: repeating it adds
/// nothing).
pub fn cite(
    root: &Path,
    id: &str,
    section: &str,
    quotes: &[String],
    note: Option<&str>,
) -> Result<Cited> {
    let _lock = lock(root)?;
    let store = load(&[root.to_path_buf()])?;
    let Some(existing) = store.get(id) else {
        bail!("no question `{id}` in {}", root.display());
    };
    if let Some(r) = &existing.retired {
        bail!(
            "question {} is retired{}: cite under the question the passage supports",
            existing.id,
            replaced_hint(r)
        );
    }
    let s = citation::resolve_section(section)?;
    let quotes: Vec<String> = quotes.iter().map(|q| q.trim().to_string()).collect();
    if quotes.is_empty() {
        bail!("a citation needs at least one --quote");
    }
    if quotes.iter().any(String::is_empty) {
        bail!("an empty --quote");
    }
    for quote in &quotes {
        if !citation::quote_found(s, quote) {
            bail!(
                "manual section {} ({}) does not say \"{quote}\" within one paragraph, list item or table cell",
                s.id,
                citation::key(s)
            );
        }
    }
    let new = Citation {
        section: citation::key(s),
        quotes,
        digest: citation::digest(s),
        at: now_utc()?,
        note: trimmed(note.map(str::to_string).as_ref()),
        void: None,
    };
    let mut q = existing.clone();
    // Same digest: the earlier one is current, so it is the same claim on the
    // same text. A stale match is not reused; void it and cite anew.
    if let Some(i) = q.citations.iter().position(|c| {
        c.void.is_none()
            && c.section == new.section
            && c.quotes == new.quotes
            && c.digest == new.digest
    }) {
        if q.citations[i].note != new.note {
            bail!(
                "citation #{} already quotes this with a different note; void it to cite anew",
                i + 1
            );
        }
        return Ok(Cited {
            number: i + 1,
            existing: true,
            outcome: cite_outcome(&store, &q),
        });
    }
    q.citations.push(new);
    check(&store, &q, None)?;
    let text = toml::to_string(&q).context("serializing question")?;
    write_atomic(&question_path(root, &q.id), text.as_bytes())?;
    Ok(Cited {
        number: q.citations.len(),
        existing: false,
        outcome: cite_outcome(&store.with(&q), &q),
    })
}

/// The question's disposition after a `cite`, plus every active citation
/// that does not count: one stale member keeps the whole support set from
/// holding, so the caller learns which to review and void.
fn cite_outcome(store: &Store, q: &Question) -> Vec<String> {
    let mut lines = vec![format!("question {} is {}", q.id, store.resolve(q))];
    for (i, c) in q.citations.iter().enumerate() {
        if c.void.is_none()
            && let Some(why) = citation::staleness(c)
        {
            lines.push(format!(
                "citation #{} is stale ({why}): review it, then void it with `void --citation {}`",
                i + 1,
                i + 1
            ));
        }
    }
    lines
}

/// Observation `number` (1-based file order) of `q`.
fn observation_mut(q: &mut Question, number: usize) -> Result<&mut Observation> {
    let count = q.observations.len();
    let id = q.id.clone();
    match number
        .checked_sub(1)
        .and_then(|i| q.observations.get_mut(i))
    {
        Some(o) => Ok(o),
        None => bail!("question {id} has no observation #{number} (it has {count})"),
    }
}

/// The current UTC time, to the second, as a TOML datetime.
fn now_utc() -> Result<Datetime> {
    let stamp = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
    stamp
        .parse::<Datetime>()
        .map_err(|e| anyhow::anyhow!("timestamp `{stamp}`: {e}"))
}

/// Question ids as given on the command line, normalized like `Store::get`.
fn ids(given: &[String]) -> Vec<String> {
    given
        .iter()
        .map(|d| d.trim().to_ascii_lowercase())
        .collect()
}

/// Append one observation to question `id` under `root`, or, under a key the
/// question already holds, confirm the identical stored one.
pub fn observe(root: &Path, id: &str, new: &NewObservation) -> Result<Observed> {
    let _lock = lock(root)?;
    let store = load(&[root.to_path_buf()])?;
    let Some(existing) = store.get(id) else {
        bail!("no question `{id}` in {}", root.display());
    };
    if let Some(r) = &existing.retired {
        bail!(
            "question {} is retired{}: record the observation under the question it tests",
            existing.id,
            replaced_hint(r)
        );
    }
    let mut q = existing.clone();

    let date = parse_date(new.date.as_deref())?;
    let date_before = parse_date(new.date_before.as_deref())?;

    let fixture_bytes = match &new.fixture {
        Some(path) => Some(
            std::fs::read(path).with_context(|| format!("reading fixture {}", path.display()))?,
        ),
        None => None,
    };
    let fixture_sha = fixture_bytes.as_deref().map(sha256_hex);
    let passed_name = new
        .fixture
        .as_ref()
        .and_then(|p| p.file_name())
        .map(|n| n.to_string_lossy().into_owned());
    // Re-observing a fixture by its store path (`fixtures/<sha>.pine`) must
    // not record the hash file name as the human name: keep the name the
    // store already knows for that hash, if any.
    let fixture_name = match (&fixture_sha, passed_name) {
        (Some(sha), Some(name)) if name == format!("{sha}.pine") => {
            known_fixture_name(store.questions(), sha)
        }
        (_, name) => name,
    };

    let observation = Observation {
        source: new.source,
        // Keys are exact: stored and matched as given, never normalized.
        key: new.key.clone(),
        date,
        date_before,
        fixture: fixture_sha.clone(),
        fixture_name,
        outcome: new.outcome,
        crash: trimmed(new.crash.as_ref()),
        result: trimmed(new.result.as_ref()),
        settings: trimmed(new.settings.as_ref()),
        inconclusive: trimmed(new.inconclusive.as_ref()),
        environment: trimmed(new.environment.as_ref()),
        note: trimmed(new.note.as_ref()),
        evidence: evidence_paths(root, &new.evidence)?,
        errors: new.errors.clone(),
        warnings: new.warnings.clone(),
        candidates: decide_candidates(&new.candidates, &new.selected, &new.refuted)?,
        void: None,
        amendments: Vec::new(),
    };
    if let Some(key) = &observation.key
        && let Some(i) = q
            .observations
            .iter()
            .position(|o| o.key.as_ref() == Some(key))
    {
        let stored = &q.observations[i];
        if let Some(v) = &stored.void {
            bail!(
                "key `{key}` belongs to observation #{} of question {}, which is void: {}",
                i + 1,
                q.id,
                v.reason
            );
        }
        // Current values only: amendment history is not part of the payload,
        // and a script still emitting a since-amended value fails here.
        let current = Observation {
            amendments: Vec::new(),
            ..stored.clone()
        };
        let diff = field_diff(&current, &observation)?;
        if !diff.is_empty() {
            bail!(
                "key `{key}` belongs to observation #{} of question {}, which differs:\n  {}",
                i + 1,
                q.id,
                diff.join("\n  ")
            );
        }
        return Ok(Observed {
            number: i + 1,
            existing: true,
            warnings: Vec::new(),
        });
    }
    q.observations.push(observation);
    let warnings = check(&store, &q, fixture_sha.as_deref())?;

    if let (Some(sha), Some(bytes)) = (&fixture_sha, &fixture_bytes) {
        store_fixture(root, sha, bytes)?;
    }
    let text = toml::to_string(&q).context("serializing question")?;
    write_atomic(&question_path(root, &q.id), text.as_bytes())?;
    Ok(Observed {
        number: q.observations.len(),
        existing: false,
        warnings,
    })
}

/// Put a fixture into the content-addressed store. An existing file is kept
/// only if it really hashes to its name; anything else (e.g. a truncated
/// leftover from an interrupted write) is replaced.
fn store_fixture(root: &Path, sha: &str, bytes: &[u8]) -> Result<()> {
    let path = fixture_path(root, sha);
    if std::fs::read(&path).is_ok_and(|existing| sha256_hex(&existing) == sha) {
        return Ok(());
    }
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).with_context(|| format!("creating {}", dir.display()))?;
    }
    write_atomic(&path, bytes)
}

/// Write `bytes` to a dot-prefixed temp file beside `path` (unique per
/// process), then rename it into place, so an interrupted write never leaves
/// a partial record or fixture under its real name. The loader ignores
/// dot-files. Callers hold the directory lock, so concurrent writers
/// serialize rather than race.
fn write_atomic(path: &Path, bytes: &[u8]) -> Result<()> {
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    let tmp = path.with_file_name(format!(".{name}.{}.tmp", std::process::id()));
    std::fs::write(&tmp, bytes).with_context(|| format!("writing {}", tmp.display()))?;
    std::fs::rename(&tmp, path).with_context(|| format!("replacing {}", path.display()))
}

/// Validate `q` as it would be written into `store` (new or replacing its
/// namesake), including the relation cycles and key clashes only the whole
/// store can show;
/// errors refuse the write.
fn check(store: &Store, q: &Question, pending_fixture: Option<&str>) -> Result<Vec<String>> {
    let after_store = store.with(q);
    let after = after_store.questions();
    let known_ids = validate::ids_in(after, &q.root);
    let mut report = Report::default();
    let cx = Context {
        known_ids: &known_ids,
        pending_fixture,
    };
    validate::question(q, &cx, &mut report);
    report.errors.extend(validate::cycles(after));
    report.errors.extend(validate::duplicate_keys(after));
    if !report.errors.is_empty() {
        bail!("refusing to write:\n  {}", report.errors.join("\n  "));
    }
    Ok(report.warnings)
}

/// The human name some earlier observation recorded for fixture `sha`.
fn known_fixture_name(questions: &[Question], sha: &str) -> Option<String> {
    questions
        .iter()
        .flat_map(|q| &q.observations)
        .filter(|o| o.fixture.as_deref() == Some(sha))
        .find_map(|o| o.fixture_name.clone())
}

/// Parse an optional `YYYY-MM-DD`; validation then checks it is date-only and
/// that exactly one of `date` / `date_before` is present.
fn parse_date(s: Option<&str>) -> Result<Option<Datetime>> {
    s.map(|d| {
        d.trim()
            .parse::<Datetime>()
            .map_err(|e| anyhow::anyhow!("date `{d}`: {e}"))
    })
    .transpose()
}

fn trimmed(s: Option<&String>) -> Option<String> {
    s.map(|v| v.trim().to_string())
}

/// Apply `--selected` / `--refuted` to the declared candidates. Each must
/// name a declared candidate, and none may be both.
fn decide_candidates(
    declared: &[Candidate],
    selected: &[String],
    refuted: &[String],
) -> Result<Vec<Candidate>> {
    for name in selected.iter().chain(refuted) {
        if !declared.iter().any(|c| &c.name == name) {
            bail!("`{name}` is not a declared candidate (declare it with --candidate)");
        }
    }
    if let Some(both) = selected.iter().find(|s| refuted.contains(s)) {
        bail!("candidate `{both}` cannot be both selected and refuted");
    }
    Ok(declared
        .iter()
        .map(|c| {
            let state = if selected.contains(&c.name) {
                CandidateState::Selected
            } else if refuted.contains(&c.name) {
                CandidateState::Refuted
            } else {
                CandidateState::Undecided
            };
            Candidate { state, ..c.clone() }
        })
        .collect())
}

/// Each evidence path must exist now; it is stored relative to the records
/// directory so the record stays valid wherever the repo is checked out.
fn evidence_paths(root: &Path, evidence: &[PathBuf]) -> Result<Vec<String>> {
    let base = root
        .canonicalize()
        .with_context(|| format!("resolving {}", root.display()))?;
    evidence
        .iter()
        .map(|p| {
            let target = p
                .canonicalize()
                .with_context(|| format!("evidence {} does not exist", p.display()))?;
            Ok(relative_path(&base, &target))
        })
        .collect()
}

/// `target` relative to `base` (both absolute and canonical), `/`-separated.
pub(super) fn relative_path(base: &Path, target: &Path) -> String {
    let b: Vec<Component<'_>> = base.components().collect();
    let t: Vec<Component<'_>> = target.components().collect();
    let common = b.iter().zip(&t).take_while(|(x, y)| x == y).count();
    let mut parts: Vec<String> = vec!["..".to_string(); b.len() - common];
    parts.extend(
        t[common..]
            .iter()
            .map(|c| c.as_os_str().to_string_lossy().into_owned()),
    );
    parts.join("/")
}

/// A new 8-hex id: FNV-1a over the question text and the current time,
/// re-salted until it collides with neither a loaded id nor a file on disk.
fn fresh_id(question: &str, taken: &BTreeSet<String>, root: &Path) -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or_default();
    let mut salt: u32 = 0;
    loop {
        let mut h: u32 = 0x811c_9dc5;
        for b in question
            .bytes()
            .chain(nanos.to_le_bytes())
            .chain(salt.to_le_bytes())
        {
            h ^= u32::from(b);
            h = h.wrapping_mul(0x0100_0193);
        }
        let id = format!("{h:08x}");
        if !taken.contains(&id) && !question_path(root, &id).exists() {
            return id;
        }
        salt += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn relative_path_climbs_out_of_the_records_dir() {
        let base = Path::new("/repo/reference/records");
        assert_eq!(
            relative_path(base, Path::new("/repo/reference/fieldwork/run26.json")),
            "../fieldwork/run26.json"
        );
        assert_eq!(
            relative_path(base, Path::new("/repo/reference/records/x.json")),
            "x.json"
        );
    }
}
