use anyhow::{Result, bail};
use clap::{Args, Parser, Subcommand, ValueEnum};
use pine_oracle::{behavior, manual, suggest};
use std::io::{IsTerminal, Read};
use std::path::{Path, PathBuf};

mod commands;
mod output;

use output::{ResolvedFormat, Style, print_json};

/// pine: Pine v6 oracle CLI. Answers semantic questions about Pine script
/// across every Pine-adjacent project. Vendors the pine-data behavior surface
/// (structured Pine v6 signatures + prose); exposes it as one-shot subcommands.
#[derive(Parser)]
#[command(name = "pine", version, about = "Pine v6 oracle CLI", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Command,

    /// Output format. Defaults to `text` when stdout is a TTY, `json` when
    /// stdout is redirected or piped. Pass `--format json` to force machine-
    /// readable output, or `--format text` to force human-readable output.
    #[arg(long, global = true, value_enum, default_value_t = OutputFormat::Auto)]
    format: OutputFormat,

    /// Suppress ANSI styling in text mode. Honoured automatically when
    /// `NO_COLOR` is set or stdout is not a TTY. Has no effect in `--format
    /// json` mode (JSON output is never styled). The flag reaches every
    /// command; only those that emit colored text today (`pine validate`)
    /// act on it visibly.
    #[arg(long, global = true)]
    no_color: bool,

    /// Suppress non-data status/note text in text mode. JSON output is
    /// unchanged regardless of this flag. The exact effect is per-subcommand.
    #[arg(long, global = true)]
    quiet: bool,
}

#[derive(ValueEnum, Clone, Copy, Debug, PartialEq, Eq)]
enum OutputFormat {
    Auto,
    Text,
    Json,
}

impl OutputFormat {
    fn resolve(self, stdout_is_tty: bool) -> ResolvedFormat {
        match self {
            OutputFormat::Json => ResolvedFormat::Json,
            OutputFormat::Text => ResolvedFormat::Text,
            OutputFormat::Auto => {
                if stdout_is_tty {
                    ResolvedFormat::Text
                } else {
                    ResolvedFormat::Json
                }
            }
        }
    }
}

#[derive(Args)]
struct PineSourceArgs {
    /// Inline Pine source, a Pine file path, or `-` to read stdin.
    #[arg(value_name = "CODE_OR_FILE")]
    input: Option<String>,
    /// Use the given inline Pine source instead of a file.
    #[arg(short, long)]
    code: Option<String>,
    /// Read Pine source from a file path.
    #[arg(short, long)]
    file: Option<PathBuf>,
}

impl PineSourceArgs {
    fn read(&self) -> Result<String> {
        match (&self.input, &self.code, &self.file) {
            (Some(_), Some(_), _) => bail!("positional input cannot combine with `--code`"),
            (Some(_), _, Some(_)) => bail!("positional input cannot combine with `--file`"),
            (_, Some(_), Some(_)) => bail!("`--code` cannot combine with `--file`"),
            (_, Some(code), None) => Ok(code.clone()),
            (_, None, Some(path)) => read_source_file(path),
            (Some(input), None, None) if input == "-" => {
                // Explicit `-` means "read from stdin". Guard against an
                // accidental hang: if the caller typed `-` but stdin is still
                // a TTY, bail with the same helpful message as the implicit
                // stdin branch. Callers that genuinely want to pipe into `-`
                // will have stdin redirected (not a TTY).
                if std::io::stdin().is_terminal() {
                    bail!("Pine source required: pass CODE_OR_FILE, `--code`, `--file`, or `-`")
                }
                read_source_stdin()
            }
            (Some(input), None, None) => {
                let path = Path::new(input);
                if path.is_file() || looks_like_pine_file(input) {
                    read_source_file(path)
                } else {
                    Ok(input.clone())
                }
            }
            (None, None, None) => {
                if std::io::stdin().is_terminal() {
                    bail!("Pine source required: pass CODE_OR_FILE, `--code`, `--file`, or `-`")
                }
                read_source_stdin()
            }
        }
    }
}

fn read_source_file(path: &Path) -> Result<String> {
    std::fs::read_to_string(path)
        .map_err(|err| anyhow::anyhow!("reading Pine source {}: {err}", path.display()))
}

