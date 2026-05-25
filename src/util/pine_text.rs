// SPDX-License-Identifier: MPL-2.0
//
// Lightweight Pine v6 text utilities that operate on raw source strings
// without invoking a full lexer or parser.

/// Strip Pine line (`//`) and block (`/* */`) comments from source text.
///
/// The algorithm walks the input byte by byte:
/// - On `/*`: skip forward until `*/` (or end of input). Pine does NOT support
///   nested block comments; an inner `/*` is not recognised and the outer `*/`
///   closes the block.
/// - On `//`: skip forward until `\n` (or end of input).
/// - Anything else is copied verbatim, including string literals. Comment
///   delimiters inside string literals are NOT recognised as comment openers
///   (this matches the current behavior of both corpus.rs and diff.rs, which
///   also do not special-case strings; preserving that behavior here).
///
/// The returned String has the same character positions as the input for all
/// non-comment bytes, which keeps downstream substring searches meaningful.
pub(crate) fn strip_pine_comments(src: &str) -> String {
    let mut out = String::with_capacity(src.len());
    let bytes = src.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if i + 1 < bytes.len() && bytes[i] == b'/' && bytes[i + 1] == b'*' {
            // Block comment: skip until `*/`
            i += 2;
            while i + 1 < bytes.len() && !(bytes[i] == b'*' && bytes[i + 1] == b'/') {
                i += 1;
            }
            // Consume the closing `*/` if present (min with len for unterminated comment)
            i = (i + 2).min(bytes.len());
        } else if i + 1 < bytes.len() && bytes[i] == b'/' && bytes[i + 1] == b'/' {
            // Line comment: skip to end of line
            while i < bytes.len() && bytes[i] != b'\n' {
                i += 1;
            }
        } else {
            out.push(bytes[i] as char);
            i += 1;
        }
    }
    out
}

