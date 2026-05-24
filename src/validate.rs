// Local-tier validator. Runs the lifted lexer + parser, surfaces the first
// lex or parse failure as a structured `Diagnostic`. The strict tier
// (`validate --strict`) belongs here later; it will shell to TradingView's
// pine-lint endpoint, no auth, no on-disk cache.
//
// The lifted parser stops at the first error rather than continuing with
// recovery, so for now the diagnostic list is either empty or single-element.
// A multi-error mode is future work that would touch the lifted parser.

use serde::Serialize;

use crate::syntax::{Lexer, LexerError, Parser, ParserError};

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Error,
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Stage {
    Lex,
    Parse,
}

#[derive(Debug, Clone, Serialize)]
pub struct Diagnostic {
    pub severity: Severity,
    pub stage: Stage,
    pub message: String,
    pub line: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub column: Option<usize>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Report {
    pub ok: bool,
    pub diagnostics: Vec<Diagnostic>,
}

/// Validate Pine v6 source. Returns a `Report` with an empty `diagnostics`
/// vec on success, or a single-element vec describing the first lex/parse
/// failure.
pub fn check(code: &str) -> Report {
    let mut lexer = Lexer::new(code);
    let tokens = match lexer.tokenize() {
        Ok(tokens) => tokens,
        Err(e) => {
            return Report {
                ok: false,
                diagnostics: vec![lex_diagnostic(&e)],
            };
        }
    };

    let mut parser = Parser::new(tokens);
    if let Err(e) = parser.parse() {
        return Report {
            ok: false,
            diagnostics: vec![parse_diagnostic(&e)],
        };
    }

    Report {
        ok: true,
        diagnostics: Vec::new(),
    }
}

fn lex_diagnostic(e: &LexerError) -> Diagnostic {
    let (line, column) = match e {
        LexerError::UnterminatedString { line, column } => (*line, Some(*column)),
        LexerError::InvalidHexColor { line, column, .. } => (*line, Some(*column)),
        LexerError::UnexpectedCharacter { line, column, .. } => (*line, Some(*column)),
        LexerError::InvalidNumber { line, column, .. } => (*line, Some(*column)),
        LexerError::IndentationError { line } => (*line, None),
    };
    Diagnostic {
        severity: Severity::Error,
        stage: Stage::Lex,
        message: e.to_string(),
        line,
        column,
    }
}

fn parse_diagnostic(e: &ParserError) -> Diagnostic {
    let line = match e {
        ParserError::UnexpectedToken(_, line) => *line,
        ParserError::ExpectedToken { line, .. } => *line,
        ParserError::ExpectedVariableName(line) => *line,
        ParserError::ExpectedParameterName(line) => *line,
        ParserError::InvalidCallTarget(line) => *line,
        ParserError::ExpectedIdentifierAfterDot(line) => *line,
    };
    Diagnostic {
        severity: Severity::Error,
        stage: Stage::Parse,
        message: e.to_string(),
        line,
        column: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_source_is_ok() {
        let r = check("");
        assert!(r.ok);
        assert!(r.diagnostics.is_empty());
    }

    #[test]
    fn simple_var_decl_is_ok() {
        let r = check("x = 1\n");
        assert!(r.ok, "expected ok, got {:?}", r.diagnostics);
    }

    #[test]
    fn unterminated_string_is_lex_error() {
        let r = check("x = \"hello\n");
        assert!(!r.ok);
        assert_eq!(r.diagnostics.len(), 1);
        let d = &r.diagnostics[0];
        assert!(matches!(d.stage, Stage::Lex));
        assert!(d.column.is_some(), "lex error should carry column");
    }

    #[test]
    fn unexpected_token_is_parse_error() {
        let r = check("x = + +\n");
        assert!(!r.ok);
        assert_eq!(r.diagnostics.len(), 1);
        let d = &r.diagnostics[0];
        assert!(matches!(d.stage, Stage::Parse));
        assert!(d.line >= 1);
    }
}
