use anyhow::{Result, bail};
use clap::{ArgGroup, Subcommand};
use pine_oracle::verdict::{
    self, AmendField, AmendTarget, Amendment, AuditRow, Citation, ClosureAction, Evidence,
    Inspector, Kind, ListFilter, NewEvidence, NewObservation, NewQuestion, Outcome, Question,
    Ranked, Relation, Source, Status, Store, VoidTarget,
};
use std::path::{Path, PathBuf};

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
        /// Mark the answer negative: it reports that the behavior asked
        /// about was not identified (only what was tested is excluded).
        #[arg(long)]
        negative: bool,
        /// Runtime: the candidates are mutually exclusive alternatives (not
        /// necessarily exhaustive). Runs selecting different ones conflict,
        /// and a counting run may select at most one.
        #[arg(long)]
        exclusive: bool,
    },

    /// Append one observation to a question.
    Observe(Box<ObserveArgs>),

    /// Withdraw a question (ill-posed, mis-filed, reworded into a different
    /// claim, abandoned; a wording fix is `amend --question`). It keeps its
    /// observations as history and has no status from then on.
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

    /// Stop the investigation of an open question nobody will resolve. It
    /// stays open (nothing was learned), shows as `open, closed`, and drops
    /// out of `list --status open`.
    Close {
        id: String,
        /// Records directory holding the question.
        #[arg(long)]
        records: PathBuf,
        /// Why the investigation stops.
        #[arg(long)]
        reason: String,
    },

    /// Resume the investigation of a closed question.
    Reopen {
        id: String,
        /// Records directory holding the question.
        #[arg(long)]
        records: PathBuf,
        /// Why the investigation resumes.
        #[arg(long)]
        reason: String,
    },

    /// Correct a question's wording, answer, negative or exclusive mark, or an observation's
    /// note, result, settings, environment or inconclusive reason. The old
    /// value, the time and the reason are kept beside the field; `show`
    /// prints them.
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

    /// Maintain and audit the evidence files observations point at.
    #[command(subcommand)]
    Evidence(EvidenceCommand),

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
        /// `open` is the owed-work view and leaves out closed questions.
        #[arg(long, value_parser = ["settled", "documented", "conflict", "open"])]
        status: Option<String>,
        /// Only open questions whose investigation is closed.
        #[arg(long, conflicts_with_all = ["status", "retired"])]
        closed: bool,
        /// Only questions whose answer is marked negative.
        #[arg(long)]
        negative: bool,
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

/// `po verdict evidence`: evidence files keep their identity (the sha256 of
/// their decoded content) apart from their location, so they can be
/// compressed or moved without breaking the records that cite them.
#[derive(Subcommand)]
pub(crate) enum EvidenceCommand {
    /// Re-point every entry whose current location is OLD to NEW, which must
    /// hold the same content, raw or decoded (an entry without a digest: as
    /// OLD reads now, and it gains that digest, marked attested). Checks
    /// every entry before writing any record; never deletes a file. Rerun
    /// after an interruption to move the rest.
    Move {
        old: PathBuf,
        new: PathBuf,
        /// Records directory to update.
        #[arg(long)]
        records: PathBuf,
        /// Why the file moved (e.g. compressed to save LFS space).
        #[arg(long)]
        reason: String,
        /// Read OLD as it is, never decoded, for entries without a digest.
        #[arg(long)]
        old_raw: bool,
    },

    /// Merge the entry at REDUNDANT into the entry at INTO, two evidence
    /// entries of one observation citing the same content (equal digests;
    /// no file is read). The redundant entry is kept under the survivor, so
    /// its locations still name it in keyed reruns. A rerun is a no-op.
    Merge {
        /// Question id.
        id: String,
        redundant: PathBuf,
        into: PathBuf,
        /// The observation's #N.
        #[arg(long)]
        observation: usize,
        /// Records directory to update.
        #[arg(long)]
        records: PathBuf,
        /// Why the two entries are one citation.
        #[arg(long)]
        reason: String,
    },

