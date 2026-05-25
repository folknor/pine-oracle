use anyhow::{Result, bail};
use clap::{Parser, Subcommand, ValueEnum};
use pine_cli::{behavior, corpus, diff, reference, search, syntax, validate};
use serde::Serialize;
use std::io::IsTerminal;

/// JSON output schema version. Bumped on any breaking shape change to a
/// subcommand's JSON output. Documented in docs/pine-oracle.md.
const SCHEMA_VERSION: u32 = 1;

/// Wrap `payload` in the versioned envelope. Objects get the field inserted
/// at the top level; arrays and scalars are wrapped as
/// `{schema_version, items}` / `{schema_version, value}`. Public-ish only
/// so the test below can pin the wire shape.
fn versioned_json<T: Serialize>(payload: &T) -> Result<serde_json::Value> {
    let mut v = serde_json::to_value(payload)?;
    let kind = match &v {
        serde_json::Value::Object(_) => 0,
        serde_json::Value::Array(_) => 1,
        _ => 2,
    };
    match kind {
        0 => {
            if let serde_json::Value::Object(ref mut obj) = v {
                obj.insert("schema_version".into(), serde_json::json!(SCHEMA_VERSION));
            }
        }
        1 => {
            v = serde_json::json!({
                "schema_version": SCHEMA_VERSION,
                "items": v,
            });
        }
        _ => {
            v = serde_json::json!({
                "schema_version": SCHEMA_VERSION,
                "value": v,
            });
        }
    }
    Ok(v)
}

fn print_json<T: Serialize>(payload: &T) -> Result<()> {
    println!("{}", serde_json::to_string(&versioned_json(payload)?)?);
    Ok(())
}

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

/// ANSI styling for terminal text output. Disabled when `--no-color` is
/// set, when the `NO_COLOR` env var is present (no-color.org convention),
/// when the resolved output format is JSON, or when stdout is not a TTY
/// (same probe `OutputFormat::Auto` uses to pick text vs json).
#[derive(Clone, Copy)]
struct Style {
    enabled: bool,
}

impl Style {
    fn resolve(no_color: bool, format: ResolvedFormat) -> Self {
        let enabled = !no_color
            && std::env::var_os("NO_COLOR").is_none()
            && format == ResolvedFormat::Text
            && std::io::stdout().is_terminal();
        Style { enabled }
    }

    fn red(self, s: &str) -> String {
        self.wrap(s, "31")
    }
    fn yellow(self, s: &str) -> String {
        self.wrap(s, "33")
    }
    fn cyan(self, s: &str) -> String {
        self.wrap(s, "36")
    }
    fn dim(self, s: &str) -> String {
        self.wrap(s, "2")
    }
    fn bold(self, s: &str) -> String {
        self.wrap(s, "1")
    }

    fn wrap(self, s: &str, code: &str) -> String {
        if self.enabled {
            format!("\x1b[{code}m{s}\x1b[0m")
        } else {
            s.to_string()
        }
    }
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
        /// Restrict hits to one source: `reference` or `probe`.
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

    /// pine-data snapshot date + corpus revision + binary version
    Version,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let format = cli.format.resolve();
    let style = Style::resolve(cli.no_color, format);

    match cli.command {
        Command::Lookup { name } => cmd_lookup(&name, format),
        Command::Search { query, limit, kind } => {
            cmd_search(&query, limit, kind.as_deref(), format, style)
        }
        Command::Validate { code, strict } => cmd_validate(&code, strict, format, style),
        Command::Parse { code } => cmd_parse(&code, format),
        Command::Tokens { code } => cmd_tokens(&code, format),
        Command::Behavior { name } => cmd_behavior(&name, format),
        Command::Probe { slug } => cmd_probe(&slug, format),
        Command::Probes { grep, feature } => {
            cmd_probes(grep.as_deref(), feature.as_deref(), format)
        }
        Command::Diff {
            probe,
            trades_csv,
            show_diffs,
        } => cmd_diff(&probe, &trades_csv, show_diffs, format),
        Command::Version => cmd_version(format),
    }
}

