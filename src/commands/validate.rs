use anyhow::Result;
use pine_cli::validate;

use crate::output::{ResolvedFormat, Style, print_json};

pub(crate) fn run(code: &str, strict: bool, format: ResolvedFormat, style: Style) -> Result<()> {
    let report = if strict {
        validate::strict(code)?
    } else {
        validate::check(code)
    };
    match format {
        ResolvedFormat::Json => {
            print_json(&report)?;
        }
        ResolvedFormat::Text => {
            if report.diagnostics.is_empty() {
                println!("{}", style.cyan("ok"));
            } else {
                if strict {
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
                    let sev = match d.severity {
                        validate::Severity::Error => style.red("error"),
                        validate::Severity::Warning => style.yellow("warning"),
                    };
                    let stage = style.dim(&format!(
                        "[{}]",
                        match d.stage {
                            validate::Stage::Lex => "lex",
                            validate::Stage::Parse => "parse",
                            validate::Stage::Type => "type",
                            validate::Stage::Semantic => "semantic",
                            validate::Stage::Strict => "strict",
                        }
                    ));
                    let loc = match d.column {
                        Some(col) => format!("{}:{}", d.line, col),
                        None => format!("{}", d.line),
                    };
                    let code = d.code.as_deref().map_or(String::new(), |code| {
                        format!("{} ", style.dim(&format!("{code}:")))
                    });
                    println!("{sev}{stage} {}: {code}{}", style.bold(&loc), d.message);
                }
                if report.ok {
                    println!("{}", style.yellow("ok (warnings only)"));
                }
            }
        }
    }
    if !report.ok {
        std::process::exit(1);
    }
    Ok(())
}
