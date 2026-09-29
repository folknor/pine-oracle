use anyhow::{Result, bail};
use clap::{ArgGroup, Subcommand};
use pine_oracle::verdict::{
    self, AmendField, AmendTarget, Amendment, Citation, Kind, ListFilter, NewObservation,
    NewQuestion, Outcome, Question, Ranked, Relation, Source, Status, Store, VoidTarget,
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

    /// Correct a question's answer, or an observation's note, result,
    /// settings, environment or inconclusive reason. The old value, the
    /// time and the reason are kept beside the field; `show` prints them.
    Amend(Box<AmendArgs>),

    /// Cite a Pine User Manual passage as support for the answer. The
    /// citations jointly assert the whole answer; po checks that the section
    /// says the quotes, never that they imply the answer. A question nothing
    /// measured decides becomes `documented`, never `settled`.
    Cite {
        id: String,
        /// Records directory holding the question.
        #[arg(long)]
        records: PathBuf,
        /// The manual section: the 8-hex id `po search` prints, or
        /// `page#anchor`.
        #[arg(long)]
        section: String,
        /// A verbatim passage from the section (within one paragraph, list
        /// item or table cell). Repeatable.
        #[arg(long = "quote", value_name = "TEXT", required = true)]
        quotes: Vec<String>,
        #[arg(long)]
        note: Option<String>,
    },

    /// Withdraw a wrongly recorded observation (a mis-transcribed outcome,
    /// code or candidate) or a citation (stale and reviewed, or applied in
    /// error). It stays visible and never counts; record the correction with
    /// `observe` or `cite`.
    #[command(group(ArgGroup::new("record").required(true).args(["observation", "citation"])))]
    Void {
        id: String,
        /// Records directory holding the question.
        #[arg(long)]
        records: PathBuf,
        /// The observation's `#N`, as `show` prints it.
        #[arg(long, value_name = "N")]
        observation: Option<usize>,
        /// The citation's `#N`, as `show` prints it.
        #[arg(long, value_name = "N")]
        citation: Option<usize>,
        /// Why it is withdrawn.
        #[arg(long)]
        reason: String,
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
        #[arg(long, value_parser = ["settled", "documented", "conflict", "open"])]
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

#[derive(clap::Args)]
#[command(group(ArgGroup::new("change").required(true).args([
    "answer", "note", "result", "settings", "environment", "inconclusive", "clear",
])))]
pub(crate) struct AmendArgs {
    /// Question id.
    id: String,
    /// Records directory holding the question.
    #[arg(long)]
    records: PathBuf,
    /// Why the old value was wrong.
    #[arg(long)]
    reason: String,
    /// The corrected answer.
    #[arg(long, conflicts_with = "observation")]
    answer: Option<String>,
    /// The observation to correct: its `#N`, as `show` prints it.
    #[arg(long, value_name = "N", required_unless_present = "answer")]
    observation: Option<usize>,
    /// The corrected note.
    #[arg(long)]
    note: Option<String>,
    /// The corrected runtime result.
    #[arg(long)]
    result: Option<String>,
    /// The corrected runtime settings.
    #[arg(long)]
    settings: Option<String>,
    /// The corrected environment.
    #[arg(long)]
    environment: Option<String>,
    /// Mark the observation inconclusive (or reword the reason): it stops
    /// counting.
    #[arg(long, value_name = "REASON")]
    inconclusive: Option<String>,
    /// Remove an optional field: `note`, `environment`, or `inconclusive`
    /// (the run counts again).
    #[arg(long, value_name = "FIELD", value_parser = ["note", "environment", "inconclusive"])]
    clear: Option<String>,
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
        VerdictCommand::Amend(args) => {
            let a = *args;
            let (target, value) = match (a.observation, a.answer) {
                (_, Some(answer)) => (AmendTarget::Answer, Some(answer)),
                (Some(number), None) => {
                    let (field, value) = match (
                        a.note,
                        a.result,
                        a.settings,
                        a.environment,
                        a.inconclusive,
                        a.clear,
                    ) {
                        (Some(v), ..) => (AmendField::Note, Some(v)),
                        (_, Some(v), ..) => (AmendField::Result, Some(v)),
                        (_, _, Some(v), ..) => (AmendField::Settings, Some(v)),
                        (_, _, _, Some(v), ..) => (AmendField::Environment, Some(v)),
                        (_, _, _, _, Some(v), _) => (AmendField::Inconclusive, Some(v)),
                        (.., Some(field)) => (field.parse()?, None),
                        _ => bail!("nothing to amend"),
                    };
                    (AmendTarget::Observation { number, field }, value)
                }
                (None, None) => bail!("name --answer or --observation N"),
            };
            let support = verdict::amend(&a.records, &a.id, target, value.as_deref(), &a.reason)?;
            if let Some(support) = support
                && !quiet
            {
                eprintln!("{support}");
            }
            Ok(())
        }
        VerdictCommand::Cite {
            id,
            records,
            section,
            quotes,
            note,
        } => {
            let cited = verdict::cite(&records, &id, &section, &quotes, note.as_deref())?;
            if cited.existing && !quiet {
                eprintln!("already cited as citation #{}: unchanged", cited.number);
            }
            Ok(())
        }
        VerdictCommand::Void {
            id,
            records,
            observation,
            citation,
            reason,
        } => {
            let target = match (observation, citation) {
                (Some(n), _) => VoidTarget::Observation(n),
                (None, Some(n)) => VoidTarget::Citation(n),
                (None, None) => bail!("name --observation N or --citation N"),
            };
            verdict::void(&records, &id, target, &reason)
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
            print_amendments("    ", &q.amendments);
            println!("  retired: {}", r.reason);
        }
        None => {
            println!("  {}", q.answer);
            print_amendments("    ", &q.amendments);
        }
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
    for (i, c) in q.citations.iter().enumerate() {
        println!();
        print_citation(i + 1, c, style);
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
    if let Some(v) = &o.void {
        // The reason is already in the header annotation.
        detail("voided", &v.at.to_string());
    }
    print_amendments("            ", &o.amendments);
}

/// One citation: `#N  manual  <8-hex id>  <at>`, then where it points and
/// what it quotes. A void or stale one is dimmed and says why.
fn print_citation(number: usize, c: &Citation, style: Style) {
    let info = c.section_info();
    let handle = info.as_ref().map_or("(gone)", |(id, _)| id.as_str());
    let mut head = format!("  #{number:<3} {:<8}  {handle}  {}", "manual", c.at);
    let not_counting = c.not_counting();
    match &not_counting {
        Some(why) => {
            head.push_str(&format!("  ({why})"));
            println!("{}", style.dim(&head));
        }
        None => println!("{head}"),
    }
    let detail = |label: &str, value: &str| println!("            {label}: {value}");
    detail("section", &c.section);
    if let Some((_, breadcrumb)) = &info {
        detail("breadcrumb", breadcrumb);
    }
    for quote in &c.quotes {
        detail("quote", quote);
    }
    if let Some(note) = &c.note {
        detail("note", note);
    }
    if let Some(v) = &c.void {
        detail("voided", &v.at.to_string());
    }
}

/// Each correction, oldest first: which field, when, why, and what it read
/// before (the value after is the next amendment's `was`, or the current
/// field).
fn print_amendments(indent: &str, amendments: &[Amendment]) {
    for a in amendments {
        println!(
            "{indent}amended {} {}: {}",
            a.field.as_str(),
            a.at,
            a.reason
        );
        let was = a.was.as_deref().unwrap_or("(absent)");
        match &a.now {
            Some(_) => println!("{indent}  was: {was}"),
            None => println!("{indent}  cleared, was: {was}"),
        }
    }
}
