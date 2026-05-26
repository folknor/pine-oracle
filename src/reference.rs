// SPDX-License-Identifier: MPL-2.0
//
// Adapted from pinecone (crates/pine-reference/src/lib.rs, MPL-2.0).
// (c) Pinecone contributors. See vendor/pine-reference/LICENSE.
//
// Changes from upstream:
//   - Dropped download_and_save_reference() and its deps (headless_chrome,
//     scraper, htmd). Spec refreshes are done by running pinecone's tool.
//   - Replaced eyre with anyhow.
//   - Cached the parsed section table behind OnceLock.
//   - Added lookup(name) and search(query) that span all categories.

use serde::Serialize;
use std::sync::OnceLock;

use crate::util::markdown;

const REFERENCE_MARKDOWN: &str = include_str!("../vendor/pine-reference/spec/v6.md");

/// A parsed section from the v6 reference. Only used internally; the public
/// API surface is `Entry` (via `lookup` / `all_entries` / `prefix_search`).
///
/// Prefer `all_entries()` for the common case of iterating over every
/// reference entry with its parent category.
#[derive(Debug, Clone)]
pub(crate) struct Section {
    pub(crate) title: String,
    pub(crate) level: u8,
    pub(crate) content: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct Entry {
    pub category: String,
    pub name: String,
    pub content: String,
}

fn sections() -> &'static [Section] {
    static SECTIONS: OnceLock<Vec<Section>> = OnceLock::new();
    SECTIONS
        .get_or_init(|| {
            markdown::sections(REFERENCE_MARKDOWN)
                .into_iter()
                .map(|s| Section {
                    title: canonicalize_title(&s.title),
                    level: s.level,
                    content: s.body,
                })
                .collect()
        })
        .as_slice()
}

pub fn categories() -> Vec<&'static str> {
    sections()
        .iter()
        .filter(|s| s.level == 2)
        .map(|s| s.title.as_str())
        .collect()
}

/// Every level-3 entry with its parent category. Used by `search` to build
/// the BM25 index and by `kind_catalog` for the reference document count.
///
/// The `Vec<Entry>` is built once and cached behind a `OnceLock`; subsequent
/// calls return a borrow of the static slice without any allocation.
pub fn all_entries() -> &'static [Entry] {
    static ENTRIES: OnceLock<Vec<Entry>> = OnceLock::new();
    ENTRIES
        .get_or_init(|| {
            let mut out = Vec::with_capacity(1024);
            let mut current_cat: &str = "";
            for s in sections() {
                if s.level == 2 {
                    current_cat = &s.title;
                } else if s.level == 3 {
                    out.push(Entry {
                        category: current_cat.to_string(),
                        name: s.title.clone(),
                        content: s.content.clone(),
                    });
                }
            }
            out
        })
        .as_slice()
}

/// Exact-match lookup across every category. First hit wins.
pub fn lookup(name: &str) -> Option<Entry> {
    let mut current_cat: &str = "";
    for s in sections() {
        if s.level == 2 {
            current_cat = &s.title;
        } else if s.level == 3 && s.title.eq_ignore_ascii_case(name) {
            return Some(Entry {
                category: current_cat.to_string(),
                name: s.title.clone(),
                content: s.content.clone(),
            });
        }
    }
    None
}

/// Case-insensitive prefix matches across every category.
pub fn prefix_search(prefix: &str) -> Vec<Entry> {
    let mut out = Vec::new();
    let mut current_cat: &str = "";
    for s in sections() {
        if s.level == 2 {
            current_cat = &s.title;
        } else if s.level == 3 && starts_with_ci(&s.title, prefix) {
            out.push(Entry {
                category: current_cat.to_string(),
                name: s.title.clone(),
                content: s.content.clone(),
            });
        }
    }
    out
}

fn starts_with_ci(haystack: &str, needle: &str) -> bool {
    // Use `get(..needle.len())` rather than a direct slice so we never panic
    // when `needle.len()` falls on a multi-byte UTF-8 boundary. The slice
    // length matches iff `needle` is purely ASCII (which is the common case
    // for Pine identifiers); non-ASCII needles simply fall through to None
    // and return false.
    haystack
        .get(..needle.len())
        .is_some_and(|prefix| prefix.eq_ignore_ascii_case(needle))
}

