// Strict read side: load every question under the named records directories,
// validate all of them, and refuse to return a store if any record is
// invalid. A successful load is therefore a clean bill of health for every
// directory passed, which is what lets piners use any read verb as a CI gate.
// For that to hold, nothing in a records directory may be silently skipped:
// every entry must be a question file, the fixture store, a markdown note, or
// a dot-file (in-flight temp files).

use anyhow::{Context as _, Result, bail};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use super::validate::{self, Context, Report};
use super::{Kind, Question, Resolved, Status};

/// Every question loaded from the given records directories, validated.
#[derive(Debug)]
pub struct Store {
    questions: Vec<Question>,
}

/// `po verdict list` filters. Every set field must match.
#[derive(Debug, Default, Clone)]
pub struct ListFilter<'a> {
    /// Exact identifier, case-insensitive.
    pub identifier: Option<&'a str>,
    /// Exact diagnostic code, case-insensitive.
    pub code: Option<&'a str>,
    /// Matches the resolved status of non-retired questions, so an inferred
    /// question whose premises are all settled matches `Settled`, and a
    /// documented one matches `Documented`, never `Open` or `Settled`.
    /// Retired questions have no status and never match.
    pub status: Option<Status>,
    /// Only inferred questions.
    pub inferred: bool,
    /// Only retired questions.
    pub retired: bool,
    pub kind: Option<Kind>,
}

/// Load and validate every question under each root. Roots are never
/// defaulted or discovered; each must be an existing directory. Any invalid
/// record or unexpected entry fails the whole load, listing every problem.
/// Warnings (e.g. empty identifiers) are a write-time concern and are not
/// reported here, so reads over a deliberate corpus stay quiet.
pub fn load(roots: &[PathBuf]) -> Result<Store> {
    if roots.is_empty() {
        bail!("no records directory given (pass --records <DIR>)");
    }
    let mut questions: Vec<Question> = Vec::new();
    let mut problems = Vec::new();
    // The same directory named twice (or by two spellings) is one root, not
    // a source of duplicate ids.
    let mut canonical_roots = BTreeSet::new();
    for root in roots {
        if !root.is_dir() {
            bail!("records directory {} does not exist", root.display());
        }
        let canonical = root
            .canonicalize()
            .with_context(|| format!("resolving {}", root.display()))?;
        if canonical_roots.insert(canonical) {
            read_root(root, &mut questions, &mut problems)?;
        }
    }

    let mut seen = BTreeSet::new();
    for q in &questions {
        if !seen.insert(q.id.clone()) {
            problems.push(format!(
                "question id {} appears in more than one records directory",
                q.id
            ));
        }
    }

    // Question relations resolve within the question's own directory only,
    // so a record validates the same whether its directory is loaded alone
    // (as the write path does) or alongside others.
    let mut ids_by_root: BTreeMap<&Path, BTreeMap<String, bool>> = BTreeMap::new();
    for q in &questions {
        ids_by_root
            .entry(q.root.as_path())
            .or_insert_with(|| validate::ids_in(&questions, &q.root));
    }
    let mut report = Report::default();
    for q in &questions {
        let cx = Context {
            known_ids: &ids_by_root[q.root.as_path()],
            pending_fixture: None,
        };
        validate::question(q, &cx, &mut report);
    }
    problems.extend(report.errors);
    problems.extend(validate::cycles(&questions));
    problems.extend(validate::duplicate_keys(&questions));
    if !problems.is_empty() {
        bail!(
            "{} invalid record problem(s):\n  {}",
            problems.len(),
            problems.join("\n  ")
        );
    }
    questions.sort_by(|a, b| a.id.cmp(&b.id));
    Ok(Store { questions })
}

