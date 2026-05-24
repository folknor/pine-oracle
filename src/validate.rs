// Validator.
//
// Local tier: `check(code)` runs the lifted lexer + parser, surfaces the
// first failure as a structured `Diagnostic`. Single-element today because
// the lifted parser bails on the first error; multi-error recovery is
// future work in src/syntax/parser.rs.
//
// Strict tier: `strict(code)` POSTs the source to TradingView's
// pine-lint endpoint and maps every error + warning back to a Diagnostic.
// No auth, no on-disk cache (cross-invocation caching has nowhere to live
// given pine-oracle's zero-on-disk contract). Network call is synchronous
// via ureq; default 10-second timeout.

use std::time::Duration;

use anyhow::{anyhow, bail};
use serde::{Deserialize, Serialize};

use crate::syntax::{Lexer, LexerError, Parser, ParserError};

const PINE_LINT_URL: &str =
    "https://pine-facade.tradingview.com/pine-facade/translate_light?user_name=admin&v=3";

const USER_AGENT: &str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) \
    AppleWebKit/537.36 (KHTML, like Gecko) Chrome/138.0.0.0 Safari/537.36";

const STRICT_TIMEOUT_SECS: u64 = 10;

// Fixed boundary for multipart/form-data. The Pine-lint endpoint expects
// multipart (httpx `files=` in the original Python, FormData in the TS port);
// urlencoded silently fails on some payloads. Boundary value is arbitrary as
// long as it does not appear in the body; an 8-byte random hex suffix keeps
// collision risk astronomical without dragging in a UUID crate.
const MULTIPART_BOUNDARY: &str = "----pine_oracle_boundary_8eb4f1c6a39d2710";

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Error,
    Warning,
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Stage {
    Lex,
    Parse,
    Strict,
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

// ---------- strict tier (TV pine-lint over HTTPS) ----------

#[derive(Debug, Deserialize)]
struct TvPosition {
    line: usize,
    column: usize,
}

#[derive(Debug, Deserialize)]
struct TvDiagnostic {
    start: TvPosition,
    #[serde(default)]
    #[allow(dead_code)]
    end: Option<TvPosition>,
    message: String,
}

#[derive(Debug, Default, Deserialize)]
struct TvResult {
    #[serde(default)]
    errors: Vec<TvDiagnostic>,
    #[serde(default)]
    warnings: Vec<TvDiagnostic>,
}

#[derive(Debug, Deserialize)]
struct TvResponse {
    #[serde(default)]
    success: bool,
    #[serde(default)]
    error: Option<String>,
    #[serde(default)]
    result: Option<TvResult>,
}

/// Send `code` to TradingView's pine-lint endpoint and return a `Report`
/// summarising every error + warning the API reports. Returns Err only on
/// transport / parse failures (network, timeout, malformed JSON); script
/// errors land in the `Report::diagnostics` vec.
pub fn strict(code: &str) -> anyhow::Result<Report> {
    let agent = ureq::AgentBuilder::new()
        .timeout(Duration::from_secs(STRICT_TIMEOUT_SECS))
        .build();
    let body = build_multipart_body(code);
    let content_type = format!("multipart/form-data; boundary={MULTIPART_BOUNDARY}");
    let resp = agent
        .post(PINE_LINT_URL)
        .set("Referer", "https://www.tradingview.com/")
        .set("User-Agent", USER_AGENT)
        .set("DNT", "1")
        .set("Content-Type", &content_type)
        .send_bytes(body.as_bytes())
        .map_err(|e| anyhow!("pine-lint request failed: {e}"))?;
    let response_body = resp
        .into_string()
        .map_err(|e| anyhow!("reading pine-lint body: {e}"))?;
    parse_strict_response(&response_body)
}

fn build_multipart_body(code: &str) -> String {
    // Standard RFC 7578 form-data part: CRLF separators, a Content-Disposition
    // header naming the field, a blank line, the value, then the closing
    // boundary marker (`--BOUNDARY--`).
    format!(
        "--{MULTIPART_BOUNDARY}\r\n\
         Content-Disposition: form-data; name=\"source\"\r\n\
         \r\n\
         {code}\r\n\
         --{MULTIPART_BOUNDARY}--\r\n"
    )
}

fn parse_strict_response(body: &str) -> anyhow::Result<Report> {
    let resp: TvResponse = serde_json::from_str(body)
        .map_err(|e| anyhow!("parsing pine-lint JSON: {e}; body: {body}"))?;
    if !resp.success {
        bail!(
            "TV pine-lint reported transport failure: {}",
            resp.error.unwrap_or_else(|| "no error message".to_string())
        );
    }
    let result = resp.result.unwrap_or_default();
    let mut diagnostics = Vec::with_capacity(result.errors.len() + result.warnings.len());
    for e in result.errors {
        diagnostics.push(Diagnostic {
            severity: Severity::Error,
            stage: Stage::Strict,
            message: e.message,
            line: e.start.line,
            column: Some(e.start.column),
        });
    }
    for w in result.warnings {
        diagnostics.push(Diagnostic {
            severity: Severity::Warning,
            stage: Stage::Strict,
            message: w.message,
            line: w.start.line,
            column: Some(w.start.column),
        });
    }
    let ok = !diagnostics
        .iter()
        .any(|d| matches!(d.severity, Severity::Error));
    Ok(Report { ok, diagnostics })
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

    #[test]
    fn parses_strict_response_with_error_and_warning() {
        let body = r#"{
            "success": true,
            "result": {
                "errors": [
                    {
                        "start": {"line": 3, "column": 5},
                        "end": {"line": 3, "column": 9},
                        "message": "undeclared identifier `foo`"
                    }
                ],
                "warnings": [
                    {
                        "start": {"line": 7, "column": 1},
                        "end": {"line": 7, "column": 4},
                        "message": "implicit conversion"
                    }
                ]
            }
        }"#;
        let report = parse_strict_response(body).expect("must parse");
        assert!(!report.ok, "errors present => ok=false");
        assert_eq!(report.diagnostics.len(), 2);
        assert!(matches!(report.diagnostics[0].severity, Severity::Error));
        assert!(matches!(report.diagnostics[0].stage, Stage::Strict));
        assert_eq!(report.diagnostics[0].line, 3);
        assert_eq!(report.diagnostics[0].column, Some(5));
        assert!(matches!(report.diagnostics[1].severity, Severity::Warning));
    }

    #[test]
    fn parses_clean_strict_response() {
        let body = r#"{"success": true, "result": {"errors": [], "warnings": []}}"#;
        let report = parse_strict_response(body).expect("must parse");
        assert!(report.ok);
        assert!(report.diagnostics.is_empty());
    }

    #[test]
    fn warnings_only_keep_ok_true() {
        let body = r#"{
            "success": true,
            "result": {
                "errors": [],
                "warnings": [{
                    "start": {"line": 1, "column": 1},
                    "message": "stylistic"
                }]
            }
        }"#;
        let report = parse_strict_response(body).expect("must parse");
        assert!(report.ok, "warnings without errors => ok=true");
        assert_eq!(report.diagnostics.len(), 1);
        assert!(matches!(report.diagnostics[0].severity, Severity::Warning));
    }

    #[test]
    fn multipart_body_shape() {
        let body = build_multipart_body("indicator(\"x\")\n");
        assert!(body.starts_with("--"));
        assert!(body.contains("Content-Disposition: form-data; name=\"source\""));
        assert!(body.contains("indicator(\"x\")\n"));
        assert!(body.ends_with("--\r\n"));
        // CRLF, not LF.
        assert!(body.contains("\r\n\r\n"));
    }

    #[test]
    fn transport_failure_returns_err() {
        let body = r#"{"success": false, "error": "rate limited"}"#;
        let err = parse_strict_response(body).expect_err("transport failure must Err");
        assert!(err.to_string().contains("rate limited"));
    }
}
