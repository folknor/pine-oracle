// Write side: `add` creates a question, `observe` appends one observation,
// `retire` withdraws a question, `close` / `reopen` stop and resume an open
// question's investigation, `amend` corrects a question's wording, its
// answer, its negative mark or an observation's prose keeping the history, `cite` records a manual passage
// supporting the answer, `void` withdraws an observation or a citation.
// Each takes the verdicts directory's lock exclusively,
// strictly loads it (never write into a store that is already invalid),
// validates the new state with the same rules the read side enforces, and
// only then touches disk. po computes fixture hashes itself from the bytes it
// stores, so a recorded hash cannot disagree with its file. `add` and
// `observe` take an optional capture key that makes a rerun a no-op.
//
// Durability: every file a verb writes is synced, and so (on Unix, see
// `sync_dir`) is the directory entry that publishes it, before the verb
// reports success; a fixture (file,
// `fixtures/`, root) is synced before the record that references it is
// renamed into place, whether this verb stored it or found it already
// stored. A no-op (keyed rerun, identical citation) writes nothing and
// certifies nothing.

use anyhow::{Context as _, Result, bail};
use std::collections::BTreeSet;
use std::path::{Component, Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use toml::value::Datetime;

use super::evidence::{self, Inspector, NewEvidence};
use super::load::load_unlocked;
use super::validate::{self, Context, Report, fixture_path, question_path, sha256_hex};
use super::{
    AmendField, Amendment, Candidate, CandidateState, Citation, Closure, ClosureAction, Diag,
    Disagreement, Disposition, Kind, Observation, Outcome, Question, Relation, Retired, Source,
    Status, Store, Void, citation, load,
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
    /// The answer reports that the behavior asked about was not identified.
    pub negative: bool,
    /// Runtime: the candidates are mutually exclusive alternatives.
    pub exclusive: bool,
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
    /// The question's id as stored (the caller's may differ in case).
    pub id: String,
    pub number: usize,
    /// The key was already held by an identical observation, whose number
    /// this is; nothing was written.
    pub existing: bool,
    pub warnings: Vec<String>,
    /// The question's disposition before and after the observation (equal
    /// for a keyed rerun), as `show` prints it.
    pub was: Disposition,
    pub now: Disposition,
}

/// Whether `observe` writes, or only reports what writing would do.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Mode {
    Write,
    DryRun,
}

/// An observation to append. Paths (`fixture`, `evidence`) are as the caller
/// named them; `observe` copies the fixture into the store, and records each
/// evidence file's location relative to the evidence directory (refusing
/// one outside it) with the digest of its decoded content.
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
    /// Declared candidates (state ignored; set from `selected` /
    /// `consistent` / `refuted`, undecided otherwise).
    pub candidates: Vec<Candidate>,
    pub selected: Vec<String>,
    pub consistent: Vec<String>,
    pub refuted: Vec<String>,
    pub evidence: Vec<NewEvidence>,
    pub environment: Option<String>,
    pub note: Option<String>,
    pub inconclusive: Option<String>,
}