/// Returns `true` when the positional input looks more like a Pine file path
/// than inline source. Two cases are accepted:
///
/// 1. Ends with `.pine` AND has no inline-syntax characters (the latter
///    guard prevents `plot(close, title="my-script.pine")` from being
///    mistaken for a file path).
/// 2. Contains a path separator (`/` or `\`) AND has no inline-syntax
///    characters (the guard prevents `bar(x / 2)` from matching on `/`).
///
/// If the heuristic is wrong, users can force interpretation via `--file`
/// (always a path) or `--code` (always inline). These two flags bypass
/// the heuristic entirely.
fn looks_like_pine_file(input: &str) -> bool {
    let has_pine_ext = Path::new(input)
        .extension()
        .is_some_and(|ext| ext == "pine");
    let has_separator = input.contains('/') || input.contains('\\');
    let inline = looks_like_inline_pine_source(input);
    (has_pine_ext || has_separator) && !inline
}

/// Returns `true` when the positional input looks like inline Pine source
/// rather than a file path. Any of the following characters indicate source:
/// whitespace, parentheses, `=`, double-quote, or single-quote. These
/// characters are common in Pine expressions and rare (or illegal) in file
/// names used on the command line without quoting.
fn looks_like_inline_pine_source(input: &str) -> bool {
    input.contains('\n')
        || input.contains('(')
        || input.contains(')')
        || input.contains('=')
        || input.contains('"')
        || input.contains('\'')
        || input.contains(' ')
}

fn read_source_stdin() -> Result<String> {
    let mut code = String::new();
    std::io::stdin()
        .read_to_string(&mut code)
        .map_err(|err| anyhow::anyhow!("reading Pine source from stdin: {err}"))?;
    Ok(code)
}

#[derive(Subcommand)]
enum Command {
    /// Describe an identifier from pine-data: structured signature with
    /// per-argument prose, remarks, see-also, and operators.
    Lookup {
        name: Option<String>,
        /// List behavior catalog entries instead of looking up one name.
        #[arg(long)]
        list: bool,
        /// Restrict `--list` to function, variable, constant, keyword, type,
        /// or annotation. Pass `?` to list the catalog.
        #[arg(long)]
        kind: Option<String>,
        /// Restrict `--list` entries by name, namespace, or detail text.
        #[arg(long)]
        grep: Option<String>,
    },

    /// Search the Pine User Manual prose (how does X work)
    Search {
        /// A query, a `page#anchor` section ref, or a page path.
        query: String,
        /// Max sections to return for a query.
        #[arg(long, default_value_t = 8)]
        limit: usize,
    },

    /// Type errors, syntax errors, behavior warnings
    Validate {
        #[command(flatten)]
        source: PineSourceArgs,
        #[arg(long)]
        strict: bool,
    },

    /// pine-data snapshot date + behavior bake counts
    Version,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    // Resolve stdout TTY state once so OutputFormat::Auto and Style::resolve
    // both see the same answer. stdin().is_terminal() is kept separate in
    // PineSourceArgs::read -- it is a different stream.
    let stdout_is_tty = std::io::stdout().is_terminal();
    let format = cli.format.resolve(stdout_is_tty);
    // `Style` is `Copy` and threaded into every command's `run` so future
    // colorization can be added without touching the dispatcher. Commands
    // that don't emit colored text today take it as `_style`.
    let style = Style::resolve(cli.no_color, format, stdout_is_tty);

    match cli.command {
        Command::Lookup {
            name,
            list,
            kind,
            grep,
        } => commands::lookup::run(
            name.as_deref(),
            list,
            kind.as_deref(),
            grep.as_deref(),
            format,
            style,
            cli.quiet,
        ),
        Command::Search { query, limit } => {
            commands::search::run(&query, limit, format, style, cli.quiet)
        }
        Command::Validate { source, strict } => {
            let code = source.read()?;
            commands::validate::run(&code, strict, format, style, cli.quiet)
        }
        Command::Version => cmd_version(format, cli.quiet),
    }
}

