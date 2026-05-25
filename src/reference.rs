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

    // Depth-first walk that concatenates all Text and Code literals that are
    // descendants of `node`. Recurses into Emph, Strong, Link, etc. so that
    // inline markup inside headings (e.g. backtick code spans) is not dropped.
    fn inline_text<'a>(node: &'a AstNode<'a>) -> String {
        let mut out = String::new();
        for child in node.children() {
            match &child.data.borrow().value {
                NodeValue::Text(t) => out.push_str(t),
                NodeValue::Code(code) => out.push_str(&code.literal),
                _ => out.push_str(&inline_text(child)),
            }
        }
        out
    }

    // Each entry stores (title, level, heading_end_0) where heading_end_0 is
    // the 0-based index of the heading's last source line (comrak sourcepos is
    // 1-based, so heading_end_0 = sourcepos.end.line - 1). The body of a
    // section starts at heading_end_0 + 1. For single-line headings (the
    // normal case in v6.md) heading_end_0 == sourcepos.start.line - 1.
    fn collect_headings<'a>(node: &'a AstNode<'a>, headings: &mut Vec<(String, u8, usize)>) {
        if let NodeValue::Heading(heading) = &node.data.borrow().value {
            let level = heading.level;
            if level == 2 || level == 3 {
                let heading_text = inline_text(node);
                let heading_end_0 = node.data.borrow().sourcepos.end.line - 1;
                headings.push((canonicalize_title(&heading_text), level, heading_end_0));
            }
        }
        for child in node.children() {
            collect_headings(child, headings);
        }
    }

    let mut headings = Vec::new();
    collect_headings(root, &mut headings);

    let mut sections = Vec::with_capacity(headings.len());
    for (i, (title, level, heading_end_0)) in headings.iter().enumerate() {
        // Body ends just before the next heading's first line. Since we store
        // heading_end_0 (0-based last line of the heading), for single-line
        // headings that equals the 0-based start line, which is exactly the
        // exclusive upper bound we need for the preceding section's body.
        let end_line = if i + 1 < headings.len() {
            headings[i + 1].2
        } else {
            lines.len()
        };

        // Body starts at the line immediately after the heading ends.
        let content: Vec<String> = lines[heading_end_0 + 1..end_line]
            .iter()
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

    // Bug-1 regression: heading immediately followed by a body line (no blank
    // line between) must not drop the first body line.
    #[test]
    fn body_not_dropped_when_heading_has_no_blank_line() {
        let md = "## Category\n### Entry\nFirst body line\nSecond body line\n";
        let sections = parse_sections(md).expect("must parse");
        let entry = sections
            .iter()
            .find(|s| s.title == "Entry")
            .expect("Entry section must exist");
        assert!(
            entry.content.contains("First body line"),
            "first body line must not be dropped; got: {:?}",
            entry.content
        );
        assert!(
            entry.content.contains("Second body line"),
            "second body line must not be dropped; got: {:?}",
            entry.content
        );
    }

    // Bug-2 regression: inline code spans inside headings must be captured in
    // the section title.
    #[test]
    fn heading_inline_code_included_in_title() {
        let md = "## Category\n### Entry with `inline_code`\nsome body\n";
        let sections = parse_sections(md).expect("must parse");
        let entry = sections
            .iter()
            .find(|s| s.title.contains("Entry with"))
            .expect("section must exist");
        assert!(
            entry.title.contains("inline_code"),
            "inline code token must appear in title; got: {:?}",
            entry.title
        );
    }
}