/// Create a question under `root` and return its generated id.
pub fn add(root: &Path, new: &NewQuestion) -> Result<Added> {
    let root = &evidence::absolute(root)?;
    let _lock = lock(root)?;
    let store = load_unlocked(root)?;
    let mut q = Question {
        id: String::new(),
        root: root.clone(),
        kind: new.kind,
        // Keys are exact: stored and matched as given, never normalized.
        key: new.key.clone(),
        question: new.question.trim().to_string(),
        answer: new.answer.trim().to_string(),
        identifiers: new.identifiers.clone(),
        follow_up_to: ids(&new.follow_up_to),
        basis: ids(&new.basis),
        negative: new.negative,
        exclusive: new.exclusive,
        legacy_derived_from: None,
        retired: None,
        closures: Vec::new(),
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
        // The question text is the claim's identity: matching a superseded
        // wording would hand a script written for the old claim (say, both
        // halves of a since-split question) the reworded question's id, and
        // its observations would land under a claim they do not test.
        // Exclusivity likewise changes how the script's future captures are
        // read. The current value is matched first, so a field amended away
        // and back accepts its current value and refuses the one between.
        for (field, given, current) in [
            (
                AmendField::Question,
                Some(q.question.clone()),
                Some(existing.question.clone()),
            ),
            (
                AmendField::Exclusive,
                q.exclusive_value(),
                existing.exclusive_value(),
            ),
        ] {
            if given == current {
                continue;
            }
            let history = || existing.amendments.iter().filter(|a| a.field == field);
            if !history().any(|a| a.was == given || a.now == given) {
                continue;
            }
            // Cite the amendment that explains the current value: for the
            // text, the latest one that reworded away from the given wording;
            // for the mark, the latest one, which set the current state.
            let Some(a) = (match field {
                AmendField::Question => history().rev().find(|a| a.was == given),
                _ => history().next_back(),
            })
            .or_else(|| history().next_back()) else {
                continue;
            };
            let id = &existing.id;
            match field {
                AmendField::Question => bail!(
                    "key `{key}` belongs to question {id}, whose question was reworded from this \
                     text ({}: {}). Its claim is now: {}\nUpdate the script to the current \
                     wording, or, if it still tests the old claim, record it under the question \
                     that claim now lives in.",
                    a.at,
                    a.reason,
                    existing.question
                ),
                _ => bail!(
                    "key `{key}` belongs to question {id}, which is {} since an amendment ({}: \
                     {}): its candidates are {}. Update the script to pass {}, since its \
                     captures will be compared under that.",
                    if existing.exclusive {
                        "exclusive"
                    } else {
                        "no longer exclusive"
                    },
                    a.at,
                    a.reason,
                    if existing.exclusive {
                        "exclusive alternatives, so runs selecting different ones conflict"
                    } else {
                        "not assumed exclusive"
                    },
                    if existing.exclusive {
                        "--exclusive"
                    } else {
                        "no --exclusive"
                    }
                ),
            }
        }
        // Only the fields `add` sets are compared: observations, citations,
        // amendments, closures and retirement come later, not part of the
        // add payload. An amended answer or negative mark matches any value
        // it has held, so amending does not break the script that added the
        // question.
        let stored = Question {
            observations: Vec::new(),
            amendments: Vec::new(),
            citations: Vec::new(),
            closures: Vec::new(),
            ..existing.clone()
        };
        let mut answer = Some(q.answer.clone());
        let mut negative = q.negative_value();
        let mut warnings = Vec::new();
        for (field, given, held) in [
            (AmendField::Answer, &mut answer, Some(stored.answer.clone())),
            (AmendField::Negative, &mut negative, stored.negative_value()),
        ] {
            if accept_held(given, &held, field, &existing.amendments) {
                warnings.push(superseded(field, &format!("question {}", existing.id)));
            }
        }
        let given = Question {
            answer: answer.unwrap_or_default(),
            negative: negative.is_some(),
            ..q.clone()
        };
        let diff = field_diff(&stored, &given)?;
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
            warnings,
        });
    }
    // Only now, creating a question, are its identifiers checked against
    // pine-data: a keyed rerun above must stay a no-op after a pine-data
    // refresh dropped one of them, and stored identifiers are never
    // re-checked.
    let unknown: Vec<String> = q
        .identifiers
        .iter()
        .filter_map(|i| validate::identifier_membership(i).err())
        .collect();
    if !unknown.is_empty() {
        bail!("refusing to write:\n  {}", unknown.join("\n  "));
    }
    let taken: BTreeSet<String> = store.questions().iter().map(|x| x.id.clone()).collect();
    q.id = fresh_id(&q.question, &taken, root);
    let warnings = check(&store, &q, None)?;
    write_question(&q)?;
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

