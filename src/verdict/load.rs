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
    /// Matches the resolved status, so a derived question whose sources are
    /// all settled matches `Settled`.
    pub status: Option<Status>,
    /// Only derived questions.
    pub derived: bool,
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

    // `derived_from` resolves within the question's own directory only, so a
    // record validates the same whether its directory is loaded alone (as
    // the write path does) or alongside others.
    let mut ids_by_root: BTreeMap<&Path, BTreeSet<String>> = BTreeMap::new();
    for q in &questions {
        ids_by_root
            .entry(q.root.as_path())
            .or_default()
            .insert(q.id.clone());
    }
    let mut report = Report::default();
    let empty = BTreeSet::new();
    for q in &questions {
        let cx = Context {
            known_ids: ids_by_root.get(q.root.as_path()).unwrap_or(&empty),
            pending_fixture: None,
        };
        validate::question(q, &cx, &mut report);
    }
    problems.extend(report.errors);
    problems.extend(derivation_cycles(&questions));
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

/// A question may not be derived, directly or transitively, from itself.
/// Only hand edits can create a cycle (`add` needs its sources to exist
/// first), so report each question found on one.
fn derivation_cycles(questions: &[Question]) -> Vec<String> {
    let edges: BTreeMap<&str, &[String]> = questions
        .iter()
        .map(|q| (q.id.as_str(), q.derived_from.as_slice()))
        .collect();
    let mut out = Vec::new();
    for q in questions {
        let mut stack: Vec<&str> = q.derived_from.iter().map(String::as_str).collect();
        let mut seen = BTreeSet::new();
        while let Some(id) = stack.pop() {
            if id == q.id {
                out.push(format!(
                    "question {} is derived from itself through `derived_from`",
                    q.id
                ));
                break;
            }
            if seen.insert(id) {
                stack.extend(
                    edges
                        .get(id)
                        .into_iter()
                        .flat_map(|d| d.iter().map(String::as_str)),
                );
            }
        }
    }
    out
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
            .filter(|q| filter.status.is_none_or(|s| self.resolve(q).status == s))
            .filter(|q| !filter.derived || q.is_derived())
            .filter(|q| filter.kind.is_none_or(|k| q.kind == k))
            .collect()
    }

    /// The status to show for `q`: its own, or for a derived question the
    /// worst resolved status among its sources (conflict < open < settled),
    /// naming the source responsible unless every source is settled. Sources
    /// may themselves be derived; `load` has refused cycles.
    pub fn resolve(&self, q: &Question) -> Resolved {
        if !q.is_derived() {
            return Resolved {
                status: q.own_status(),
                derived: false,
                via: None,
            };
        }
        let mut worst: Option<(Status, &str)> = None;
        for id in &q.derived_from {
            let status = self
                .get(id)
                .map_or(Status::Open, |s| self.resolve(s).status);
            if worst.is_none_or(|(w, _)| status < w) {
                worst = Some((status, id));
            }
        }
        let (status, via) = worst.unwrap_or((Status::Open, ""));
        Resolved {
            status,
            derived: true,
            via: (status != Status::Settled).then(|| via.to_string()),
        }
    }
}
