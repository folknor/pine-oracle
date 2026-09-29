use anyhow::{Result, bail};
use clap::{ArgGroup, Subcommand};
use pine_oracle::verdict::{
    self, Kind, ListFilter, NewObservation, NewQuestion, Outcome, Question, Ranked, Relation,
    Source, Status, Store,
};
use std::path::PathBuf;

use crate::output::Style;

/// `po verdict` records and answers what TradingView measurably did, per
/// oracle source. It is deliberately separate from `lookup` / `search`:
/// verdicts are not baked into the binary, every verb names its records
/// directory with `--records`, and nothing is ever read from a default
/// location.
#[derive(Subcommand)]
pub(crate) enum VerdictCommand {
    /// Create a question. Prints its generated 8-hex id.
    Add {
        /// Records directory to write into (must already exist).
        #[arg(long)]
        records: PathBuf,
        #[arg(long, value_parser = ["compile", "runtime"])]
        kind: String,
        /// Capture key, unique in the directory. If a question already holds
        /// it, print that question's id when the payload matches (writing
        /// nothing) and fail when it differs, so a capture script can rerun.
        #[arg(long)]
        key: Option<String>,
        /// The question, in one sentence.
        #[arg(long)]
        question: String,
        /// The answer, in one sentence.
        #[arg(long)]
        answer: String,
        /// A pine-data identifier the question concerns. Repeatable.
        #[arg(long = "identifier")]
        identifiers: Vec<String>,
        /// Id of a question this investigation grew out of (lineage; never
        /// affects status). Repeatable.
        #[arg(long = "follow-up-to", value_name = "ID")]
        follow_up_to: Vec<String>,
        /// Id of a premise the answer is concluded from. Makes the question
        /// inferred: settled only when every premise is. Repeatable.
        #[arg(long = "basis", value_name = "ID")]
        basis: Vec<String>,
    },

    /// Append one observation to a question.
    Observe(Box<ObserveArgs>),

    /// Withdraw a question (ill-posed, mis-filed, reworded, abandoned). It
    /// keeps its observations as history and has no status from then on.
    Retire {
        id: String,
        /// Records directory holding the question.
        #[arg(long)]
        records: PathBuf,
        /// Why the question is withdrawn.
        #[arg(long)]
        reason: String,
        /// Id of a question the investigation continued in. Repeatable.
        #[arg(long = "replaced-by", value_name = "ID")]
        replaced_by: Vec<String>,
    },

    /// BM25 search over questions, answers, results, messages and fixture
    /// source. Prints `<id>  <status>  <kind>  <question>` rows.
    Search {
        query: String,
        /// Records directory to read. Repeatable.
        #[arg(long, required = true)]
        records: Vec<PathBuf>,
        #[arg(long, default_value_t = 8)]
        limit: usize,
    },

    /// Print question(s) in full, strongest observation first.
    Show {
        #[arg(required = true)]
        ids: Vec<String>,
        /// Records directory to read. Repeatable.
        #[arg(long, required = true)]
        records: Vec<PathBuf>,
    },

    /// List questions, optionally filtered. A successful run means every
    /// record under the given directories is valid.
    List {
        /// Records directory to read. Repeatable.
        #[arg(long, required = true)]
        records: Vec<PathBuf>,
        /// Only questions concerning this identifier.
        #[arg(long)]
        identifier: Option<String>,
        /// Only questions whose observations carry this diagnostic code.
        #[arg(long)]
        code: Option<String>,
        /// Resolved status; retired questions have none and never match.
        #[arg(long, value_parser = ["settled", "conflict", "open"])]
        status: Option<String>,
        /// Only inferred questions (answered from `basis` premises).
        #[arg(long)]
        inferred: bool,
        /// Only retired questions.
        #[arg(long, conflicts_with_all = ["status", "inferred"])]
        retired: bool,
        #[arg(long, value_parser = ["compile", "runtime"])]
        kind: Option<String>,
    },
}

