use anyhow::Result;
use serde::Serialize;

use crate::output::{ResolvedFormat, Style, print_json};

#[derive(Serialize)]
struct TokenOut<'a> {
    typ: &'a piners_syntax::TokenKind,
    lexeme: &'a str,
    line: u32,
    column: u32,
}

#[derive(Serialize)]
struct LexErrorOut {
    code: Option<String>,
    message: String,
    line: u32,
    column: u32,
}

/// Wrapping envelope emitted in JSON mode when lex errors are present.
/// When there are no errors the envelope is still used, keeping the schema
/// stable: callers can always key on `.tokens` and `.errors`.
#[derive(Serialize)]
struct TokensOutput<'a> {
    tokens: Vec<TokenOut<'a>>,
    errors: Vec<LexErrorOut>,
}

pub(crate) fn run(code: &str, format: ResolvedFormat, _style: Style) -> Result<()> {
    let lexed = piners_syntax::lex_result(code);

    let tokens = lexed
        .tokens
        .iter()
        .map(|token| {
            let location = token.source_span(&lexed.linebreaks);
            TokenOut {
                typ: &token.kind,
                lexeme: token_text(code, token),
                line: location.start.line,
                column: location.start.column,
            }
        })
        .collect::<Vec<_>>();

    let errors: Vec<LexErrorOut> = lexed
        .diagnostics
        .iter()
        .map(|d| {
            let loc = d.span.source_span(&lexed.linebreaks);
            LexErrorOut {
                code: d.code.as_deref().map(str::to_owned),
                message: d.message.clone(),
                line: loc.start.line,
                column: loc.start.column,
            }
        })
        .collect();

    match format {
        ResolvedFormat::Json => {
            print_json(&TokensOutput { tokens, errors })?;
        }
        ResolvedFormat::Text => {
            for token in &tokens {
                println!(
                    "{:>4}:{:<3}  {:?}  {:?}",
                    token.line, token.column, token.typ, token.lexeme
                );
            }
            if !errors.is_empty() {
                eprintln!();
                eprintln!("lex errors ({}):", errors.len());
                for e in &errors {
                    let code_prefix = e
                        .code
                        .as_deref()
                        .map_or(String::new(), |c| format!("{c}: "));
                    eprintln!(
                        "  {:>4}:{:<3}  {}{}",
                        e.line, e.column, code_prefix, e.message
                    );
                }
            }
        }
    }
    Ok(())
}

fn token_text<'a>(code: &'a str, token: &piners_syntax::Token) -> &'a str {
    let start = token.span.start as usize;
    let end = token.span.end as usize;
    // `get` returns None only when the byte range is out of bounds or
    // crosses a UTF-8 char boundary. piners-syntax spans are always
    // valid UTF-8 boundaries in practice; the empty fallback is a
    // defensive guard, not a normal path. If callers observe an empty
    // lexeme on a token that should have text, it indicates a
    // piners-syntax span invariant violation.
    code.get(start..end).unwrap_or("")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_token_locations() {
        let lexed = piners_syntax::lex_result("indicator(\"x\")\n");
        assert!(
            lexed.diagnostics.is_empty(),
            "clean source has no lex errors"
        );
        let first = lexed.tokens.first().expect("token");
        assert_eq!(token_text("indicator(\"x\")\n", first), "indicator");
        assert_eq!(first.source_span(&lexed.linebreaks).start.line, 1);
    }

    #[test]
    fn invalid_utf8_boundary_returns_empty_lexeme() {
        // "e" in a multibyte context: span 5..6 cuts inside the 2-byte UTF-8 sequence for 'e'
        // but we want to test the non-char-boundary path. Use a 2-byte codepoint so the
        // byte range 5..6 is not a valid char boundary, which causes `get` to return None.
        let code = "x = \"\u{00e9}\""; // é is 2 bytes; bytes 5..6 split inside it
        let token = piners_syntax::Token::new(
            piners_syntax::TokenKind::StringLit(String::new()),
            piners_syntax::Span::new(5, 6),
        );
        assert_eq!(token_text(code, &token), "");
    }

    #[test]
    fn lex_error_still_produces_tokens() {
        // "x @\ny" - the stray `@` is a lex error but `x` and `y` should survive.
        let lexed = piners_syntax::lex_result("x @\ny");
        assert!(
            !lexed.diagnostics.is_empty(),
            "should have at least one lex error"
        );
        let idents: Vec<&str> = lexed
            .tokens
            .iter()
            .filter_map(|t| {
                if let piners_syntax::TokenKind::Ident(name) = &t.kind {
                    Some(name.as_str())
                } else {
                    None
                }
            })
            .collect();
        assert!(idents.contains(&"x"), "x token recovered");
        assert!(idents.contains(&"y"), "y token recovered");
    }

    #[test]
    fn json_output_carries_errors_field() {
        let lexed = piners_syntax::lex_result("x @\ny");
        let errors: Vec<LexErrorOut> = lexed
            .diagnostics
            .iter()
            .map(|d| {
                let loc = d.span.source_span(&lexed.linebreaks);
                LexErrorOut {
                    code: d.code.as_deref().map(str::to_owned),
                    message: d.message.clone(),
                    line: loc.start.line,
                    column: loc.start.column,
                }
            })
            .collect();
        assert!(!errors.is_empty(), "errors field has at least one entry");
        assert_eq!(
            errors[0].code.as_deref(),
            Some("PINE0107"),
            "unexpected-char code"
        );
    }

    #[test]
    fn clean_input_produces_no_errors() {
        let lexed = piners_syntax::lex_result("x = 1\n");
        assert!(
            lexed.diagnostics.is_empty(),
            "clean source produces empty errors list"
        );
        let has_ident = lexed
            .tokens
            .iter()
            .any(|t| matches!(&t.kind, piners_syntax::TokenKind::Ident(n) if n == "x"));
        assert!(has_ident, "x token present in clean output");
    }
}