    /// Record the digest of entries that have none (those at the given
    /// paths, or every one in the store), read now and marked attested: it
    /// identifies the content from now on, not the original capture.
    Attest {
        paths: Vec<PathBuf>,
        /// Records directory to update.
        #[arg(long)]
        records: PathBuf,
        /// Why the files are taken as the observed ones.
        #[arg(long)]
        reason: String,
        /// Read the files as they are, never decoded.
        #[arg(long)]
        raw: bool,
    },

    /// Read every evidence file of the given questions (all when none) and
    /// compare it with its digest. Fails when any file is missing,
    /// unreadable, undecodable, an unfetched Git LFS pointer, or changed.
    /// Entries without a digest fail only when unavailable.
    Check {
        ids: Vec<String>,
        /// Records directory to read. Repeatable.
        #[arg(long, required = true)]
        records: Vec<PathBuf>,
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
    /// Runtime: a candidate the run establishes. Repeatable (candidates are
    /// not assumed exclusive).
    #[arg(long = "selected", value_name = "NAME")]
    selected: Vec<String>,
    /// Runtime: a candidate the result fits without establishing it (it
    /// fits another too). Never decides, never conflicts. Repeatable.
    #[arg(long = "consistent", value_name = "NAME")]
    consistent: Vec<String>,
    /// Runtime: a candidate the evidence refuted. Repeatable.
    #[arg(long = "refuted", value_name = "NAME")]
    refuted: Vec<String>,
    /// Why this observation adjudicates nothing. Shown, never counted.
    #[arg(long, value_name = "REASON")]
    inconclusive: Option<String>,
    /// Evidence file (export, probe JSON), read now: po records its location
    /// and the sha256 of its content (a gzip or zstd file decoded).
    /// Repeatable.
    #[arg(long = "evidence", value_name = "PATH")]
    evidence: Vec<PathBuf>,
    /// Evidence file hashed as it is, never decoded: for an artifact that
    /// merely starts like a gzip or zstd stream. Repeatable.
    #[arg(long = "evidence-raw", value_name = "PATH")]
    evidence_raw: Vec<PathBuf>,
    /// Free text, e.g. `TradingView Desktop 3.4.1`.
    #[arg(long)]
    environment: Option<String>,
    #[arg(long)]
    note: Option<String>,
    /// Check the observation and print the question's resulting status,
    /// writing nothing.
    #[arg(long)]
    dry_run: bool,
}

#[derive(clap::Args)]
#[command(group(ArgGroup::new("change").required(true).multiple(true).args([
    "question", "answer", "negative", "exclusive", "note", "result", "settings", "environment", "inconclusive", "clear",
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
    /// The reworded question: a wording correction, not a new claim (a
    /// claim whose evidence splits is retired and replaced). Refused on a
    /// retired question.
    #[arg(long, conflicts_with = "observation")]
    question: Option<String>,
    /// The corrected answer. Combines with `--negative` or `--clear
    /// negative`, so an answer and its classification change in one write.
    #[arg(long, conflicts_with = "observation")]
    answer: Option<String>,
    /// Mark the answer negative: it reports that the behavior asked about
    /// was not identified.
    #[arg(long, conflicts_with = "observation")]
    negative: bool,
    /// Mark the runtime candidates exclusive alternatives: runs selecting
    /// different ones then conflict. Refused while a counting run selects
    /// two.
    #[arg(long, conflicts_with = "observation")]
    exclusive: bool,
    /// The observation to correct: its `#N`, as `show` prints it.
    #[arg(long, value_name = "N")]
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
    /// Remove an optional field: the question's `negative` or `exclusive`
    /// mark, or an observation's `note`, `environment`, or `inconclusive`
    /// (the run counts again).
    #[arg(long, value_name = "FIELD", value_parser = ["negative", "exclusive", "note", "environment", "inconclusive"])]
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
            negative,
            exclusive,
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
                    negative,
                    exclusive,
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
        } => {
            print_warnings(
                &verdict::retire(&records, &id, &reason, &replaced_by)?,
                quiet,
            );
            Ok(())
        }
        VerdictCommand::Observe(args) => {
            let new = new_observation(&args)?;
            let observed = if args.dry_run {
                verdict::observe_dry_run(&args.records, &args.id, &new)?
            } else {
                verdict::observe(&args.records, &args.id, &new)?
            };
            print_warnings(&observed.warnings, quiet);
            let (id, n, was, now) = (&observed.id, observed.number, &observed.was, &observed.now);
            // The answer a dry run was asked for, so stdout and never quieted.
            if args.dry_run {
                if observed.existing {
                    println!(
                        "dry run: already recorded as observation #{n}; question {id} stays {now}"
                    );
                } else if was == now {
                    println!("dry run: observation #{n} would leave question {id} {now}");
                } else {
                    println!("dry run: observation #{n} would make question {id} {now} (is {was})");
                }
            } else if observed.existing && !quiet {
                eprintln!("already recorded as observation #{n}: unchanged");
            }
            Ok(())
        }
        VerdictCommand::Close {
            id,
            records,
            reason,
        } => {
            print_warnings(&verdict::close(&records, &id, &reason)?, quiet);
            Ok(())
        }
        VerdictCommand::Reopen {
            id,
            records,
            reason,
        } => {
            print_warnings(&verdict::reopen(&records, &id, &reason)?, quiet);
            Ok(())
        }
        VerdictCommand::Amend(args) => {
            let a = *args;
            let changes = amend_changes(&a)?;
            let borrowed: Vec<(AmendTarget, Option<&str>)> =
                changes.iter().map(|(t, v)| (*t, v.as_deref())).collect();
            let amended = verdict::amend_all(&a.records, &a.id, &borrowed, &a.reason)?;
            print_warnings(&amended.warnings, quiet);
            if let Some(support) = amended.support
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
            print_warnings(&cited.warnings, quiet);
            if !quiet {
                if cited.existing {
                    eprintln!("already cited as citation #{}: unchanged", cited.number);
                }
                for line in &cited.outcome {
                    eprintln!("{line}");
                }
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
            print_warnings(&verdict::void(&records, &id, target, &reason)?, quiet);
            Ok(())
        }
        VerdictCommand::Evidence(command) => run_evidence(command, quiet),
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
            let mut inspector = Inspector::new();
            for (i, id) in ids.iter().enumerate() {
                let Some(q) = store.get(id) else {
                    bail!("no question `{id}` under the given --records");
                };
                if i > 0 {
                    println!();
                }
                print_question(&store, q, &mut inspector, style);
            }
            inspector.stable()
        }
        VerdictCommand::List {
            records,
            identifier,
            code,
            status,
            closed,
            negative,
            inferred,
            retired,
            kind,
        } => {
            let store = verdict::load(&records)?;
            let filter = ListFilter {
                identifier: identifier.as_deref(),
                code: code.as_deref(),
                status: status.as_deref().map(str::parse::<Status>).transpose()?,
                closed,
                negative,
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

fn run_evidence(command: EvidenceCommand, quiet: bool) -> Result<()> {
    match command {
        EvidenceCommand::Move {
            old,
            new,
            records,
            reason,
            old_raw,
        } => {
            let moved = verdict::relocate(&records, &old, &new, old_raw, &reason)?;
            if moved.entries == 0 {
                println!(
                    "every entry that was at {} has moved already: nothing moved",
                    old.display()
                );
            } else {
                println!(
                    "moved {} remaining evidence entr{} in {} question(s){}",
                    moved.entries,
                    if moved.entries == 1 { "y" } else { "ies" },
                    moved.questions,
                    if moved.attested > 0 {
                        format!(
                            "; {} had no digest and now carry one attested by this move",
                            moved.attested
                        )
                    } else {
                        String::new()
                    }
                );
                if !quiet {
                    eprintln!(
                        "{} is no longer referenced by this store; other stores may still cite it",
                        old.display()
                    );
                }
            }
            Ok(())
        }
        EvidenceCommand::Merge {
            id,
            redundant,
            into,
            observation,
            records,
            reason,
        } => {
            match verdict::merge(&records, &id, observation, &redundant, &into, &reason)? {
                verdict::Merged::Merged => println!(
                    "merged {} into {} ({id} #{observation})",
                    redundant.display(),
                    into.display()
                ),
                verdict::Merged::Already => println!(
                    "{} is already merged into {}: nothing to do",
                    redundant.display(),
                    into.display()
                ),
            }
            Ok(())
        }
        EvidenceCommand::Attest {
            paths,
            records,
            reason,
            raw,
        } => {
            let done = verdict::attest(&records, &paths, raw, &reason)?;
            if done.entries == 0 {
                println!("no evidence entry without a digest: nothing attested");
            } else {
                println!(
                    "attested {} evidence entr{} ({} file(s)) in {} question(s)",
                    done.entries,
                    if done.entries == 1 { "y" } else { "ies" },
                    done.files,
                    done.questions
                );
            }
            Ok(())
        }
        EvidenceCommand::Check { ids, records } => {
            let store = verdict::load(&records)?;
            let audit = verdict::audit(&store, &ids)?;
            let row =
                |r: &AuditRow| format!("{} #{}  {}  {}", r.id, r.observation, r.path, r.inspection);
            for r in &audit.failing {
                println!("{}", row(r));
            }
            if !quiet {
                for r in &audit.unhashed {
                    eprintln!("{}", row(r));
                }
            }
            let summary = format!(
                "{} evidence entr{}: {} match, {} without a digest, {} failing",
                audit.entries,
                if audit.entries == 1 { "y" } else { "ies" },
                audit.matching,
                audit.unhashed.len(),
                audit.failing.len()
            );
            if !audit.failing.is_empty() {
                bail!("{summary}");
            }
            println!("{summary}");
            Ok(())
        }
    }
}

/// The changes one `amend` call makes. Question level: `--question`,
/// `--answer`, the negative mark (`--negative` / `--clear negative`) and the
/// exclusive mark (`--exclusive` / `--clear exclusive`), in one write. With
/// `--observation N`: exactly one of its prose fields.
fn amend_changes(a: &AmendArgs) -> Result<Vec<(AmendTarget, Option<String>)>> {
    let clear_negative = a.clear.as_deref() == Some("negative");
    let clear_exclusive = a.clear.as_deref() == Some("exclusive");
    let question_clear = clear_negative || clear_exclusive;
    let observation_fields: Vec<(AmendField, Option<String>)> = [
        (AmendField::Note, a.note.clone()),
        (AmendField::Result, a.result.clone()),
        (AmendField::Settings, a.settings.clone()),
        (AmendField::Environment, a.environment.clone()),
        (AmendField::Inconclusive, a.inconclusive.clone()),
    ]
    .into_iter()
    .filter(|(_, v)| v.is_some())
    .chain(
        a.clear
            .as_deref()
            .filter(|_| !question_clear)
            .map(|f| f.parse().map(|field| (field, None)))
            .transpose()?,
    )
    .collect();
    let Some(number) = a.observation else {
        if !observation_fields.is_empty() {
            bail!("an observation field needs --observation N");
        }
        if a.negative && clear_negative {
            bail!("--negative and --clear negative contradict each other");
        }
        let mut changes = Vec::new();
        if let Some(question) = &a.question {
            changes.push((AmendTarget::Question, Some(question.clone())));
        }
        if let Some(answer) = &a.answer {
            changes.push((AmendTarget::Answer, Some(answer.clone())));
        }
        if a.negative {
            changes.push((AmendTarget::Negative, Some("true".to_string())));
        }
        if clear_negative {
            changes.push((AmendTarget::Negative, None));
        }
        if a.exclusive && clear_exclusive {
            bail!("--exclusive and --clear exclusive contradict each other");
        }
        if a.exclusive {
            changes.push((AmendTarget::Exclusive, Some("true".to_string())));
        }
        if clear_exclusive {
            changes.push((AmendTarget::Exclusive, None));
        }
        if changes.is_empty() {
            bail!(
                "name --question, --answer, --negative, --exclusive, --clear negative, --clear exclusive or --observation N"
            );
        }
        return Ok(changes);
    };
    if let Some(field) = a.clear.as_deref().filter(|_| question_clear) {
        bail!("`{field}` is a question field: drop --observation");
    }
    match observation_fields.as_slice() {
        [(field, value)] => Ok(vec![(
            AmendTarget::Observation {
                number,
                field: *field,
            },
            value.clone(),
        )]),
        [] => bail!("nothing to amend on observation #{number}"),
        _ => bail!("amend one observation field at a time"),
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
        consistent: a.consistent.clone(),
        refuted: a.refuted.clone(),
        evidence: a
            .evidence
            .iter()
            .map(|p| (p, false))
            .chain(a.evidence_raw.iter().map(|p| (p, true)))
            .map(|(path, raw)| NewEvidence {
                path: path.clone(),
                raw,
            })
            .collect(),
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
        .map(|q| store.disposition(q).to_string())
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

fn print_question(store: &Store, q: &Question, inspector: &mut Inspector, style: Style) {
    println!(
        "{}  {}  {}",
        style.bold(&q.id),
        q.kind.as_str(),
        style.bold(&store.disposition(q).to_string())
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
    if q.negative {
        let which = if q.is_retired() {
            "former answer"
        } else {
            "answer"
        };
        println!("  negative: the {which} does not identify the behavior asked about");
    }
    if q.exclusive {
        // A retired question's observations are no longer compared, so its
        // mark is history, like a retired negative mark.
        if q.is_retired() {
            println!("  exclusive: the candidates were declared mutually exclusive alternatives");
        } else {
            println!(
                "  exclusive: the candidates are mutually exclusive alternatives (runs selecting different ones conflict)"
            );
        }
    }
    for c in &q.closures {
        let action = match c.action {
            ClosureAction::Close => "closed",
            ClosureAction::Reopen if c.automatic => "reopened automatically",
            ClosureAction::Reopen => "reopened",
        };
        println!("  {action} {}: {}", c.at, c.reason);
    }
    // A closure po never ended (hand edits, or a read-time change such as a
    // citation turning current) is suspended while the question is decided.
    if q.is_closed() && !q.is_retired() && !store.disposition(q).closed {
        println!("  closure suspended: the question does not resolve open");
    }
    if q.every_candidate_refuted() {
        println!("  candidates: every declared candidate refuted, none selected");
    }
    if q.identifiers.is_empty() {
        println!("  identifiers: none");
    } else {
        // Stored identifiers are never re-checked against pine-data (a
        // refresh must not fail the load), so one a refresh renamed or
        // dropped is marked here, prompting a hand edit.
        let snapshot = pine_oracle::behavior::snapshot().generated_at;
        let idents: Vec<String> = q
            .identifiers
            .iter()
            .map(|i| {
                if verdict::identifier_known(i) {
                    i.clone()
                } else {
                    format!("{i} (not in pine-data snapshot {snapshot})")
                }
            })
            .collect();
        println!("  identifiers: {}", idents.join(", "));
    }
    // Each relation both ways. Linked questions print with their own
    // disposition (a premise or replacement may itself be retired).
    let with_disposition = |ids: &[String]| -> String {
        ids.iter()
            .map(|id| match store.get(id) {
                Some(linked) => format!("{id} ({})", store.disposition(linked)),
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
        print_observation(&r, &q.root, inspector, style);
    }
}

fn print_observation(r: &Ranked<'_>, root: &Path, inspector: &mut Inspector, style: Style) {
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
        print_evidence(e, root, inspector);
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

/// One evidence entry: its location, the file's state read now, then its
/// history.
fn print_evidence(e: &Evidence, root: &Path, inspector: &mut Inspector) {
    let indent = "            ";
    println!(
        "{indent}evidence: {} ({})",
        e.path,
        inspector.inspect(root, e)
    );
    print_evidence_history(e, &format!("{indent}  "));
}

/// Where an entry's digest came from, the locations it left, and the
/// entries merged into it (never read: history, each with its own).
fn print_evidence_history(e: &Evidence, indent: &str) {
    if let Some(sha) = &e.sha256 {
        match &e.attested {
            Some(a) => println!(
                "{indent}sha256 {sha}, attested {}: {} (identifies the content from then on, not the capture)",
                a.at, a.reason
            ),
            None => println!("{indent}sha256 {sha}, recorded at capture"),
        }
    }
    for m in &e.moves {
        println!("{indent}moved {} from {}: {}", m.at, m.from, m.reason);
    }
    for m in &e.merged {
        println!(
            "{indent}merged {} from {}: {}",
            m.at, m.entry.path, m.reason
        );
        print_evidence_history(&m.entry, &format!("{indent}  "));
    }
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