#[derive(clap::Args)]
#[command(group(ArgGroup::new("fixture_choice").required(true).args(["fixture", "no_fixture"])))]
#[command(group(ArgGroup::new("outcome").args(["accepted", "rejected", "crashed"])))]
#[command(group(ArgGroup::new("when").required(true).args(["date", "date_before"])))]
pub(crate) struct ObserveArgs {
    /// Question id from `po verdict add`.
    id: String,
    /// Records directory holding the question.
    #[arg(long)]
    records: PathBuf,
    #[arg(long, value_parser = ["editor", "endpoint", "chart"])]
    source: String,
    /// Capture key, unique in the question. If an observation already holds
    /// it, do nothing when the payload matches and fail when it differs, so
    /// a capture script can rerun.
    #[arg(long)]
    key: Option<String>,
    /// When the observation was taken, YYYY-MM-DD.
    #[arg(long)]
    date: Option<String>,
    /// For a capture whose date was never recorded: a YYYY-MM-DD upper bound
    /// (e.g. the date of the commit that added the export).
    #[arg(long, value_name = "DATE")]
    date_before: Option<String>,
    /// The exact file measured. po copies it into the store, named by sha256.
    #[arg(long)]
    fixture: Option<PathBuf>,
    /// State that no byte-exact fixture was measured (e.g. inline code).
    #[arg(long)]
    no_fixture: bool,
    /// Compile: TradingView accepted the script.
    #[arg(long)]
    accepted: bool,
    /// Compile: TradingView rejected the script.
    #[arg(long)]
    rejected: bool,
    /// Compile: the oracle itself crashed, with this message.
    #[arg(long, value_name = "MESSAGE")]
    crashed: Option<String>,
    /// `CODE|message|detail`. Detail: a `line:col-line:col` span, `bar=N`,
    /// or `key=value` template context, comma-separated. Repeatable.
    #[arg(long = "error", value_name = "SPEC")]
    errors: Vec<String>,
    /// Compile warning, same spec as `--error`. Repeatable.
    #[arg(long = "warning", value_name = "SPEC")]
    warnings: Vec<String>,
    /// Runtime: one sentence saying what TradingView did.
    #[arg(long)]
    result: Option<String>,
    /// Runtime: chart type, symbol, timeframe and Strategy Properties.
    #[arg(long)]
    settings: Option<String>,
    /// Runtime: `name|model`, a competing model the fixture separates.
    /// Repeatable.
    #[arg(long = "candidate", value_name = "SPEC")]
    candidates: Vec<String>,
    /// Runtime: a candidate the evidence selected. Repeatable.
    #[arg(long = "selected", value_name = "NAME")]
    selected: Vec<String>,
    /// Runtime: a candidate the evidence refuted. Repeatable.
    #[arg(long = "refuted", value_name = "NAME")]
    refuted: Vec<String>,
    /// Why this observation adjudicates nothing. Shown, never counted.
    #[arg(long, value_name = "REASON")]
    inconclusive: Option<String>,
    /// Evidence file (export, probe JSON). Must exist. Repeatable.
    #[arg(long = "evidence", value_name = "PATH")]
    evidence: Vec<PathBuf>,
    /// Free text, e.g. `TradingView Desktop 3.4.1`.
    #[arg(long)]
    environment: Option<String>,
    #[arg(long)]
    note: Option<String>,
}

