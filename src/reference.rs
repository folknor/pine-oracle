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

use anyhow::Result;
use comrak::nodes::{AstNode, NodeValue};
use comrak::{Arena, Options, parse_document};
use serde::Serialize;
use std::sync::OnceLock;

const REFERENCE_MARKDOWN: &str = include_str!("../vendor/pine-reference/spec/v6.md");

#[derive(Debug, Clone)]
pub struct Section {
    pub title: String,
    pub level: u8,
    pub content: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct Entry {
    pub category: String,
    pub name: String,
    pub content: String,
}

fn parse_sections(markdown_content: &str) -> Result<Vec<Section>> {
    let arena = Arena::new();
    let options = Options::default();
    let root = parse_document(&arena, markdown_content, &options);

    let lines: Vec<&str> = markdown_content.lines().collect();

    fn collect_headings<'a>(node: &'a AstNode<'a>, headings: &mut Vec<(String, u8, usize)>) {
        if let NodeValue::Heading(heading) = &node.data.borrow().value {
            let level = heading.level;
            if level == 2 || level == 3 {
                let mut heading_text = String::new();
                for child in node.children() {
                    if let NodeValue::Text(text) = &child.data.borrow().value {
                        heading_text.push_str(text);
                    }
                }
                let line_num = node.data.borrow().sourcepos.start.line;
                headings.push((canonicalize_title(&heading_text), level, line_num));
            }
        }
        for child in node.children() {
            collect_headings(child, headings);
        }
    }

    let mut headings = Vec::new();
    collect_headings(root, &mut headings);

    let mut sections = Vec::with_capacity(headings.len());
    for (i, (title, level, start_line)) in headings.iter().enumerate() {
        let end_line = if i + 1 < headings.len() {
            headings[i + 1].2 - 1
        } else {
            lines.len()
        };

        let content: Vec<String> = lines[*start_line..end_line]
            .iter()
            .skip(1)
            .map(|s| (*s).to_string())
            .collect();

        sections.push(Section {
            title: title.clone(),
            level: *level,
            content: content.join("\n").trim_end().to_string(),
        });
    }

    Ok(sections)
}

fn sections() -> &'static [Section] {
    static SECTIONS: OnceLock<Vec<Section>> = OnceLock::new();
    SECTIONS
        .get_or_init(|| parse_sections(REFERENCE_MARKDOWN).expect("vendored v6.md must parse"))
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
/// the BM25 index.
pub fn all_entries() -> Vec<Entry> {
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
    haystack.len() >= needle.len() && haystack[..needle.len()].eq_ignore_ascii_case(needle)
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

    #[test]
    fn parses_at_least_seven_categories() {
        let cats = categories();
        assert!(
            cats.len() >= 7,
            "expected >= 7 categories, got {}: {:?}",
            cats.len(),
            cats
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
}
