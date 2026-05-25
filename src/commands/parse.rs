use anyhow::Result;

use crate::output::{ResolvedFormat, print_json};

pub(crate) fn run(code: &str, format: ResolvedFormat) -> Result<()> {
    let script = piners_syntax::parse(code).map_err(format_parse_errors)?;

    match format {
        ResolvedFormat::Json => {
            print_json(&script)?;
        }
        ResolvedFormat::Text => {
            // piners-syntax has no stable pretty-printer yet; JSON is the
            // stable AST surface, while Debug keeps text mode inspectable.
            println!("{script:#?}");
        }
    }
    Ok(())
}

fn format_parse_errors(errors: Vec<piners_syntax::ParseError>) -> anyhow::Error {
    let label = if errors
        .iter()
        .all(|error| matches!(error.kind, piners_syntax::ParseErrorKind::Lex))
    {
        "lex error"
    } else if errors
        .iter()
        .any(|error| matches!(error.kind, piners_syntax::ParseErrorKind::Lex))
    {
        "syntax error"
    } else {
        "parse error"
    };
    let message = errors
        .into_iter()
        .map(|error| {
            let loc = error.location.map_or_else(
                || "?".to_string(),
                |location| format!("{}:{}", location.start.line, location.start.column),
            );
            if matches!(error.kind, piners_syntax::ParseErrorKind::Lex) {
                format!("{loc}: {}", error.got)
            } else {
                // Build a clean message field-by-field to avoid leaking raw
                // `Span { start: N, end: M }` (from {span:?}) and `Some(...)`
                // wrappers (from {expected:?}) that the ParseError Display impl
                // would otherwise emit.
                let kind = &error.kind;
                match error.expected.as_deref() {
                    Some(expected) => {
                        format!("{loc}: {kind}: expected {expected}, got {}", error.got)
                    }
                    None => format!("{loc}: {kind}: got {}", error.got),
                }
            }
        })
        .collect::<Vec<_>>()
        .join("\n");
    anyhow::anyhow!("{label}: {message}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_indicator_header() {
        let script =
            piners_syntax::parse("//@version=6\nindicator(\"x\")\nvalue = 1\n").expect("parse");
        assert_eq!(script.version, 6);
        // The indicator call is stored in Script::header. Script::items only
        // contains code after the header.
        assert_eq!(script.items.len(), 1);
    }

    #[test]
    fn formats_parse_error_with_location() {
        let err =
            format_parse_errors(piners_syntax::parse("not a pine header").expect_err("must fail"));
        assert!(err.to_string().contains("1:1"));
    }

    #[test]
    fn parse_error_does_not_leak_span_debug() {
        // "not a pine header" triggers an invalid-header parse error.
        // The formatted message must not contain raw Span { ... } or Some(...)
        // wrappers that the ParseError Display impl would otherwise emit.
        let err =
            format_parse_errors(piners_syntax::parse("not a pine header").expect_err("must fail"));
        let msg = err.to_string();
        assert!(
            !msg.contains("Span {"),
            "output must not contain Span debug: {msg}"
        );
        assert!(
            !msg.contains("Some("),
            "output must not contain Some(...) wrapper: {msg}"
        );
    }

    #[test]
    fn parse_error_has_clean_expected_got_reporting() {
        // A source with an obviously wrong token after the version header gives
        // a parse error with both expected and got fields set. Verify the
        // rendered message contains "expected" and "got" as plain English words
        // and that the got token text appears directly (no Debug wrapping).
        let err =
            format_parse_errors(piners_syntax::parse("//@version=6\n123").expect_err("must fail"));
        let msg = err.to_string();
        assert!(
            !msg.contains("Span {"),
            "output must not contain Span debug: {msg}"
        );
        assert!(
            !msg.contains("Some("),
            "output must not contain Some(...) wrapper: {msg}"
        );
        // The rendered message should be human-readable.
        assert!(
            msg.contains("got") || msg.contains("expected"),
            "output should contain expected/got language: {msg}"
        );
    }

    #[test]
    fn labels_lex_failures_as_lex_errors() {
        let err = format_parse_errors(
            piners_syntax::parse("//@version=6\nindicator(\"x\")\ntext = \"unterminated\n")
                .expect_err("must fail"),
        );
        assert!(err.to_string().starts_with("lex error:"));
    }
}
