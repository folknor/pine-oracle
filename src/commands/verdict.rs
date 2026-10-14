use anyhow::{Context as _, Result, bail};
use clap::{ArgGroup, Subcommand};
use pine_oracle::verdict::{
    self, AmendField, AmendTarget, Amendment, AuditRow, Citation, ClosureAction, Evidence,
    Inspector, Kind, ListFilter, NewEvidence, NewObservation, NewQuestion, Outcome, Question,
    Ranked, Relation, ReleaseScope, Source, Status, Store, VoidTarget, cited,
};
use std::path::{Path, PathBuf};

use crate::output::Style;

/// `po verdict` records and answers what TradingView measurably did, per
/// oracle source. It is deliberately separate from `lookup` / `search`:
/// verdicts are not baked into the binary but live in `verdicts/` at the root
/// of the git repository the command runs in.
#[derive(Subcommand)]
pub(crate) enum VerdictCommand {
    /// Create a question. Prints its generated 8-hex id.
    Add {
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
        /// Why the investigation stops.
        #[arg(long)]
        reason: String,
    },

    /// Resume the investigation of a closed question.
    Reopen {
        id: String,
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
        #[arg(long, default_value_t = 8)]
        limit: usize,
    },

    /// Check every verdict cited in the repository: each cited id exists and
    /// is settled, or its line says `open`, `conflict` or `documented` after
    /// the id, matching its status; a retired one always fails. A citation
    /// is an 8-hex id after the word `verdict` or `verdicts`. Prints
    /// `path:line: verdict <id> ...` per failure; fails on any, and when
    /// nothing is cited at all. `verdicts/` is never scanned.
    Citations {
        /// Git pathspecs to scan (tracked files, and untracked ones git does
        /// not ignore), relative to the working directory; `:(exclude)...`
        /// works. None scans the whole repository.
        #[arg(value_name = "PATHSPEC")]
        pathspecs: Vec<String>,
    },

    /// Print question(s) in full, strongest observation first (with
    /// `--json`, observations are in file order).
    Show {
        #[arg(required = true)]
        ids: Vec<String>,
        /// Print JSON instead (`{"schema": 1, "questions": [...]}`, one
        /// question per id in the order given; see docs/verdict.md). Paths
        /// are from the repository root; evidence files are not read.
        #[arg(long)]
        json: bool,
    },

    /// List questions, optionally filtered. A successful run means every
    /// record under `verdicts/` is valid.
    List {
        /// Print the listed questions as JSON, in full, as `show --json`
        /// does (an empty `questions` when none match).
        #[arg(long)]
        json: bool,
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
        /// Why the two entries are one citation.
        #[arg(long)]
        reason: String,
    },

