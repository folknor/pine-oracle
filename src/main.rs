use anyhow::{bail, Result};
use clap::{Parser, Subcommand, ValueEnum};
use pine_cli::{behavior, corpus, diff, reference, search, syntax, validate};
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
    Probe { slug: String },

    /// List baked corpus probes
    Probes {
        #[arg(long)]
        grep: Option<String>,
    },

    /// Tier-classify a piners trade list against the probe's tv_trades
    Diff { probe: String, trades_csv: String },

    /// pine-data snapshot date + corpus revision + binary version
    Version,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let format = cli.format.resolve();

    match cli.command {
        Command::Lookup { name } => cmd_lookup(&name, format),
        Command::Search { query, limit } => cmd_search(&query, limit, format),
        Command::Validate { code, strict } => cmd_validate(&code, strict, format),
        Command::Parse { code } => cmd_parse(&code, format),
        Command::Tokens { code } => cmd_tokens(&code, format),
        Command::Behavior { name } => cmd_behavior(&name, format),
        Command::Probe { slug } => cmd_probe(&slug, format),
        Command::Probes { grep } => cmd_probes(grep.as_deref(), format),
        Command::Diff { probe, trades_csv } => cmd_diff(&probe, &trades_csv, format),
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

fn cmd_probe(slug: &str, format: ResolvedFormat) -> Result<()> {
    let probe = corpus::load_probe(slug)?;
    match format {
        ResolvedFormat::Json => {
            println!("{}", serde_json::to_string(&probe)?);
        }
        ResolvedFormat::Text => {
            println!("slug: {}", probe.slug);
            match probe.summary {
                Some(summary) => println!("summary: {summary}"),
                None => println!("summary: (none)"),
            }
            let trade_lines = probe.tv_trades_csv.lines().count();
            let trade_bytes = probe.tv_trades_csv.len();
            println!("tv_trades.csv: {trade_lines} lines, {trade_bytes} bytes (use --format json for full content)");
            println!(
                "inputs.json: {}",
                if probe.inputs_json.is_some() {
                    "present"
                } else {
                    "(none)"
                }
            );
            println!("\nstrategy.pine:\n{}", probe.strategy_pine);
        }
    }
    Ok(())
}

fn cmd_probes(grep: Option<&str>, format: ResolvedFormat) -> Result<()> {
    let probes = corpus::list_probes(grep)?;
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

fn cmd_diff(probe_slug: &str, trades_csv_path: &str, format: ResolvedFormat) -> Result<()> {
    let user_csv = std::fs::read_to_string(trades_csv_path)
        .map_err(|e| anyhow::anyhow!("reading {trades_csv_path}: {e}"))?;
    let report = diff::diff(probe_slug, &user_csv)?;
    match format {
        ResolvedFormat::Json => {
            println!("{}", serde_json::to_string(&report)?);
        }
        ResolvedFormat::Text => print_diff_text(&report),
    }
    Ok(())
}

fn print_diff_text(r: &diff::DiffReport) {
    println!("probe:       {}", r.probe_slug);
    println!("profile:     {:?}", r.profile);
    println!(
        "TV trades:   {}  user trades: {}  matched: {}",
        r.tv_trade_count, r.user_trade_count, r.matched_count
    );
    println!(
        "count delta:           {:>10.4}%  (threshold {:>7.4}%)",
        r.count_delta * 100.0,
        r.thresholds.count * 100.0
    );
    println!(
        "entry-price p90 delta: {:>10.4}%  (threshold {:>7.4}%)",
        r.entry_p90_delta * 100.0,
        r.thresholds.entry * 100.0
    );
    println!(
        "exit-price  p90 delta: {:>10.4}%  (threshold {:>7.4}%)",
        r.exit_p90_delta * 100.0,
        r.thresholds.exit * 100.0
    );
    println!(
        "pnl         p90 delta: {:>10.4}%  (threshold {:>7.4}%)",
        r.pnl_p90_delta * 100.0,
        r.thresholds.pnl * 100.0
    );
    println!("tier:        {:?}", r.tier);
}

fn cmd_behavior(name: &str, format: ResolvedFormat) -> Result<()> {
    let Some(b) = behavior::lookup(name) else {
        bail!("no behavior data for `{name}`");
    };
    match format {
        ResolvedFormat::Json => {
            println!("{}", serde_json::to_string(&b)?);
        }
        ResolvedFormat::Text => print_behavior_text(&b),
    }
    Ok(())
}

fn print_behavior_text(b: &behavior::Behavior) {
    match b {
        behavior::Behavior::Function(f) => {
            println!("function {}", f.name);
            if let Some(ns) = &f.namespace {
                println!("  namespace: {ns}");
            }
            println!("  syntax: {}", f.syntax);
            if !f.returns.is_empty() {
                println!("  returns: {}", f.returns);
            }
            if !f.parameters.is_empty() {
                println!("  parameters:");
                for p in &f.parameters {
                    let req = if p.required { "required" } else { "optional" };
                    println!("    - {} : {} ({req})", p.name, p.ty);
                }
            }
            if f.flags.top_level_only {
                println!("  flags: top-level only");
            }
            if let Some(beh) = &f.behavior {
                let poly = if beh.polymorphic.is_polymorphic() {
                    "yes"
                } else {
                    "no"
                };
                println!("  polymorphic: {poly}");
                if let Some(detail) = beh.polymorphic.detail() {
                    if let Some(rtp) = &detail.return_type_param {
                        println!("    return-type-param: {rtp}");
                    }
                    if let Some(strat) = &detail.strategy {
                        println!("    strategy: {strat}");
                    }
                    if !detail.allowed_types.is_empty() {
                        println!("    allowed-types: {}", detail.allowed_types.join(", "));
                    }
                }
                if let Some(ord) = &beh.argument_ordering {
                    println!("  argument-ordering: {ord}");
                }
                if !beh.observed_return_types.is_empty() {
                    println!(
                        "  observed-return-types: {}",
                        beh.observed_return_types.join(", ")
                    );
                }
                if let Some(reason) = &beh.reason {
                    println!("  reason: {reason}");
                }
            }
        }
        behavior::Behavior::Variable(v) => {
            println!("variable {}", v.name);
            println!("  type: {}", v.ty);
            println!("  qualifier: {}", v.qualifier);
        }
        behavior::Behavior::Constant(c) => {
            println!("constant {}", c.name);
            if let Some(ns) = &c.namespace {
                println!("  namespace: {ns}");
            }
            if let Some(short) = &c.short_name {
                println!("  short-name: {short}");
            }
            println!("  type: {}", c.ty);
        }
        behavior::Behavior::Keyword(k) => {
            println!("keyword {}", k.name);
        }
    }
}

fn cmd_validate(code: &str, strict: bool, format: ResolvedFormat) -> Result<()> {
    let report = if strict {
        validate::strict(code)?
    } else {
        validate::check(code)
    };
    match format {
        ResolvedFormat::Json => {
            println!("{}", serde_json::to_string(&report)?);
        }
        ResolvedFormat::Text => {
            if report.diagnostics.is_empty() {
                println!("ok");
            } else {
                for d in &report.diagnostics {
                    let sev = match d.severity {
                        validate::Severity::Error => "error",
                        validate::Severity::Warning => "warning",
                    };
                    let stage = match d.stage {
                        validate::Stage::Lex => "lex",
                        validate::Stage::Parse => "parse",
                        validate::Stage::Strict => "strict",
                    };
                    let loc = match d.column {
                        Some(col) => format!("{}:{}", d.line, col),
                        None => format!("{}", d.line),
                    };
                    println!("{sev}[{stage}] {loc}: {}", d.message);
                }
                if report.ok {
                    println!("ok (warnings only)");
                }
            }
        }
    }
    if !report.ok {
        std::process::exit(1);
    }
    Ok(())
}

fn cmd_parse(code: &str, format: ResolvedFormat) -> Result<()> {
    let mut lexer = syntax::Lexer::new(code);
    let tokens = lexer
        .tokenize()
        .map_err(|e| anyhow::anyhow!("lex error: {e}"))?;
    let mut parser = syntax::Parser::new(tokens);
    let statements = parser
        .parse()
        .map_err(|e| anyhow::anyhow!("parse error: {e}"))?;
    let program = syntax::Program::new(statements);

    match format {
        ResolvedFormat::Json => {
            println!("{}", serde_json::to_string(&program)?);
        }
        ResolvedFormat::Text => {
            println!("{}", serde_json::to_string_pretty(&program)?);
        }
    }
    Ok(())
}

fn cmd_tokens(code: &str, format: ResolvedFormat) -> Result<()> {
    let mut lexer = syntax::Lexer::new(code);
    let tokens = lexer
        .tokenize()
        .map_err(|e| anyhow::anyhow!("lex error: {e}"))?;

    match format {
        ResolvedFormat::Json => {
            println!("{}", serde_json::to_string(&tokens)?);
        }
        ResolvedFormat::Text => {
            for t in &tokens {
                println!("{:>4}:{:<3}  {:?}  {:?}", t.line, t.column, t.typ, t.lexeme);
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