pub(crate) fn run(command: VerdictCommand, style: Style, quiet: bool) -> Result<()> {
    match command {
        VerdictCommand::Add {
            records,
            kind,
            key,
            question,
            answer,
            identifiers,
            follow_up_to,
            basis,
        } => {
            let added = verdict::add(
                &records,
                &NewQuestion {
                    kind: kind.parse()?,
                    key,
                    question,
                    answer,
                    identifiers,
                    follow_up_to,
                    basis,
                },
            )?;
            print_warnings(&added.warnings, quiet);
            if added.existing && !quiet {
                eprintln!("already recorded: the key's question is unchanged");
            }
            println!("{}", added.id);
            Ok(())
        }
        VerdictCommand::Retire {
            id,
            records,
            reason,
            replaced_by,
        } => verdict::retire(&records, &id, &reason, &replaced_by),
        VerdictCommand::Observe(args) => {
            let observed = verdict::observe(&args.records, &args.id, &new_observation(&args)?)?;
            print_warnings(&observed.warnings, quiet);
            if observed.existing && !quiet {
                eprintln!(
                    "already recorded as observation #{}: unchanged",
                    observed.number
                );
            }
            Ok(())
        }
        VerdictCommand::Search {
            query,
            records,
            limit,
        } => {
            let store = verdict::load(&records)?;
            let hits = store.search(&query, limit)?;
            if hits.is_empty() {
                bail!("no verdict matches `{query}`");
            }
            let rows: Vec<&Question> = hits.iter().filter_map(|h| store.get(&h.id)).collect();
            print_rows(&store, &rows, style);
            Ok(())
        }
        VerdictCommand::Show { ids, records } => {
            let store = verdict::load(&records)?;
            for (i, id) in ids.iter().enumerate() {
                let Some(q) = store.get(id) else {
                    bail!("no question `{id}` under the given --records");
                };
                if i > 0 {
                    println!();
                }
                print_question(&store, q, style);
            }
            Ok(())
        }
        VerdictCommand::List {
            records,
            identifier,
            code,
            status,
            inferred,
            retired,
            kind,
        } => {
            let store = verdict::load(&records)?;
            let filter = ListFilter {
                identifier: identifier.as_deref(),
                code: code.as_deref(),
                status: status.as_deref().map(str::parse::<Status>).transpose()?,
                inferred,
                retired,
                kind: kind.as_deref().map(str::parse::<Kind>).transpose()?,
            };
            let rows = store.list(&filter);
            if rows.is_empty() && !quiet {
                eprintln!("no questions");
            }
            print_rows(&store, &rows, style);
            Ok(())
        }
    }
}

fn new_observation(a: &ObserveArgs) -> Result<NewObservation> {
    let outcome = match (a.accepted, a.rejected, a.crashed.is_some()) {
        (true, _, _) => Some(Outcome::Accepted),
        (_, true, _) => Some(Outcome::Rejected),
        (_, _, true) => Some(Outcome::Crashed),
        _ => None,
    };
    Ok(NewObservation {
        source: a.source.parse::<Source>()?,
        key: a.key.clone(),
        date: a.date.clone(),
        date_before: a.date_before.clone(),
        fixture: a.fixture.clone(),
        outcome,
        crash: a.crashed.clone(),
        errors: parse_all(&a.errors, verdict::parse_diag)?,
        warnings: parse_all(&a.warnings, verdict::parse_diag)?,
        result: a.result.clone(),
        settings: a.settings.clone(),
        candidates: parse_all(&a.candidates, verdict::parse_candidate)?,
        selected: a.selected.clone(),
        refuted: a.refuted.clone(),
        evidence: a.evidence.clone(),
        environment: a.environment.clone(),
        note: a.note.clone(),
        inconclusive: a.inconclusive.clone(),
    })
}

fn parse_all<T>(specs: &[String], parse: impl Fn(&str) -> Result<T>) -> Result<Vec<T>> {
    specs.iter().map(|s| parse(s)).collect()
}

fn print_warnings(warnings: &[String], quiet: bool) {
    if quiet {
        return;
    }
    for w in warnings {
        eprintln!("warning: {w}");
    }
}

/// `<id>  <status>  <kind>  <question>` rows. The status column is padded to
/// the widest status in the batch, since an inferred or retired disposition
/// names other questions (`inferred (open via 3fa91c02)`).
fn print_rows(store: &Store, questions: &[&Question], style: Style) {
    let statuses: Vec<String> = questions
        .iter()
        .map(|q| store.resolve(q).to_string())
        .collect();
    let width = statuses.iter().map(String::len).max().unwrap_or(0);
    for (q, status) in questions.iter().zip(&statuses) {
        println!(
            "{}  {status:<width$}  {:<7}  {}",
            style.bold(&q.id),
            q.kind.as_str(),
            q.question
        );
    }
}

