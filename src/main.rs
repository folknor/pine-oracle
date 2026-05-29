use anyhow::{Result, bail};
use clap::{Args, Parser, Subcommand, ValueEnum};
use pine_oracle::{behavior, corpus, indicator, reference, search};
use std::io::{IsTerminal, Read};
use std::path::{Path, PathBuf};

mod commands;
mod output;

use output::{ResolvedFormat, Style, print_json};

/// pine: Pine v6 oracle CLI. Answers semantic questions about Pine script
/// across every Pine-adjacent project. Vendors the TradingView v6 reference
/// and the PineForge cross-validation corpus; exposes them as one-shot
/// subcommands.
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
    /// command; only those that emit colored text today (`pine search`,
    /// `pine validate`) act on it visibly.
    #[arg(long, global = true)]
    no_color: bool,

    /// Suppress non-data status/note text in text mode. JSON output is
    /// unchanged regardless of this flag. The exact effect is per-subcommand;
    /// commands whose entire text output is data (e.g. `pine parse`,
    /// `pine tokens`, `pine diff`) treat `--quiet` as a no-op.
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
    /// Function / constant / variable details
    Lookup { name: String },

    /// BM25 search across reference, corpus, docs, audit, and behavior data
    Search {
        query: String,
        #[arg(long, default_value_t = 25)]
        limit: usize,
        /// Restrict hits to one source. Pass `?` to list source kinds.
        #[arg(long)]
        kind: Option<String>,
    },

    /// Type errors, syntax errors, behavior warnings
    Validate {
        #[command(flatten)]
        source: PineSourceArgs,
        #[arg(long)]
        strict: bool,
    },

    /// AST as JSON
    Parse {
        #[command(flatten)]
        source: PineSourceArgs,
    },

    /// Lexer tokens with line / indent
    Tokens {
        #[command(flatten)]
        source: PineSourceArgs,
    },

    /// Polymorphism, side-effects, series-vs-simple, na-propagation
    Behavior {
        name: Option<String>,
        /// List behavior catalog entries instead of looking up one name.
        #[arg(long)]
        list: bool,
        /// Restrict `--list` to function, variable, constant, or keyword.
        /// Pass `?` to list the behavior-kind catalog.
        #[arg(long)]
        kind: Option<String>,
        /// Restrict `--list` entries by name, namespace, or detail text.
        #[arg(long)]
        grep: Option<String>,
    },

    /// Probe contents: strategy.pine + tv_trades.csv + summary
    Probe { slug: String },

    /// List baked corpus probes
    Probes {
        #[arg(long)]
        grep: Option<String>,
        /// Restrict to probes whose strategy.pine uses the named Pine
        /// feature (e.g. `oca`, `trail`, `pyramiding`, `mtf`, `varip`,
        /// `udt`). Pass `?` to list every catalog entry with its
        /// description.
        #[arg(long)]
        feature: Option<String>,
    },

    /// Tier-classify a piners trade list against the probe's tv_trades
    Diff {
        probe: String,
        trades_csv: String,
        /// Emit the worst-N matched pairs (ranked descending by the per-
        /// pair max of entry / exit / pnl normalized deltas) plus every
        /// unmatched trade from the trimmed window. 0 (default) keeps
        /// the report headline-only.
        #[arg(long, default_value_t = 0)]
        show_diffs: usize,
    },

    /// Per-bar indicator parity against a baked baseline
    Indicator {
        slug: Option<String>,
        /// Run strict per-bar parity for the named indicator fixture.
        #[arg(long)]
        strict: bool,
        /// Run strict per-bar parity for every matching fixture.
        #[arg(long)]
        all: bool,
        /// Run the fixture once and print piners-runner actual outputs
        /// without comparing against expect.json.
        #[arg(long)]
        actual: bool,
        /// Print fixture detail without compiling/running the Pine source.
        #[arg(long)]
        metadata_only: bool,
        /// List baked indicator fixtures.
        #[arg(long)]
        list: bool,
        /// Restrict `--list` or `--all` by slug or fixture metadata.
        #[arg(long)]
        grep: Option<String>,
        /// Restrict `--list` or `--all` to smoke or tv. Pass `?` to list
        /// baseline kinds.
        #[arg(long)]
        baseline: Option<String>,
    },

    /// pine-data snapshot date + corpus revision + binary version
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
        Command::Lookup { name } => commands::lookup::run(&name, format, style, cli.quiet),
        Command::Search { query, limit, kind } => {
            commands::search::run(&query, limit, kind.as_deref(), format, style, cli.quiet)
        }
        Command::Validate { source, strict } => {
            let code = source.read()?;
            commands::validate::run(&code, strict, format, style, cli.quiet)
        }
        Command::Parse { source } => {
            let code = source.read()?;
            commands::parse::run(&code, format, style)
        }
        Command::Tokens { source } => {
            let code = source.read()?;
            commands::tokens::run(&code, format, style)
        }
        Command::Behavior {
            name,
            list,
            kind,
            grep,
        } => commands::behavior::run(
            name.as_deref(),
            list,
            kind.as_deref(),
            grep.as_deref(),
            format,
            style,
            cli.quiet,
        ),
        Command::Probe { slug } => commands::probe::run(&slug, format, style, cli.quiet),
        Command::Probes { grep, feature } => commands::probes::run(
            grep.as_deref(),
            feature.as_deref(),
            format,
            style,
            cli.quiet,
        ),
        Command::Diff {
            probe,
            trades_csv,
            show_diffs,
        } => commands::diff::run(&probe, &trades_csv, show_diffs, format, style),
        // `indicator` uses an `Args` struct rather than a positional arg list
        // because it has 8+ flags. Other subcommands use positional lists;
        // this is the only exception and is intentional.
        Command::Indicator {
            slug,
            strict,
            all,
            actual,
            metadata_only,
            list,
            grep,
            baseline,
        } => commands::indicator::run(
            &commands::indicator::Args {
                slug: slug.as_deref(),
                strict,
                list,
                all,
                actual,
                metadata_only,
                quiet: cli.quiet,
                grep: grep.as_deref(),
                baseline: baseline.as_deref(),
            },
            format,
            style,
        ),
        Command::Version => cmd_version(format, cli.quiet),
    }
}

