use anyhow::Result;
use serde::Serialize;

use crate::output::{ResolvedFormat, print_json};

#[derive(Serialize)]
struct TokenOut<'a> {
    typ: &'a piners_syntax::TokenKind,
    lexeme: &'a str,
    line: u32,
    column: u32,
}

pub(crate) fn run(code: &str, format: ResolvedFormat) -> Result<()> {
    let lexed =
        piners_syntax::lex_with_linebreaks(code).map_err(|e| anyhow::anyhow!("lex error: {e}"))?;
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

    match format {
        ResolvedFormat::Json => {
            print_json(&tokens)?;
        }
        ResolvedFormat::Text => {
            for token in &tokens {
                println!(
                    "{:>4}:{:<3}  {:?}  {:?}",
                    token.line, token.column, token.typ, token.lexeme
                );
            }
        }
    }
    Ok(())
}

fn token_text<'a>(code: &'a str, token: &piners_syntax::Token) -> &'a str {
    let start = token.span.start as usize;
    let end = token.span.end as usize;
    code.get(start..end).unwrap_or("")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_token_locations() {
        let lexed = piners_syntax::lex_with_linebreaks("indicator(\"x\")\n").expect("lex");
        let first = lexed.tokens.first().expect("token");
        assert_eq!(token_text("indicator(\"x\")\n", first), "indicator");
        assert_eq!(first.source_span(&lexed.linebreaks).start.line, 1);
    }

    #[test]
    fn invalid_utf8_boundary_returns_empty_lexeme() {
        let code = "x = \"é\"";
        let token = piners_syntax::Token::new(
            piners_syntax::TokenKind::StringLit(String::new()),
            piners_syntax::Span::new(5, 6),
        );
        assert_eq!(token_text(code, &token), "");
    }
}