    /// Record the digest of entries that have none (those at the given
    /// paths, or every one in the store), read now and marked attested: it
    /// identifies the content from now on, not the original capture.
    Attest {
        paths: Vec<PathBuf>,
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
    /// Entries without a digest fail only when unavailable; released ones
    /// are not read.
    Check { ids: Vec<String> },

    /// Stop requiring evidence files of a void observation: the entries keep
    /// their digests and locations as the record of what was cited, and are
    /// never read again. No file is read or deleted; prints which other
    /// entries still require each location.
    #[command(group(ArgGroup::new("which").required(true).args(["paths", "all"])))]
    Release {
        /// Question id.
        id: String,
        /// Locations of the entries to release (any location of an entry).
        paths: Vec<PathBuf>,
        /// The void observation's #N.
        #[arg(long)]
        observation: usize,
        /// Release every evidence entry of the observation.
        #[arg(long)]
        all: bool,
        /// Why the files are no longer needed.
        #[arg(long)]
        reason: String,
    },
}

#[derive(clap::Args)]
#[command(group(ArgGroup::new("fixture_choice").required(true).args(["fixture", "no_fixture"])))]
#[command(group(ArgGroup::new("outcome").args(["accepted", "rejected", "crashed"])))]
#[command(group(ArgGroup::new("when").required(true).args(["date", "date_before"])))]
pub(crate) struct ObserveArgs {
    /// Question id from `po verdict add`.
    id: String,
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
    /// Evidence file (export, probe JSON) under `verdicts/evidence/`, read
    /// now: po records its location and the sha256 of its content (a gzip or
    /// zstd file decoded). Repeatable.
    #[arg(long = "evidence", value_name = "PATH")]
    evidence: Vec<PathBuf>,
    /// Evidence file under `verdicts/evidence/` hashed as it is, never
    /// decoded: for an artifact that merely starts like a gzip or zstd
    /// stream. Repeatable.
    #[arg(long = "evidence-raw", value_name = "PATH")]
    evidence_raw: Vec<PathBuf>,
    /// The earlier observation `#N` this one replaces (a reanalysis, a
    /// corrected transcription), voided in the same write unless it already
    /// is. Needs --supersede-reason.
    #[arg(long, value_name = "N", requires = "supersede_reason")]
    supersedes: Option<usize>,
    /// Why the new observation replaces the old one.
    #[arg(long, value_name = "REASON", requires = "supersedes")]
    supersede_reason: Option<String>,
    /// Release this evidence of the superseded observation (any of its
    /// locations) in the same write, for the supersede reason. Repeatable.
    #[arg(
        long,
        value_name = "PATH",
        requires = "supersedes",
        conflicts_with = "release_all"
    )]
    release: Vec<PathBuf>,
    /// Release every evidence entry of the superseded observation.
    #[arg(long, requires = "supersedes")]
    release_all: bool,
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
    let root = verdict::locate()?;
    match command {
        VerdictCommand::Add {
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
                &root,
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
            reason,
            replaced_by,
        } => {
            print_warnings(&verdict::retire(&root, &id, &reason, &replaced_by)?, quiet);
            Ok(())
        }
        VerdictCommand::Observe(args) => {
            let new = new_observation(&args)?;
            let observed = if args.dry_run {
                verdict::observe_dry_run(&root, &args.id, &new)?
            } else {
                verdict::observe(&root, &args.id, &new)?
            };
            print_warnings(&observed.warnings, quiet);
            if !quiet {
                print_holders(&observed.released, true, args.dry_run && !observed.existing);
            }
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
        VerdictCommand::Close { id, reason } => {
            print_warnings(&verdict::close(&root, &id, &reason)?, quiet);
            Ok(())
        }
        VerdictCommand::Reopen { id, reason } => {
            print_warnings(&verdict::reopen(&root, &id, &reason)?, quiet);
            Ok(())
        }
        VerdictCommand::Amend(args) => {
            let a = *args;
            let changes = amend_changes(&a)?;
            let borrowed: Vec<(AmendTarget, Option<&str>)> =
                changes.iter().map(|(t, v)| (*t, v.as_deref())).collect();
            let amended = verdict::amend_all(&root, &a.id, &borrowed, &a.reason)?;
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
            section,
            quotes,
            note,
        } => {
            let cited = verdict::cite(&root, &id, &section, &quotes, note.as_deref())?;
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
            observation,
            citation,
            reason,
        } => {
            let target = match (observation, citation) {
                (Some(n), _) => VoidTarget::Observation(n),
                (None, Some(n)) => VoidTarget::Citation(n),
                (None, None) => bail!("name --observation N or --citation N"),
            };
            print_warnings(&verdict::void(&root, &id, target, &reason)?, quiet);
            Ok(())
        }
        VerdictCommand::Evidence(command) => run_evidence(command, &root, quiet),
        VerdictCommand::Search { query, limit } => {
            let store = verdict::load(&root)?;
            let hits = store.search(&query, limit)?;
            if hits.is_empty() {
                bail!("no verdict matches `{query}`");
            }
            let rows: Vec<&Question> = hits.iter().filter_map(|h| store.get(&h.id)).collect();
            print_rows(&store, &rows, style);
            Ok(())
        }
        VerdictCommand::Citations { pathspecs } => {
            let store = verdict::load(&root)?;
            let repo = root.parent().unwrap_or(&root);
            let cwd = std::env::current_dir()?;
            let (mut cited, mut failing, mut files) = (0, 0, 0);
            let mut ids = std::collections::BTreeSet::new();
            for path in cited::files(repo, &cwd, &pathspecs)? {
                let full = repo.join(&path);
                // Skipped, as citing nothing here: a tracked file deleted
                // from the working tree, a submodule (listed as its
                // directory; its own repository is checked on its own), and
                // a link (its target is another file, or outside the tree).
                match std::fs::symlink_metadata(&full) {
                    Err(e) if e.kind() == std::io::ErrorKind::NotFound => continue,
                    Ok(m) if m.is_dir() || m.file_type().is_symlink() => continue,
                    _ => {}
                }
                let text = match std::fs::read(&full) {
                    // Binary (a NUL byte, valid UTF-8 or not) cites nothing;
                    // text that does not decode might, so it is named.
                    Ok(bytes) if bytes.contains(&0) => continue,
                    Ok(bytes) => match String::from_utf8(bytes) {
                        Ok(text) => text,
                        Err(_) => {
                            if !quiet {
                                eprintln!("warning: {path} is not UTF-8 text: not scanned");
                            }
                            continue;
                        }
                    },
                    Err(e) => return Err(e).with_context(|| format!("reading {path}")),
                };
                files += 1;
                let found = cited::scan(&text);
                cited += found.len();
                ids.extend(found.iter().map(|c| c.id.clone()));
                for f in cited::check(&store, &found) {
                    failing += 1;
                    println!("{path}:{}: verdict {} {}", f.line, f.id, f.why);
                }
            }
            if cited == 0 {
                bail!(
                    "no verdict citations in {files} file(s): the pathspecs match nothing that cites, \
                     so the check proves nothing"
                );
            }
            if failing > 0 {
                bail!("{failing} of {cited} citation(s) break the settlement rule");
            }
            // A fixed line, so a CI step can match it; the counts go to
            // stderr.
            println!("citations ok.");
            if !quiet {
                eprintln!("{cited} cited, {} ids, {files} files", ids.len());
            }
            Ok(())
        }
        VerdictCommand::Show { ids, json: true } => {
            let store = verdict::load(&root)?;
            let mut out = Vec::new();
            for id in &ids {
                let Some(q) = store.get(id) else {
                    bail!("no question `{id}` in {}", root.display());
                };
                out.push(store.json(q));
            }
            println!(
                "{}",
                serde_json::to_string_pretty(&verdict::json::envelope(&out))?
            );
            Ok(())
        }
        VerdictCommand::Show { ids, json: false } => {
            let store = verdict::load(&root)?;
            let mut inspector = Inspector::new();
            for (i, id) in ids.iter().enumerate() {
                let Some(q) = store.get(id) else {
                    bail!("no question `{id}` in {}", root.display());
                };
                if i > 0 {
                    println!();
                }
                print_question(&store, q, &mut inspector, style);
            }
            inspector.stable()
        }
        VerdictCommand::List {
            json,
            identifier,
            code,
            status,
            closed,
            negative,
            inferred,
            retired,
            kind,
        } => {
            let store = verdict::load(&root)?;
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
            if json {
                let out: Vec<_> = rows.iter().map(|q| store.json(q)).collect();
                println!(
                    "{}",
                    serde_json::to_string_pretty(&verdict::json::envelope(&out))?
                );
                return Ok(());
            }
            if rows.is_empty() && !quiet {
                eprintln!("no questions");
            }
            print_rows(&store, &rows, style);
            Ok(())
        }
    }
}