/// Hold the verdicts directory's write lock until the returned file drops.
/// Every write verb holds it from its strict load through its last write, so
/// a check (key absent, question not retired) cannot go stale before the
/// write that relies on it. The lock file is a persistent dot-file (the
/// loader ignores dot-files); it is never deleted, since recreating it would
/// let two processes lock different files. po owns the directory, so the
/// first write verb creates it to hold the lock; `records/` is created only
/// when a record is written.
pub(super) fn lock(root: &Path) -> Result<std::fs::File> {
    ensure_dir(root)?;
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
pub fn retire(root: &Path, id: &str, reason: &str, replaced_by: &[String]) -> Result<Vec<String>> {
    let root = &evidence::absolute(root)?;
    let _lock = lock(root)?;
    let store = load_unlocked(root)?;
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
    let warnings = check(&store, &q, None)?;
    write_question(&q)?;
    Ok(warnings)
}

/// Stop the investigation of open question `id` for `reason`: it stays open
/// (nothing was learned) and shows as `open, closed`, and `list --status
/// open`, the owed-work view, leaves it out. Refused on a retired question,
/// on an inferred one (its openness names the premise that owes the work:
/// close or pursue that premise instead), on one already closed, and on one
/// that does not resolve open.
pub fn close(root: &Path, id: &str, reason: &str) -> Result<Vec<String>> {
    let root = &evidence::absolute(root)?;
    let _lock = lock(root)?;
    let store = load_unlocked(root)?;
    let Some(existing) = store.get(id) else {
        bail!("no question `{id}` in {}", root.display());
    };
    if let Some(r) = &existing.retired {
        bail!(
            "question {} is retired{}: it has no investigation to close",
            existing.id,
            replaced_hint(r)
        );
    }
    if existing.is_inferred() {
        bail!(
            "question {} is inferred: its openness names the premise that owes the work, so close or pursue that premise instead",
            existing.id
        );
    }
    if existing.is_closed() {
        bail!("question {} is already closed", existing.id);
    }
    let resolved = store.resolve(existing);
    if resolved.status() != Some(Status::Open) {
        bail!(
            "question {} is {resolved}: only an open question's investigation can be closed",
            existing.id
        );
    }
    let mut q = existing.clone();
    q.closures.push(Closure {
        at: now_utc()?,
        action: ClosureAction::Close,
        reason: reason.trim().to_string(),
        automatic: false,
    });
    let warnings = check(&store, &q, None)?;
    write_question(&q)?;
    Ok(warnings)
}

/// Resume the investigation of closed question `id`. Allowed whenever the
/// last closure entry is a close, even while the closure is suspended (the
/// question currently resolves decided), so a dormant closure can be ended
/// before it resumes. Refused on a retired question, whose closure is history.
pub fn reopen(root: &Path, id: &str, reason: &str) -> Result<Vec<String>> {
    let root = &evidence::absolute(root)?;
    let _lock = lock(root)?;
    let store = load_unlocked(root)?;
    let Some(existing) = store.get(id) else {
        bail!("no question `{id}` in {}", root.display());
    };
    if existing.is_retired() {
        bail!(
            "question {} is retired: its closure is history",
            existing.id
        );
    }
    if !existing.is_closed() {
        bail!("question {} is not closed", existing.id);
    }
    let mut q = existing.clone();
    q.closures.push(Closure {
        at: now_utc()?,
        action: ClosureAction::Reopen,
        reason: reason.trim().to_string(),
        automatic: false,
    });
    let warnings = check(&store, &q, None)?;
    write_question(&q)?;
    Ok(warnings)
}

/// End the closure of `q` (as it is about to be written into `store`) when
/// the write leaves it decided: an active, measured, closed question that no
/// longer resolves open gets an automatic reopen in the same write, so voiding
/// whatever decided it later leaves it open (owed work) rather than silently
/// closed again. `did` says what the write did, for the reason. Retired
/// questions are left alone: their closure is history.
fn end_closure_if_decided(store: &Store, q: &mut Question, did: &str) -> Result<()> {
    if q.is_retired() || q.is_inferred() || !q.is_closed() {
        return Ok(());
    }
    let resolved = store.with(q).resolve(q);
    if resolved.status() == Some(Status::Open) {
        return Ok(());
    }
    q.closures.push(Closure {
        at: now_utc()?,
        action: ClosureAction::Reopen,
        reason: format!("the question is now {resolved} after {did}"),
        automatic: true,
    });
    Ok(())
}

/// What `amend` corrects: the question's text, answer or negative mark, or
/// one prose field of the observation numbered `number` (1-based file order,
/// the `#N` `show` prints).
#[derive(Debug, Clone, Copy)]
pub enum AmendTarget {
    /// A wording correction of the question text. Refused on a retired
    /// question, whose text is what its replacements replaced.
    Question,
    Answer,
    /// A value marks the answer negative; `None` clears the mark.
    Negative,
    /// A value marks the candidates exclusive alternatives; `None` clears
    /// the mark.
    Exclusive,
    Observation {
        number: usize,
        field: AmendField,
    },
}

/// What `amend` did beyond the write.
#[derive(Debug, Clone)]
pub struct Amended {
    /// For a question-level amendment, what the answer now rests on (see
    /// `answer_support`).
    pub support: Option<String>,
    /// Non-blocking warnings: validation warnings about the question as
    /// written, then the disagreements the amendment introduced (see
    /// `amend_contradictions`).
    pub warnings: Vec<String>,
}

/// `amend_all` with one change.
pub fn amend(
    root: &Path,
    id: &str,
    target: AmendTarget,
    value: Option<&str>,
    reason: &str,
) -> Result<Amended> {
    amend_all(root, id, &[(target, value)], reason)
}

/// Correct question `id`'s text, answer, negative or exclusive mark, or observations'
/// prose fields, each to its value (`None` clears an optional field),
/// recording the old and new value, the time and `reason` beside each field,
/// in one write (an answer rewrite and its classification change together,
/// never through a contradictory intermediate record). Allowed on retired
/// questions too, except for the question text: it is a correction of the
/// record, not a measurement. Validation then runs as for any write, so e.g.
/// marking a run inconclusive recomputes its status and clearing a required
/// runtime `result` is refused. Clearing `inconclusive` (a run counts again)
/// or marking the question exclusive (selections rival) can add
/// disagreements, which are warned about as `observe` warns of a new one.
pub fn amend_all(
    root: &Path,
    id: &str,
    changes: &[(AmendTarget, Option<&str>)],
    reason: &str,
) -> Result<Amended> {
    let root = &evidence::absolute(root)?;
    let _lock = lock(root)?;
    let store = load_unlocked(root)?;
    let Some(existing) = store.get(id) else {
        bail!("no question `{id}` in {}", root.display());
    };
    if changes.is_empty() {
        bail!("nothing to amend");
    }
    let mut q = existing.clone();
    let at = now_utc()?;
    let reason = reason.trim().to_string();
    let mut question_level = false;
    for (target, value) in changes {
        let now = value.map(|v| v.trim().to_string());
        match *target {
            AmendTarget::Question => {
                question_level = true;
                if let Some(r) = &existing.retired {
                    bail!(
                        "question {} is retired{}: its text is what was replaced, so it stays as worded",
                        existing.id,
                        replaced_hint(r)
                    );
                }
                let Some(text) = &now else {
                    bail!("the question cannot be cleared, only reworded");
                };
                if *text == q.question {
                    bail!("the question already reads that");
                }
                let was = std::mem::replace(&mut q.question, text.clone());
                q.amendments.push(Amendment {
                    at,
                    field: AmendField::Question,
                    was: Some(was),
                    now,
                    reason: reason.clone(),
                });
            }
            AmendTarget::Answer => {
                question_level = true;
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
                    reason: reason.clone(),
                });
            }
            AmendTarget::Negative => {
                question_level = true;
                let negative = now.is_some();
                if negative == q.negative {
                    bail!(
                        "the answer is already {}marked negative",
                        if negative { "" } else { "not " }
                    );
                }
                let was = q.negative_value();
                q.negative = negative;
                q.amendments.push(Amendment {
                    at,
                    field: AmendField::Negative,
                    was,
                    now: q.negative_value(),
                    reason: reason.clone(),
                });
            }
            AmendTarget::Exclusive => {
                question_level = true;
                let exclusive = now.is_some();
                if exclusive == q.exclusive {
                    bail!(
                        "the candidates are already {}marked exclusive",
                        if exclusive { "" } else { "not " }
                    );
                }
                let was = q.exclusive_value();
                q.exclusive = exclusive;
                q.amendments.push(Amendment {
                    at,
                    field: AmendField::Exclusive,
                    was,
                    now: q.exclusive_value(),
                    reason: reason.clone(),
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
                    reason: reason.clone(),
                });
            }
        }
    }
    end_closure_if_decided(&store, &mut q, "an amendment")?;
    let mut warnings = check(&store, &q, None)?;
    let disposition = (store.disposition(existing), store.with(&q).disposition(&q));
    warnings.extend(amend_contradictions(existing, &q, &disposition));
    write_question(&q)?;
    // Computed from the state just written, under the lock, so it describes
    // what was written and not what another writer made of it since.
    Ok(Amended {
        support: question_level.then(|| answer_support(&store.with(&q), &q)),
        warnings,
    })
}

