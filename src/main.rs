use anyhow::{Result, bail};
use clap::{Args, Parser, Subcommand, ValueEnum};
use pine_cli::{behavior, corpus, indicator, reference, search};
use std::io::{IsTerminal, Read};
use std::path::{Path, PathBuf};

mod commands;
mod output;

use output::{ResolvedFormat, Style, print_json};

#[derive(Parser)]
#[command(name = "pine", version, about = "Pine v6 oracle CLI", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Command,

    #[arg(long, global = true, value_enum, default_value_t = OutputFormat::Auto)]
    format: OutputFormat,

    #[arg(long, global = true)]
    no_color: bool,

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
            (Some(input), None, None) if input == "-" => read_source_stdin(),
            (Some(input), None, None) => {
                let path = Path::new(input);
                if path.is_file() || looks_like_source_path(input) {
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

fn looks_like_source_path(input: &str) -> bool {
    let path = Path::new(input);
    path.extension().is_some_and(|ext| ext == "pine")
        || ((input.contains('/') || input.contains('\\')) && !looks_like_inline_source(input))
}

fn looks_like_inline_source(input: &str) -> bool {
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
    // Style is currently threaded into validate and search, the two commands
    // that emit colored text today. TODO: extend to every command's run()
    // signature for forward compat once a clean pass is made across all
    // command modules.
    let style = Style::resolve(cli.no_color, format, stdout_is_tty);

    match cli.command {
        Command::Lookup { name } => commands::lookup::run(&name, format, cli.quiet),
        Command::Search { query, limit, kind } => {
            commands::search::run(&query, limit, kind.as_deref(), format, style, cli.quiet)
        }
        Command::Validate { source, strict } => {
            let code = source.read()?;
            commands::validate::run(&code, strict, format, style, cli.quiet)
        }
        Command::Parse { source } => {
            let code = source.read()?;
            commands::parse::run(&code, format)
        }
        Command::Tokens { source } => {
            let code = source.read()?;
            commands::tokens::run(&code, format)
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
            cli.quiet,
        ),
        Command::Probe { slug } => commands::probe::run(&slug, format, cli.quiet),
        Command::Probes { grep, feature } => {
            commands::probes::run(grep.as_deref(), feature.as_deref(), format, cli.quiet)
        }
        Command::Diff {
            probe,
            trades_csv,
            show_diffs,
        } => commands::diff::run(&probe, &trades_csv, show_diffs, format),
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
                    "function_behavior_count": pine_data.function_behavior_count,
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
            println!("pine {binary}");
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
                "behavior:       {} functions, {} variables, {} constants, {} keywords, {} behavior entries, {behavior_docs} searchable docs",
                pine_data.function_count,
                pine_data.variable_count,
                pine_data.constant_count,
                pine_data.keyword_count,
                pine_data.function_behavior_count
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

    #[test]
    fn looks_like_source_path_positive_and_negative() {
        // .pine extension -> path
        assert!(looks_like_source_path("foo.pine"));
        // path with separator -> path
        assert!(looks_like_source_path("path/to/file"));
        // inline Pine with parens -> not a path
        assert!(!looks_like_source_path("indicator(\"x\")"));
        // inline Pine with function call -> not a path
        assert!(!looks_like_source_path("plot(close)"));
    }

    #[test]
    fn looks_like_inline_source_positive_and_negative() {
        // newline -> inline
        assert!(looks_like_inline_source("\n"));
        // assignment -> inline
        assert!(looks_like_inline_source("x = 1"));
        // space -> inline (file paths usually don't contain spaces)
        assert!(looks_like_inline_source("just text"));
        // bare identifier -> not inline (could be a slug or filename)
        assert!(!looks_like_inline_source("identifier"));
    }

    #[test]
    fn cmd_version_json_shape() {
        use crate::output::versioned_json;
        use pine_cli::{behavior, corpus, indicator, reference, search};

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
                "function_behavior_count": pine_data.function_behavior_count,
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
        assert!(v["behavior"]["function_behavior_count"].is_number());
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
}
