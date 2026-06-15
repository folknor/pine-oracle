// Authored TA-recipe surface, backing `po recipe`. Unlike `manual` (scraped from
// TradingView) and `behavior` (pine-data exports), this corpus is hand-authored
// in-repo: technical-analysis instruments that have no TradingView builtin and
// no manual page (custom moving averages, composite indicators, candlestick
// patterns, market-structure concepts). Each entry is a markdown file under
// `assets/recipes/<category>/<name>.md` carrying `title` + optional `aliases`
// frontmatter and a freeform body (prose + a Pine v6 code fence). The tree is
// embedded with `include_dir` and rendered through `crate::render`, exactly like
// the manual - the difference is provenance, not shape.
//
// This is the "how do I build X in Pine" half of the oracle: where `po lookup`
// answers "what is `ta.sma`", `po recipe` answers "how do I compute an HMA".

use include_dir::{Dir, include_dir};
use std::collections::{BTreeMap, HashMap};
use std::sync::OnceLock;

static RECIPES: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/assets/recipes");

/// One authored TA recipe: an instrument with no TradingView builtin, described
/// as prose + a Pine v6 snippet.
#[derive(Debug, Clone)]
pub struct Recipe {
    /// Lookup key: the file stem (e.g. `hma`). Lowercase by authoring
    /// convention; matched case-insensitively.
    pub name: String,
    /// Human title from frontmatter (e.g. `Hull Moving Average`).
    pub title: String,
    /// Category = the first path component under `assets/recipes`
    /// (e.g. `moving-average`, `candlestick`).
    pub category: String,
    /// Alternative lookup handles from frontmatter `aliases` (comma-separated).
    pub aliases: Vec<String>,
    /// Recipe markdown body (everything after the frontmatter), rendered via
    /// `crate::render`.
    pub body: String,
}

/// A category and how many recipes it holds - the `--category ?` catalog row.
#[derive(Debug, Clone)]
pub struct CategoryCount {
    pub category: String,
    pub count: usize,
}

/// All parsed recipes, in stable (category, name) order. Built once, cached.
pub fn recipes() -> &'static [Recipe] {
    static ALL: OnceLock<Vec<Recipe>> = OnceLock::new();
    ALL.get_or_init(build_recipes).as_slice()
}

/// Number of recipes in the corpus.
pub fn count() -> usize {
    recipes().len()
}

/// Categories present, each with its recipe count, in sorted order.
pub fn categories() -> Vec<CategoryCount> {
    let mut counts: BTreeMap<&str, usize> = BTreeMap::new();
    for r in recipes() {
        *counts.entry(r.category.as_str()).or_insert(0) += 1;
    }
    counts
        .into_iter()
        .map(|(category, count)| CategoryCount {
            category: category.to_string(),
            count,
        })
        .collect()
}

/// Exact recipe by name or alias (case-insensitive). The lookup table maps both
/// names and aliases to recipe indices; names win on a name/alias clash because
/// they are inserted last.
pub fn lookup(query: &str) -> Option<&'static Recipe> {
    let key = query.trim().to_ascii_lowercase();
    index().get(key.as_str()).map(|&i| &recipes()[i])
}

/// Recipes filtered by `category` (exact, case-insensitive) and `grep`
/// (substring over name / title / aliases / category, case-insensitive), in
/// stable order. Either filter may be `None`.
pub fn list(category: Option<&str>, grep: Option<&str>) -> Vec<&'static Recipe> {
    let cat = category.map(|c| c.trim().to_ascii_lowercase());
    let needle = grep.map(|g| g.trim().to_ascii_lowercase());
    recipes()
        .iter()
        .filter(|r| match &cat {
            Some(c) => r.category.eq_ignore_ascii_case(c),
            None => true,
        })
        .filter(|r| match &needle {
            Some(n) => recipe_matches(r, n),
            None => true,
        })
        .collect()
}

