// Validator.
//
// Local tier: `check(code)` runs piners-syntax's lex / parse / type /
// semantic pipeline and returns every diagnostic it can recover.
//
// Strict tier: `strict(code)` POSTs the source to TradingView's
// pine-lint endpoint and maps every error + warning back to a Diagnostic.
// No auth, no on-disk cache (cross-invocation caching has nowhere to live
// given pine-oracle's zero-on-disk contract). Network call is synchronous
// via ureq; default 10-second timeout.

use std::time::Duration;

use anyhow::{anyhow, bail};
use piners_syntax::line_col_from_breaks;
use serde::{Deserialize, Serialize};

use crate::behavior;

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
    Type,
    Semantic,
    Strict,
}

#[derive(Debug, Clone, Serialize)]
pub struct Diagnostic {
    pub severity: Severity,
    pub stage: Stage,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
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

/// Validate Pine v6 source. Empty source is invalid under the piners-syntax
/// contract because a script header is required. Returns a `Report` with an
/// empty `diagnostics` vec on success.
pub fn check(code: &str) -> Report {
    let report = piners_syntax::validate(code, behavior::syntax_builtins());
    let linebreaks = report
        .ast
        .as_ref()
        .map_or_else(|| linebreaks(code), |ast| ast.linebreaks.clone());
    let diagnostics = report
        .diagnostics
        .iter()
        .map(|diagnostic| local_diagnostic(code, &linebreaks, diagnostic))
        .collect::<Vec<_>>();
    let ok = !diagnostics
        .iter()
        .any(|diagnostic| matches!(diagnostic.severity, Severity::Error));
    Report { ok, diagnostics }
}

fn local_diagnostic(
    code: &str,
    linebreaks: &[u32],
    diagnostic: &piners_syntax::Diagnostic,
) -> Diagnostic {
    let loc = line_col_from_breaks(linebreaks, diagnostic.span.start);
    let column = if diagnostic.span.start as usize <= code.len() {
        Some(loc.column as usize)
    } else {
        None
    };
    Diagnostic {
        severity: match diagnostic.severity {
            piners_syntax::Severity::Error => Severity::Error,
            piners_syntax::Severity::Warning | piners_syntax::Severity::Hint => Severity::Warning,
        },
        stage: match diagnostic.stage {
            piners_syntax::Stage::Lex => Stage::Lex,
            piners_syntax::Stage::Parse => Stage::Parse,
            piners_syntax::Stage::Type => Stage::Type,
            piners_syntax::Stage::Semantic => Stage::Semantic,
        },
        code: diagnostic
            .code
            .as_ref()
            .map(|code| code.as_str().to_string()),
        message: diagnostic.message.clone(),
        line: loc.line as usize,
        column,
    }
}

fn linebreaks(code: &str) -> Vec<u32> {
    code.bytes()
        .enumerate()
        .filter_map(|(idx, byte)| (byte == b'\n').then_some(idx as u32))
        .collect()
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
    let config = ureq::Agent::config_builder()
        .timeout_global(Some(Duration::from_secs(STRICT_TIMEOUT_SECS)))
        .build();
    let agent = ureq::Agent::new_with_config(config);
    let body = build_multipart_body(code);
    let content_type = format!("multipart/form-data; boundary={MULTIPART_BOUNDARY}");
    let mut response = agent
        .post(PINE_LINT_URL)
        .header("Referer", "https://www.tradingview.com/")
        .header("User-Agent", USER_AGENT)
        .header("DNT", "1")
        .header("Content-Type", &content_type)
        .send(body.as_bytes())
        .map_err(|e| anyhow!("pine-lint request failed: {e}"))?;
    let response_body = response
        .body_mut()
        .read_to_string()
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
            code: None,
            message: e.message,
            line: e.start.line,
            column: Some(e.start.column),
        });
    }
    for w in result.warnings {
        diagnostics.push(Diagnostic {
            severity: Severity::Warning,
            stage: Stage::Strict,
            code: None,
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
    fn empty_source_is_parse_error() {
        let r = check("");
        assert!(!r.ok);
        assert!(
            r.diagnostics
                .iter()
                .any(|d| matches!(d.stage, Stage::Parse))
        );
    }

    #[test]
    fn simple_indicator_is_ok() {
        let r = check("//@version=6\nindicator(\"x\")\nx = 1\n");
        assert!(r.ok, "expected ok, got {:?}", r.diagnostics);
    }

    #[test]
    fn unterminated_string_is_lex_error() {
        let r = check("//@version=6\nindicator(\"x\")\nx = \"hello\n");
        assert!(!r.ok);
        let d = r
            .diagnostics
            .iter()
            .find(|d| matches!(d.stage, Stage::Lex))
            .expect("lex diagnostic");
        assert!(matches!(d.stage, Stage::Lex));
        assert!(d.column.is_some(), "lex error should carry column");
    }

    #[test]
    fn unexpected_token_is_parse_error() {
        let r = check("//@version=6\nindicator(\"x\")\nx = + +\n");
        assert!(!r.ok);
        let d = r
            .diagnostics
            .iter()
            .find(|d| matches!(d.stage, Stage::Parse))
            .expect("parse diagnostic");
        assert!(matches!(d.stage, Stage::Parse));
        assert!(d.line >= 1);
    }

    #[test]
    fn local_validation_includes_type_and_semantic_diagnostics() {
        let r = check("//@version=6\nindicator(\"x\")\nint value = \"bad\"\nvalue == na\n");
        assert!(!r.ok);
        assert!(
            r.diagnostics.iter().any(|d| matches!(d.stage, Stage::Type)),
            "{:?}",
            r.diagnostics
        );
        assert!(
            r.diagnostics
                .iter()
                .any(|d| matches!(d.stage, Stage::Semantic)),
            "{:?}",
            r.diagnostics
        );
    }

    #[test]
    fn local_validation_accepts_bool_plotshape_and_plotchar_series() {
        let r = check(
            "//@version=6\nindicator(\"x\")\ncondition = close > open\nplotshape(condition)\nplotchar(condition)\n",
        );
        assert!(r.ok, "expected ok, got {:?}", r.diagnostics);
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
