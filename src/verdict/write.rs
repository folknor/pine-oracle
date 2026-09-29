// Write side: `add` creates a question, `observe` appends one observation,
// `retire` withdraws a question. Each takes the records directory's lock,
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
    Candidate, CandidateState, Diag, Kind, Observation, Outcome, Question, Relation, Retired,
    Source, Store, load,
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
        // Only the plain fields `add` sets are compared: observations and
        // retirement are later history, not part of the add payload.
        let stored = Question {
            observations: Vec::new(),
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
/// mis-filed, reworded, abandoned), naming where the investigation continued. Refuses an already-retired
/// question, and one an active question still uses as a premise (retiring it
/// must first force a review of that inference).
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
    };
    if let Some(key) = &observation.key
        && let Some(i) = q
            .observations
            .iter()
            .position(|o| o.key.as_ref() == Some(key))
    {
        let diff = field_diff(&q.observations[i], &observation)?;
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
    let mut after: Vec<Question> = store
        .questions()
        .iter()
        .filter(|x| x.id != q.id)
        .cloned()
        .collect();
    after.push(q.clone());
    let known_ids = validate::ids_in(&after, &q.root);
    let mut report = Report::default();
    let cx = Context {
        known_ids: &known_ids,
        pending_fixture,
    };
    validate::question(q, &cx, &mut report);
    report.errors.extend(validate::cycles(&after));
    report.errors.extend(validate::duplicate_keys(&after));
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
