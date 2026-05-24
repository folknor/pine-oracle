use anyhow::{bail, Result};
use clap::{Parser, Subcommand, ValueEnum};
use pine_cli::{corpus, reference, search};
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
        Command::Probe {
            slug,
            engine_history,
        } => cmd_probe(&slug, engine_history, format),
        Command::Probes { feature, grep } => {
            cmd_probes(feature.as_deref(), grep.as_deref(), format)
        }
        Command::Diff { .. } => bail!("diff: not implemented yet"),
        Command::Corpus { action } => match action {
            CorpusAction::Install => cmd_corpus_install(format),
            CorpusAction::Update => cmd_corpus_update(format),
        },
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
    let hits = search::query(query, limit)?;
    match format {
        ResolvedFormat::Json => {
            let matches: Vec<_> = hits
                .iter()
                .map(|h| {
                    serde_json::json!({
                        "name": h.name,
                        "category": h.category,
                        "score": h.score,
                    })
                })
                .collect();
            println!(
                "{}",
                serde_json::to_string(&serde_json::json!({
                    "query": query,
                    "matches": matches,
                }))?
            );
        }
        ResolvedFormat::Text => {
            if hits.is_empty() {
                eprintln!("no matches");
                return Ok(());
            }
            for h in &hits {
                println!("{:>6.2}  {}  ({})", h.score, h.name, h.category);
            }
        }
    }
    Ok(())
}

fn cmd_probe(slug: &str, engine_history: bool, format: ResolvedFormat) -> Result<()> {
    let probe = corpus::load_probe(slug, engine_history)?;
    match format {
        ResolvedFormat::Json => {
            println!("{}", serde_json::to_string(&probe)?);
        }
        ResolvedFormat::Text => {
            println!("slug: {}", probe.slug);
            if let Some(summary) = &probe.summary {
                println!("summary: {summary}");
            } else {
                println!("summary: (none)");
            }
            println!("tv_trades.csv: {}", probe.tv_trades_csv.display());
            if probe.inputs_json.is_some() {
                println!("inputs.json: present");
            } else {
                println!("inputs.json: (none)");
            }
            println!("\nstrategy.pine:\n{}", probe.strategy_pine);
        }
    }
    Ok(())
}

fn cmd_probes(feature: Option<&str>, grep: Option<&str>, format: ResolvedFormat) -> Result<()> {
    let probes = corpus::list_probes(feature, grep)?;
    match format {
        ResolvedFormat::Json => {
            println!("{}", serde_json::to_string(&probes)?);
        }
        ResolvedFormat::Text => {
            if probes.is_empty() {
                eprintln!("no probes matched");
                return Ok(());
            }
            for p in &probes {
                match &p.summary {
                    Some(s) => {
                        let snippet: String = s.chars().take(80).collect();
                        println!("{}  -  {snippet}", p.slug);
                    }
                    None => println!("{}", p.slug),
                }
            }
        }
    }
    Ok(())
}

fn cmd_corpus_install(format: ResolvedFormat) -> Result<()> {
    corpus::install()?;
    match format {
        ResolvedFormat::Json => {
            println!(
                "{}",
                serde_json::to_string(&serde_json::json!({"status": "ok"}))?
            );
        }
        ResolvedFormat::Text => println!("corpus install: ok"),
    }
    Ok(())
}

fn cmd_corpus_update(format: ResolvedFormat) -> Result<()> {
    corpus::update()?;
    match format {
        ResolvedFormat::Json => {
            println!(
                "{}",
                serde_json::to_string(&serde_json::json!({"status": "ok"}))?
            );
        }
        ResolvedFormat::Text => println!("corpus update: ok"),
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