/// Closest recipes to `query` for the lookup-miss "did you mean ...?" path. The
/// corpus is name-keyed and modest enough that a scored linear scan beats
/// standing up a second BM25 index: an exact name/alias hit ranks first, then
/// substring containment, then shared-token overlap. Zero-scoring recipes are
/// dropped; ties keep stable corpus order.
pub fn suggest(query: &str, limit: usize) -> Vec<&'static Recipe> {
    let q = query.trim().to_ascii_lowercase();
    if q.is_empty() {
        return Vec::new();
    }
    let q_tokens: Vec<&str> = q.split_whitespace().collect();
    let mut scored: Vec<(i32, usize)> = recipes()
        .iter()
        .enumerate()
        .filter_map(|(i, r)| {
            let score = suggest_score(r, &q, &q_tokens);
            (score > 0).then_some((score, i))
        })
        .collect();
    // Higher score first; stable index order breaks ties.
    scored.sort_by(|a, b| b.0.cmp(&a.0).then(a.1.cmp(&b.1)));
    scored
        .into_iter()
        .take(limit)
        .map(|(_, i)| &recipes()[i])
        .collect()
}

fn suggest_score(r: &Recipe, q: &str, q_tokens: &[&str]) -> i32 {
    let title = r.title.to_ascii_lowercase();
    // Every handle this recipe answers to, plus its title.
    let mut handles: Vec<&str> = vec![r.name.as_str(), title.as_str()];
    handles.extend(r.aliases.iter().map(String::as_str));

    let mut score = 0;
    for h in &handles {
        if *h == q {
            score += 100;
        } else if h.contains(q) || q.contains(*h) {
            score += 40;
        }
    }
    // Token overlap catches multi-word queries that no single handle contains.
    let hay = format!("{} {} {}", r.name, title, r.aliases.join(" "));
    for t in q_tokens {
        if t.len() >= 2 && hay.contains(t) {
            score += 5;
        }
    }
    score
}

fn recipe_matches(r: &Recipe, needle: &str) -> bool {
    r.name.to_ascii_lowercase().contains(needle)
        || r.title.to_ascii_lowercase().contains(needle)
        || r.category.to_ascii_lowercase().contains(needle)
        || r.aliases
            .iter()
            .any(|a| a.to_ascii_lowercase().contains(needle))
}

// ---------- index ----------

/// name/alias (lowercase) -> recipe index. Aliases inserted first, names last,
/// so a name shadows a colliding alias on lookup.
fn index() -> &'static HashMap<String, usize> {
    static INDEX: OnceLock<HashMap<String, usize>> = OnceLock::new();
    INDEX.get_or_init(|| {
        let mut map = HashMap::new();
        for (i, r) in recipes().iter().enumerate() {
            for alias in &r.aliases {
                map.insert(alias.to_ascii_lowercase(), i);
            }
        }
        for (i, r) in recipes().iter().enumerate() {
            map.insert(r.name.to_ascii_lowercase(), i);
        }
        map
    })
}

// ---------- parsing ----------

fn build_recipes() -> Vec<Recipe> {
    let mut files: Vec<_> = RECIPES
        .find("**/*.md")
        .into_iter()
        .flatten()
        .filter_map(|entry| entry.as_file())
        .collect();
    files.sort_by_key(|f| f.path().to_path_buf());
    let mut out = Vec::new();
    for file in files {
        let Some(path) = file.path().to_str() else {
            continue;
        };
        let Some(text) = file.contents_utf8() else {
            continue;
        };
        if let Some(recipe) = parse_recipe(path, text) {
            out.push(recipe);
        }
    }
    out
}

/// Parse one recipe file. `path` is relative to `assets/recipes`
/// (`<category>/<name>.md`). Files without frontmatter, without a `title`, or
/// not nested under a category directory are skipped.
fn parse_recipe(path: &str, text: &str) -> Option<Recipe> {
    let (category, file) = path.split_once('/')?;
    let name = file.strip_suffix(".md")?;
    let (front, body) = split_frontmatter(text)?;
    let title = front_value(front, "title")?.to_string();
    let aliases = front_value(front, "aliases")
        .map(parse_aliases)
        .unwrap_or_default();
    Some(Recipe {
        name: name.to_string(),
        title,
        category: category.to_string(),
        aliases,
        body: body.trim_end().to_string(),
    })
}