fn print_question(store: &Store, q: &Question, style: Style) {
    println!(
        "{}  {}  {}",
        style.bold(&q.id),
        q.kind.as_str(),
        style.bold(&store.resolve(q).to_string())
    );
    println!("  {}", q.question);
    match &q.retired {
        // A retired question asserts nothing: its answer is history.
        Some(r) => {
            println!("  former answer: {}", q.answer);
            println!("  retired: {}", r.reason);
        }
        None => println!("  {}", q.answer),
    }
    if q.identifiers.is_empty() {
        println!("  identifiers: none");
    } else {
        println!("  identifiers: {}", q.identifiers.join(", "));
    }
    // Each relation both ways. Linked questions print with their own
    // disposition (a premise or replacement may itself be retired).
    let with_disposition = |ids: &[String]| -> String {
        ids.iter()
            .map(|id| match store.get(id) {
                Some(linked) => format!("{id} ({})", store.resolve(linked)),
                None => id.clone(),
            })
            .collect::<Vec<_>>()
            .join(", ")
    };
    let referrers = |relation: Relation| -> Vec<String> {
        store
            .referrers(&q.id, relation)
            .into_iter()
            .map(|x| x.id.clone())
            .collect()
    };
    for (label, ids) in [
        ("follows up", q.follow_up_to.clone()),
        ("follow-ups", referrers(Relation::FollowUpTo)),
        ("basis", q.basis.clone()),
        ("premise of", referrers(Relation::Basis)),
        ("replaced by", Relation::ReplacedBy.of(q).to_vec()),
        ("replaces", referrers(Relation::ReplacedBy)),
    ] {
        if !ids.is_empty() {
            println!("  {label}: {}", with_disposition(&ids));
        }
    }
    // A replacement that was itself retired: say where the work stands now,
    // so the start of a chain does not need opening link by link.
    let replaced_by = Relation::ReplacedBy.of(q);
    if replaced_by
        .iter()
        .any(|id| store.get(id).is_some_and(Question::is_retired))
    {
        let current: Vec<String> = store
            .current_successors(q)
            .into_iter()
            .map(|s| s.id.clone())
            .collect();
        if !current.is_empty() {
            println!("  now continued in: {}", with_disposition(&current));
        }
    }
    if q.observations.is_empty() {
        println!("  observations: none");
        return;
    }
    for r in q.ranked() {
        println!();
        print_observation(&r, style);
    }
}

fn print_observation(r: &Ranked<'_>, style: Style) {
    let o = r.observation;
    let verdict = o.outcome.map_or("run", Outcome::as_str);
    let mut head = format!(
        "  #{:<3} {:<8}  {}  {verdict}",
        r.number,
        o.source.as_str(),
        o.date_string()
    );
    if let Some(note) = &r.annotation {
        head.push_str(&format!("  ({note})"));
    }
    if o.counts() {
        println!("{head}");
    } else {
        println!("{}", style.dim(&head));
    }
    let detail = |label: &str, value: &str| println!("            {label}: {value}");
    if let Some(crash) = &o.crash {
        detail("crash", crash);
    }
    for (label, diags) in [("error", &o.errors), ("warning", &o.warnings)] {
        for d in diags {
            let mut line = match &d.message {
                Some(message) => format!("{} {message}", d.code),
                None => format!("{} (message not recorded)", d.code),
            };
            if let Some(span) = &d.span {
                line.push_str(&format!(" ({span})"));
            }
            if let Some(bar) = d.bar {
                line.push_str(&format!(" (bar {bar})"));
            }
            if !d.ctx.is_empty() {
                // Quote values holding commas, as they are spelled on input.
                let ctx: Vec<String> = d
                    .ctx
                    .iter()
                    .map(|(k, v)| {
                        if v.contains(',') {
                            format!("{k}=\"{v}\"")
                        } else {
                            format!("{k}={v}")
                        }
                    })
                    .collect();
                line.push_str(&format!(" [{}]", ctx.join(", ")));
            }
            detail(label, &line);
        }
    }
    if let Some(result) = &o.result {
        detail("result", result);
    }
    if let Some(settings) = &o.settings {
        detail("settings", settings);
    }
    for c in &o.candidates {
        let marker = c.state.as_str();
        match &c.model {
            Some(model) => detail("candidate", &format!("{} {marker}: {model}", c.name)),
            None => detail("candidate", &format!("{} {marker}", c.name)),
        }
    }
    match (&o.fixture, &o.fixture_name) {
        (Some(sha), Some(name)) => detail("fixture", &format!("fixtures/{sha}.pine ({name})")),
        (Some(sha), None) => detail("fixture", &format!("fixtures/{sha}.pine")),
        (None, _) => detail("fixture", "none (no byte-exact source measured)"),
    }
    if let Some(env) = &o.environment {
        detail("environment", env);
    }
    for e in &o.evidence {
        detail("evidence", e);
    }
    if let Some(note) = &o.note {
        detail("note", note);
    }
}
