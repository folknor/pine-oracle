use anyhow::Result;
use clap::{Parser, Subcommand, ValueEnum};
use pine_cli::{corpus, indicator, reference, search};
use std::io::IsTerminal;

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
    fn resolve(self) -> ResolvedFormat {
        match self {
            OutputFormat::Json => ResolvedFormat::Json,
            OutputFormat::Text => ResolvedFormat::Text,
            OutputFormat::Auto => {
                if std::io::stdout().is_terminal() {
                    ResolvedFormat::Text
                } else {
                    ResolvedFormat::Json
                }
            }
        }
    }
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
        /// Restrict hits to one source: reference, probe, audit, docs, or behavior.
        #[arg(long)]
        kind: Option<String>,
    },

    /// Type errors, syntax errors, behavior warnings
    Validate {
        code: String,
        #[arg(long)]
        strict: bool,
    },

    /// AST as JSON
    Parse { code: String },

    /// Lexer tokens with line / indent
    Tokens { code: String },

    /// Polymorphism, side-effects, series-vs-simple, na-propagation
    Behavior { name: String },

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
        /// List baked indicator fixtures.
        #[arg(long)]
        list: bool,
    },

    /// pine-data snapshot date + corpus revision + binary version
    Version,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let format = cli.format.resolve();
    let style = Style::resolve(cli.no_color, format);

    match cli.command {
        Command::Lookup { name } => commands::lookup::run(&name, format),
        Command::Search { query, limit, kind } => {
            commands::search::run(&query, limit, kind.as_deref(), format, style)
        }
        Command::Validate { code, strict } => commands::validate::run(&code, strict, format, style),
        Command::Parse { code } => commands::parse::run(&code, format),
        Command::Tokens { code } => commands::tokens::run(&code, format),
        Command::Behavior { name } => commands::behavior::run(&name, format),
        Command::Probe { slug } => commands::probe::run(&slug, format),
        Command::Probes { grep, feature } => {
            commands::probes::run(grep.as_deref(), feature.as_deref(), format)
        }
        Command::Diff {
            probe,
            trades_csv,
            show_diffs,
        } => commands::diff::run(&probe, &trades_csv, show_diffs, format),
        Command::Indicator { slug, strict, list } => {
            commands::indicator::run(slug.as_deref(), strict, list, format)
        }
        Command::Version => cmd_version(format),
    }
}

fn cmd_version(format: ResolvedFormat) -> Result<()> {
    let binary = env!("CARGO_PKG_VERSION");
    let categories = reference::categories();
    let reference_entry_count = reference::all_entries().len();
    let probe_count = corpus::list_probes(None, None).map_or(0, |v| v.len());
    let probe_summary_count = corpus::list_probes(None, None)
        .map_or(0, |v| v.iter().filter(|p| p.summary.is_some()).count());
    let audit_sections = search::audit_section_count();
    let docs_sections = search::docs_section_count();
    let behavior_docs = search::behavior_doc_count();
    let indicator_counts =
        indicator::fixture_counts().unwrap_or(indicator::IndicatorFixtureCounts {
            total: 0,
            smoke: 0,
            tv: 0,
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
            println!("pine-cli {binary}");
            println!(
                "v6 reference:   {reference_entry_count} entries across {} categories ({})",
                categories.len(),
                categories.join(", ")
            );
            println!(
                "corpus:         {probe_count} baked probes ({probe_summary_count} with author summaries)"
            );
            println!(
                "pineforge docs: {audit_sections} audit sections + {docs_sections} narrative sections"
            );
            println!("behavior:      {behavior_docs} searchable pine-data docs");
            println!(
                "indicator:     {} strict fixtures ({} smoke, {} tv)",
                indicator_counts.total, indicator_counts.smoke, indicator_counts.tv
            );
        }
    }
    Ok(())
}