/// Split a comma-separated `aliases:` value into trimmed, non-empty handles.
fn parse_aliases(value: &str) -> Vec<String> {
    value
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .collect()
}

/// Split a `---\n...\n---\n` YAML frontmatter block off the front. Returns
/// `(frontmatter_lines, body)` or `None` when there is no frontmatter. Mirrors
/// the manual's parser - the two corpora share a frontmatter convention.
fn split_frontmatter(text: &str) -> Option<(&str, &str)> {
    let rest = text.strip_prefix("---\n")?;
    let end = rest.find("\n---")?;
    let front = &rest[..end];
    let body = rest[end + 4..].trim_start_matches('\n');
    Some((front, body))
}

/// Read a `key: value` line out of a frontmatter block.
fn front_value<'a>(front: &'a str, key: &str) -> Option<&'a str> {
    front.lines().find_map(|line| {
        let (k, v) = line.split_once(':')?;
        (k.trim() == key).then(|| v.trim())
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn corpus_loads_and_counts_are_sane() {
        assert!(count() >= 2, "expected at least the seed recipes");
        assert!(!categories().is_empty(), "categories must be populated");
        assert_eq!(
            categories().iter().map(|c| c.count).sum::<usize>(),
            count(),
            "category counts must sum to the corpus size"
        );
    }

    #[test]
    fn every_recipe_has_title_and_body() {
        for r in recipes() {
            assert!(!r.title.is_empty(), "{} has empty title", r.name);
            assert!(!r.body.is_empty(), "{} has empty body", r.name);
            assert!(!r.category.is_empty(), "{} has empty category", r.name);
        }
    }

    #[test]
    fn names_and_aliases_are_unique() {
        use std::collections::HashSet;
        let mut seen = HashSet::new();
        for r in recipes() {
            assert!(
                seen.insert(r.name.to_ascii_lowercase()),
                "duplicate recipe name {}",
                r.name
            );
        }
        // An alias must not shadow another recipe's name, or lookup is
        // ambiguous. (An alias colliding with its own name is harmless.)
        for r in recipes() {
            for alias in &r.aliases {
                let a = alias.to_ascii_lowercase();
                if a == r.name.to_ascii_lowercase() {
                    continue;
                }
                assert!(
                    !seen.contains(&a),
                    "alias {alias} on {} shadows another recipe name",
                    r.name
                );
            }
        }
    }

    #[test]
    fn lookup_resolves_by_name_and_alias() {
        let by_name = lookup("hma").expect("hma by name");
        assert_eq!(by_name.title, "Hull Moving Average");
        assert_eq!(by_name.category, "moving-average");
        assert!(by_name.body.contains("ta.wma"), "body carries the Pine");

        // Case-insensitive, and an alias resolves to the same recipe.
        assert_eq!(lookup("HMA").map(|r| &r.name), Some(&"hma".to_string()));
        let by_alias = lookup("hull moving average").expect("hma by alias");
        assert_eq!(by_alias.name, "hma");

        assert!(lookup("does-not-exist").is_none());
    }

    #[test]
    fn list_filters_by_category_and_grep() {
        let mas = list(Some("moving-average"), None);
        assert!(mas.iter().any(|r| r.name == "hma"));
        assert!(
            mas.iter().all(|r| r.category == "moving-average"),
            "category filter must be exact"
        );
        // grep crosses name / title / alias.
        assert!(list(None, Some("hull")).iter().any(|r| r.name == "hma"));
        assert!(
            list(None, Some("engulf"))
                .iter()
                .any(|r| r.name == "engulfing")
        );
        assert!(list(Some("candlestick"), Some("hull")).is_empty());
    }

    #[test]
    fn suggest_ranks_exact_handle_first() {
        let hits = suggest("hull moving avg", 5);
        assert!(
            hits.iter().any(|r| r.name == "hma"),
            "fuzzy query should surface hma, got {:?}",
            hits.iter().map(|r| &r.name).collect::<Vec<_>>()
        );
        assert!(suggest("", 5).is_empty(), "empty query suggests nothing");
    }
}