/// Parse every question file under `root` and vet every other entry.
/// Parse failures and unexpected entries are collected as problems; I/O
/// failures abort.
fn read_root(root: &Path, questions: &mut Vec<Question>, problems: &mut Vec<String>) -> Result<()> {
    let mut paths = Vec::new();
    for path in dir_entries(root)? {
        let name = file_name(&path);
        if name.starts_with('.') || (name.ends_with(".md") && path.is_file()) {
            continue;
        }
        if name == "fixtures" && path.is_dir() {
            check_fixture_store(&path, problems)?;
            continue;
        }
        match name.strip_suffix(".toml") {
            Some(stem) if validate::is_id(stem) && path.is_file() => paths.push(path),
            _ => problems.push(format!(
                "{}: not a question file (`<8-hex id>.toml`), the `fixtures` store, or a markdown note",
                path.display()
            )),
        }
    }
    paths.sort();
    for path in paths {
        let stem = file_name(&path).trim_end_matches(".toml").to_string();
        let text = std::fs::read_to_string(&path)
            .with_context(|| format!("reading {}", path.display()))?;
        match toml::from_str::<Question>(&text) {
            Ok(mut q) => {
                q.id = stem;
                q.root = root.to_path_buf();
                questions.push(q);
            }
            Err(e) => problems.push(format!("{}: {e}", path.display())),
        }
    }
    Ok(())
}

/// Every stored fixture must be `<sha256>.pine` and hash to its name,
/// including ones no observation references yet (a leftover from an
/// interrupted write must not lurk until something points at it).
fn check_fixture_store(dir: &Path, problems: &mut Vec<String>) -> Result<()> {
    for path in dir_entries(dir)? {
        let name = file_name(&path);
        if name.starts_with('.') {
            continue;
        }
        let Some(sha) = name.strip_suffix(".pine").filter(|_| path.is_file()) else {
            problems.push(format!("{}: not a `<sha256>.pine` fixture", path.display()));
            continue;
        };
        let bytes = std::fs::read(&path).with_context(|| format!("reading {}", path.display()))?;
        let actual = validate::sha256_hex(&bytes);
        if actual != sha {
            problems.push(format!(
                "fixture {} hashes to {actual}, not its name",
                path.display()
            ));
        }
    }
    Ok(())
}

fn dir_entries(dir: &Path) -> Result<Vec<PathBuf>> {
    let mut out = Vec::new();
    for entry in std::fs::read_dir(dir).with_context(|| format!("reading {}", dir.display()))? {
        out.push(
            entry
                .with_context(|| format!("reading {}", dir.display()))?
                .path(),
        );
    }
    out.sort();
    Ok(out)
}

fn file_name(path: &Path) -> String {
    path.file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default()
}

impl Store {
    /// Every question, sorted by id.
    pub fn questions(&self) -> &[Question] {
        &self.questions
    }

    /// This store with `q` added, or replacing its namesake: the state a
    /// write is about to produce.
    pub(super) fn with(&self, q: &Question) -> Store {
        let mut questions: Vec<Question> = self
            .questions
            .iter()
            .filter(|x| x.id != q.id)
            .cloned()
            .collect();
        questions.push(q.clone());
        questions.sort_by(|a, b| a.id.cmp(&b.id));
        Store { questions }
    }

    pub fn get(&self, id: &str) -> Option<&Question> {
        let id = id.trim().to_ascii_lowercase();
        self.questions.iter().find(|q| q.id == id)
    }

    /// Questions matching every set filter field, in id order.
    pub fn list(&self, filter: &ListFilter<'_>) -> Vec<&Question> {
        self.questions
            .iter()
            .filter(|q| {
                filter
                    .identifier
                    .is_none_or(|i| q.identifiers.iter().any(|x| x.eq_ignore_ascii_case(i)))
            })
            .filter(|q| {
                filter
                    .code
                    .is_none_or(|c| q.codes().iter().any(|x| x.eq_ignore_ascii_case(c)))
            })
            .filter(|q| {
                filter
                    .status
                    .is_none_or(|s| self.resolve(q).status() == Some(s))
            })
            .filter(|q| !filter.inferred || q.is_inferred())
            .filter(|q| !filter.retired || q.is_retired())
            .filter(|q| filter.kind.is_none_or(|k| q.kind == k))
            .collect()
    }