fn run_evidence(command: EvidenceCommand, root: &Path, quiet: bool) -> Result<()> {
    match command {
        EvidenceCommand::Move {
            old,
            new,
            reason,
            old_raw,
        } => {
            let moved = verdict::relocate(root, &old, &new, old_raw, &reason)?;
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
            reason,
        } => {
            match verdict::merge(root, &id, observation, &redundant, &into, &reason)? {
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
        EvidenceCommand::Attest { paths, reason, raw } => {
            let done = verdict::attest(root, &paths, raw, &reason)?;
            for location in &done.skipped {
                println!("skipped {location}: only released entries are there");
            }
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
        EvidenceCommand::Check { ids } => {
            let store = verdict::load(root)?;
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
                "{} evidence entr{}: {} match, {} without a digest, {} released, {} failing",
                audit.entries,
                if audit.entries == 1 { "y" } else { "ies" },
                audit.matching,
                audit.unhashed.len(),
                audit.released,
                audit.failing.len()
            );
            if !audit.failing.is_empty() {
                bail!("{summary}");
            }
            println!("{summary}");
            Ok(())
        }
        EvidenceCommand::Release {
            id,
            paths,
            observation,
            all,
            reason,
        } => {
            let scope = if all {
                ReleaseScope::All
            } else {
                ReleaseScope::Paths(&paths)
            };
            let done = verdict::release(root, &id, observation, scope, &reason)?;
            println!(
                "released {} evidence entr{} of {id} #{observation}{}",
                done.released,
                if done.released == 1 { "y" } else { "ies" },
                if done.already > 0 {
                    format!(" ({} already released)", done.already)
                } else {
                    String::new()
                }
            );
            print_holders(&done.holders, false, false);
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
        supersedes: a.supersedes,
        supersede_reason: a.supersede_reason.clone(),
        release: a.release.clone(),
        release_all: a.release_all,
        environment: a.environment.clone(),
        note: a.note.clone(),
        inconclusive: a.inconclusive.clone(),
    })
}

/// Per released location, the unreleased entries still requiring the file
/// (stdout, or stderr beside `observe`'s own report). `would`: a dry run's
/// prediction, nothing released yet.
fn print_holders(holders: &[(String, Vec<String>)], stderr: bool, would: bool) {
    for (location, still) in holders {
        let line = match (would, still.is_empty()) {
            (false, true) => {
                format!("released {location}: no unreleased entry in this store requires it")
            }
            (false, false) => format!(
                "released {location}: still required by {}",
                still.join(", ")
            ),
            (true, true) => {
                format!("would release {location}: no unreleased entry would require it")
            }
            (true, false) => format!(
                "would release {location}: still required by {}",
                still.join(", ")
            ),
        };
        if stderr {
            eprintln!("{line}");
        } else {
            println!("{line}");
        }
    }
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
        print_observation(&r, q, inspector, style);
    }
}

fn print_observation(r: &Ranked<'_>, q: &Question, inspector: &mut Inspector, style: Style) {
    let root = &q.root;
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
    if let Some(s) = &o.supersedes {
        detail(
            "supersedes",
            &format!("#{} {}: {}", s.observation, s.at, s.reason),
        );
    }
    if let Some(next) = q.observations.iter().position(|x| {
        x.supersedes
            .as_ref()
            .is_some_and(|s| s.observation == r.number)
    }) {
        detail("superseded by", &format!("#{}", next + 1));
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