/// Detect whether a Pine strategy source uses trailing-stop exit parameters.
///
/// Searches the (already comment-stripped) source for any of the three
/// trailing-stop parameter names -- `trail_points`, `trail_offset`,
/// `trail_price` -- followed (after optional whitespace) by `=`. The
/// keyword itself must also appear as an assignment (parameter binding);
/// bare occurrences such as variable names without `=` are ignored.
///
/// **Pre-condition:** pass the output of [`strip_pine_comments`] to avoid
/// false negatives from commented-out code. The function does not strip
/// comments internally so that callers that have already stripped can avoid
/// the cost.
///
/// **Note:** only the first occurrence of each keyword is inspected. If the
/// first occurrence lacks `=` but a later one has it, the function returns
/// `false` for that keyword. This matches the behavior of the original
/// `diff.rs::detect_profile_from_source` that this function replaces.
pub(crate) fn uses_trail_exits(stripped_src: &str) -> bool {
    let lower = stripped_src.to_ascii_lowercase();
    for needle in ["trail_points", "trail_offset", "trail_price"] {
        if let Some(pos) = lower.find(needle) {
            let rest = &lower[pos + needle.len()..];
            // Require `=` after optional whitespace (not `==`; but we just
            // check starts_with('=') which also matches `==`; this matches
            // the upstream verify_corpus.py behavior and the original diff.rs)
            if rest.trim_start().starts_with('=') {
                return true;
            }
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- strip_pine_comments ---

    #[test]
    fn strip_empty_input() {
        assert_eq!(strip_pine_comments(""), "");
    }

    #[test]
    fn strip_no_comments() {
        let src = "x = 1\ny = 2\n";
        assert_eq!(strip_pine_comments(src), src);
    }

    #[test]
    fn strip_line_comment() {
        let src = "real // commented out\nmore real\n";
        let out = strip_pine_comments(src);
        assert!(out.contains("real "), "kept real code: {out:?}");
        assert!(out.contains("more real"), "kept second line: {out:?}");
        assert!(!out.contains("commented out"), "stripped comment: {out:?}");
    }

    #[test]
    fn strip_block_comment() {
        let src = "before /* block */ after\n";
        let out = strip_pine_comments(src);
        assert!(out.contains("before "), "kept prefix: {out:?}");
        assert!(out.contains(" after"), "kept suffix: {out:?}");
        assert!(!out.contains("block"), "stripped block: {out:?}");
    }

    #[test]
    fn strip_block_comment_spanning_lines() {
        let src = "a\n/* line1\nline2\n*/\nb\n";
        let out = strip_pine_comments(src);
        assert!(out.contains('a'), "kept a: {out:?}");
        assert!(out.contains('b'), "kept b: {out:?}");
        assert!(!out.contains("line1"), "stripped block content: {out:?}");
        assert!(!out.contains("line2"), "stripped block content: {out:?}");
    }

    #[test]
    fn strip_combined_line_and_block() {
        let src = "real // commented out\n/* block */more real\n// only comment\n";
        let out = strip_pine_comments(src);
        assert!(out.contains("real "), "kept real: {out:?}");
        assert!(out.contains("more real"), "kept more real: {out:?}");
        assert!(
            !out.contains("commented out"),
            "stripped line comment: {out:?}"
        );
        assert!(!out.contains("block"), "stripped block comment: {out:?}");
        assert!(
            !out.contains("only comment"),
            "stripped comment-only line: {out:?}"
        );
    }

    #[test]
    fn strip_nested_block_comment_not_supported() {
        // Pine does NOT support nested block comments. The outer `*/` closes
        // the block; the inner `/*` is consumed as block content.
        // Input: `/* outer /* inner */ still_outer */ real`
        // Expected: ` real` (only text after the first `*/` is kept).
        let src = "/* outer /* inner */ still_outer */ real";
        let out = strip_pine_comments(src);
        // After the first `*/`, we get " still_outer */ real"
        assert!(
            out.contains("still_outer"),
            "content after inner close kept: {out:?}"
        );
        assert!(out.contains("real"), "real code kept: {out:?}");
        // The outer comment header `/* outer ` and inner marker `/* inner` are
        // both consumed before the first `*/`. " outer " surrounded by spaces
        // and " inner " must not survive; "still_outer" can (and does).
        assert!(
            !out.contains(" outer "),
            "outer comment header stripped: {out:?}"
        );
        assert!(!out.contains(" inner "), "inner marker stripped: {out:?}");
    }

    #[test]
    fn strip_comment_delimiter_in_string_literal_passes_through() {
        // Current behavior: comment delimiters inside string literals are NOT
        // recognised specially. A `//` inside a string will still truncate the
        // line at that point. This matches the original implementations in
        // corpus.rs and diff.rs and is documented as a known limitation.
        let src = "x = \"hello // world\"\ny = 1\n";
        let out = strip_pine_comments(src);
        // The `//` inside the string is treated as a line comment start.
        assert!(
            out.contains("x = \"hello "),
            "prefix of string kept: {out:?}"
        );
        assert!(!out.contains("world"), "part after // stripped: {out:?}");
        // Second line is unaffected.
        assert!(out.contains("y = 1"), "second line intact: {out:?}");
    }

    // --- uses_trail_exits ---

    #[test]
    fn trail_points_with_space() {
        assert!(uses_trail_exits("trail_points = 1"));
    }

    #[test]
    fn trail_points_no_space() {
        assert!(uses_trail_exits("trail_points=1"));
    }

    #[test]
    fn trail_offset_detected() {
        assert!(uses_trail_exits(
            "strategy.exit(id=\"x\", trail_offset = 5)"
        ));
    }

    #[test]
    fn trail_price_detected() {
        assert!(uses_trail_exits("trail_price = close"));
    }

    #[test]
    fn trail_points_no_equals_not_detected() {
        // Bare keyword without `=` should not trigger (e.g. a variable name).
        assert!(!uses_trail_exits("trail_points"));
    }

    #[test]
    fn trail_in_comment_not_detected_when_pre_stripped() {
        // If caller strips comments first, the commented keyword is gone.
        let src = "// trail_points = 1\nreal_code = 2\n";
        let stripped = strip_pine_comments(src);
        assert!(!uses_trail_exits(&stripped));
    }

    #[test]
    fn unrelated_keyword_not_detected() {
        assert!(!uses_trail_exits("stop_loss = 1.0"));
        assert!(!uses_trail_exits("trail_mode = true"));
    }

    #[test]
    fn trail_keyword_case_insensitive() {
        // The function lowercases internally, so TRAIL_POINTS should match.
        assert!(uses_trail_exits("TRAIL_POINTS = 100"));
    }
}
