// Where verdicts are cited: `po verdict citations`. A repository that cites
// verdict ids in code comments and docs keeps a rule: every cited id exists
// and is settled, or the citing line says which status it rests on. This
// module finds the citations (`scan`) and judges them against a store
// (`check`), so the caller needs no scanner and no reading of `show`'s text.
//
// A citation is an 8-hex lowercase id after the word `verdict` or `verdicts`
// (any case), in any text. A list continues onto the next line when a line
// ends in the keyword, in `and`, or in an id followed by a comma; a line with
// no word tokens (a bare `//` or `#`) keeps it open. Inside a list the status
// words `open`, `conflict` and `documented` do not end it, so
// `verdicts a (open), b` cites both; any other word does.
//
// A cited id passes when it is settled (`inferred (settled)` and `settled,
// negative` included). A documented, open or conflict one passes only when
// that exact word appears, lowercase and whole, on its line after the id and
// before the next id: a stale marker fails. A retired one always fails, and
// an id not in the store fails.

use anyhow::{Context as _, Result, bail};
use std::path::Path;

use super::{Status, Store};

/// One cited id.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cited {
    pub id: String,
    /// 1-based line.
    pub line: usize,
    /// The text after the id on its line, up to the next id on that line:
    /// where a status marker for it may stand.
    pub tail: String,
}

/// A citation that breaks the rule.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Failure {
    pub line: usize,
    pub id: String,
    /// What is wrong and what to do, as one phrase after `verdict <id>`.
    pub why: String,
}

/// Runs of ASCII letters and digits, as byte ranges.
fn tokens(line: &str) -> Vec<(usize, usize)> {
    let mut out = Vec::new();
    let mut start = None;
    for (index, c) in line.char_indices() {
        if c.is_ascii_alphanumeric() {
            start.get_or_insert(index);
        } else if let Some(begin) = start.take() {
            out.push((begin, index));
        }
    }
    if let Some(begin) = start {
        out.push((begin, line.len()));
    }
    out
}

