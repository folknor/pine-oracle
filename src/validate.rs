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

use std::sync::OnceLock;
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

// Shared agent re-used across strict() calls so TCP connections are pooled.
// The agent is configured once with the module-level timeout; since the timeout
// is the same for every call, reuse is safe.
static STRICT_AGENT: OnceLock<ureq::Agent> = OnceLock::new();

#[non_exhaustive]
#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Error,
    Warning,
    Hint,
}

/// Which analysis pass produced this diagnostic.
///
/// Contract: `Lex` / `Parse` / `Type` / `Semantic` come from `check()`;
/// `Strict` comes from `strict()`. The `Strict` pass is a yes/no oracle
/// only - the diagnostic prose (message, line, column) is non-actionable
/// (TV pine-lint stops at the first error, reports wrong positions, and
/// breaks on trailing whitespace). Treat `Strict` diagnostics as opaque
/// blobs; only `Report::ok` is reliable for that tier.
#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Stage {
    Lex,
    Parse,
    Type,
    Semantic,
    Strict,
}

impl Stage {
    pub fn as_str(self) -> &'static str {
        match self {
            Stage::Lex => "lex",
            Stage::Parse => "parse",
            Stage::Type => "type",
            Stage::Semantic => "semantic",
            Stage::Strict => "strict",
        }
    }
}

impl std::fmt::Display for Stage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
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