/// The warning for the disagreements an amendment introduced, `before` ->
/// `after`: an observation made to count again (cleared inconclusive) or
/// the question marked exclusive can each add an axis, even to a pair that
/// already conflicted. Every pair of same-strength observations counting
/// after the write is compared under the rules before and after (nothing
/// before, if either did not count), and each pair with new axes is named
/// once with just those axes. Stopping counting or clearing exclusivity
/// only removes axes, and prose fixes touch none.
fn amend_contradictions(
    before: &Question,
    after: &Question,
    (was, now): &(Disposition, Disposition),
) -> Option<String> {
    if after.is_retired() {
        return None;
    }
    let mut did = Vec::new();
    if after.exclusive && !before.exclusive {
        did.push("marking the question exclusive".to_string());
    }
    for (i, (b, a)) in before
        .observations
        .iter()
        .zip(&after.observations)
        .enumerate()
    {
        if !b.counts() && a.counts() {
            did.push(format!("clearing inconclusive on #{}", i + 1));
        }
    }
    let (old, new) = (&before.observations, &after.observations);
    let mut pairs = Vec::new();
    let mut rival = false;
    for i in 0..new.len() {
        for j in i + 1..new.len() {
            let (a, b) = (&new[i], &new[j]);
            if !a.counts() || !b.counts() || a.source.strength() != b.source.strength() {
                continue;
            }
            let held = if old[i].counts() && old[j].counts() {
                old[i].disagreements(&old[j], before.rules())
            } else {
                Vec::new()
            };
            let (us, them) = (format!("#{}", i + 1), format!("#{}", j + 1));
            let axes: Vec<String> = a
                .disagreements(b, after.rules())
                .into_iter()
                .filter(|d| !held.contains(d))
                .inspect(|d| rival |= matches!(d, Disagreement::Rival(_)))
                .map(|d| d.describe(&us, &them))
                .collect();
            if !axes.is_empty() {
                // Endpoint pairs never decide status: say so per pair, since
                // one amendment can revive both tiers.
                let tier = if a.decides() {
                    ""
                } else {
                    "weaker source only, "
                };
                pairs.push(format!(
                    "{tier}{us} {} {} and {them} {} {} [{}]",
                    a.source.as_str(),
                    a.verdict_summary(after.kind),
                    b.source.as_str(),
                    b.verdict_summary(after.kind),
                    axes.join("; ")
                ));
            }
        }
    }
    if pairs.is_empty() {
        return None;
    }
    let id = &after.id;
    let status = if was == now {
        format!("question {id} stays {now}")
    } else {
        format!("question {id} is now {now} (was {was})")
    };
    let exclusive_hint = if rival {
        " If the candidates are not true alternatives, `amend --clear exclusive`."
    } else {
        ""
    };
    Some(format!(
        "after {}, observations newly contradict: {}: {status}. If the fixtures test \
         different claims (a control is the usual case: it is its own question), they \
         belong to separate questions: void the misfiled one (or mark it inconclusive \
         again) and observe it under its own question. If one was mis-recorded, void that \
         one.{exclusive_hint}",
        if did.is_empty() {
            "the amendment".to_string()
        } else {
            did.join(" and ")
        },
        pairs.join(", ")
    ))
}