fn cmd_version(format: ResolvedFormat, quiet: bool) -> Result<()> {
    let binary = env!("CARGO_PKG_VERSION");
    let categories = reference::categories();
    let reference_entry_count = reference::all_entries().len();
    let probe_count = corpus::probe_count();
    let probe_summary_count = corpus::probe_summary_count();
    let audit_sections = search::audit_section_count();
    let docs_sections = search::docs_section_count();
    let behavior_docs = search::behavior_doc_count();
    let pine_data = behavior::snapshot();
    let indicator_counts = indicator::fixture_counts().unwrap_or_else(|err| {
        eprintln!("warning: indicator fixture counts unavailable: {err}");
        indicator::IndicatorFixtureCounts {
            total: 0,
            smoke: 0,
            tv: 0,
        }
    });
    match format {
        ResolvedFormat::Json => {
            print_json(&serde_json::json!({
                "binary": binary,
                "reference": {
                    "categories": categories,
                    "entry_count": reference_entry_count,
                },
                "corpus": {
                    "probe_count": probe_count,
                    "probe_summary_count": probe_summary_count,
                },
                "pineforge_docs": {
                    "audit_sections": audit_sections,
                    "narrative_sections": docs_sections,
                },
                "behavior": {
                    "pine_data_version": pine_data.version,
                    "generated_at": pine_data.generated_at,
                    "function_count": pine_data.function_count,
                    "variable_count": pine_data.variable_count,
                    "constant_count": pine_data.constant_count,
                    "keyword_count": pine_data.keyword_count,
                    "type_count": pine_data.type_count,
                    "annotation_count": pine_data.annotation_count,
                    "polymorphic_function_count": pine_data.polymorphic_function_count,
                    "search_doc_count": behavior_docs,
                },
                "indicator": {
                    "fixture_count": indicator_counts.total,
                    "smoke_fixture_count": indicator_counts.smoke,
                    "tv_fixture_count": indicator_counts.tv,
                },
            }))?;
        }
        ResolvedFormat::Text => {
            println!("po {binary}");
            if quiet {
                return Ok(());
            }
            println!(
                "v6 reference:   {reference_entry_count} entries across {} categories ({})",
                categories.len(),
                categories.join(", ")
            );
            println!(
                "corpus:         {probe_count} baked probes ({probe_summary_count} with author summaries)"
            );
            println!(
                "PineForge docs: {audit_sections} audit sections + {docs_sections} narrative sections"
            );
            println!(
                "pine-data:      v{} generated {}",
                pine_data.version, pine_data.generated_at
            );
            println!(
                "behavior:       {} functions ({} polymorphic), {} variables, {} constants, {} keywords, {} types, {} annotations, {behavior_docs} searchable docs",
                pine_data.function_count,
                pine_data.polymorphic_function_count,
                pine_data.variable_count,
                pine_data.constant_count,
                pine_data.keyword_count,
                pine_data.type_count,
                pine_data.annotation_count
            );
            println!(
                "indicator:      {} strict fixtures ({} smoke, {} tv)",
                indicator_counts.total, indicator_counts.smoke, indicator_counts.tv
            );
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
        use pine_oracle::{behavior, corpus, indicator, reference, search};

        let binary = env!("CARGO_PKG_VERSION");
        let categories = reference::categories();
        let reference_entry_count = reference::all_entries().len();
        let probe_count = corpus::probe_count();
        let probe_summary_count = corpus::probe_summary_count();
        let audit_sections = search::audit_section_count();
        let docs_sections = search::docs_section_count();
        let behavior_docs = search::behavior_doc_count();
        let pine_data = behavior::snapshot();
        let indicator_counts =
            indicator::fixture_counts().unwrap_or(indicator::IndicatorFixtureCounts {
                total: 0,
                smoke: 0,
                tv: 0,
            });

        let payload = serde_json::json!({
            "binary": binary,
            "reference": {
                "categories": categories,
                "entry_count": reference_entry_count,
            },
            "corpus": {
                "probe_count": probe_count,
                "probe_summary_count": probe_summary_count,
            },
            "pineforge_docs": {
                "audit_sections": audit_sections,
                "narrative_sections": docs_sections,
            },
            "behavior": {
                "pine_data_version": pine_data.version,
                "generated_at": pine_data.generated_at,
                "function_count": pine_data.function_count,
                "variable_count": pine_data.variable_count,
                "constant_count": pine_data.constant_count,
                "keyword_count": pine_data.keyword_count,
                "type_count": pine_data.type_count,
                "annotation_count": pine_data.annotation_count,
                "polymorphic_function_count": pine_data.polymorphic_function_count,
                "search_doc_count": behavior_docs,
            },
            "indicator": {
                "fixture_count": indicator_counts.total,
                "smoke_fixture_count": indicator_counts.smoke,
                "tv_fixture_count": indicator_counts.tv,
            },
        });

        let v = versioned_json(&payload).expect("must wrap");

        // Pin all eight top-level fields; any rename must update this test.
        assert!(
            v["schema_version"].is_number(),
            "schema_version must be present"
        );
        assert!(v["binary"].is_string(), "binary field must be a string");
        assert!(
            v["reference"].is_object(),
            "reference field must be an object"
        );
        assert!(v["corpus"].is_object(), "corpus field must be an object");
        assert!(
            v["pineforge_docs"].is_object(),
            "pineforge_docs field must be an object"
        );
        assert!(
            v["behavior"].is_object(),
            "behavior field must be an object"
        );
        assert!(
            v["indicator"].is_object(),
            "indicator field must be an object"
        );

        // Spot-check nested fields so renames inside objects are caught too.
        assert!(v["reference"]["entry_count"].is_number());
        assert!(v["reference"]["categories"].is_array());
        assert!(v["corpus"]["probe_count"].is_number());
        assert!(v["corpus"]["probe_summary_count"].is_number());
        assert!(v["pineforge_docs"]["audit_sections"].is_number());
        assert!(v["pineforge_docs"]["narrative_sections"].is_number());
        assert!(v["behavior"]["pine_data_version"].is_string());
        assert!(v["behavior"]["generated_at"].is_string());
        assert!(v["behavior"]["function_count"].is_number());
        assert!(v["behavior"]["variable_count"].is_number());
        assert!(v["behavior"]["constant_count"].is_number());
        assert!(v["behavior"]["keyword_count"].is_number());
        assert!(v["behavior"]["type_count"].is_number());
        assert!(v["behavior"]["annotation_count"].is_number());
        assert!(v["behavior"]["polymorphic_function_count"].is_number());
        assert!(v["behavior"]["search_doc_count"].is_number());
        assert!(v["indicator"]["fixture_count"].is_number());
        assert!(v["indicator"]["smoke_fixture_count"].is_number());
        assert!(v["indicator"]["tv_fixture_count"].is_number());

        // Count must be > 0 for reference and corpus to flag regressions.
        assert!(v["reference"]["entry_count"].as_u64().unwrap_or(0) > 0);
        assert!(v["corpus"]["probe_count"].as_u64().unwrap_or(0) > 0);
    }

    #[test]
    fn behavior_list_parses_without_name() {
        let cli = Cli::try_parse_from([
            "pine",
            "behavior",
            "--list",
            "--kind",
            "function",
            "--grep",
            "plotshape",
        ])
        .expect("behavior list args should parse");

        match cli.command {
            Command::Behavior {
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
            _ => panic!("expected behavior command"),
        }
    }

    #[test]
    fn behavior_kind_catalog_parses_without_name() {
        let cli = Cli::try_parse_from(["pine", "behavior", "--kind", "?"])
            .expect("behavior kind catalog args should parse");

        match cli.command {
            Command::Behavior {
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
            _ => panic!("expected behavior command"),
        }
    }

    #[test]
    fn search_kind_catalog_parses_literal_question_mark() {
        let cli = Cli::try_parse_from(["pine", "search", "x", "--kind", "?"])
            .expect("search kind catalog args should parse");

        match cli.command {
            Command::Search { query, limit, kind } => {
                assert_eq!(query, "x");
                assert_eq!(limit, 25);
                assert_eq!(kind.as_deref(), Some("?"));
            }
            _ => panic!("expected search command"),
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
    fn parse_accepts_file_flag_source() {
        let cli = Cli::try_parse_from([
            "pine",
            "parse",
            "--file",
            "indicators/smoke-close/source.pine",
        ])
        .expect("parse --file source should parse");

        match cli.command {
            Command::Parse { source } => {
                let code = source.read().expect("source file");
                assert!(code.contains("Smoke close"));
                assert!(code.contains("plot(close)"));
            }
            _ => panic!("expected parse command"),
        }
    }

    #[test]
    fn tokens_accepts_positional_file_source() {
        let cli = Cli::try_parse_from(["pine", "tokens", "indicators/smoke-close/source.pine"])
            .expect("tokens path source should parse");

        match cli.command {
            Command::Tokens { source } => {
                let code = source.read().expect("source file");
                assert!(code.contains("Smoke close"));
                assert!(code.contains("plot(close)"));
            }
            _ => panic!("expected tokens command"),
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

    #[test]
    fn indicator_list_accepts_filters() {
        let cli = Cli::try_parse_from([
            "pine",
            "indicator",
            "--list",
            "--grep",
            "request",
            "--baseline",
            "smoke",
        ])
        .expect("indicator list args should parse");

        match cli.command {
            Command::Indicator {
                slug,
                strict,
                all,
                actual,
                metadata_only,
                list,
                grep,
                baseline,
            } => {
                assert_eq!(slug, None);
                assert!(!strict);
                assert!(!all);
                assert!(!actual);
                assert!(!metadata_only);
                assert!(list);
                assert_eq!(grep.as_deref(), Some("request"));
                assert_eq!(baseline.as_deref(), Some("smoke"));
            }
            _ => panic!("expected indicator command"),
        }
    }

    #[test]
    fn indicator_all_accepts_filters() {
        let cli = Cli::try_parse_from([
            "pine",
            "indicator",
            "--strict",
            "--all",
            "--baseline",
            "smoke",
        ])
        .expect("indicator all args should parse");

        match cli.command {
            Command::Indicator {
                slug,
                strict,
                all,
                actual,
                metadata_only,
                list,
                grep,
                baseline,
            } => {
                assert_eq!(slug, None);
                assert!(strict);
                assert!(all);
                assert!(!actual);
                assert!(!metadata_only);
                assert!(!list);
                assert_eq!(grep, None);
                assert_eq!(baseline.as_deref(), Some("smoke"));
            }
            _ => panic!("expected indicator command"),
        }
    }

    #[test]
    fn indicator_detail_parses_slug_without_flags() {
        let cli = Cli::try_parse_from(["pine", "indicator", "smoke-close"])
            .expect("indicator detail args should parse");

        match cli.command {
            Command::Indicator {
                slug,
                strict,
                all,
                actual,
                metadata_only,
                list,
                grep,
                baseline,
            } => {
                assert_eq!(slug.as_deref(), Some("smoke-close"));
                assert!(!strict);
                assert!(!all);
                assert!(!actual);
                assert!(!metadata_only);
                assert!(!list);
                assert_eq!(grep, None);
                assert_eq!(baseline, None);
            }
            _ => panic!("expected indicator command"),
        }
    }

    #[test]
    fn indicator_actual_parses_slug_with_flag() {
        let cli = Cli::try_parse_from(["pine", "indicator", "smoke-close", "--actual"])
            .expect("indicator actual args should parse");

        match cli.command {
            Command::Indicator {
                slug,
                strict,
                all,
                actual,
                metadata_only,
                list,
                grep,
                baseline,
            } => {
                assert_eq!(slug.as_deref(), Some("smoke-close"));
                assert!(!strict);
                assert!(!all);
                assert!(actual);
                assert!(!metadata_only);
                assert!(!list);
                assert_eq!(grep, None);
                assert_eq!(baseline, None);
            }
            _ => panic!("expected indicator command"),
        }
    }

    #[test]
    fn indicator_metadata_only_parses_slug_with_flag() {
        let cli = Cli::try_parse_from(["pine", "indicator", "smoke-close", "--metadata-only"])
            .expect("indicator metadata-only args should parse");

        match cli.command {
            Command::Indicator {
                slug,
                strict,
                all,
                actual,
                metadata_only,
                list,
                grep,
                baseline,
            } => {
                assert_eq!(slug.as_deref(), Some("smoke-close"));
                assert!(!strict);
                assert!(!all);
                assert!(!actual);
                assert!(metadata_only);
                assert!(!list);
                assert_eq!(grep, None);
                assert_eq!(baseline, None);
            }
            _ => panic!("expected indicator command"),
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
