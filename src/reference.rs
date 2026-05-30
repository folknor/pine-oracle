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

/// Prose enrichment lifted from a v6 reference entry body for the merged
/// `po lookup` view. These are the sections pine-data's structured `behavior`
/// surface does not carry: free-text `Remarks`, the `See also` cross-reference
/// list, and per-argument prose descriptions (the structured param list has
/// types but no prose).
#[derive(Debug, Clone, Serialize, Default)]
pub struct Enrichment {
    pub category: String,
    pub remarks: Option<String>,
    pub see_also: Vec<String>,
    pub arguments: Vec<ArgProse>,
}

impl Enrichment {
    /// True when nothing beyond the category was extracted - lets callers skip
    /// rendering an empty enrichment block.
    pub fn is_empty(&self) -> bool {
        self.remarks.is_none() && self.see_also.is_empty() && self.arguments.is_empty()
    }
}

/// One `name -> prose` pair from an entry's `Arguments` section.
#[derive(Debug, Clone, Serialize)]
pub struct ArgProse {
    pub name: String,
    pub description: String,
}

/// The bare label lines v6.md uses to delimit sub-sections inside an entry
/// body. They appear as standalone paragraph lines (not markdown headings),
/// each followed by a blank line then the section content. A body line whose
/// trimmed text matches one of these exactly starts a new section.
const ENTRY_SECTION_LABELS: [&str; 9] = [
    "Syntax",
    "Syntax & Overloads",
    "Arguments",
    "Example",
    "Returns",
    "Remarks",
    "See also",
    "Type",
    "Fields",
];

/// Parse the prose enrichment for `name` out of its v6 reference entry.
/// Returns `None` when the name has no reference entry at all.
pub fn enrichment(name: &str) -> Option<Enrichment> {
    let entry = lookup(name)?;
    let sections = entry_sections(&entry.content);
    let mut out = Enrichment {
        category: entry.category,
        ..Default::default()
    };
    for (label, body) in &sections {
        match label.as_str() {
            "Remarks" => {
                let trimmed = body.trim();
                if !trimmed.is_empty() {
                    out.remarks = Some(trimmed.to_string());
                }
            }
            "See also" => out.see_also = parse_see_also(body),
            "Arguments" => out.arguments = parse_arguments(body),
            _ => {}
        }
    }
    Some(out)
}

/// Split an entry body into `(label, section_body)` pairs delimited by the
/// bare `ENTRY_SECTION_LABELS` lines. Body text before the first label (the
/// entry's lead description) is dropped, since the structured behavior surface
/// already carries the description.
fn entry_sections(body: &str) -> Vec<(String, String)> {
    let mut out: Vec<(String, String)> = Vec::new();
    let mut current: Option<(String, Vec<&str>)> = None;
    for line in body.lines() {
        if ENTRY_SECTION_LABELS.contains(&line.trim()) {
            if let Some((label, lines)) = current.take() {
                out.push((label, lines.join("\n").trim().to_string()));
            }
            current = Some((line.trim().to_string(), Vec::new()));
        } else if let Some((_, lines)) = current.as_mut() {
            lines.push(line);
        }
    }
    if let Some((label, lines)) = current.take() {
        out.push((label, lines.join("\n").trim().to_string()));
    }
    out
}

/// Extract the cross-referenced names from a `See also` block. The block is a
/// run of markdown links like `[ta.ema()](#fun_ta.ema)[ta.rma()](#fun_ta.rma)`;
/// we keep each link's display text with the trailing `()` stripped so it
/// matches the name a user would pass back to `po lookup`.
fn parse_see_also(body: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut rest = body;
    while let Some(open) = rest.find('[') {
        let after = &rest[open + 1..];
        let Some(close) = after.find(']') else { break };
        let text = after[..close].trim();
        let name = text.strip_suffix("()").unwrap_or(text).trim();
        if !name.is_empty() {
            out.push(name.to_string());
        }
        rest = &after[close + 1..];
    }
    out
}

/// Parse the `Arguments` section into `name -> prose` pairs. Each argument is a
/// single line of the form `name (type) prose...`; we take the leading token as
/// the name and everything after the `(type)` parenthetical as the prose. Lines
/// that don't fit the shape are skipped rather than guessed at.
fn parse_arguments(body: &str) -> Vec<ArgProse> {
    let mut out = Vec::new();
    for line in body.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let Some(paren) = line.find(" (") else {
            continue;
        };
        let name = line[..paren].trim();
        let after_open = &line[paren + 2..];
        let Some(close) = after_open.find(')') else {
            continue;
        };
        let prose = after_open[close + 1..].trim();
        if name.is_empty() || prose.is_empty() {
            continue;
        }
        out.push(ArgProse {
            name: name.to_string(),
            description: prose.to_string(),
        });
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

    // Enrichment: ta.sma carries Remarks, See also, and two prose arguments.
    // Pin the parse against the real vendored entry so a format drift in v6.md
    // (or a regression in entry_sections) fails here.
    #[test]
    fn enrichment_extracts_ta_sma_sections() {
        let e = enrichment("ta.sma").expect("ta.sma must have a reference entry");
        assert_eq!(e.category, "Functions");
        assert_eq!(
            e.remarks.as_deref(),
            Some("`na` values in the `source` series are ignored."),
            "ta.sma Remarks must be captured verbatim"
        );
        assert!(
            e.see_also.iter().any(|s| s == "ta.ema"),
            "See also must include ta.ema (with () stripped); got {:?}",
            e.see_also
        );
        assert!(
            !e.see_also.iter().any(|s| s.contains("()")),
            "see-also names must have () stripped; got {:?}",
            e.see_also
        );
        let source = e
            .arguments
            .iter()
            .find(|a| a.name == "source")
            .expect("source argument prose must be parsed");
        assert_eq!(source.description, "Series of values to process.");
    }

    // Variable entries enrich too: `close` carries both a Remarks line and a
    // See-also list of the other OHLC variables.
    #[test]
    fn enrichment_captures_variable_remarks_and_see_also() {
        let e = enrichment("close").expect("close must have a reference entry");
        assert_eq!(e.category, "Variables");
        assert!(
            e.remarks.is_some(),
            "close should carry a Remarks line; got {e:?}"
        );
        assert!(
            e.see_also.iter().any(|s| s == "open"),
            "close See-also should include open; got {:?}",
            e.see_also
        );
        // A variable has no Arguments section.
        assert!(e.arguments.is_empty());
    }

    // is_empty() is the signal the lookup view uses to skip the enrichment
    // block; pin it against a default (no sections extracted).
    #[test]
    fn enrichment_default_is_empty() {
        assert!(Enrichment::default().is_empty());
    }

    #[test]
    fn enrichment_none_for_unknown_name() {
        assert!(enrichment("definitely_not_a_pine_name_xyz").is_none());
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