    /// The disposition to show for `q`. A retired question has no status. A
    /// measured one is documented when nothing measured decides it and its
    /// citations hold. An inferred one is settled when every premise
    /// resolves settled, documented via its worst premise when that is
    /// documented, and otherwise open via its worst premise (conflict < open
    /// < documented, first in `basis` order on a tie): a conflicting premise
    /// blocks the inference but is not a conflict in it, since nobody
    /// measured it. Premises may
    /// themselves be inferred; `load` has refused cycles, and validation
    /// keeps retired questions out of an active basis.
    pub fn resolve(&self, q: &Question) -> Resolved {
        if let Some(r) = &q.retired {
            return Resolved::Retired {
                replaced_by: r.replaced_by.clone(),
            };
        }
        if !q.is_inferred() {
            // A measurement that settles or conflicts wins over the manual;
            // citations only lift what nothing measured decides.
            return match q.own_status() {
                Status::Open if q.citations_hold() => Resolved::Documented,
                status => Resolved::Measured(status),
            };
        }
        let mut worst: Option<(Status, &str)> = None;
        for id in &q.basis {
            let status = self
                .get(id)
                .and_then(|p| self.resolve(p).status())
                .unwrap_or(Status::Open);
            if worst.is_none_or(|(w, _)| status < w) {
                worst = Some((status, id));
            }
        }
        match worst {
            Some((Status::Settled, _)) | None => Resolved::Inferred {
                status: Status::Settled,
                via: None,
            },
            Some((Status::Documented, via)) => Resolved::Inferred {
                status: Status::Documented,
                via: Some(via.to_string()),
            },
            Some((_, via)) => Resolved::Inferred {
                status: Status::Open,
                via: Some(via.to_string()),
            },
        }
    }

    /// Where the investigation behind retired question `q` stands now: the
    /// non-retired questions reached through `replaced_by`, following chains
    /// through replacements that were retired in turn (A -> B -> C, D gives
    /// C, D). In first-reached order, each once; empty when nothing replaced
    /// it. `load` has refused replacement cycles.
    pub fn current_successors(&self, q: &Question) -> Vec<&Question> {
        let mut out: Vec<&Question> = Vec::new();
        let mut seen = BTreeSet::new();
        let mut queue: std::collections::VecDeque<&str> = Relation::ReplacedBy
            .of(q)
            .iter()
            .map(String::as_str)
            .collect();
        while let Some(id) = queue.pop_front() {
            if !seen.insert(id) {
                continue;
            }
            let Some(next) = self.get(id) else { continue };
            if next.is_retired() {
                queue.extend(Relation::ReplacedBy.of(next).iter().map(String::as_str));
            } else {
                out.push(next);
            }
        }
        out
    }

    /// Questions that name `id` through `relation`, in id order: the reverse
    /// links `show` prints (follow-ups, premise users, replaces).
    pub fn referrers(&self, id: &str, relation: Relation) -> Vec<&Question> {
        self.questions
            .iter()
            .filter(|q| relation.of(q).iter().any(|x| x == id))
            .collect()
    }
}

/// One of the three question-to-question relations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Relation {
    FollowUpTo,
    Basis,
    ReplacedBy,
}

impl Relation {
    /// The ids `q` names through this relation.
    pub fn of(self, q: &Question) -> &[String] {
        match self {
            Relation::FollowUpTo => &q.follow_up_to,
            Relation::Basis => &q.basis,
            Relation::ReplacedBy => q.retired.as_ref().map_or(&[], |r| &r.replaced_by),
        }
    }

    /// The TOML field, as validation messages name it.
    pub fn field(self) -> &'static str {
        match self {
            Relation::FollowUpTo => "follow_up_to",
            Relation::Basis => "basis",
            Relation::ReplacedBy => "retired.replaced_by",
        }
    }
}