/// Canonicalise a heading text into the name users actually type.
/// Function entries in v6.md ship as `math.max()`, `ta.rsi()` etc.; users
/// type `math.max`. Strip the trailing `()` and surrounding whitespace.
fn canonicalize_title(raw: &str) -> String {
    let trimmed = raw.trim();
    trimmed
        .strip_suffix("()")
        .unwrap_or(trimmed)
        .trim()
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// v6.md has exactly 7 H2 category headings: Variables, Constants,
    /// Functions, Keywords, Types, Operators, Annotations. Pin the exact count
    /// so a dropped category fails the test rather than a loose `>= 7` mask.
    #[test]
    fn parses_exactly_seven_categories() {
        let cats = categories();
        assert_eq!(
            cats.len(),
            7,
            "expected exactly 7 categories, got {}: {:?}",
            cats.len(),
            cats
        );
        // Pin the canonical names so a rename also fails the test.
        let expected = [
            "Variables",
            "Constants",
            "Functions",
            "Keywords",
            "Types",
            "Operators",
            "Annotations",
        ];
        assert_eq!(
            cats, expected,
            "category names or order changed; expected {expected:?}, got {cats:?}"
        );
    }

    #[test]
    fn looks_up_math_max() {
        let e = lookup("math.max").expect("math.max must exist");
        assert_eq!(e.category, "Functions");
        assert_eq!(e.name, "math.max");
        assert!(!e.content.is_empty());
    }

    #[test]
    fn looks_up_close_variable() {
        let e = lookup("close").expect("close must exist");
        assert_eq!(e.category, "Variables");
    }

    #[test]
    fn prefix_search_finds_math_namespace() {
        let hits = prefix_search("math.");
        assert!(hits.len() > 10, "math. prefix should hit many functions");
    }

    // Escaped-underscore regression: v6.md writes `### bar\_index` (backslash
    // escapes the underscore to suppress inline italic). Comrak unescapes `\_`
    // to `_` in Text nodes, so lookup("bar_index") must succeed. If a future
    // comrak upgrade changes unescaping behavior, this test catches it.
    #[test]
    fn looks_up_escaped_underscore_identifier() {
        let e = lookup("bar_index").expect("bar_index must exist");
        assert_eq!(e.category, "Variables");
        // Canonicalized name must not contain the backslash.
        assert!(
            !e.name.contains('\\'),
            "name must not contain backslash; got: {:?}",
            e.name
        );
    }

    // Second underscore-escaped name pin.
    #[test]
    fn looks_up_last_bar_index() {
        let e = lookup("last_bar_index").expect("last_bar_index must exist");
        assert_eq!(e.category, "Variables");
    }

    // Bug-1 regression: heading immediately followed by a body line (no blank
    // line between) must not drop the first body line.
    // Now exercises util::markdown::sections through sections() -> all_entries().
    // The canonical regression pin lives in util/markdown.rs; this test pins
    // the reference-layer mapping (canonicalize_title + OnceLock accumulation)
    // against the real vendored data.
    #[test]
    fn body_not_dropped_for_real_entry() {
        // `close` is followed immediately by body text with no blank line.
        let e = lookup("close").expect("close must exist");
        assert!(
            !e.content.is_empty(),
            "close entry must have non-empty body"
        );
    }

    // Bug-2 regression: inline code spans inside headings must be captured.
    // Verified against the shared util::markdown::sections tests; this layer
    // asserts that canonicalize_title doesn't destroy inline code content.
    #[test]
    fn inline_code_in_heading_survives_canonicalization() {
        // math.max() -> "math.max" (strip `()` suffix, keep the name)
        let e = lookup("math.max").expect("math.max must exist");
        assert!(
            e.name.contains("math.max"),
            "canonicalized name must preserve identifier; got: {:?}",
            e.name
        );
    }

    // starts_with_ci must not panic when given a multi-byte UTF-8 needle.
    #[test]
    fn starts_with_ci_non_ascii_needle_does_not_panic() {
        // The current vendored data has no non-ASCII H3 titles, but the helper
        // must not panic when a caller passes a multi-byte needle. An empty
        // result (not a panic) is the correct outcome.
        let hits = prefix_search("\u{2192}"); // U+2192 RIGHTWARDS ARROW
        // No heading starts with an arrow, so we expect no hits - but no panic.
        assert!(
            hits.is_empty(),
            "no heading should start with an arrow; got: {hits:?}"
        );
    }
}