fn cmd_lookup(name: &str, format: ResolvedFormat) -> Result<()> {
    if let Some(entry) = reference::lookup(name) {
        match format {
            ResolvedFormat::Json => {
                print_json(&entry)?;
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
            print_json(&serde_json::json!({
                "query": name,
                "exact": false,
                "matches": names,
            }))?;
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

fn cmd_search(
    query: &str,
    limit: usize,
    kind_filter: Option<&str>,
    format: ResolvedFormat,
    style: Style,
) -> Result<()> {
    // Over-fetch when filtering so the post-filter list still fills the limit.
    let raw_limit = if kind_filter.is_some() {
        limit * 4
    } else {
        limit
    };
    let mut hits = search::query(query, raw_limit)?;
    if let Some(k) = kind_filter {
        hits.retain(|h| h.kind == k);
        hits.truncate(limit);
    }
    match format {
        ResolvedFormat::Json => {
            let matches: Vec<_> = hits
                .iter()
                .map(|h| {
                    serde_json::json!({
                        "kind": h.kind,
                        "name": h.name,
                        "category": h.category,
                        "score": h.score,
                        "content": h.content,
                    })
                })
                .collect();
            print_json(&serde_json::json!({
                "query": query,
                "matches": matches,
            }))?;
        }
        ResolvedFormat::Text => {
            if hits.is_empty() {
                eprintln!("no matches");
                return Ok(());
            }
            for h in &hits {
                let score = style.dim(&format!("{:>6.2}", h.score));
                let kind = style.cyan(&format!("[{:<9}]", h.kind));
                let name = style.bold(&h.name);
                let category = style.dim(&format!("({})", h.category));
                println!("{score}  {kind} {name}  {category}");
                let snippet = snippet_first_line(&h.content, 120);
                if !snippet.is_empty() {
                    println!("        {}", style.dim(&snippet));
                }
            }
        }
    }
    Ok(())
}

fn snippet_first_line(content: &str, max_chars: usize) -> String {
    let first = content
        .lines()
        .map(str::trim)
        .find(|l| !l.is_empty())
        .unwrap_or("");
    let collected: String = first.chars().take(max_chars).collect();
    if first.chars().count() > max_chars {
        format!("{collected}...")
    } else {
        collected
    }
}

fn cmd_probe(slug: &str, format: ResolvedFormat) -> Result<()> {
    let probe = corpus::load_probe(slug)?;
    match format {
        ResolvedFormat::Json => {
            print_json(&probe)?;
        }
        ResolvedFormat::Text => {
            println!("slug: {}", probe.slug);
            match probe.summary {
                Some(summary) => println!("summary: {summary}"),
                None => println!("summary: (none)"),
            }
            let trade_lines = probe.tv_trades_csv.lines().count();
            let trade_bytes = probe.tv_trades_csv.len();
            println!(
                "tv_trades.csv: {trade_lines} lines, {trade_bytes} bytes (use --format json for full content)"
            );
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

fn cmd_probes(grep: Option<&str>, feature: Option<&str>, format: ResolvedFormat) -> Result<()> {
    if matches!(feature, Some("?")) {
        return print_feature_catalog(format);
    }
    let probes = corpus::list_probes(grep, feature)?;
    match format {
        ResolvedFormat::Json => {
            print_json(&probes)?;
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

fn print_feature_catalog(format: ResolvedFormat) -> Result<()> {
    let catalog = corpus::feature_catalog();
    match format {
        ResolvedFormat::Json => {
            let items: Vec<_> = catalog
                .iter()
                .map(|(name, desc)| serde_json::json!({"name": name, "description": desc}))
                .collect();
            print_json(&items)?;
        }
        ResolvedFormat::Text => {
            for (name, desc) in &catalog {
                println!("{name:<28}  {desc}");
            }
        }
    }
    Ok(())
}

fn cmd_diff(
    probe_slug: &str,
    trades_csv_path: &str,
    show_diffs: usize,
    format: ResolvedFormat,
) -> Result<()> {
    let user_csv = std::fs::read_to_string(trades_csv_path)
        .map_err(|e| anyhow::anyhow!("reading {trades_csv_path}: {e}"))?;
    let report = diff::diff(probe_slug, &user_csv, diff::DiffOptions { show_diffs })?;
    match format {
        ResolvedFormat::Json => {
            print_json(&report)?;
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
    print_diff_details(r);
}

fn print_diff_details(r: &diff::DiffReport) {
    if !r.pair_diffs.is_empty() {
        println!();
        println!(
            "worst {} matched pair(s) (ranked by max of entry / exit / pnl deltas):",
            r.pair_diffs.len()
        );
        for (i, p) in r.pair_diffs.iter().enumerate() {
            let dir = match p.direction {
                diff::Direction::Long => "long ",
                diff::Direction::Short => "short",
            };
            let pnl_cell = match p.pnl_delta {
                Some(d) => format!("{:>6.2}%", d * 100.0),
                None => "  (na)".to_string(),
            };
            println!(
                "  {:>2}. {dir}  worst {:>6.2}%  skew {:>+5}s",
                i + 1,
                p.worst_delta * 100.0,
                p.time_skew_seconds
            );
            println!(
                "      tv:   entry {} @ {:>10.4}    exit @ {:>10.4}    pnl {:>+10.4}",
                p.tv_entry_time, p.tv_entry_price, p.tv_exit_price, p.tv_pnl
            );
            println!(
                "      user: entry {} @ {:>10.4}    exit @ {:>10.4}    pnl {:>+10.4}",
                p.user_entry_time, p.user_entry_price, p.user_exit_price, p.user_pnl
            );
            println!(
                "      delta:                entry {:>6.2}%      exit {:>6.2}%      pnl {pnl_cell}",
                p.entry_delta * 100.0,
                p.exit_delta * 100.0
            );
        }
    }
    print_orphan_block("TV-only", &r.tv_orphans);
    print_orphan_block("user-only", &r.user_orphans);
}

fn print_orphan_block(label: &str, rows: &[diff::TradeRow]) {
    if rows.is_empty() {
        return;
    }
    println!();
    println!("{label} trades ({} unmatched):", rows.len());
    for t in rows {
        let dir = match t.direction {
            diff::Direction::Long => "long ",
            diff::Direction::Short => "short",
        };
        println!(
            "  {dir}  entry {} @ {:>10.4}    exit @ {:>10.4}    pnl {:>+10.4}",
            t.entry_time, t.entry_price, t.exit_price, t.pnl
        );
    }
}

fn cmd_behavior(name: &str, format: ResolvedFormat) -> Result<()> {
    let Some(b) = behavior::lookup(name) else {
        bail!("no behavior data for `{name}`");
    };
    match format {
        ResolvedFormat::Json => {
            print_json(&b)?;
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
            if !f.examples.is_empty() {
                let n = f.examples.len();
                let label = if n == 1 { "example" } else { "examples" };
                println!("  {label}: {n}");
                for (i, ex) in f.examples.iter().enumerate() {
                    if n > 1 {
                        println!("    --- example {} ---", i + 1);
                    }
                    for line in ex.lines() {
                        println!("    {line}");
                    }
                }
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

fn cmd_validate(code: &str, strict: bool, format: ResolvedFormat, style: Style) -> Result<()> {
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
                            validate::Stage::Strict => "strict",
                        }
                    ));
                    let loc = match d.column {
                        Some(col) => format!("{}:{}", d.line, col),
                        None => format!("{}", d.line),
                    };
                    println!("{sev}{stage} {}: {}", style.bold(&loc), d.message);
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
            print_json(&program)?;
        }
        ResolvedFormat::Text => {
            let mut out = String::new();
            render_program(&program, &mut out);
            print!("{out}");
        }
    }
    Ok(())
}

// ---------- AST pretty-printer ----------
//
// Indented-text rendering of the parsed AST. Each node prints one line
// with its kind + a brief identifier (variable name, operator, literal
// value, etc.); children indent by two spaces. ASCII-only.

fn render_program(p: &syntax::Program, out: &mut String) {
    out.push_str("Program\n");
    for stmt in &p.statements {
        render_stmt(stmt, 1, out);
    }
}

fn render_stmt(stmt: &syntax::ast::Stmt, depth: usize, out: &mut String) {
    use syntax::ast::Stmt;
    let pad = indent(depth);
    match stmt {
        Stmt::VarDecl {
            name,
            type_qualifier,
            type_annotation,
            initializer,
            is_varip,
        } => {
            let kw = if *is_varip { "varip" } else { "var" };
            let q = type_qualifier
                .as_ref()
                .map(|q| format!("{q:?} "))
                .unwrap_or_default();
            let ty = type_annotation
                .as_ref()
                .map(|t| format!(": {t}"))
                .unwrap_or_default();
            out.push_str(&format!("{pad}VarDecl {kw} {q}{name}{ty}\n"));
            if let Some(init) = initializer {
                render_expr(init, depth + 1, out);
            }
        }
        Stmt::Assignment { target, value } => {
            out.push_str(&format!("{pad}Assignment\n"));
            out.push_str(&format!("{}target:\n", indent(depth + 1)));
            render_expr(target, depth + 2, out);
            out.push_str(&format!("{}value:\n", indent(depth + 1)));
            render_expr(value, depth + 2, out);
        }
        Stmt::TupleAssignment { names, value } => {
            out.push_str(&format!("{pad}TupleAssignment [{}]\n", names.join(", ")));
            render_expr(value, depth + 1, out);
        }
        Stmt::Expression(e) => {
            out.push_str(&format!("{pad}Expression\n"));
            render_expr(e, depth + 1, out);
        }
        Stmt::If {
            condition,
            then_branch,
            else_if_branches,
            else_branch,
        } => {
            out.push_str(&format!("{pad}If\n"));
            out.push_str(&format!("{}cond:\n", indent(depth + 1)));
            render_expr(condition, depth + 2, out);
            out.push_str(&format!("{}then:\n", indent(depth + 1)));
            for s in then_branch {
                render_stmt(s, depth + 2, out);
            }
            for (i, (cond, body)) in else_if_branches.iter().enumerate() {
                out.push_str(&format!("{}else-if[{i}] cond:\n", indent(depth + 1)));
                render_expr(cond, depth + 2, out);
                out.push_str(&format!("{}else-if[{i}] body:\n", indent(depth + 1)));
                for s in body {
                    render_stmt(s, depth + 2, out);
                }
            }
            if let Some(eb) = else_branch {
                out.push_str(&format!("{}else:\n", indent(depth + 1)));
                for s in eb {
                    render_stmt(s, depth + 2, out);
                }
            }
        }
        Stmt::For {
            var_name,
            from,
            to,
            body,
        } => {
            out.push_str(&format!("{pad}For {var_name}\n"));
            out.push_str(&format!("{}from:\n", indent(depth + 1)));
            render_expr(from, depth + 2, out);
            out.push_str(&format!("{}to:\n", indent(depth + 1)));
            render_expr(to, depth + 2, out);
            out.push_str(&format!("{}body:\n", indent(depth + 1)));
            for s in body {
                render_stmt(s, depth + 2, out);
            }
        }
        Stmt::ForIn {
            index_var,
            item_var,
            collection,
            body,
        } => {
            let pat = match index_var {
                Some(idx) => format!("[{idx}, {item_var}]"),
                None => item_var.clone(),
            };
            out.push_str(&format!("{pad}ForIn {pat}\n"));
            out.push_str(&format!("{}collection:\n", indent(depth + 1)));
            render_expr(collection, depth + 2, out);
            out.push_str(&format!("{}body:\n", indent(depth + 1)));
            for s in body {
                render_stmt(s, depth + 2, out);
            }
        }
        Stmt::While { condition, body } => {
            out.push_str(&format!("{pad}While\n"));
            out.push_str(&format!("{}cond:\n", indent(depth + 1)));
            render_expr(condition, depth + 2, out);
            out.push_str(&format!("{}body:\n", indent(depth + 1)));
            for s in body {
                render_stmt(s, depth + 2, out);
            }
        }
        Stmt::Break => out.push_str(&format!("{pad}Break\n")),
        Stmt::Continue => out.push_str(&format!("{pad}Continue\n")),
        Stmt::TypeDecl {
            name,
            fields,
            export,
        } => {
            let e = if *export { "export " } else { "" };
            out.push_str(&format!("{pad}TypeDecl {e}{name}\n"));
            for f in fields {
                out.push_str(&format!(
                    "{}field {} : {}\n",
                    indent(depth + 1),
                    f.name,
                    f.type_annotation
                ));
                if let Some(dv) = &f.default_value {
                    render_expr(dv, depth + 2, out);
                }
            }
        }
        Stmt::MethodDecl {
            name,
            params,
            body,
            export,
        } => {
            let e = if *export { "export " } else { "" };
            let plist = params
                .iter()
                .map(|p| p.name.clone())
                .collect::<Vec<_>>()
                .join(", ");
            out.push_str(&format!("{pad}MethodDecl {e}{name}({plist})\n"));
            for s in body {
                render_stmt(s, depth + 1, out);
            }
        }
        Stmt::EnumDecl {
            name,
            fields,
            export,
        } => {
            let e = if *export { "export " } else { "" };
            out.push_str(&format!("{pad}EnumDecl {e}{name}\n"));
            for f in fields {
                let t = f
                    .title
                    .as_ref()
                    .map(|t| format!(" = {t:?}"))
                    .unwrap_or_default();
                out.push_str(&format!("{}variant {}{t}\n", indent(depth + 1), f.name));
            }
        }
        Stmt::FunctionDecl {
            name,
            params,
            body,
            export,
        } => {
            let e = if *export { "export " } else { "" };
            let plist = params
                .iter()
                .map(|p| p.name.clone())
                .collect::<Vec<_>>()
                .join(", ");
            out.push_str(&format!("{pad}FunctionDecl {e}{name}({plist})\n"));
            for s in body {
                render_stmt(s, depth + 1, out);
            }
        }
        Stmt::Export { item } => {
            use syntax::ast::ExportItem;
            let label = match item {
                ExportItem::Type(n) => format!("type {n}"),
                ExportItem::Function(n) => format!("function {n}"),
            };
            out.push_str(&format!("{pad}Export {label}\n"));
        }
        Stmt::Import { path, alias } => {
            out.push_str(&format!("{pad}Import {path} as {alias}\n"));
        }
    }
}

fn render_expr(expr: &syntax::ast::Expr, depth: usize, out: &mut String) {
    use syntax::ast::Expr;
    let pad = indent(depth);
    match expr {
        Expr::Literal(lit) => {
            out.push_str(&format!("{pad}Literal {}\n", render_literal(lit)));
        }
        Expr::Variable(name) => {
            out.push_str(&format!("{pad}Variable {name}\n"));
        }
        Expr::Binary { left, op, right } => {
            out.push_str(&format!("{pad}Binary {op:?}\n"));
            render_expr(left, depth + 1, out);
            render_expr(right, depth + 1, out);
        }
        Expr::Unary { op, expr: inner } => {
            out.push_str(&format!("{pad}Unary {op:?}\n"));
            render_expr(inner, depth + 1, out);
        }
        Expr::Call {
            callee,
            type_args,
            args,
        } => {
            let targs = if type_args.is_empty() {
                String::new()
            } else {
                format!("<{}>", type_args.join(", "))
            };
            out.push_str(&format!("{pad}Call{targs}\n"));
            out.push_str(&format!("{}callee:\n", indent(depth + 1)));
            render_expr(callee, depth + 2, out);
            for (i, arg) in args.iter().enumerate() {
                use syntax::ast::Argument;
                match arg {
                    Argument::Positional(e) => {
                        out.push_str(&format!("{}arg[{i}]:\n", indent(depth + 1)));
                        render_expr(e, depth + 2, out);
                    }
                    Argument::Named { name, value } => {
                        out.push_str(&format!("{}arg[{i}] {name}=:\n", indent(depth + 1)));
                        render_expr(value, depth + 2, out);
                    }
                }
            }
        }
        Expr::Index { expr: inner, index } => {
            out.push_str(&format!("{pad}Index\n"));
            render_expr(inner, depth + 1, out);
            out.push_str(&format!("{}[\n", indent(depth + 1)));
            render_expr(index, depth + 2, out);
            out.push_str(&format!("{}]\n", indent(depth + 1)));
        }
        Expr::MemberAccess { object, member } => {
            out.push_str(&format!("{pad}MemberAccess .{member}\n"));
            render_expr(object, depth + 1, out);
        }
        Expr::Ternary {
            condition,
            then_expr,
            else_expr,
        } => {
            out.push_str(&format!("{pad}Ternary\n"));
            out.push_str(&format!("{}cond:\n", indent(depth + 1)));
            render_expr(condition, depth + 2, out);
            out.push_str(&format!("{}then:\n", indent(depth + 1)));
            render_expr(then_expr, depth + 2, out);
            out.push_str(&format!("{}else:\n", indent(depth + 1)));
            render_expr(else_expr, depth + 2, out);
        }
        Expr::Function { params, body } => {
            let plist = params
                .iter()
                .map(|p| p.name.clone())
                .collect::<Vec<_>>()
                .join(", ");
            out.push_str(&format!("{pad}Function ({plist})\n"));
            for s in body {
                render_stmt(s, depth + 1, out);
            }
        }
        Expr::Array(items) => {
            out.push_str(&format!("{pad}Array [{}]\n", items.len()));
            for item in items {
                render_expr(item, depth + 1, out);
            }
        }
        Expr::Switch { value, cases } => {
            out.push_str(&format!("{pad}Switch\n"));
            out.push_str(&format!("{}value:\n", indent(depth + 1)));
            render_expr(value, depth + 2, out);
            for (i, (pat, result)) in cases.iter().enumerate() {
                out.push_str(&format!("{}case[{i}] pattern:\n", indent(depth + 1)));
                render_expr(pat, depth + 2, out);
                out.push_str(&format!("{}case[{i}] result:\n", indent(depth + 1)));
                render_expr(result, depth + 2, out);
            }
        }
        Expr::IfExpr {
            condition,
            then_expr,
            else_if_branches,
            else_expr,
        } => {
            out.push_str(&format!("{pad}IfExpr\n"));
            out.push_str(&format!("{}cond:\n", indent(depth + 1)));
            render_expr(condition, depth + 2, out);
            out.push_str(&format!("{}then:\n", indent(depth + 1)));
            render_expr(then_expr, depth + 2, out);
            for (i, (c, e)) in else_if_branches.iter().enumerate() {
                out.push_str(&format!("{}else-if[{i}] cond:\n", indent(depth + 1)));
                render_expr(c, depth + 2, out);
                out.push_str(&format!("{}else-if[{i}] expr:\n", indent(depth + 1)));
                render_expr(e, depth + 2, out);
            }
            if let Some(e) = else_expr {
                out.push_str(&format!("{}else:\n", indent(depth + 1)));
                render_expr(e, depth + 2, out);
            }
        }
    }
}

fn render_literal(lit: &syntax::ast::Literal) -> String {
    use syntax::ast::Literal;
    match lit {
        Literal::Number(n) => format!("Number {n}"),
        Literal::String(s) => format!("String {s:?}"),
        Literal::Bool(b) => format!("Bool {b}"),
        Literal::Na => "Na".to_string(),
        Literal::HexColor(c) => format!("HexColor {c}"),
    }
}

fn indent(depth: usize) -> String {
    "  ".repeat(depth)
}

fn cmd_tokens(code: &str, format: ResolvedFormat) -> Result<()> {
    let mut lexer = syntax::Lexer::new(code);
    let tokens = lexer
        .tokenize()
        .map_err(|e| anyhow::anyhow!("lex error: {e}"))?;

    match format {
        ResolvedFormat::Json => {
            print_json(&tokens)?;
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
    let reference_entry_count = reference::all_entries().len();
    let probe_count = corpus::list_probes(None, None).map_or(0, |v| v.len());
    let probe_summary_count = corpus::list_probes(None, None)
        .map_or(0, |v| v.iter().filter(|p| p.summary.is_some()).count());
    let audit_sections = search::audit_section_count();
    let docs_sections = search::docs_section_count();
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
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn object_payload_gets_schema_version_inline() {
        let payload = serde_json::json!({ "name": "x", "n": 1 });
        let v = versioned_json(&payload).expect("must wrap");
        assert_eq!(v["schema_version"], serde_json::json!(SCHEMA_VERSION));
        assert_eq!(v["name"], "x");
        assert_eq!(v["n"], 1);
    }

    #[test]
    fn array_payload_gets_wrapped_under_items() {
        let payload = serde_json::json!([1, 2, 3]);
        let v = versioned_json(&payload).expect("must wrap");
        assert_eq!(v["schema_version"], serde_json::json!(SCHEMA_VERSION));
        assert_eq!(v["items"], serde_json::json!([1, 2, 3]));
        assert!(v.get("name").is_none());
    }

    #[test]
    fn scalar_payload_gets_wrapped_under_value() {
        let v = versioned_json(&42).expect("must wrap");
        assert_eq!(v["schema_version"], serde_json::json!(SCHEMA_VERSION));
        assert_eq!(v["value"], 42);
    }

    #[test]
    fn schema_version_is_one() {
        // Hard-pin: bumping SCHEMA_VERSION requires updating this test AND
        // the docs in docs/pine-oracle.md "Schema versioning" section.
        assert_eq!(SCHEMA_VERSION, 1);
    }

    #[test]
    fn ast_pretty_print_smoke() {
        let mut lexer = syntax::Lexer::new("x = 5 + 3\n");
        let tokens = lexer.tokenize().expect("lex");
        let mut parser = syntax::Parser::new(tokens);
        let stmts = parser.parse().expect("parse");
        let program = syntax::Program::new(stmts);
        let mut out = String::new();
        render_program(&program, &mut out);
        // Expect a tree-shaped header + the binary expression decomposed
        // into operator + operands.
        assert!(out.starts_with("Program\n"));
        assert!(out.contains("VarDecl"));
        assert!(out.contains("Binary Add"));
        assert!(out.contains("Number 5"));
        assert!(out.contains("Number 3"));
    }

    #[test]
    fn style_disabled_returns_unwrapped_text() {
        let s = Style { enabled: false };
        assert_eq!(s.red("err"), "err");
        assert_eq!(s.yellow("warn"), "warn");
        assert_eq!(s.bold("name"), "name");
        assert_eq!(s.dim("12"), "12");
        assert_eq!(s.cyan("[reference]"), "[reference]");
    }

    #[test]
    fn style_enabled_wraps_with_ansi_escape_codes() {
        let s = Style { enabled: true };
        assert_eq!(s.red("err"), "\x1b[31merr\x1b[0m");
        assert_eq!(s.yellow("warn"), "\x1b[33mwarn\x1b[0m");
        assert_eq!(s.bold("x"), "\x1b[1mx\x1b[0m");
        assert_eq!(s.dim("12"), "\x1b[2m12\x1b[0m");
    }

    #[test]
    fn style_resolve_disables_for_json_output() {
        // Even if --no-color is unset and stdout were a tty, JSON output
        // must never carry escape codes.
        let s = Style::resolve(false, ResolvedFormat::Json);
        assert!(!s.enabled);
    }

    #[test]
    fn style_resolve_disables_with_no_color_flag() {
        // Forcing --no-color overrides any TTY auto-detect.
        let s = Style::resolve(true, ResolvedFormat::Text);
        assert!(!s.enabled);
    }
}
