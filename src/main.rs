use anyhow::{bail, Result};
use clap::{Parser, Subcommand, ValueEnum};
use pine_cli::reference;
use std::io::IsTerminal;

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

#[derive(Clone, Copy, PartialEq, Eq)]
enum ResolvedFormat {
    Text,
    Json,
}

#[derive(Subcommand)]
enum Command {
    /// Function / constant / variable details
    Lookup { name: String },

    /// Substring search across the v6 reference (BM25 in v2)
    Search {
        query: String,
        #[arg(long, default_value_t = 25)]
        limit: usize,
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
    Probe {
        slug: String,
        #[arg(long)]
        engine_history: bool,
    },

    /// List corpus probes
    Probes {
        #[arg(long)]
        feature: Option<String>,
        #[arg(long)]
        grep: Option<String>,
    },

    /// Tier-classify a piners trade list against the probe's tv_trades
    Diff { probe: String, trades_csv: String },

    /// Manage the local corpus
    Corpus {
        #[command(subcommand)]
        action: CorpusAction,
    },

    /// pine-data snapshot date + corpus revision + binary version
    Version,
}

#[derive(Subcommand)]
enum CorpusAction {
    /// Fetch PineForge corpus into XDG data dir
    Install,
    /// git pull the corpus
    Update,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let format = cli.format.resolve();

    match cli.command {
        Command::Lookup { name } => cmd_lookup(&name, format),
        Command::Search { query, limit } => cmd_search(&query, limit, format),
        Command::Validate { .. } => bail!("validate: not implemented yet"),
        Command::Parse { .. } => bail!("parse: not implemented yet"),
        Command::Tokens { .. } => bail!("tokens: not implemented yet"),
        Command::Behavior { .. } => bail!("behavior: not implemented yet"),
        Command::Probe { .. } => bail!("probe: not implemented yet"),
        Command::Probes { .. } => bail!("probes: not implemented yet"),
        Command::Diff { .. } => bail!("diff: not implemented yet"),
        Command::Corpus { .. } => bail!("corpus: not implemented yet"),
        Command::Version => cmd_version(format),
    }
}

fn cmd_lookup(name: &str, format: ResolvedFormat) -> Result<()> {
    if let Some(entry) = reference::lookup(name) {
        match format {
            ResolvedFormat::Json => {
                println!("{}", serde_json::to_string(&entry)?);
            }
            ResolvedFormat::Text => {
                println!("{} ({})\n", entry.name, entry.category);
                println!("{}", entry.content);
            }
        }
        return Ok(());
    }

    let prefix_hits = reference::prefix_search(name);
    if prefix_hits.is_empty() {
        bail!("no match for `{name}`");
    }

    match format {
        ResolvedFormat::Json => {
            let names: Vec<&str> = prefix_hits.iter().map(|e| e.name.as_str()).collect();
            println!(
                "{}",
                serde_json::to_string(&serde_json::json!({
                    "query": name,
                    "exact": false,
                    "matches": names,
                }))?
            );
        }
        ResolvedFormat::Text => {
            eprintln!("no exact match; {} prefix hit(s):", prefix_hits.len());
            for e in &prefix_hits {
                println!("{}  ({})", e.name, e.category);
            }
        }
    }
    Ok(())
}

fn cmd_search(query: &str, limit: usize, format: ResolvedFormat) -> Result<()> {
    let hits = reference::search(query, limit);
    match format {
        ResolvedFormat::Json => {
            let summarised: Vec<_> = hits
                .iter()
                .map(|e| {
                    serde_json::json!({
                        "name": e.name,
                        "category": e.category,
                    })
                })
                .collect();
            println!(
                "{}",
                serde_json::to_string(&serde_json::json!({
                    "query": query,
                    "matches": summarised,
                }))?
            );
        }
        ResolvedFormat::Text => {
            if hits.is_empty() {
                eprintln!("no matches");
                return Ok(());
            }
            for e in &hits {
                println!("{}  ({})", e.name, e.category);
            }
        }
    }
    Ok(())
}

fn cmd_version(format: ResolvedFormat) -> Result<()> {
    let binary = env!("CARGO_PKG_VERSION");
    let categories = reference::categories();
    match format {
        ResolvedFormat::Json => {
            println!(
                "{}",
                serde_json::to_string(&serde_json::json!({
                    "binary": binary,
                    "reference_categories": categories,
                }))?
            );
        }
        ResolvedFormat::Text => {
            println!("pine-cli {binary}");
            println!("v6 reference categories: {}", categories.join(", "));
        }
    }
    Ok(())
}
