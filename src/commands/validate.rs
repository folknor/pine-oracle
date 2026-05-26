use std::io::Write as _;

use anyhow::Result;
use pine_oracle::validate;

use crate::output::{ResolvedFormat, Style, print_json};

pub(crate) fn run(
    source: &str,
    strict: bool,
    format: ResolvedFormat,
    style: Style,
    quiet: bool,
) -> Result<()> {
    let report = if strict {
        validate::strict(source)?
    } else {
        validate::check(source)
    };
    match format {
        ResolvedFormat::Json => {
            print_json(&report)?;
        }
        ResolvedFormat::Text => {
            if report.diagnostics.is_empty() {
                if !quiet {
                    println!("{}", style.cyan("ok"));
                }
            } else {
                if strict && !quiet {
                    eprintln!(
                        "note: TV's pine-lint diagnostics are non-actionable - first error only,"
                    );
                    eprintln!(
                        "      wrong line/column numbers, breaks on trailing whitespace. The"
                    );
                    eprintln!(
                        "      success bit is the only trustworthy signal. Use `pine validate`"
                    );
                    eprintln!("      (no --strict) for IDE-quality errors when iterating.");
                }
                for d in &report.diagnostics {
                    // Wildcard arm exists because `validate::Severity` is
                    // `#[non_exhaustive]` across crate boundaries; future
                    // variants render as cyan placeholders until classified.
                    #[allow(clippy::match_same_arms)]
                    let sev = match d.severity {
                        validate::Severity::Error => style.red("error"),
                        validate::Severity::Warning => style.yellow("warning"),
                        validate::Severity::Hint => style.cyan("hint"),
                        _ => style.cyan("hint"),
                    };
                    let stage = style.dim(&format!("[{}]", d.stage));
                    let loc = match d.column {
                        Some(col) => format!("{}:{}", d.line, col),
                        None => format!("{}", d.line),
                    };
                    let diag_code = d.code.as_deref().map_or(String::new(), |code| {
                        format!("{} ", style.dim(&format!("{code}:")))
                    });
                    println!(
                        "{sev}{stage} {}: {diag_code}{}",
                        style.bold(&loc),
                        d.message
                    );
                    // Skip caret frame for Strict diagnostics: TV's pine-lint
                    // reports wrong line / column numbers by design, so
                    // rendering a frame would visually claim accuracy that
                    // the tier explicitly disavows.
                    if !matches!(d.stage, validate::Stage::Strict)
                        && let Some(frame) = diagnostic_frame(source, d)
                    {
                        print_diagnostic_frame(&frame, style);
                    }
                }
                if report.ok && !quiet {
                    let has_warn = report
                        .diagnostics
                        .iter()
                        .any(|d| matches!(d.severity, validate::Severity::Warning));
                    if has_warn {
                        println!("{}", style.yellow("ok (warnings only)"));
                    } else {
                        println!("{}", style.cyan("ok (hints only)"));
                    }
                }
            }
        }
    }
    if !report.ok {
        // Flush stdout before process::exit so block-buffered output
        // (e.g. piped to a file) is not truncated. Ignore flush errors;
        // we are exiting anyway.
        std::io::stdout().flush().ok();
        std::process::exit(1);
    }
    Ok(())
}

#[derive(Debug, PartialEq, Eq)]
struct DiagnosticFrame {
    line: usize,
    text: String,
    marker: String,
}

fn diagnostic_frame(code: &str, diagnostic: &validate::Diagnostic) -> Option<DiagnosticFrame> {
    let column = diagnostic.column?;
    let text = code.lines().nth(diagnostic.line.checked_sub(1)?)?;
    let marker = caret_marker(text, column);
    Some(DiagnosticFrame {
        line: diagnostic.line,
        text: text.to_string(),
        marker,
    })
}

fn caret_marker(line: &str, column: usize) -> String {
    // piners-syntax reports one-based byte columns (spans are byte offsets).
    // Convert the byte prefix back to chars before building the visual marker.
    let byte_index = column.max(1).saturating_sub(1).min(line.len());
    let boundary = if line.is_char_boundary(byte_index) {
        byte_index
    } else {
        (0..byte_index)
            .rev()
            .find(|index| line.is_char_boundary(*index))
            .unwrap_or(0)
    };
    let prefix = line[..boundary].chars();
    let mut marker = String::new();
    for ch in prefix {
        if ch == '\t' {
            marker.push('\t');
        } else {
            marker.push(' ');
        }
    }
    marker.push('^');
    marker
}

fn print_diagnostic_frame(frame: &DiagnosticFrame, style: Style) {
    let width = frame.line.to_string().len();
    println!("  {:>width$} | {}", frame.line, frame.text, width = width);
    println!(
        "  {:>width$} | {}",
        "",
        style.red(&frame.marker),
        width = width
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn diagnostic_frame_marks_one_based_column() {
        let diagnostic = validate::Diagnostic {
            severity: validate::Severity::Error,
            stage: validate::Stage::Parse,
            code: None,
            message: "bad".to_string(),
            line: 2,
            column: Some(5),
        };
        assert_eq!(
            diagnostic_frame("one\ntwo = 1\n", &diagnostic),
            Some(DiagnosticFrame {
                line: 2,
                text: "two = 1".to_string(),
                marker: "    ^".to_string(),
            })
        );
    }

    #[test]
    fn diagnostic_frame_omits_unknown_column() {
        let diagnostic = validate::Diagnostic {
            severity: validate::Severity::Error,
            stage: validate::Stage::Parse,
            code: None,
            message: "bad".to_string(),
            line: 1,
            column: None,
        };
        assert_eq!(diagnostic_frame("one\n", &diagnostic), None);
    }

    #[test]
    fn diagnostic_frame_clamps_column_to_first_character() {
        assert_eq!(caret_marker("abc", 0), "^");
        assert_eq!(caret_marker("abc", 1), "^");
        assert_eq!(caret_marker("abc", 3), "  ^");
    }

    #[test]
    fn diagnostic_frame_treats_columns_as_byte_offsets() {
        assert_eq!(caret_marker("éx", 3), " ^");
        assert_eq!(caret_marker("éx", 2), "^");
    }
}