/// What an amended answer now rests on, for the person asserting it: po
/// never checks that support implies an answer, so it shows the support
/// instead. Measured: the observations that decide status, weaker ones that
/// count, and active citations (stale ones marked); inferred: the premises.
/// A retired question's answer is history and rests on nothing, but what it
/// replaces is still history worth naming. Either way, the questions resting
/// on this one follow.
fn answer_support(store: &Store, q: &Question) -> String {
    // What rests on this question in turn: a reworded claim or answer is
    // what those inferences and replacements now point at.
    let dependents: String = [
        ("premise of", Relation::Basis),
        ("replaces", Relation::ReplacedBy),
    ]
    .into_iter()
    .filter_map(|(label, relation)| {
        let ids: Vec<String> = store
            .referrers(&q.id, relation)
            .into_iter()
            .filter(|r| relation == Relation::ReplacedBy || !r.is_retired())
            .map(|r| match relation {
                Relation::Basis => format!("{} ({})", r.id, store.disposition(r)),
                _ => r.id.clone(),
            })
            .collect();
        (!ids.is_empty()).then(|| format!("; {label} {}", ids.join(", ")))
    })
    .collect();
    if q.is_retired() {
        return format!(
            "question {} is retired: the amended former answer has no active disposition{dependents}",
            q.id
        );
    }
    let resolved = store.disposition(q);
    let support = if q.is_inferred() {
        let premises: Vec<String> = q
            .basis
            .iter()
            .map(|id| match store.get(id) {
                Some(p) => format!("{id} ({})", store.disposition(p)),
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
        // Top-tier counting runs, not "deciding": a run whose candidates are
        // all consistent or undecided counts without settling anything.
        let top = numbered(Observation::decides);
        let weaker = numbered(|o| o.counts() && !o.decides());
        let mut parts = Vec::new();
        if top.is_empty() {
            parts.push("no counting editor or chart observation".to_string());
        } else {
            parts.push(format!(
                "counting editor/chart observations {}",
                top.join(", ")
            ));
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
    format!("the answer rests on {support}; status {resolved}{dependents}")
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
pub fn void(root: &Path, id: &str, target: VoidTarget, reason: &str) -> Result<Vec<String>> {
    let root = &evidence::absolute(root)?;
    let _lock = lock(root)?;
    let store = load_unlocked(root)?;
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
    let did = match target {
        VoidTarget::Observation(n) => format!("voiding observation #{n}"),
        VoidTarget::Citation(n) => format!("voiding citation #{n}"),
    };
    end_closure_if_decided(&store, &mut q, &did)?;
    let warnings = check(&store, &q, None)?;
    write_question(&q)?;
    Ok(warnings)
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
    /// Non-blocking validation warnings about the question as written.
    pub warnings: Vec<String>,
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
    let root = &evidence::absolute(root)?;
    let _lock = lock(root)?;
    let store = load_unlocked(root)?;
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
            warnings: Vec::new(),
        });
    }
    q.citations.push(new);
    let did = format!("adding citation #{}", q.citations.len());
    end_closure_if_decided(&store, &mut q, &did)?;
    let warnings = check(&store, &q, None)?;
    write_question(&q)?;
    Ok(Cited {
        number: q.citations.len(),
        existing: false,
        outcome: cite_outcome(&store.with(&q), &q),
        warnings,
    })
}

/// The question's disposition after a `cite`, plus every active citation
/// that does not count: one stale member keeps the whole support set from
/// holding, so the caller learns which to review and void.
fn cite_outcome(store: &Store, q: &Question) -> Vec<String> {
    let mut lines = vec![format!("question {} is {}", q.id, store.disposition(q))];
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
pub(super) fn now_utc() -> Result<Datetime> {
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
    observe_in(root, id, new, Mode::Write)
}

/// What `observe` would do, with every check and warning it makes, writing
/// nothing: a capture script can see the resulting disposition (a stray
/// conflict, say) before the observation becomes a record that must be voided.
pub fn observe_dry_run(root: &Path, id: &str, new: &NewObservation) -> Result<Observed> {
    observe_in(root, id, new, Mode::DryRun)
}

fn observe_in(root: &Path, id: &str, new: &NewObservation, mode: Mode) -> Result<Observed> {
    // A dry run is a read: like the read verbs it loads under the shared
    // lock (so it works on a read-only checkout), and what it reports may be
    // overtaken by a writer once the load returns, as any read's may.
    let root = &evidence::absolute(root)?;
    let (_lock, store) = match mode {
        Mode::Write => (Some(lock(root)?), load_unlocked(root)?),
        Mode::DryRun => (None, load(root)?),
    };
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
    // not record the hash file name as the human name. Which human name it
    // gets is decided below, once the capture key is resolved.
    let by_store_path = matches!(
        (&fixture_sha, &passed_name),
        (Some(sha), Some(name)) if *name == format!("{sha}.pine")
    );
    let fixture_name = if by_store_path { None } else { passed_name };
    let mut inspector = Inspector::new();

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
        // Read below: a keyed rerun matches evidence by location and digest,
        // a new observation records it.
        evidence: Vec::new(),
        errors: new.errors.clone(),
        warnings: new.warnings.clone(),
        candidates: decide_candidates(
            &new.candidates,
            &new.selected,
            &new.consistent,
            &new.refuted,
        )?,
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
        // Amendment history is not part of the payload. An amended field
        // matches any value it has held, so the landing script that wrote
        // the original stays idempotent after the correction; a superseded
        // value is reported, since a script still emitting it is likely the
        // template for the next landing.
        let mut given = observation.clone();
        // A rerun passing the stored copy of its own fixture carries the
        // name that observation recorded; any other name is compared as
        // given, so two files with identical bytes still differ.
        if by_store_path && stored.fixture == given.fixture {
            given.fixture_name.clone_from(&stored.fixture_name);
        }
        let mut warnings = Vec::new();
        for field in AmendField::ALL {
            if let (Some(held), Some(value)) = (stored.field(field), given.field_mut(field))
                && accept_held(value, held, field, &stored.amendments)
            {
                warnings.push(superseded(field, &format!("observation #{}", i + 1)));
            }
        }
        // Evidence is compared on its own: by location (a former one names
        // the entry, since a landing script may predate a move) and by the
        // digest of whatever file is there now.
        let (mut diff, moved) =
            evidence::rerun_differences(root, &stored.evidence, &new.evidence, &mut inspector)?;
        warnings.extend(moved);
        given.evidence.clone_from(&stored.evidence);
        let current = Observation {
            amendments: Vec::new(),
            ..stored.clone()
        };
        diff.splice(0..0, field_diff(&current, &given)?);
        if !diff.is_empty() {
            bail!(
                "key `{key}` belongs to observation #{} of question {}, which differs:\n  {}",
                i + 1,
                q.id,
                diff.join("\n  ")
            );
        }
        inspector.unchanged()?;
        let now = store.disposition(existing);
        return Ok(Observed {
            id: q.id.clone(),
            number: i + 1,
            existing: true,
            warnings,
            was: now.clone(),
            now,
        });
    }
    let mut observation = observation;
    observation.evidence = evidence::capture(root, &new.evidence, &mut inspector)?;
    if by_store_path && let Some(sha) = &fixture_sha {
        observation.fixture_name = known_fixture_name(store.questions(), sha);
    }
    q.observations.push(observation);
    let did = format!("adding observation #{}", q.observations.len());
    end_closure_if_decided(&store, &mut q, &did)?;
    let mut warnings = check(&store, &q, fixture_sha.as_deref())?;
    let (was, now) = (store.disposition(existing), store.with(&q).disposition(&q));
    warnings.extend(contradiction(&q, q.observations.len(), (&was, &now), mode));

    inspector.unchanged()?;
    if mode == Mode::Write {
        if let (Some(sha), Some(bytes)) = (&fixture_sha, &fixture_bytes) {
            store_fixture(root, sha, bytes)?;
        }
        write_question(&q)?;
    }
    Ok(Observed {
        id: q.id.clone(),
        number: q.observations.len(),
        existing: false,
        warnings,
        was,
        now,
    })
}

/// For a keyed rerun: a given value of amendable `field` that the field has
/// held at some point (its original, or any later amendment) is taken as the
/// current value, so amending never breaks the script that landed the record.
/// The stored value stands; the rerun writes nothing either way. Returns
/// whether the given value was a superseded one.
fn accept_held(
    given: &mut Option<String>,
    current: &Option<String>,
    field: AmendField,
    amendments: &[Amendment],
) -> bool {
    if given == current {
        return false;
    }
    let held = amendments
        .iter()
        .filter(|a| a.field == field)
        .any(|a| a.was == *given || a.now == *given);
    if held {
        given.clone_from(current);
    }
    held
}

/// The note for a keyed rerun that gave a superseded value of `field`.
fn superseded(field: AmendField, of: &str) -> String {
    format!(
        "the given `{}` is a superseded value of {of}; the amended value stands. \
         Update the script that emits it.",
        field.as_str()
    )
}

/// The warning for observation `number` of `after`, just appended, that
/// contradicts other counting observations of the same source strength,
/// naming each disagreement axis (`amend_contradictions` is the amend
/// counterpart). Mixed outcomes or codes
/// under one question usually mean fixtures testing different claims (a
/// control filed under the question it controls, most often), which belong
/// to separate questions; at the top strength they make the question a
/// conflict. Weaker-vs-stronger disagreement is not warned about: it is the
/// expected endpoint gap, and `show` annotates it.
fn contradiction(
    after: &Question,
    number: usize,
    (was, now): (&Disposition, &Disposition),
    mode: Mode,
) -> Option<String> {
    let new = after.observations.get(number.checked_sub(1)?)?;
    if !new.counts() {
        return None;
    }
    let us = format!("#{number}");
    let against: Vec<String> = after
        .observations
        .iter()
        .enumerate()
        .filter(|(i, o)| {
            i + 1 != number && o.counts() && o.source.strength() == new.source.strength()
        })
        .filter_map(|(i, o)| {
            let them = format!("#{}", i + 1);
            let axes: Vec<String> = new
                .disagreements(o, after.rules())
                .iter()
                .map(|d| d.describe(&us, &them))
                .collect();
            (!axes.is_empty()).then(|| {
                format!(
                    "{them} {} {} [{}]",
                    o.source.as_str(),
                    o.verdict_summary(after.kind),
                    axes.join("; ")
                )
            })
        })
        .collect();
    if against.is_empty() {
        return None;
    }
    // `was` / `now` are the disposition `show` prints, not the
    // measurement-only status: a documented or inferred question need not
    // end up a conflict.
    let tier = if new.decides() {
        ""
    } else {
        "weaker source only, "
    };
    let id = &after.id;
    let status = match (mode, was == now) {
        (Mode::Write, true) => format!("{tier}question {id} stays {now}"),
        (Mode::Write, false) => format!("{tier}question {id} is now {now} (was {was})"),
        (Mode::DryRun, true) => format!("{tier}question {id} would stay {now}"),
        (Mode::DryRun, false) => format!("{tier}question {id} would be {now} (is {was})"),
    };
    let advice = match mode {
        Mode::Write => format!(
            "`void --observation {number}` and observe it under its own question. If one \
             was mis-recorded, void that one."
        ),
        Mode::DryRun => "observe it under its own question instead. If an earlier one \
             was mis-recorded, void it; if this one is, correct it before landing."
            .to_string(),
    };
    Some(format!(
        "observation #{number} {} {} contradicts {}: {status}. If the fixtures test \
         different claims (a control is the usual case: it is its own question), they \
         belong to separate questions: {advice}",
        new.source.as_str(),
        new.verdict_summary(after.kind),
        against.join(", ")
    ))
}

/// Put a fixture into the content-addressed store and make it durable: file,
/// `fixtures/` and the root's entry for `fixtures/`. An existing file is kept
/// only if it really hashes to its name; anything else (e.g. a truncated
/// leftover from an interrupted write) is replaced. A kept file is synced
/// too: correct bytes do not prove an earlier writer's sync succeeded.
fn store_fixture(root: &Path, sha: &str, bytes: &[u8]) -> Result<()> {
    let path = fixture_path(root, sha);
    let dir = path.parent().unwrap_or(root);
    if std::fs::read(&path).is_ok_and(|existing| sha256_hex(&existing) == sha) {
        sync_file(&path)?;
        sync_dir(dir)?;
    } else {
        std::fs::create_dir_all(dir).with_context(|| format!("creating {}", dir.display()))?;
        write_atomic(
            &path,
            bytes,
            "the fixture is stored, but no record references it yet",
        )?;
    }
    sync_dir(root)
}

/// Write the question record `q` durably.
pub(super) fn write_question(q: &Question) -> Result<()> {
    let text = toml::to_string(q).context("serializing question")?;
    ensure_dir(&validate::records_dir(&q.root))?;
    write_atomic(
        &question_path(&q.root, &q.id),
        text.as_bytes(),
        "the record is in place, so check it with `po verdict show` before retrying an unkeyed write",
    )
}

/// Write `bytes` to a dot-prefixed temp file beside `path` (unique per
/// process), sync it, rename it into place and sync the directory, so an
/// interrupted write or a crash never leaves a partial file under its real
/// name, and a success means the file survives a crash. The loader ignores
/// dot-files. Callers hold the directory lock, so concurrent writers
/// serialize rather than race. `landed` explains the state when only the
/// directory sync after the rename fails.
fn write_atomic(path: &Path, bytes: &[u8], landed: &str) -> Result<()> {
    use std::io::Write as _;
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    let tmp = path.with_file_name(format!(".{name}.{}.tmp", std::process::id()));
    let mut file =
        std::fs::File::create(&tmp).with_context(|| format!("writing {}", tmp.display()))?;
    file.write_all(bytes)
        .and_then(|()| file.sync_all())
        .with_context(|| format!("writing {}", tmp.display()))?;
    drop(file);
    std::fs::rename(&tmp, path).with_context(|| format!("replacing {}", path.display()))?;
    let dir = path.parent().unwrap_or(Path::new("."));
    sync_dir(dir).with_context(|| {
        format!(
            "{} was replaced but may not survive a crash: {landed}",
            path.display()
        )
    })
}

/// Create directory `dir` if it is missing, and make its entry in the parent
/// durable, so a write that reports success cannot lose the directory it
/// wrote into. The parent must exist (the repository root, or `verdicts/`).
/// The parent is synced even when the directory already exists: whoever
/// created it may not have synced yet (or failed to), and this write's
/// success must not rest on that.
fn ensure_dir(dir: &Path) -> Result<()> {
    if !dir.is_dir() {
        match std::fs::create_dir(dir) {
            Ok(()) => {}
            // Another writer won the race to create it.
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists && dir.is_dir() => {}
            Err(e) => return Err(e).with_context(|| format!("creating {}", dir.display())),
        }
    }
    sync_dir(dir.parent().unwrap_or(Path::new(".")))
}

fn sync_file(path: &Path) -> Result<()> {
    std::fs::File::open(path)
        .and_then(|f| f.sync_all())
        .with_context(|| format!("syncing {}", path.display()))
}

/// Make the entries of directory `dir` durable. Unix only: elsewhere a
/// directory cannot be opened for syncing, and rename durability is the
/// filesystem's.
fn sync_dir(dir: &Path) -> Result<()> {
    #[cfg(unix)]
    {
        sync_file(dir)
    }
    #[cfg(not(unix))]
    {
        let _ = dir;
        Ok(())
    }
}

/// Validate `q` as it would be written into `store` (new or replacing its
/// namesake), including the relation cycles and key clashes only the whole
/// store can show;
/// errors refuse the write.
fn check(store: &Store, q: &Question, pending_fixture: Option<&str>) -> Result<Vec<String>> {
    let after_store = store.with(q);
    let after = after_store.questions();
    let known_ids = validate::ids_in(after);
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

/// `check` for a write that changes several questions of one store at once:
/// each is validated against the store with all of them in place.
pub(super) fn check_many(store: &Store, changed: &[Question]) -> Result<()> {
    let after_store = store.with_all(changed);
    let after = after_store.questions();
    let mut report = Report::default();
    let known_ids = validate::ids_in(after);
    for q in changed {
        let cx = Context {
            known_ids: &known_ids,
            pending_fixture: None,
        };
        validate::question(q, &cx, &mut report);
    }
    report.errors.extend(validate::cycles(after));
    report.errors.extend(validate::duplicate_keys(after));
    if !report.errors.is_empty() {
        bail!("refusing to write:\n  {}", report.errors.join("\n  "));
    }
    Ok(())
}

/// The human name the store's observations agree on for fixture `sha`, if
/// they agree. Identical bytes may have been captured under several names;
/// picking one of them would attribute another capture's file, so a
/// disagreement records no name. Void observations do not vote: a withdrawn
/// record may carry a mis-recorded name.
fn known_fixture_name(questions: &[Question], sha: &str) -> Option<String> {
    let names: BTreeSet<&String> = questions
        .iter()
        .flat_map(|q| &q.observations)
        .filter(|o| o.void.is_none() && o.fixture.as_deref() == Some(sha))
        .filter_map(|o| o.fixture_name.as_ref())
        .collect();
    match names.into_iter().collect::<Vec<_>>().as_slice() {
        [name] => Some((*name).clone()),
        _ => None,
    }
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

/// Apply `--selected` / `--consistent` / `--refuted` to the declared
/// candidates (undecided otherwise). Each must name a declared candidate, and
/// each candidate takes at most one state.
fn decide_candidates(
    declared: &[Candidate],
    selected: &[String],
    consistent: &[String],
    refuted: &[String],
) -> Result<Vec<Candidate>> {
    let given = [
        (CandidateState::Selected, selected),
        (CandidateState::Consistent, consistent),
        (CandidateState::Refuted, refuted),
    ];
    for name in given.iter().flat_map(|(_, names)| names.iter()) {
        if !declared.iter().any(|c| &c.name == name) {
            bail!("`{name}` is not a declared candidate (declare it with --candidate)");
        }
    }
    declared
        .iter()
        .map(|c| {
            let states: Vec<CandidateState> = given
                .iter()
                .filter(|(_, names)| names.contains(&c.name))
                .map(|(state, _)| *state)
                .collect();
            let state = match states[..] {
                [] => CandidateState::Undecided,
                [one] => one,
                [a, b, ..] => bail!(
                    "candidate `{}` cannot be both {} and {}",
                    c.name,
                    a.as_str(),
                    b.as_str()
                ),
            };
            Ok(Candidate { state, ..c.clone() })
        })
        .collect()
}

/// `target` relative to `base` (both absolute and normalized), `/`-separated.
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
    fn relative_path_below_the_evidence_dir() {
        let base = Path::new("/repo/verdicts/evidence");
        assert_eq!(
            relative_path(base, Path::new("/repo/verdicts/evidence/run26/export.json")),
            "run26/export.json"
        );
        assert_eq!(
            relative_path(base, Path::new("/repo/verdicts/evidence/x.json")),
            "x.json"
        );
    }
}