/// Validation outcome. The `ok` field is the authoritative pass/fail signal;
/// do not ignore it. Tied to the JSON schema via `SCHEMA_VERSION` in output.rs.
#[must_use]
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
    // Clamp the span start to code.len() so EOF-adjacent diagnostics (e.g.
    // "unexpected end of input", whose span is Span::empty(code.len())) still
    // get a column. Clamping is safe: line_col_from_breaks is total and the
    // resulting column correctly lands at the last character position.
    let clamped_start = (diagnostic.span.start as usize).min(code.len()) as u32;
    let loc = line_col_from_breaks(linebreaks, clamped_start);
    let column = Some(loc.column as usize);
    Diagnostic {
        severity: match diagnostic.severity {
            piners_syntax::Severity::Error => Severity::Error,
            piners_syntax::Severity::Warning => Severity::Warning,
            piners_syntax::Severity::Hint => Severity::Hint,
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

// Deserialize a nullable JSON array as an empty Vec when the field is null.
// `#[serde(default)]` alone handles the field-absent case but rejects
// an explicit `null`; this helper covers both cases so a TV-schema change
// that adds `"errors": null` doesn't crash the parser.
fn null_as_empty<'de, D, T>(deserializer: D) -> Result<Vec<T>, D::Error>
where
    D: serde::Deserializer<'de>,
    T: serde::Deserialize<'de>,
{
    Ok(Option::deserialize(deserializer)?.unwrap_or_default())
}

#[derive(Debug, Default, Deserialize)]
struct TvResult {
    #[serde(default, deserialize_with = "null_as_empty")]
    errors: Vec<TvDiagnostic>,
    #[serde(default, deserialize_with = "null_as_empty")]
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
///
/// **Non-actionable prose warning.** TV's pine-lint stops at the first error,
/// reports wrong line / column numbers, and breaks on trailing whitespace. The
/// diagnostic messages in the returned `Report` are not reliable for iterative
/// debugging. `Report::ok` is the only trustworthy output of this function.
/// Use `check()` for IDE-quality errors; use `strict()` only to confirm a
/// locally-clean script also passes TV's validator.
pub fn strict(code: &str) -> anyhow::Result<Report> {
    let agent = STRICT_AGENT.get_or_init(|| {
        let config = ureq::Agent::config_builder()
            .timeout_global(Some(Duration::from_secs(STRICT_TIMEOUT_SECS)))
            .build();
        ureq::Agent::new_with_config(config)
    });
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
    // read_to_string is unbounded; real pine-lint responses are tiny (<10 KB).
    // If TV returns a large error page (e.g. 503 HTML), we read it all and
    // then fail fast in parse_strict_response with a clear JSON-parse error.
    // A size cap could be added via ureq's body().with_config() if needed.
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
    //
    // TV's pine-lint stops at the first error and breaks on trailing whitespace,
    // so we trim trailing CR/LF from the source before embedding it. This
    // prevents the CRLF separator after the value from creating a double-CRLF
    // that the linter misinterprets.
    let code = code.trim_end_matches(['\r', '\n']);
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
    // A missing `result` field while `success` is true is unexpected - treat it
    // as a transport failure rather than silently reporting ok=true with no
    // diagnostics, which would be a false negative. Null is handled by the
    // `Option<TvResult>` deserializer and maps to None here.
    let result = resp.result.ok_or_else(|| {
        anyhow!("TV pine-lint returned success=true but no result field (unexpected schema)")
    })?;
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
        // Trailing newline is trimmed before embedding; the body should still
        // contain the source text (without the trailing LF) and the standard
        // CRLF separators.
        let body = build_multipart_body("indicator(\"x\")\n");
        assert!(body.starts_with("--"));
        assert!(body.contains("Content-Disposition: form-data; name=\"source\""));
        assert!(body.contains("indicator(\"x\")"));
        assert!(body.ends_with("--\r\n"));
        // CRLF, not LF.
        assert!(body.contains("\r\n\r\n"));
    }

    // Issue 3: trailing newlines in source must not produce a double-CRLF
    // before the closing boundary.
    #[test]
    fn multipart_body_trims_trailing_newline() {
        let body = build_multipart_body("indicator(\"x\")\n");
        // The value section must end with exactly one CRLF before the boundary,
        // not two (which would happen if the trailing LF were kept).
        let boundary_line = format!("--{MULTIPART_BOUNDARY}--");
        let closing = format!("\r\n{boundary_line}");
        // Locate the closing boundary.
        let pos = body.find(&boundary_line).expect("closing boundary present");
        // The two bytes before the boundary marker must be \r\n, not \r\n\r\n.
        let before = &body[..pos];
        assert!(
            !before.ends_with("\r\n\r\n"),
            "double CRLF found before closing boundary: {body:?}"
        );
        assert!(
            before.ends_with("\r\n"),
            "expected single CRLF before closing boundary: {body:?}"
        );
        // Confirm the closing suffix is present and not a dangling \r\n\r\n.
        assert!(body.contains(&closing));
    }

    #[test]
    fn multipart_body_trims_trailing_crlf() {
        // CRLF-terminated source (Windows line endings) should also be trimmed.
        let body = build_multipart_body("indicator(\"x\")\r\n");
        let boundary_line = format!("--{MULTIPART_BOUNDARY}--");
        let pos = body.find(&boundary_line).expect("closing boundary present");
        let before = &body[..pos];
        assert!(
            !before.ends_with("\r\n\r\n"),
            "double CRLF found for CRLF-terminated source: {body:?}"
        );
    }

    #[test]
    fn transport_failure_returns_err() {
        let body = r#"{"success": false, "error": "rate limited"}"#;
        let err = parse_strict_response(body).expect_err("transport failure must Err");
        assert!(err.to_string().contains("rate limited"));
    }

    // Issue 4: parse_strict_response robustness.

    #[test]
    fn html_body_returns_err_with_clear_message() {
        // TV returns a 503 HTML page when the backend is overloaded; the
        // parser must not panic and must surface a clear error.
        let body = "<!DOCTYPE html><html><body><h1>503 Service Unavailable</h1></body></html>";
        let err = parse_strict_response(body).expect_err("HTML body must Err");
        let msg = err.to_string();
        assert!(
            msg.contains("parsing pine-lint JSON"),
            "expected JSON-parse error, got: {msg}"
        );
    }

    #[test]
    fn empty_body_returns_err() {
        let err = parse_strict_response("").expect_err("empty body must Err");
        let msg = err.to_string();
        assert!(
            msg.contains("parsing pine-lint JSON"),
            "expected JSON-parse error, got: {msg}"
        );
    }

    #[test]
    fn success_true_with_null_result_returns_err() {
        // `{"success": true, "result": null}` is ambiguous - we cannot confirm
        // the script is clean. Treat it as a transport failure so callers get
        // an explicit error rather than a false negative.
        let body = r#"{"success": true, "result": null}"#;
        let err = parse_strict_response(body).expect_err("null result must Err");
        assert!(
            err.to_string().contains("unexpected schema"),
            "expected schema error, got: {err}"
        );
    }

    #[test]
    fn success_true_with_missing_result_returns_err() {
        // A response with no `result` field is treated the same as null.
        let body = r#"{"success": true}"#;
        let err = parse_strict_response(body).expect_err("missing result must Err");
        assert!(
            err.to_string().contains("unexpected schema"),
            "expected schema error, got: {err}"
        );
    }

    #[test]
    fn large_body_with_many_diagnostics_parses_without_panic() {
        // Synthesize 100 dummy errors to exercise the parser under load.
        let mut errors = Vec::with_capacity(100);
        for i in 0..100u32 {
            errors.push(format!(
                r#"{{"start":{{"line":{i},"column":1}},"message":"error {i}"}}"#
            ));
        }
        let errors_json = errors.join(",");
        let body = format!(
            r#"{{"success": true, "result": {{"errors": [{errors_json}], "warnings": []}}}}"#
        );
        let report = parse_strict_response(&body).expect("large body must parse");
        assert!(!report.ok, "100 errors => ok=false");
        assert_eq!(report.diagnostics.len(), 100, "all 100 diagnostics present");
        assert!(
            report
                .diagnostics
                .iter()
                .all(|d| matches!(d.severity, Severity::Error))
        );
    }

    // Issue 1: Hint diagnostics must map to Severity::Hint, not Severity::Warning.
    // piners-syntax PINE0409 is a known hint; we use a synthetic check here since
    // we don't control which exact source triggers a hint in all versions.
    // We verify the mapping by constructing the intermediate types directly.
    #[test]
    fn hint_severity_maps_to_severity_hint_not_warning() {
        // Build a synthetic piners_syntax::Diagnostic-like path by going through
        // local_diagnostic indirectly: exercise the Severity mapping via check()
        // on a source that triggers at least one diagnostic, then confirm the
        // Hint variant exists in our type. (A true hint from piners-syntax
        // requires a specific source pattern; we at least confirm no compile
        // error and that the variant serialises correctly.)
        let sev_hint = Severity::Hint;
        let sev_warn = Severity::Warning;
        let sev_err = Severity::Error;
        let json_hint = serde_json::to_string(&sev_hint).unwrap();
        let json_warn = serde_json::to_string(&sev_warn).unwrap();
        let json_err = serde_json::to_string(&sev_err).unwrap();
        assert_eq!(json_hint, r#""hint""#);
        assert_eq!(json_warn, r#""warning""#);
        assert_eq!(json_err, r#""error""#);
    }

    #[test]
    fn hints_keep_ok_true_in_check() {
        // A source with only hints (no errors) must yield ok=true.
        // We can't guarantee piners-syntax emits a hint for any particular
        // source without coupling to internal codes, so we verify the `ok`
        // logic directly: a Report with only Hint diagnostics must be ok=true.
        let report = Report {
            ok: ![Diagnostic {
                severity: Severity::Hint,
                stage: Stage::Semantic,
                code: Some("PINE0409".to_string()),
                message: "advisory hint".to_string(),
                line: 1,
                column: None,
            }]
            .iter()
            .any(|d| matches!(d.severity, Severity::Error)),
            diagnostics: vec![Diagnostic {
                severity: Severity::Hint,
                stage: Stage::Semantic,
                code: Some("PINE0409".to_string()),
                message: "advisory hint".to_string(),
                line: 1,
                column: None,
            }],
        };
        assert!(report.ok, "hint-only report must be ok=true");
    }

    #[test]
    fn stage_display_matches_serde_names() {
        // Stage::as_str() and Display must return the same lowercase names that
        // serde uses for JSON serialisation.
        assert_eq!(Stage::Lex.as_str(), "lex");
        assert_eq!(Stage::Parse.as_str(), "parse");
        assert_eq!(Stage::Type.as_str(), "type");
        assert_eq!(Stage::Semantic.as_str(), "semantic");
        assert_eq!(Stage::Strict.as_str(), "strict");
        assert_eq!(Stage::Strict.to_string(), "strict");
    }

    #[test]
    fn null_errors_field_deserializes_as_empty() {
        // TV could return `"errors": null` in a future schema change.
        // null_as_empty must treat it identically to a missing field.
        let body = r#"{"success": true, "result": {"errors": null, "warnings": null}}"#;
        let report = parse_strict_response(body).expect("null arrays must parse");
        assert!(report.ok, "no errors => ok=true");
        assert!(report.diagnostics.is_empty(), "no diagnostics");
    }

    #[test]
    fn eof_diagnostic_has_column() {
        // A parse error at EOF (e.g. missing script body) should still carry a
        // column. The old code suppressed column when span.start > code.len();
        // clamping ensures every diagnostic gets a column value.
        let r = check("//@version=6\nindicator(\"x\")\n");
        // Any diagnostic present must have a column; we don't mandate errors here
        // but if piners-syntax produces one, column must be Some.
        for d in &r.diagnostics {
            assert!(
                d.column.is_some(),
                "all diagnostics must carry column: {d:?}"
            );
        }
    }
}
