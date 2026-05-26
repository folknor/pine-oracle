// SPDX-License-Identifier: MPL-2.0
//
// Shared markdown-section extractor used by reference.rs and search.rs.
//
// Both callers need to split a markdown document into (level, title, body)
// triples for H2 and H3 headings. The logic was previously duplicated as
// `parse_sections` (reference.rs) and `parse_md_sections` (search.rs). This
// module is the canonical single copy.

use comrak::nodes::{AstNode, NodeValue};
use comrak::{Arena, Options, parse_document};

/// One markdown section: heading level (2 or 3), heading text, and body text.
///
/// `title` is the concatenated inline text of all `Text` and `Code` nodes
/// inside the heading (backtick code spans included). `body` is the raw
/// source lines from the line immediately after the heading down to (but not
/// including) the next H2/H3 heading at any level, with trailing whitespace
/// trimmed.
///
/// H1 headings and headings below H3 are silently skipped.
#[derive(Debug, Clone)]
pub(crate) struct MdSection {
    pub(crate) level: u8,
    pub(crate) title: String,
    pub(crate) body: String,
}

/// Extract all H2 and H3 sections from `markdown`.
///
/// Returns one `MdSection` per H2/H3 heading in document order. The body of
/// each section is the source lines between the current heading and the next
/// H2/H3 heading (or end-of-document), trailing-whitespace trimmed.
///
/// Inline markup inside headings (bold, italic, links, backtick code spans) is
/// flattened to plain text so the caller receives the words users actually
/// type (e.g. `inline_code` from `### Entry with \`inline_code\``).
///
/// H1 introductory text (before the first H2/H3) is not included in any
/// section. Callers that want to surface H1 body prose should prepend it to
/// the first section's body or index it separately.
pub(crate) fn sections(markdown: &str) -> Vec<MdSection> {
    let arena = Arena::new();
    let opts = Options::default();
    let root = parse_document(&arena, markdown, &opts);
    let lines: Vec<&str> = markdown.lines().collect();

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

    // Each entry stores (raw_title, level, heading_end_0) where heading_end_0
    // is the 0-based index of the heading's last source line (comrak sourcepos
    // is 1-based, so heading_end_0 = sourcepos.end.line - 1). The body of a
    // section starts at heading_end_0 + 1. For single-line headings (the
    // normal case) heading_end_0 == sourcepos.start.line - 1.
    fn collect<'a>(node: &'a AstNode<'a>, out: &mut Vec<(String, u8, usize)>) {
        if let NodeValue::Heading(h) = &node.data.borrow().value
            && (h.level == 2 || h.level == 3)
        {
            let text = inline_text(node);
            let heading_end_0 = node.data.borrow().sourcepos.end.line - 1;
            out.push((text, h.level, heading_end_0));
        }
        for child in node.children() {
            collect(child, out);
        }
    }

    let mut headings: Vec<(String, u8, usize)> = Vec::new();
    collect(root, &mut headings);

    let mut out = Vec::with_capacity(headings.len());
    for (i, (raw_title, level, heading_end_0)) in headings.iter().enumerate() {
        // Body ends just before the next heading's first line. Since we store
        // heading_end_0 (0-based last line of the heading), for single-line
        // headings that equals the 0-based start line, which is exactly the
        // exclusive upper bound we need for the preceding section's body.
        let end = if i + 1 < headings.len() {
            headings[i + 1].2
        } else {
            lines.len()
        };
        // Body starts at the line immediately after the heading ends.
        let body = lines[heading_end_0 + 1..end]
            .join("\n")
            .trim_end()
            .to_string();
        out.push(MdSection {
            level: *level,
            title: raw_title.trim().to_string(),
            body,
        });
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn h2_and_h3_sections_are_extracted() {
        let md = "# Ignored H1\n\n## Category\n\n### Entry\nbody line\n";
        let s = sections(md);
        assert_eq!(s.len(), 2, "expected 2 sections (H2 + H3), got {}", s.len());
        assert_eq!(s[0].level, 2);
        assert_eq!(s[0].title, "Category");
        assert_eq!(s[1].level, 3);
        assert_eq!(s[1].title, "Entry");
        assert!(
            s[1].body.contains("body line"),
            "H3 body must include content; got: {:?}",
            s[1].body
        );
    }

    #[test]
    fn body_not_dropped_when_heading_has_no_blank_line() {
        let md = "## Section\nFirst body line\nSecond body line\n";
        let s = sections(md);
        assert_eq!(s.len(), 1);
        assert!(
            s[0].body.contains("First body line"),
            "first body line must not be dropped; got: {:?}",
            s[0].body
        );
        assert!(
            s[0].body.contains("Second body line"),
            "second body line must not be dropped; got: {:?}",
            s[0].body
        );
    }

    #[test]
    fn heading_inline_code_included_in_title() {
        let md = "## Section with `inline_code` token\nsome body\n";
        let s = sections(md);
        assert_eq!(s.len(), 1);
        assert!(
            s[0].title.contains("inline_code"),
            "inline code token must appear in title; got: {:?}",
            s[0].title
        );
    }

    #[test]
    fn h1_body_is_not_included_in_any_section() {
        let md = "# Top level\n\nIntro text that should not appear.\n\n## Section\nbody\n";
        let s = sections(md);
        // Only the H2 is returned; the H1 intro is not in any section body.
        assert_eq!(s.len(), 1, "only H2 section expected");
        assert_eq!(s[0].level, 2);
        // The intro text must not bleed into the H2's body.
        assert!(
            !s[0].body.contains("Intro text"),
            "H1 intro must not appear in H2 body; got: {:?}",
            s[0].body
        );
    }

    #[test]
    fn trailing_whitespace_trimmed_from_body() {
        let md = "## Section\nbody\n\n\n";
        let s = sections(md);
        assert_eq!(s.len(), 1);
        assert_eq!(s[0].body, "body", "trailing newlines must be trimmed");
    }
}
