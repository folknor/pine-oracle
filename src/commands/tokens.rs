use anyhow::Result;
use pine_cli::syntax;

use crate::output::{ResolvedFormat, print_json};

pub(crate) fn run(code: &str, format: ResolvedFormat) -> Result<()> {
    let mut lexer = syntax::Lexer::new(code);
    let tokens = lexer
        .tokenize()
        .map_err(|e| anyhow::anyhow!("lex error: {e}"))?;

    match format {
        ResolvedFormat::Json => {
            print_json(&tokens)?;
        }
        ResolvedFormat::Text => {
            for t in &tokens {
                println!("{:>4}:{:<3}  {:?}  {:?}", t.line, t.column, t.typ, t.lexeme);
            }
        }
    }
    Ok(())
}