fn is_id(token: &str) -> bool {
    token.len() == 8
        && token
            .bytes()
            .all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f'))
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Last {
    Keyword,
    Id,
    And,
}

/// Every citation in `source`, in order.
pub fn scan(source: &str) -> Vec<Cited> {
    let mut out = Vec::new();
    let mut in_list = false;
    let mut last = Last::Keyword;
    for (index, line) in source.lines().enumerate() {
        let toks = tokens(line);
        let mut found: Vec<(usize, usize)> = Vec::new();
        for &(start, end) in &toks {
            let token = &line[start..end];
            if token.eq_ignore_ascii_case("verdict") || token.eq_ignore_ascii_case("verdicts") {
                in_list = true;
                last = Last::Keyword;
            } else if !in_list {
                continue;
            } else if is_id(token) {
                found.push((start, end));
                last = Last::Id;
            } else if token == "and" {
                last = Last::And;
            } else if matches!(token, "open" | "conflict" | "documented") {
                // A marker beside an id must not end the list its siblings
                // follow in: `verdicts a (open), b`.
            } else {
                in_list = false;
            }
        }
        // A line with no tokens, such as a bare comment marker, keeps the list.
        if in_list && !toks.is_empty() {
            in_list = match last {
                Last::Keyword | Last::And => true,
                Last::Id => line.trim_end().ends_with(','),
            };
        }
        // A tail ends at the next id on the line, cited or not: a marker
        // past an uncited id (`verdict a; compare b (open)`) is b's.
        let id_starts: Vec<usize> = toks
            .iter()
            .filter(|&&(s, e)| is_id(&line[s..e]))
            .map(|&(s, _)| s)
            .collect();
        for &(start, end) in &found {
            let stop = id_starts
                .iter()
                .copied()
                .find(|&s| s >= end)
                .unwrap_or(line.len());
            out.push(Cited {
                id: line[start..end].to_string(),
                line: index + 1,
                tail: line[end..stop].to_string(),
            });
        }
    }
    out
}

/// The citations in `cited` that break the rule against `store`.
pub fn check(store: &Store, cited: &[Cited]) -> Vec<Failure> {
    let mut failures = Vec::new();
    for c in cited {
        let fail = |why: String| Failure {
            line: c.line,
            id: c.id.clone(),
            why,
        };
        let Some(q) = store.get(&c.id) else {
            failures.push(fail("is not in the store".to_string()));
            continue;
        };
        let Some(status) = store.resolve(q).status() else {
            let now: Vec<String> = store
                .current_successors(q)
                .into_iter()
                .map(|s| s.id.clone())
                .collect();
            failures.push(fail(if now.is_empty() {
                "is retired and nothing replaced it; drop the citation".to_string()
            } else {
                format!("is retired; cite {} instead", now.join(", "))
            }));
            continue;
        };
        if status == Status::Settled {
            continue;
        }
        let word = status.as_str();
        let marked = tokens(&c.tail)
            .iter()
            .any(|&(start, end)| &c.tail[start..end] == word);
        if !marked {
            failures.push(fail(format!(
                "is {word}; write `{word}` after the id on this line, or cite a settled verdict"
            )));
        }
    }
    failures
}

/// The files git sees under `pathspecs` (tracked, or untracked and not
/// ignored), spelled from the repository root `repo`, never inside
/// `verdicts/`, whose ids are data. Pathspecs are git's, relative to `cwd`,
/// so `:(exclude)...` and `:(glob)...` work; none means the whole
/// repository. A pathspec that matches nothing is an error (a misspelled
/// area must not drop out of the check silently), and so is git's top level
/// not being `repo` (`GIT_DIR` / `GIT_WORK_TREE`, as hooks set them, would
/// otherwise list one tree while po judges against another's store). A git
/// failure is an error, never an empty listing.
pub fn files(repo: &Path, cwd: &Path, pathspecs: &[String]) -> Result<Vec<String>> {
    let top = std::process::Command::new("git")
        .arg("-C")
        .arg(cwd)
        .args(["rev-parse", "--show-toplevel"])
        .output()
        .context("running git rev-parse")?;
    if !top.status.success() {
        bail!(
            "git rev-parse failed: {}",
            String::from_utf8_lossy(&top.stderr).trim()
        );
    }
    let top = std::path::PathBuf::from(String::from_utf8_lossy(&top.stdout).trim());
    let real = |p: &Path| p.canonicalize().ok();
    if real(&top).is_none() || real(&top) != real(repo) {
        bail!(
            "git's top level is {}, but the verdicts store is in {}: \
             is GIT_DIR or GIT_WORK_TREE set?",
            top.display(),
            repo.display()
        );
    }
    // None: the whole repository (`:/`, its top), not just `cwd`'s subtree.
    let whole = [":/".to_string()];
    let scope = if pathspecs.is_empty() {
        &whole[..]
    } else {
        pathspecs
    };
    let output = std::process::Command::new("git")
        .arg("-C")
        .arg(cwd)
        .args([
            "ls-files",
            "-z",
            "--full-name",
            "--error-unmatch",
            "--cached",
            "--others",
            "--exclude-standard",
            "--",
        ])
        .args(scope)
        .output()
        .context("running git ls-files")?;
    if !output.status.success() {
        bail!(
            "git ls-files failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    let mut out: Vec<String> = String::from_utf8(output.stdout)
        .context("git listed a path that is not UTF-8")?
        .split('\0')
        .filter(|p| !p.is_empty() && !p.starts_with("verdicts/"))
        .map(str::to_string)
        .collect();
    out.sort();
    out.dedup();
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ids(source: &str) -> Vec<(String, usize)> {
        scan(source).into_iter().map(|c| (c.id, c.line)).collect()
    }

    fn one(id: &str, line: usize) -> Vec<(String, usize)> {
        vec![(id.to_string(), line)]
    }

    #[test]
    fn an_id_after_the_keyword_is_cited_in_any_case() {
        assert_eq!(
            ids("// TV refuses it. verdict 12e3f8ef\n"),
            one("12e3f8ef", 1)
        );
        assert_eq!(ids("Verdict 12e3f8ef holds.\n"), one("12e3f8ef", 1));
    }

    #[test]
    fn lists_and_their_continuations() {
        let all: Vec<String> = scan("// verdicts 0a1df0ea, d8a565c9 and 0e5acd94.\n")
            .into_iter()
            .map(|c| c.id)
            .collect();
        assert_eq!(all, ["0a1df0ea", "d8a565c9", "0e5acd94"]);
        // A trailing comma or keyword carries the list to the next line.
        assert_eq!(
            ids("// verdicts 0a1df0ea,\n// d8a565c9.\n"),
            vec![("0a1df0ea".to_string(), 1), ("d8a565c9".to_string(), 2)]
        );
        assert_eq!(
            ids("/// the control of verdict\n/// 5e7fb86a.\n"),
            one("5e7fb86a", 2)
        );
        // A bare comment line keeps it open.
        assert_eq!(ids("// verdicts 0a1df0ea,\n//\n// d8a565c9\n").len(), 2);
        // Without the comma, or after another word, it ends.
        assert_eq!(
            ids("// verdict 0a1df0ea\n// d8a565c9 is something else\n").len(),
            1
        );
        assert_eq!(ids("// verdict 0a1df0ea, but not d8a565c9\n").len(), 1);
    }

    #[test]
    fn what_is_not_a_citation() {
        assert!(scan("// see 0a1df0ea in the notes\n").is_empty());
        assert!(scan("// verdict 0a1df0ea1 and 0a1df0\n").is_empty());
        assert!(scan("// verdict 0A1DF0EA\n").is_empty());
    }

    #[test]
    fn a_marker_word_keeps_the_list_and_tails_stop_at_the_next_id() {
        let found = scan("// verdicts 0a1df0ea (open), d8a565c9\n");
        assert_eq!(found.len(), 2);
        assert_eq!(found[0].tail, " (open), ");
        assert_eq!(found[1].tail, "");
        // An uncited id ends the tail too: the marker is not the first id's.
        let found = scan("// verdict 12e3f8ef; compare 8a141aa5 (open)\n");
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].tail, "; compare ");
    }

    /// No pathspec means the whole repository, from any directory: git
    /// alone would list only the working directory's subtree.
    #[test]
    fn files_default_to_the_whole_repository_from_a_subdirectory() {
        let repo = Path::new(env!("CARGO_MANIFEST_DIR"));
        let listed = files(repo, &repo.join("src/verdict"), &[]).expect("git lists");
        assert!(listed.iter().any(|p| p == "docs/verdict.md"), "{listed:?}");
        assert!(listed.iter().any(|p| p == "src/verdict/cited.rs"));
        // A pathspec stays relative to the working directory.
        let here = files(repo, &repo.join("src"), &["verdict".to_string()]).expect("lists");
        assert!(
            here.iter().all(|p| p.starts_with("src/verdict/")),
            "{here:?}"
        );
        // One that matches nothing fails.
        assert!(files(repo, repo, &["no-such-area".to_string()]).is_err());
    }

    /// The judgement against the real run 26 store fixtures lives in the
    /// verdict end-to-end tests, which can build a store.
    #[test]
    fn tokens_are_ascii_alphanumeric_runs() {
        let line = "a(open), b-c";
        let words: Vec<&str> = tokens(line).iter().map(|&(s, e)| &line[s..e]).collect();
        assert_eq!(words, ["a", "open", "b", "c"]);
    }
}