fn cmd_version(format: ResolvedFormat, quiet: bool) -> Result<()> {
    let binary = env!("CARGO_PKG_VERSION");
    let indexed_names = suggest::indexed_name_count();
    let pine_data = behavior::snapshot();
    let manual_pages = manual::page_count();
    let manual_sections = manual::section_count();
    match format {
        ResolvedFormat::Json => {
            print_json(&serde_json::json!({
                    "binary": binary,
                    "behavior": {
                        "pine_data_version": pine_data.version,
                        "generated_at": pine_data.generated_at,
                        "function_count": pine_data.function_count,
                        "variable_count": pine_data.variable_count,
                        "constant_count": pine_data.constant_count,
                        "keyword_count": pine_data.keyword_count,
                        "type_count": pine_data.type_count,
                        "annotation_count": pine_data.annotation_count,
                        "operator_count": pine_data.operator_count,
                        "polymorphic_function_count": pine_data.polymorphic_function_count,
                        "indexed_name_count": indexed_names,
                },
                "manual": {
                    "page_count": manual_pages,
                    "section_count": manual_sections,
                },
            }))?;
        }
        ResolvedFormat::Text => {
            println!("po {binary}");
            if quiet {
                return Ok(());
            }
            println!(
                "pine-data:      v{} generated {}",
                pine_data.version, pine_data.generated_at
            );
            println!(
                "behavior:       {} functions ({} polymorphic), {} variables, {} constants, {} keywords, {} types, {} annotations, {} operators, {indexed_names} indexed names",
                pine_data.function_count,
                pine_data.polymorphic_function_count,
                pine_data.variable_count,
                pine_data.constant_count,
                pine_data.keyword_count,
                pine_data.type_count,
                pine_data.annotation_count,
                pine_data.operator_count
            );
            println!("manual:         {manual_pages} pages, {manual_sections} sections");
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Pin matrix for `looks_like_pine_file`. The key property: `.pine`
    /// extension alone is NOT enough -- the input must also lack inline-source
    /// characters (so `plot(close, title="my.pine")` stays inline).
    #[test]
    fn looks_like_pine_file_pin_matrix() {
        // .pine extension with no inline chars -> path
        assert!(looks_like_pine_file("foo.pine"), "bare .pine slug");
        assert!(looks_like_pine_file("dir/foo.pine"), "dir + .pine");

        // .pine extension BUT has inline chars -> NOT a path (was a bug before the &&)
        assert!(
            !looks_like_pine_file("plot(close, title=\"my.pine\")"),
            ".pine inside inline expression must stay inline"
        );
        assert!(
            !looks_like_pine_file("x = something.pine"),
            ".pine in assignment must stay inline"
        );

        // path with separator and no inline chars -> path
        assert!(looks_like_pine_file("path/to/file"), "slash-separated path");

        // separator BUT has inline chars -> not a path (slash in expression)
        assert!(
            !looks_like_pine_file("bar(x / 2)"),
            "slash inside function call must stay inline"
        );
        assert!(
            !looks_like_pine_file("plot(close / 2)"),
            "slash inside plot call must stay inline"
        );

        // no extension, no separator, no inline chars -> not a path
        assert!(!looks_like_pine_file("identifier"), "bare identifier");
        assert!(!looks_like_pine_file("plotshape"), "bare built-in name");
    }

    /// Pin matrix for `looks_like_inline_pine_source`.
    #[test]
    fn looks_like_inline_pine_source_pin_matrix() {
        // Characters that signal source code
        assert!(
            looks_like_inline_pine_source("\n"),
            "newline signals multi-line source"
        );
        assert!(
            looks_like_inline_pine_source("x = 1"),
            "assignment signals source"
        );
        assert!(
            looks_like_inline_pine_source("just text"),
            "space signals source (paths rarely have spaces on CLI)"
        );
        assert!(
            looks_like_inline_pine_source("plot(close)"),
            "parens signal function call"
        );
        assert!(
            looks_like_inline_pine_source("indicator(\"x\")"),
            "parens + quotes signal source"
        );
        assert!(
            looks_like_inline_pine_source("f = 'hi'"),
            "single-quote signals string literal"
        );

        // Plain identifiers: not inline (could be a slug, filename, probe name)
        assert!(
            !looks_like_inline_pine_source("identifier"),
            "bare identifier is not inline"
        );
        assert!(
            !looks_like_inline_pine_source("ema"),
            "bare built-in name is not inline"
        );
        assert!(
            !looks_like_inline_pine_source("foo.pine"),
            "bare .pine filename is not inline"
        );
    }

    #[test]
    fn cmd_version_json_shape() {
        use crate::output::versioned_json;
        use pine_oracle::{behavior, manual, suggest};

        let binary = env!("CARGO_PKG_VERSION");
        let indexed_names = suggest::indexed_name_count();
        let pine_data = behavior::snapshot();

        let payload = serde_json::json!({
            "binary": binary,
            "behavior": {
                "pine_data_version": pine_data.version,
                "generated_at": pine_data.generated_at,
                "function_count": pine_data.function_count,
                "variable_count": pine_data.variable_count,
                "constant_count": pine_data.constant_count,
                "keyword_count": pine_data.keyword_count,
                "type_count": pine_data.type_count,
                "annotation_count": pine_data.annotation_count,
                "operator_count": pine_data.operator_count,
                "polymorphic_function_count": pine_data.polymorphic_function_count,
                "indexed_name_count": indexed_names,
            },
            "manual": {
                "page_count": manual::page_count(),
                "section_count": manual::section_count(),
            },
        });

        let v = versioned_json(&payload).expect("must wrap");

        // Pin all top-level fields; any rename must update this test.
        assert!(
            v["schema_version"].is_number(),
            "schema_version must be present"
        );
        assert!(v["binary"].is_string(), "binary field must be a string");
        assert!(
            v["behavior"].is_object(),
            "behavior field must be an object"
        );

        // Spot-check nested fields so renames inside objects are caught too.
        assert!(v["behavior"]["pine_data_version"].is_string());
        assert!(v["behavior"]["generated_at"].is_string());
        assert!(v["behavior"]["function_count"].is_number());
        assert!(v["behavior"]["variable_count"].is_number());
        assert!(v["behavior"]["constant_count"].is_number());
        assert!(v["behavior"]["keyword_count"].is_number());
        assert!(v["behavior"]["type_count"].is_number());
        assert!(v["behavior"]["annotation_count"].is_number());
        assert!(v["behavior"]["operator_count"].is_number());
        assert!(v["behavior"]["polymorphic_function_count"].is_number());
        assert!(v["behavior"]["indexed_name_count"].is_number());

        // Counts must be > 0 to flag a regression in the bake.
        assert!(v["behavior"]["function_count"].as_u64().unwrap_or(0) > 0);
        assert!(v["behavior"]["operator_count"].as_u64().unwrap_or(0) > 0);
    }

    #[test]
    fn lookup_list_parses_without_name() {
        let cli = Cli::try_parse_from([
            "pine",
            "lookup",
            "--list",
            "--kind",
            "function",
            "--grep",
            "plotshape",
        ])
        .expect("lookup list args should parse");

        match cli.command {
            Command::Lookup {
                name,
                list,
                kind,
                grep,
            } => {
                assert_eq!(name, None);
                assert!(list);
                assert_eq!(kind.as_deref(), Some("function"));
                assert_eq!(grep.as_deref(), Some("plotshape"));
            }
            _ => panic!("expected lookup command"),
        }
    }

    #[test]
    fn lookup_kind_catalog_parses_without_name() {
        let cli = Cli::try_parse_from(["pine", "lookup", "--kind", "?"])
            .expect("lookup kind catalog args should parse");

        match cli.command {
            Command::Lookup {
                name,
                list,
                kind,
                grep,
            } => {
                assert_eq!(name, None);
                assert!(!list);
                assert_eq!(kind.as_deref(), Some("?"));
                assert_eq!(grep, None);
            }
            _ => panic!("expected lookup command"),
        }
    }

    #[test]
    fn validate_accepts_inline_positional_source() {
        let cli = Cli::try_parse_from(["pine", "validate", "indicator(\"x\")"])
            .expect("validate inline source should parse");

        match cli.command {
            Command::Validate { source, strict } => {
                assert!(!strict);
                assert_eq!(source.read().expect("source"), "indicator(\"x\")");
            }
            _ => panic!("expected validate command"),
        }
    }

    #[test]
    fn validate_accepts_code_flag_source() {
        let cli = Cli::try_parse_from(["pine", "validate", "--code", "indicator(\"x\")"])
            .expect("validate --code source should parse");

        match cli.command {
            Command::Validate { source, strict } => {
                assert!(!strict);
                assert_eq!(source.read().expect("source"), "indicator(\"x\")");
            }
            _ => panic!("expected validate command"),
        }
    }

    #[test]
    fn source_rejects_positional_and_code_flag() {
        let cli = Cli::try_parse_from([
            "pine",
            "validate",
            "indicator(\"x\")",
            "--code",
            "indicator(\"y\")",
        ])
        .expect("source conflict is resolved after parsing");

        match cli.command {
            Command::Validate { source, strict: _ } => {
                let err = source.read().expect_err("must reject conflicting sources");
                assert!(err.to_string().contains("cannot combine"));
            }
            _ => panic!("expected validate command"),
        }
    }

    #[test]
    fn source_rejects_missing_pine_path() {
        let cli = Cli::try_parse_from(["pine", "validate", "does-not-exist.pine"])
            .expect("missing path should parse");

        match cli.command {
            Command::Validate { source, strict: _ } => {
                let err = source.read().expect_err("must reject missing .pine file");
                assert!(err.to_string().contains("reading Pine source"));
                assert!(err.to_string().contains("does-not-exist.pine"));
            }
            _ => panic!("expected validate command"),
        }
    }

    #[test]
    fn inline_source_containing_operator_slash_stays_inline() {
        let cli = Cli::try_parse_from(["pine", "validate", "plot(close / 2)"])
            .expect("inline source should parse");

        match cli.command {
            Command::Validate { source, strict: _ } => {
                assert_eq!(source.read().expect("source"), "plot(close / 2)");
            }
            _ => panic!("expected validate command"),
        }
    }

    // --- OutputFormat + global flag parsing pins ---

    /// Pin that `--format json` parses to the Json variant.
    #[test]
    fn format_flag_json_parses() {
        let cli = Cli::try_parse_from(["pine", "--format", "json", "lookup", "plot"])
            .expect("--format json should parse");
        assert_eq!(cli.format, OutputFormat::Json);
    }

    /// Pin that `--format text` parses to the Text variant.
    #[test]
    fn format_flag_text_parses() {
        let cli = Cli::try_parse_from(["pine", "--format", "text", "lookup", "plot"])
            .expect("--format text should parse");
        assert_eq!(cli.format, OutputFormat::Text);
    }

    /// Pin that the default variant is Auto (no --format flag given).
    #[test]
    fn format_flag_defaults_to_auto() {
        let cli =
            Cli::try_parse_from(["pine", "lookup", "plot"]).expect("no --format should parse");
        assert_eq!(cli.format, OutputFormat::Auto);
    }

    /// Pin that `--no-color` sets the flag.
    #[test]
    fn no_color_flag_sets_field() {
        let cli = Cli::try_parse_from(["pine", "--no-color", "lookup", "plot"])
            .expect("--no-color should parse");
        assert!(cli.no_color);
    }

    /// Pin that `--quiet` sets the flag.
    #[test]
    fn quiet_flag_sets_field() {
        let cli = Cli::try_parse_from(["pine", "--quiet", "lookup", "plot"])
            .expect("--quiet should parse");
        assert!(cli.quiet);
    }

    /// Pin that the explicit `-` positional is parsed into PineSourceArgs.input
    /// as the string "-". The actual stdin read (and TTY guard) is not tested
    /// here since stdin cannot be mocked in unit tests; the parse-level
    /// structural pin is sufficient to guard against accidental removal of
    /// the special case.
    #[test]
    fn validate_dash_stdin_sentinel_parses() {
        let cli = Cli::try_parse_from(["pine", "validate", "-"]).expect("-  should parse");

        match cli.command {
            Command::Validate { source, strict: _ } => {
                assert_eq!(
                    source.input.as_deref(),
                    Some("-"),
                    "explicit - must land in input field"
                );
                assert!(source.code.is_none());
                assert!(source.file.is_none());
            }
            _ => panic!("expected validate command"),
        }
    }

    /// Pin that `--code X --file Y` (both exclusive flags together) is
    /// rejected at read time, not at parse time.
    #[test]
    fn source_rejects_code_and_file_flag_together() {
        let cli =
            Cli::try_parse_from(["pine", "validate", "--code", "plot(1)", "--file", "x.pine"])
                .expect("parse should succeed; conflict detected at read time");

        match cli.command {
            Command::Validate { source, strict: _ } => {
                let err = source.read().expect_err("--code + --file must be rejected");
                assert!(
                    err.to_string().contains("cannot combine"),
                    "error must mention 'cannot combine', got: {err}"
                );
            }
            _ => panic!("expected validate command"),
        }
    }

    /// Pin that `OutputFormat::resolve` returns Json for Auto when stdout is
    /// not a TTY, and Text when it is a TTY.
    #[test]
    fn output_format_auto_resolves_by_tty() {
        assert_eq!(
            OutputFormat::Auto.resolve(false),
            ResolvedFormat::Json,
            "Auto + not-a-tty -> Json"
        );
        assert_eq!(
            OutputFormat::Auto.resolve(true),
            ResolvedFormat::Text,
            "Auto + is-a-tty -> Text"
        );
        // Non-Auto variants are identity regardless of TTY.
        assert_eq!(OutputFormat::Json.resolve(false), ResolvedFormat::Json);
        assert_eq!(OutputFormat::Json.resolve(true), ResolvedFormat::Json);
        assert_eq!(OutputFormat::Text.resolve(false), ResolvedFormat::Text);
        assert_eq!(OutputFormat::Text.resolve(true), ResolvedFormat::Text);
    }
}
