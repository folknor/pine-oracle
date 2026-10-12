// End-to-end verdict tests: write through `add` / `observe` into a scratch
// verdicts directory under `target/`, read back through the strict `load`.
// Runs 24 and 26 use the real capture-campaign fixtures and editor-probe
// evidence from piners (copied into `testdata/verdict/`); their sha256s are
// the ones TradingView's editor probe recorded.

use std::path::{Path, PathBuf};

use super::*;

const RUN26_SHA: &str = "520b4a5046d1594580fbf346d728996449c17ee33ee193ea051a4c70826e28b8";
const RUN24_SHA: &str = "193ed148bfdbd3b0a0ddef8066a5ba29c57d981dc203a56f75caed60ef6e580a";

fn testdata(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("testdata/verdict")
        .join(name)
}

/// A fresh verdicts directory unique to one test, with empty `records/` and
/// `evidence/`.
fn scratch(test: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("target/verdict-tests")
        .join(test);
    if dir.exists() {
        std::fs::remove_dir_all(&dir).expect("clear scratch dir");
    }
    std::fs::create_dir_all(dir.join("records")).expect("create scratch dir");
    std::fs::create_dir_all(dir.join("evidence")).expect("create evidence dir");
    dir
}

/// The file question `id` lives in under `root`.
fn record(root: &Path, id: &str) -> PathBuf {
    root.join("records").join(format!("{id}.toml"))
}

/// The testdata file `name` copied into `root`'s evidence directory.
fn evidence_copy(root: &Path, name: &str) -> PathBuf {
    let path = root.join("evidence").join(name);
    std::fs::copy(testdata(name), &path).expect("copy evidence");
    path
}

/// An evidence file to `observe`, read by its leading bytes.
fn ev(path: PathBuf) -> NewEvidence {
    NewEvidence { path, raw: false }
}

fn question(kind: Kind, identifiers: &[&str]) -> NewQuestion {
    NewQuestion {
        kind,
        key: None,
        question: "Q?".to_string(),
        answer: "A.".to_string(),
        identifiers: identifiers.iter().map(ToString::to_string).collect(),
        follow_up_to: Vec::new(),
        basis: Vec::new(),
        negative: false,
        exclusive: false,
    }
}

fn obs(source: Source, date: &str) -> NewObservation {
    NewObservation {
        source,
        key: None,
        date: Some(date.to_string()),
        date_before: None,
        fixture: None,
        outcome: None,
        crash: None,
        errors: Vec::new(),
        warnings: Vec::new(),
        result: None,
        settings: None,
        candidates: Vec::new(),
        selected: Vec::new(),
        consistent: Vec::new(),
        refuted: Vec::new(),
        evidence: Vec::new(),
        environment: None,
        note: None,
        inconclusive: None,
    }
}

fn compile(source: Source, date: &str, outcome: Outcome) -> NewObservation {
    NewObservation {
        outcome: Some(outcome),
        ..obs(source, date)
    }
}

fn runtime(date: &str) -> NewObservation {
    NewObservation {
        result: Some("TV did X.".to_string()),
        settings: Some("NASDAQ:AAPL 1D, defaults".to_string()),
        ..obs(Source::Chart, date)
    }
}

fn reload(root: &Path, id: &str) -> Question {
    load(root)
        .expect("store loads")
        .get(id)
        .expect("question present")
        .clone()
}

#[test]
fn run26_editor_reject_outranks_endpoint_accept() {
    let root = scratch("run26");
    let added = add(
        &root,
        &NewQuestion {
            question: "Can an indicator script declare an exported function?".to_string(),
            answer: "No. Only libraries can contain exported functions (CE10099).".to_string(),
            ..question(Kind::Compile, &["export"])
        },
    )
    .expect("add");
    assert!(added.warnings.is_empty(), "{:?}", added.warnings);

    let editor = NewObservation {
        fixture: Some(testdata("editor-export-outside-library.pine")),
        errors: vec![
            parse_diag("CE10099|Only libraries can contain exported functions.|9:1-10:5")
                .expect("diag"),
        ],
        environment: Some("TradingView Desktop 3.4.1".to_string()),
        evidence: vec![ev(evidence_copy(
            &root,
            "run26-export-outside-library.json",
        ))],
        ..compile(Source::Editor, "2026-09-27", Outcome::Rejected)
    };
    observe(&root, &added.id, &editor).expect("editor observation");
    let endpoint = NewObservation {
        note: Some("Taken on inline `export f() => 1` before the fixture file existed.".into()),
        ..compile(Source::Endpoint, "2026-09-06", Outcome::Accepted)
    };
    observe(&root, &added.id, &endpoint).expect("endpoint observation");

    let q = reload(&root, &added.id);
    assert_eq!(q.own_status(), Status::Settled);
    let ranked = q.ranked();
    assert_eq!(ranked.len(), 2);
    assert_eq!(ranked[0].observation.source, Source::Editor);
    assert_eq!(ranked[0].annotation, None);
    assert_eq!(ranked[1].observation.source, Source::Endpoint);
    assert_eq!(
        ranked[1].annotation.as_deref(),
        Some("weaker source; editor disagrees")
    );

    // po hashed the stored bytes itself, and they match TV's probe record.
    let editor_obs = ranked[0].observation;
    assert_eq!(editor_obs.fixture.as_deref(), Some(RUN26_SHA));
    assert_eq!(
        editor_obs.fixture_name.as_deref(),
        Some("editor-export-outside-library.pine")
    );
    assert!(
        root.join("fixtures")
            .join(format!("{RUN26_SHA}.pine"))
            .is_file()
    );
    assert_eq!(ranked[1].observation.fixture, None);
    // Evidence is stored relative to the evidence directory.
    let evidence = &editor_obs.evidence[0];
    assert_eq!(evidence.path, "run26-export-outside-library.json");
    assert!(root.join("evidence").join(&evidence.path).is_file());
    // ... with the digest of its content, recorded at capture.
    let bytes = std::fs::read(testdata("run26-export-outside-library.json")).expect("read");
    assert_eq!(
        evidence.sha256.as_deref(),
        Some(validate::sha256_hex(&bytes).as_str())
    );
    assert_eq!(evidence.attested, None);
    assert_eq!(q.codes().into_iter().collect::<Vec<_>>(), vec!["CE10099"]);
}

#[test]
fn run24_editor_confirms_endpoint() {
    let root = scratch("run24");
    let id = add(
        &root,
        &question(Kind::Compile, &["strategy.risk.max_drawdown"]),
    )
    .expect("add")
    .id;
    let endpoint = NewObservation {
        note: Some("Both spellings: bare na literal and through an int na binding.".into()),
        ..compile(Source::Endpoint, "2026-09-10", Outcome::Accepted)
    };
    observe(&root, &id, &endpoint).expect("endpoint");
    assert_eq!(
        reload(&root, &id).own_status(),
        Status::Open,
        "endpoint alone settles nothing"
    );

    let editor = NewObservation {
        fixture: Some(testdata("editor-risk-drawdown-na.pine")),
        evidence: vec![ev(evidence_copy(&root, "run24-risk-drawdown-na.json"))],
        ..compile(Source::Editor, "2026-09-27", Outcome::Accepted)
    };
    observe(&root, &id, &editor).expect("editor");
    let q = reload(&root, &id);
    assert_eq!(q.own_status(), Status::Settled);
    let ranked = q.ranked();
    assert_eq!(ranked[0].observation.fixture.as_deref(), Some(RUN24_SHA));
    assert_eq!(
        ranked[1].annotation.as_deref(),
        Some("weaker source; confirmed by editor")
    );
}

#[test]
fn crashes_and_inconclusive_runs_never_count() {
    let root = scratch("not-counting");
    let id = add(&root, &question(Kind::Compile, &["export"]))
        .expect("add")
        .id;
    let crash = NewObservation {
        crash: Some("Cannot read properties of undefined (reading 'line')".into()),
        ..compile(Source::Endpoint, "2026-09-10", Outcome::Crashed)
    };
    observe(&root, &id, &crash).expect("crash");
    let q = reload(&root, &id);
    assert_eq!(q.own_status(), Status::Open);
    assert_eq!(
        q.ranked()[0].annotation.as_deref(),
        Some("oracle crashed, does not count")
    );

    let rt = add(&root, &question(Kind::Runtime, &[])).expect("add").id;
    let misfire = NewObservation {
        inconclusive: Some("the rule never armed".into()),
        ..runtime("2026-09-20")
    };
    observe(&root, &rt, &misfire).expect("inconclusive");
    assert_eq!(
        reload(&root, &rt).own_status(),
        Status::Open,
        "an inconclusive chart run must not settle a question"
    );
}

#[test]
fn runtime_candidates_select_refute_and_conflict() {
    let root = scratch("candidates");
    let id = add(&root, &question(Kind::Runtime, &[])).expect("add").id;
    let candidates = vec![
        parse_candidate("A|gross").expect("A"),
        parse_candidate("B|net").expect("B"),
        parse_candidate("C").expect("C"),
    ];
    let first = NewObservation {
        candidates: candidates.clone(),
        refuted: vec!["A".into()],
        ..runtime("2026-09-20")
    };
    observe(&root, &id, &first).expect("first");
    let q = reload(&root, &id);
    assert_eq!(q.own_status(), Status::Settled);
    let states: Vec<_> = q.observations[0]
        .candidates
        .iter()
        .map(|c| (c.name.as_str(), c.state))
        .collect();
    assert_eq!(
        states,
        vec![
            ("A", CandidateState::Refuted),
            ("B", CandidateState::Undecided),
            ("C", CandidateState::Undecided),
        ]
    );

    let second = NewObservation {
        candidates,
        selected: vec!["A".into()],
        ..runtime("2026-09-21")
    };
    observe(&root, &id, &second).expect("second");
    assert_eq!(reload(&root, &id).own_status(), Status::Conflict);
}

#[test]
fn contradicting_observations_warn_at_observe_time() {
    let root = scratch("contradiction-warning");
    let id = add(&root, &question(Kind::Compile, &["export"]))
        .expect("add")
        .id;
    let quiet = |o: &NewObservation| {
        let observed = observe(&root, &id, o).expect("observe");
        assert!(observed.warnings.is_empty(), "{:?}", observed.warnings);
    };
    let warned = |o: &NewObservation| -> String {
        let observed = observe(&root, &id, o).expect("observe");
        assert_eq!(observed.warnings.len(), 1, "{:?}", observed.warnings);
        observed.warnings[0].clone()
    };

    quiet(&compile(Source::Editor, "2026-09-20", Outcome::Accepted));
    // The endpoint disagreeing with the editor is the known gap, not a
    // contradiction worth a warning.
    let reject = NewObservation {
        errors: vec![parse_diag("CE10099").expect("diag")],
        ..compile(Source::Editor, "2026-09-21", Outcome::Rejected)
    };
    quiet(&NewObservation {
        source: Source::Endpoint,
        date: Some("2026-09-20".into()),
        ..reject.clone()
    });
    assert_eq!(
        warned(&reject),
        "observation #3 editor rejected (CE10099) contradicts #1 editor accepted [the \
         outcome differs]: question {id} is now conflict (was settled). If the fixtures \
         test different claims (a control is the usual case: it is its own question), \
         they belong to separate questions: `void --observation 3` and observe it under \
         its own question. If one was mis-recorded, void that one."
            .replace("{id}", &id)
    );
    // Agreeing with #3 still contradicts #1.
    let again = NewObservation {
        key: Some("again".into()),
        ..reject.clone()
    };
    assert!(
        warned(&again).contains("contradicts #1 editor accepted [the outcome differs]: question")
    );
    assert!(warned_rerun_is_silent(&root, &id, &again));
    // Same-strength weaker sources contradicting each other warn too.
    let w = warned(&compile(Source::Endpoint, "2026-09-22", Outcome::Accepted));
    assert!(
        w.contains(
            "contradicts #2 endpoint rejected (CE10099) [the outcome differs]: weaker \
             source only, question \
             {id} stays conflict"
                .replace("{id}", &id)
                .as_str()
        ),
        "{w}"
    );
    // A non-counting observation decides nothing, so contradicts nothing.
    quiet(&NewObservation {
        inconclusive: Some("fixture bug".into()),
        ..compile(Source::Editor, "2026-09-23", Outcome::Accepted)
    });
    // Chart and editor share the top tier; every contradicted row is named.
    let w = warned(&compile(Source::Chart, "2026-09-24", Outcome::Accepted));
    assert!(
        w.contains(
            "observation #7 chart accepted contradicts #3 editor rejected (CE10099) [the \
             outcome differs], #4 editor rejected (CE10099) [the outcome differs]: question"
        ),
        "{w}"
    );

    let rt = add(&root, &question(Kind::Runtime, &[])).expect("add").id;
    let candidates = vec![parse_candidate("A").expect("A")];
    observe(
        &root,
        &rt,
        &NewObservation {
            candidates: candidates.clone(),
            refuted: vec!["A".into()],
            ..runtime("2026-09-20")
        },
    )
    .expect("first run");
    let second = observe(
        &root,
        &rt,
        &NewObservation {
            candidates,
            selected: vec!["A".into()],
            ..runtime("2026-09-21")
        },
    )
    .expect("second run");
    assert!(
        second.warnings.last().expect("warning").contains(
            "observation #2 chart runs clean; selects A contradicts #1 chart runs clean; \
             refutes A [#2 selects A that #1 refutes]: question"
        ),
        "{:?}",
        second.warnings
    );
}

#[test]
fn contradiction_warning_reports_the_shown_disposition() {
    let root = scratch("contradiction-disposition");
    let last_warning = |id: &str, o: &NewObservation| -> String {
        observe(&root, id, o)
            .expect("observe")
            .warnings
            .last()
            .cloned()
            .unwrap_or_default()
    };
    let rejected = NewObservation {
        errors: vec![parse_diag("CE10099").expect("diag")],
        ..compile(Source::Endpoint, "2026-09-20", Outcome::Rejected)
    };

    // Documented: endpoint rows never decide, so the citations still hold.
    let documented = add(&root, &question(Kind::Compile, &["export"]))
        .expect("add")
        .id;
    cite(&root, &documented, LOWER_TF, &quotes(&[LOOKAHEAD_ON]), None).expect("cite");
    observe(&root, &documented, &rejected).expect("endpoint reject");
    let w = last_warning(
        &documented,
        &compile(Source::Endpoint, "2026-09-21", Outcome::Accepted),
    );
    assert!(
        w.contains(&format!(
            "weaker source only, question {documented} stays documented."
        )),
        "{w}"
    );

    // Inferred: it may carry only weaker observations, and they never
    // decide it.
    let inferred = add(
        &root,
        &NewQuestion {
            basis: vec![documented.clone()],
            ..question(Kind::Compile, &["export"])
        },
    )
    .expect("add")
    .id;
    observe(&root, &inferred, &rejected).expect("endpoint reject");
    let w = last_warning(
        &inferred,
        &compile(Source::Endpoint, "2026-09-21", Outcome::Accepted),
    );
    assert!(
        w.contains(&format!(
            "weaker source only, question {inferred} stays inferred (documented via \
             {documented})."
        )),
        "{w}"
    );

    // Runtime runs with only undecided candidates are open, yet differing
    // halts still contradict.
    let rt = add(&root, &question(Kind::Runtime, &["strategy.entry"]))
        .expect("add")
        .id;
    let undecided = NewObservation {
        candidates: vec![parse_candidate("A").expect("A")],
        ..runtime("2026-09-20")
    };
    observe(&root, &rt, &undecided).expect("clean run");
    let w = last_warning(
        &rt,
        &NewObservation {
            errors: vec![parse_diag("RE10044||bar=100").expect("diag")],
            ..undecided.clone()
        },
    );
    assert!(
        w.contains(&format!(
            "observation #2 chart halts (RE10044) contradicts #1 chart runs clean [error \
             codes differ (only in #2: RE10044; only in #1: none)]: question {rt} is now \
             conflict (was open)."
        )),
        "{w}"
    );
}

/// The d4892d70 case: a run landed without the structured RE10134 its
/// siblings carry. A dry run shows the stray conflict and writes nothing.
#[test]
fn dry_run_reports_the_resulting_status_and_writes_nothing() {
    let root = scratch("dry-run");
    let id = add(&root, &question(Kind::Runtime, &["strategy.entry"]))
        .expect("add")
        .id;
    let halt = NewObservation {
        errors: vec![parse_diag("RE10134||bar=5").expect("diag")],
        ..runtime("2026-09-20")
    };
    observe(&root, &id, &halt).expect("halting run");
    let before = std::fs::read(record(&root, &id)).expect("read");

    let missing_code = NewObservation {
        fixture: Some(testdata("editor-export-outside-library.pine")),
        ..runtime("2026-09-21")
    };
    let dry = observe_dry_run(&root, &id, &missing_code).expect("dry run");
    assert_eq!((dry.number, dry.existing), (2, false));
    assert_eq!(
        (dry.was.to_string(), dry.now.to_string()),
        ("settled".to_string(), "conflict".to_string())
    );
    assert_eq!(
        dry.warnings,
        vec![
            "observation #2 chart runs clean contradicts #1 chart halts (RE10134) [error \
             codes differ (only in #2: none; only in #1: RE10134)]: question {id} would be \
             conflict (is settled). If the fixtures test different claims (a control is the \
             usual case: it is its own question), they belong to separate questions: \
             observe it under its own question instead. If an earlier one was mis-recorded, \
             void it; if this one is, correct it before landing."
                .replace("{id}", &id)
        ]
    );
    assert_eq!(std::fs::read(record(&root, &id)).expect("read"), before);
    assert!(!root.join("fixtures").exists());

    // An agreeing run reports the status it leaves, and a refused one fails
    // the dry run as it would the write.
    let agreeing = observe_dry_run(&root, &id, &halt).expect("agreeing");
    assert!(agreeing.warnings.is_empty(), "{:?}", agreeing.warnings);
    assert_eq!(agreeing.now.resolved, Resolved::Measured(Status::Settled));
    refused(
        observe_dry_run(
            &root,
            &id,
            &NewObservation {
                result: None,
                ..halt.clone()
            },
        ),
        "needs a `result`",
    );

    // Under a held key the dry run reports the existing observation and the
    // status it leaves; the id comes back as stored, whatever its case.
    let keyed = NewObservation {
        key: Some("k".into()),
        ..halt.clone()
    };
    observe(&root, &id, &keyed).expect("keyed");
    let rerun = observe_dry_run(&root, &id.to_ascii_uppercase(), &keyed).expect("keyed dry");
    assert_eq!((rerun.number, rerun.existing), (2, true));
    assert_eq!(rerun.id, id);
    assert_eq!(rerun.now.resolved, Resolved::Measured(Status::Settled));

    // A retired question refuses the dry run as it would the observation.
    retire(&root, &id, "reworded", &[]).expect("retire");
    refused(observe_dry_run(&root, &id, &halt), "is retired");
}

/// Reads hold `.lock` shared: a read waits out a writer instead of reading
/// a directory mid-change. Where the lock file can neither be created nor
/// opened (a read-only checkout), the read goes unlocked rather than failing.
#[cfg(unix)]
#[test]
fn reads_wait_for_writers_and_read_only_checkouts_still_load() {
    use std::os::unix::fs::PermissionsExt as _;
    use std::time::Duration;

    let root = scratch("read-lock");
    let id = add(&root, &question(Kind::Compile, &["export"]))
        .expect("add")
        .id;
    let writer = std::fs::OpenOptions::new()
        .write(true)
        .open(root.join(".lock"))
        .expect("open lock");
    // A load in another thread, which reports when it is about to start and
    // when it is done.
    let start_load = |root: PathBuf| {
        let (tx, rx) = std::sync::mpsc::channel();
        let reader = std::thread::spawn(move || {
            tx.send(None).expect("send");
            tx.send(load(&root).map(|s| s.questions().len()).ok())
                .expect("send");
        });
        assert_eq!(rx.recv().expect("started"), None);
        (reader, rx)
    };

    // Another reader holding it shared does not block a read.
    writer.lock_shared().expect("shared");
    let (reader, rx) = start_load(root.clone());
    assert_eq!(
        rx.recv_timeout(Duration::from_secs(10)).expect("read"),
        Some(1),
        "a read waited for another reader"
    );
    reader.join().expect("reader");
    writer.unlock().expect("unlock");

    // A writer holding it exclusively does.
    writer.lock().expect("exclusive");
    let (reader, rx) = start_load(root.clone());
    assert!(
        rx.recv_timeout(Duration::from_millis(300)).is_err(),
        "the read finished while a writer held the lock"
    );
    writer.unlock().expect("unlock");
    assert_eq!(
        rx.recv_timeout(Duration::from_secs(10)).expect("read"),
        Some(1)
    );
    reader.join().expect("reader");

    // Read-only copies: one that never had a lock file reads unlocked
    // (creating nothing), one with a read-only `.lock` locks it read-only.
    let set_mode = |path: &Path, mode: u32| {
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(mode)).expect("chmod");
    };
    let read_only_copy = |name: &str, with_lock: bool| {
        let ro = scratch(name);
        std::fs::copy(record(&root, &id), record(&ro, &id)).expect("copy");
        if with_lock {
            std::fs::write(ro.join(".lock"), "").expect("lock file");
            set_mode(&ro.join(".lock"), 0o444);
        }
        set_mode(&ro, 0o555);
        ro
    };
    let fresh = read_only_copy("read-lock-ro", false);
    // A privileged test run ignores the permissions; the fallback cannot be
    // reached then, only the load checked.
    let enforced = std::fs::File::create(fresh.join(".probe")).is_err();
    let loaded = load(&fresh).map(|s| s.questions().len());
    let created = fresh.join(".lock").exists();
    set_mode(&fresh, 0o755);
    assert_eq!(loaded.expect("read-only load"), 1);
    if enforced {
        assert!(!created, "a read in a read-only directory created .lock");
    }
    let locked = read_only_copy("read-lock-ro-locked", true);
    let loaded = load(&locked).map(|s| s.questions().len());
    set_mode(&locked, 0o755);
    set_mode(&locked.join(".lock"), 0o644);
    assert_eq!(loaded.expect("read-only lock file"), 1);
}

/// A keyed rerun writes nothing, so it warns about nothing.
fn warned_rerun_is_silent(root: &Path, id: &str, o: &NewObservation) -> bool {
    let observed = observe(root, id, o).expect("rerun");
    observed.existing && observed.warnings.is_empty()
}

#[test]
fn runtime_errors_take_re_codes_and_bars() {
    let root = scratch("runtime-errors");
    let id = add(&root, &question(Kind::Runtime, &[])).expect("add").id;
    let halt = NewObservation {
        errors: vec![parse_diag("RE10044|Loop takes too long|bar=100").expect("diag")],
        ..runtime("2026-09-20")
    };
    observe(&root, &id, &halt).expect("RE code on runtime");
    let q = reload(&root, &id);
    assert_eq!(q.observations[0].errors[0].bar, Some(100));
    assert!(q.codes().contains("RE10044"));
}

#[test]
fn invalid_observations_are_refused_and_nothing_is_written() {
    let root = scratch("refusals");
    let c = add(&root, &question(Kind::Compile, &["export"]))
        .expect("add")
        .id;
    let r = add(&root, &question(Kind::Runtime, &[])).expect("add").id;
    let before_c = std::fs::read(record(&root, &c)).expect("read");

    let re_on_compile = NewObservation {
        errors: vec![parse_diag("RE10044|m").expect("diag")],
        ..compile(Source::Editor, "2026-09-27", Outcome::Rejected)
    };
    let accepted_with_error = NewObservation {
        errors: vec![parse_diag("CE10099|m").expect("diag")],
        ..compile(Source::Editor, "2026-09-27", Outcome::Accepted)
    };
    let crash_without_message = compile(Source::Endpoint, "2026-09-27", Outcome::Crashed);
    let runtime_fields_on_compile = NewObservation {
        result: Some("x".into()),
        ..compile(Source::Editor, "2026-09-27", Outcome::Accepted)
    };
    let bad_date = compile(Source::Editor, "2026-09-27T10:00:00", Outcome::Accepted);
    let fixture_to_store = NewObservation {
        fixture: Some(testdata("editor-export-outside-library.pine")),
        ..re_on_compile.clone()
    };
    for (o, needle) in [
        (&re_on_compile, "must be CE followed by five digits"),
        (&accepted_with_error, "accepted but carries errors"),
        (&crash_without_message, "crashed without a `crash` message"),
        (
            &runtime_fields_on_compile,
            "`result` belongs to runtime observations",
        ),
        (&bad_date, "must be a plain YYYY-MM-DD date"),
        (&fixture_to_store, "must be CE followed by five digits"),
    ] {
        refused(observe(&root, &c, o), needle);
    }
    assert_eq!(std::fs::read(record(&root, &c)).expect("read"), before_c);
    assert!(
        !root.join("fixtures").exists(),
        "a refused observation must not store its fixture"
    );

    let ce_on_runtime = NewObservation {
        errors: vec![parse_diag("CE10099|m").expect("diag")],
        ..runtime("2026-09-20")
    };
    let runtime_from_endpoint = NewObservation {
        source: Source::Endpoint,
        ..runtime("2026-09-20")
    };
    let no_settings = NewObservation {
        settings: None,
        ..runtime("2026-09-20")
    };
    let undeclared_candidate = NewObservation {
        selected: vec!["A".into()],
        ..runtime("2026-09-20")
    };
    let both_states = NewObservation {
        candidates: vec![parse_candidate("A").expect("A")],
        selected: vec!["A".into()],
        refuted: vec!["A".into()],
        ..runtime("2026-09-20")
    };
    let before_r = std::fs::read(record(&root, &r)).expect("read");
    for (o, needle) in [
        (&ce_on_runtime, "must be RE followed by five digits"),
        (&runtime_from_endpoint, "come from a chart run"),
        (&no_settings, "needs `settings`"),
        (&undeclared_candidate, "is not a declared candidate"),
        (&both_states, "cannot be both selected and refuted"),
    ] {
        refused(observe(&root, &r, o), needle);
    }
    assert_eq!(std::fs::read(record(&root, &r)).expect("read"), before_r);

    refused(
        add(&root, &question(Kind::Compile, &["no.such.function"])),
        "is not in pine-data",
    );
    refused(
        add(&root, &question(Kind::Compile, &["export", "EXPORT"])),
        "listed twice",
    );
    refused(
        add(
            &root,
            &NewQuestion {
                basis: vec!["deadbeef".into()],
                ..question(Kind::Compile, &["export"])
            },
        ),
        "`basis` names unknown question",
    );
    refused(
        observe(
            &root,
            "00000000",
            &compile(Source::Editor, "2026-09-27", Outcome::Accepted),
        ),
        "no question `00000000`",
    );
}

/// Assert `result` failed and its message names the rule that fired.
fn refused<T: std::fmt::Debug>(result: anyhow::Result<T>, needle: &str) {
    let err = result.expect_err(needle).to_string();
    assert!(err.contains(needle), "expected `{needle}` in: {err}");
}

#[test]
fn piners_identifiers_resolve() {
    let root = scratch("identifiers");
    add(
        &root,
        &question(Kind::Compile, &["?:", ":=", "for", "and", "var", "export"]),
    )
    .expect("operator and keyword identifiers resolve in pine-data");
}

#[test]
fn empty_identifiers_warn_but_write() {
    let root = scratch("empty-identifiers");
    let added = add(&root, &question(Kind::Runtime, &[])).expect("add");
    assert!(added.warnings.iter().any(|w| w.contains("no identifiers")));
}

/// The disposition `id` resolves to under `root`.
fn resolved(root: &Path, id: &str) -> Resolved {
    let store = load(root).expect("load");
    let q = store.get(id).expect("question").clone();
    store.resolve(&q)
}

fn ids_of(store: &Store, filter: &ListFilter<'_>) -> Vec<String> {
    store
        .list(filter)
        .into_iter()
        .map(|q| q.id.clone())
        .collect()
}

#[test]
fn inferred_questions() {
    let root = scratch("inferred");
    let base = add(&root, &question(Kind::Compile, &["export"]))
        .expect("add")
        .id;
    let other = add(&root, &question(Kind::Compile, &["export"]))
        .expect("add")
        .id;
    let inferred = add(
        &root,
        &NewQuestion {
            basis: vec![base.clone(), other.clone()],
            ..question(Kind::Compile, &["export"])
        },
    )
    .expect("add inferred")
    .id;

    // Both premises open: the first is named.
    let r = resolved(&root, &inferred);
    assert_eq!(r.status(), Some(Status::Open));
    assert_eq!(r.to_string(), format!("inferred (open via {base})"));

    // One premise settled: the other, still open, is named.
    observe(
        &root,
        &base,
        &compile(Source::Editor, "2026-09-27", Outcome::Accepted),
    )
    .expect("settle base");
    assert_eq!(
        resolved(&root, &inferred).to_string(),
        format!("inferred (open via {other})")
    );

    // Every premise settled.
    observe(
        &root,
        &other,
        &compile(Source::Editor, "2026-09-27", Outcome::Accepted),
    )
    .expect("settle other");
    assert_eq!(resolved(&root, &inferred).to_string(), "inferred (settled)");

    // A conflicting premise blocks the inference; it is not a conflict in it.
    let reject = NewObservation {
        errors: vec![parse_diag("CE10099|m").expect("d")],
        ..compile(Source::Editor, "2026-09-27", Outcome::Rejected)
    };
    observe(&root, &base, &reject).expect("conflict base");
    assert_eq!(
        resolved(&root, &inferred).to_string(),
        format!("inferred (open via {base})")
    );

    // Premises may be inferred themselves.
    let chained = add(
        &root,
        &NewQuestion {
            basis: vec![inferred.clone()],
            ..question(Kind::Compile, &["export"])
        },
    )
    .expect("add chained")
    .id;
    assert_eq!(
        resolved(&root, &chained).to_string(),
        format!("inferred (open via {inferred})")
    );

    // Non-counting and weaker observations may sit on an inferred question;
    // neither ends the inference.
    let misfire = NewObservation {
        inconclusive: Some("fixture bug".into()),
        ..compile(Source::Editor, "2026-09-27", Outcome::Rejected)
    };
    observe(&root, &inferred, &misfire).expect("inconclusive");
    observe(
        &root,
        &inferred,
        &compile(Source::Endpoint, "2026-09-10", Outcome::Accepted),
    )
    .expect("an endpoint accept is weak evidence, not a measurement of the answer");
    assert!(matches!(
        resolved(&root, &inferred),
        Resolved::Inferred { .. }
    ));
    // A top-strength measurement must not: the question is either measured
    // or inferred, never silently both.
    refused(
        observe(
            &root,
            &inferred,
            &compile(Source::Editor, "2026-09-28", Outcome::Accepted),
        ),
        "an inferred question (non-empty `basis`) carries a counting editor/chart observation",
    );

    // Filtering matches the resolved status; only measured questions conflict.
    let store = load(&root).expect("load");
    assert_eq!(
        ids_of(
            &store,
            &ListFilter {
                status: Some(Status::Conflict),
                ..Default::default()
            }
        ),
        vec![base.clone()]
    );
    let mut both = vec![inferred.clone(), chained.clone()];
    both.sort();
    assert_eq!(
        ids_of(
            &store,
            &ListFilter {
                inferred: true,
                ..Default::default()
            }
        ),
        both
    );
}

#[test]
fn follow_up_is_lineage_only() {
    let root = scratch("follow-up");
    let parent = add(&root, &question(Kind::Compile, &["export"]))
        .expect("add")
        .id;
    let child = add(
        &root,
        &NewQuestion {
            follow_up_to: vec![parent.clone()],
            ..question(Kind::Compile, &["export"])
        },
    )
    .expect("add child")
    .id;
    observe(
        &root,
        &child,
        &compile(Source::Editor, "2026-09-27", Outcome::Accepted),
    )
    .expect("a follow-up is measured like any question");
    assert_eq!(resolved(&root, &child), Resolved::Measured(Status::Settled));
    assert_eq!(resolved(&root, &parent), Resolved::Measured(Status::Open));
    let store = load(&root).expect("load");
    assert_eq!(
        store
            .referrers(&parent, Relation::FollowUpTo)
            .iter()
            .map(|q| q.id.as_str())
            .collect::<Vec<_>>(),
        vec![child.as_str()]
    );
}

/// The two piners cases that motivated retirement, rebuilt on scratch
/// records: an ill-posed question (every observation inconclusive, its
/// sub-questions measured separately) and a mis-filed one (two fixtures
/// testing different claims under one question, so a false conflict).
#[test]
fn retiring_ill_posed_and_misfiled_questions() {
    let root = scratch("retire");
    let reject = |code: &str, date: &str| NewObservation {
        errors: vec![parse_diag(code).expect("d")],
        ..compile(Source::Editor, date, Outcome::Rejected)
    };

    // Ill-posed: the method declaration is refused before the question
    // arises, so no observation can ever answer it.
    let ill_posed = add(&root, &question(Kind::Compile, &["method", "ta.ema"]))
        .expect("add")
        .id;
    let confounded = NewObservation {
        inconclusive: Some("refused for CE10236 before the qualifier question arose".into()),
        ..reject("CE10236", "2026-09-27")
    };
    observe(&root, &ill_posed, &confounded).expect("inconclusive");
    let mut subs = Vec::new();
    for outcome in [Outcome::Rejected, Outcome::Accepted, Outcome::Accepted] {
        let sub = add(
            &root,
            &NewQuestion {
                follow_up_to: vec![ill_posed.clone()],
                ..question(Kind::Compile, &["method"])
            },
        )
        .expect("add sub-question")
        .id;
        let o = match outcome {
            Outcome::Rejected => reject("CE10236", "2026-09-27"),
            _ => compile(Source::Editor, "2026-09-27", outcome),
        };
        observe(&root, &sub, &o).expect("measure sub-question");
        subs.push(sub);
    }
    assert_eq!(
        resolved(&root, &ill_posed),
        Resolved::Measured(Status::Open)
    );
    retire(&root, &ill_posed, "No valid Pine form answers it.", &subs).expect("retire");
    let r = resolved(&root, &ill_posed);
    assert_eq!(r.status(), None, "retired is not settled, open or conflict");
    assert_eq!(
        r.to_string(),
        format!("retired (replaced by {})", subs.join(", "))
    );

    // Mis-filed: two fixtures with different codes make a false conflict.
    let misfiled = add(&root, &question(Kind::Compile, &["max_bars_back"]))
        .expect("add")
        .id;
    observe(&root, &misfiled, &reject("CE10013", "2026-09-28")).expect("b40");
    observe(&root, &misfiled, &reject("CE10120", "2026-09-28")).expect("b39");
    assert_eq!(
        resolved(&root, &misfiled),
        Resolved::Measured(Status::Conflict)
    );
    let mut refiled = Vec::new();
    for code in ["CE10120", "CE10013"] {
        let id = add(&root, &question(Kind::Compile, &["max_bars_back"]))
            .expect("add")
            .id;
        observe(&root, &id, &reject(code, "2026-09-28")).expect("refile");
        refiled.push(id);
    }
    retire(
        &root,
        &misfiled,
        "Two fixtures testing different claims.",
        &refiled,
    )
    .expect("retire");

    // Neither shows up as owed work any more, and both keep their evidence.
    let store = load(&root).expect("load");
    for status in [Status::Open, Status::Conflict] {
        let work = ids_of(
            &store,
            &ListFilter {
                status: Some(status),
                ..Default::default()
            },
        );
        assert!(
            !work.contains(&ill_posed) && !work.contains(&misfiled),
            "{work:?}"
        );
    }
    let mut retired = vec![ill_posed.clone(), misfiled.clone()];
    retired.sort();
    assert_eq!(
        ids_of(
            &store,
            &ListFilter {
                retired: true,
                ..Default::default()
            }
        ),
        retired
    );
    assert_eq!(store.get(&misfiled).expect("q").observations.len(), 2);
    assert_eq!(
        ids_of(&store, &ListFilter::default()).len(),
        store.questions().len(),
        "an unfiltered list keeps retired rows"
    );

    // A retired question takes no more observations, and retires once.
    refused(
        observe(&root, &misfiled, &reject("CE10120", "2026-09-29")),
        "is retired (replaced by",
    );
    refused(retire(&root, &misfiled, "again", &[]), "already retired");
}

#[test]
fn retirement_rules() {
    let root = scratch("retire-rules");
    let q = |basis: Vec<String>| NewQuestion {
        basis,
        ..question(Kind::Compile, &["export"])
    };
    let premise = add(&root, &q(Vec::new())).expect("add").id;
    let inference = add(&root, &q(vec![premise.clone()])).expect("add").id;

    // An active inference pins its premise until the basis is reworked.
    refused(
        retire(&root, &premise, "ill-posed", &[]),
        &format!("premise of active question(s) {inference}"),
    );
    retire(&root, &inference, "superseded", &[]).expect("retire the inference");
    retire(&root, &premise, "ill-posed", &[]).expect("no active user now");
    // A new inference cannot rest on a retired premise.
    refused(
        add(&root, &q(vec![premise.clone()])),
        "`basis` names retired question",
    );

    // Replacement chains: A replaced by B, B later split into C and D.
    let a = add(&root, &q(Vec::new())).expect("add").id;
    let b = add(&root, &q(Vec::new())).expect("add").id;
    let c = add(&root, &q(Vec::new())).expect("add").id;
    let d = add(&root, &q(Vec::new())).expect("add").id;
    retire(&root, &a, "reworded", std::slice::from_ref(&b)).expect("A -> B");
    retire(&root, &b, "split", &[c.clone(), d.clone()]).expect("B -> C, D");
    let store = load(&root).expect("load");
    assert_eq!(
        store.resolve(store.get(&a).expect("a")).to_string(),
        format!("retired (replaced by {b})")
    );
    assert_eq!(
        store.referrers(&b, Relation::ReplacedBy)[0].id,
        a,
        "B shows it replaces A"
    );
    let now: Vec<&str> = store
        .current_successors(store.get(&a).expect("a"))
        .into_iter()
        .map(|q| q.id.as_str())
        .collect();
    assert_eq!(now, vec![c.as_str(), d.as_str()], "A continues in C and D");
    // ...but never back to where it started.
    refused(
        retire(&root, &c, "loop", std::slice::from_ref(&a)),
        "reaches itself through `retired.replaced_by`",
    );

    refused(retire(&root, &c, "   ", &[]), "`retired.reason` is empty");
    refused(
        retire(&root, &c, "self", std::slice::from_ref(&c)),
        "`retired.replaced_by` names the question itself",
    );
    refused(retire(&root, &c, "dup", &[d.clone(), d.clone()]), "lists");
}

#[test]
fn retired_observations_are_not_compared() {
    let root = scratch("retired-annotations");
    let id = add(&root, &question(Kind::Compile, &["export"]))
        .expect("add")
        .id;
    observe(
        &root,
        &id,
        &compile(Source::Editor, "2026-09-27", Outcome::Accepted),
    )
    .expect("editor");
    observe(
        &root,
        &id,
        &compile(Source::Endpoint, "2026-09-10", Outcome::Accepted),
    )
    .expect("endpoint");
    assert_eq!(
        reload(&root, &id).ranked()[1].annotation.as_deref(),
        Some("weaker source; confirmed by editor")
    );
    retire(&root, &id, "the two runs used different fixtures", &[]).expect("retire");
    let q = reload(&root, &id);
    assert!(q.ranked().iter().all(|r| r.annotation.is_none()));
    assert_eq!(resolved(&root, &id).to_string(), "retired");
}

#[test]
fn legacy_derived_from_names_the_split() {
    let root = scratch("legacy-derived-from");
    let a = add(&root, &question(Kind::Compile, &["export"]))
        .expect("add")
        .id;
    let b = add(&root, &question(Kind::Compile, &["export"]))
        .expect("add")
        .id;
    let path = record(&root, &b);
    let text = std::fs::read_to_string(&path).expect("read");

    // The empty list every record written before the split carries is inert,
    // and the next write drops it.
    std::fs::write(&path, format!("derived_from = []\n{text}")).expect("write");
    load(&root).expect("an empty legacy list claims nothing");
    observe(
        &root,
        &b,
        &compile(Source::Editor, "2026-09-27", Outcome::Accepted),
    )
    .expect("observe rewrites the record");
    let rewritten = std::fs::read_to_string(&path).expect("read");
    assert!(!rewritten.contains("derived_from"), "{rewritten}");

    // A populated one is a decision the author has to make.
    std::fs::write(&path, format!("derived_from = [\"{a}\"]\n{text}")).expect("write");
    refused(load(&root), "`derived_from` was split");
}

#[test]
fn compile_agreement_includes_error_and_warning_codes() {
    let root = scratch("compile-codes");
    let id = add(&root, &question(Kind::Compile, &["export"]))
        .expect("add")
        .id;
    let with_warning = NewObservation {
        warnings: vec![parse_diag("CW10001|wrapped string").expect("d")],
        ..compile(Source::Editor, "2026-09-20", Outcome::Accepted)
    };
    observe(&root, &id, &with_warning).expect("warned accept");
    // A reworded message for the same code still agrees.
    let reworded = NewObservation {
        warnings: vec![parse_diag("CW10001|string is wrapped").expect("d")],
        ..compile(Source::Editor, "2026-09-27", Outcome::Accepted)
    };
    observe(&root, &id, &reworded).expect("reworded accept");
    assert_eq!(reload(&root, &id).own_status(), Status::Settled);

    // An endpoint accept without the warning: same outcome, different codes.
    observe(
        &root,
        &id,
        &compile(Source::Endpoint, "2026-09-10", Outcome::Accepted),
    )
    .expect("endpoint");
    let q = reload(&root, &id);
    assert_eq!(
        q.ranked()[2].annotation.as_deref(),
        Some("weaker source; same outcome as editor, different codes")
    );

    // An editor accept without the warning conflicts.
    observe(
        &root,
        &id,
        &compile(Source::Editor, "2026-09-28", Outcome::Accepted),
    )
    .expect("clean accept");
    assert_eq!(reload(&root, &id).own_status(), Status::Conflict);

    // Two editor rejects with different error codes conflict.
    let rejects = add(&root, &question(Kind::Compile, &["export"]))
        .expect("add")
        .id;
    for spec in ["CE10099|m", "CE10129|m"] {
        let o = NewObservation {
            errors: vec![parse_diag(spec).expect("d")],
            ..compile(Source::Editor, "2026-09-27", Outcome::Rejected)
        };
        observe(&root, &rejects, &o).expect("reject");
    }
    assert_eq!(reload(&root, &rejects).own_status(), Status::Conflict);
}

#[test]
fn runtime_undecided_runs_stay_open_and_halts_must_agree() {
    let root = scratch("runtime-rules");
    let id = add(&root, &question(Kind::Runtime, &[])).expect("add").id;
    let undecided = NewObservation {
        candidates: vec![
            parse_candidate("A").expect("A"),
            parse_candidate("B").expect("B"),
        ],
        ..runtime("2026-09-20")
    };
    observe(&root, &id, &undecided).expect("undecided");
    assert_eq!(
        reload(&root, &id).own_status(),
        Status::Open,
        "a run that decides no candidate settles nothing"
    );

    let halts = add(&root, &question(Kind::Runtime, &[])).expect("add").id;
    observe(&root, &halts, &runtime("2026-09-20")).expect("clean run");
    assert_eq!(reload(&root, &halts).own_status(), Status::Settled);
    let halted = NewObservation {
        errors: vec![parse_diag("RE10001|halted|bar=0").expect("d")],
        ..runtime("2026-09-21")
    };
    observe(&root, &halts, &halted).expect("halted run");
    assert_eq!(
        reload(&root, &halts).own_status(),
        Status::Conflict,
        "one run halts and one runs clean"
    );
}

#[test]
fn code_less_rejects_are_compared_on_outcome_only() {
    let root = scratch("code-less-reject");
    let id = add(&root, &question(Kind::Compile, &["export"]))
        .expect("add")
        .id;
    // A reject whose code was never recorded, then one carrying CE10147.
    observe(
        &root,
        &id,
        &compile(Source::Editor, "2026-09-20", Outcome::Rejected),
    )
    .expect("code-less reject");
    let coded = NewObservation {
        errors: vec![parse_diag("CE10147").expect("d")],
        ..compile(Source::Editor, "2026-09-27", Outcome::Rejected)
    };
    observe(&root, &id, &coded).expect("coded reject");
    assert_eq!(reload(&root, &id).own_status(), Status::Settled);

    // A weaker code-less reject says its codes were not recorded.
    observe(
        &root,
        &id,
        &compile(Source::Endpoint, "2026-09-10", Outcome::Rejected),
    )
    .expect("endpoint reject");
    let q = reload(&root, &id);
    assert_eq!(
        q.ranked()[2].annotation.as_deref(),
        Some("weaker source; same outcome as editor; codes not recorded")
    );

    // Two coded rejects that differ still conflict, code-less one or not.
    let other = NewObservation {
        errors: vec![parse_diag("CE10099").expect("d")],
        ..compile(Source::Editor, "2026-09-28", Outcome::Rejected)
    };
    observe(&root, &id, &other).expect("other coded reject");
    assert_eq!(reload(&root, &id).own_status(), Status::Conflict);

    // A clean accept still compares: it means no warning, not unrecorded.
    let warned = add(&root, &question(Kind::Compile, &["export"]))
        .expect("add")
        .id;
    let with_warning = NewObservation {
        warnings: vec![parse_diag("CW10001").expect("d")],
        ..compile(Source::Editor, "2026-09-20", Outcome::Accepted)
    };
    observe(&root, &warned, &with_warning).expect("warned");
    observe(
        &root,
        &warned,
        &compile(Source::Editor, "2026-09-21", Outcome::Accepted),
    )
    .expect("clean");
    assert_eq!(reload(&root, &warned).own_status(), Status::Conflict);
}

#[test]
fn re_observing_a_stored_fixture_keeps_its_name() {
    let root = scratch("fixture-name");
    let id = add(&root, &question(Kind::Compile, &["export"]))
        .expect("add")
        .id;
    let first = NewObservation {
        fixture: Some(testdata("editor-export-outside-library.pine")),
        ..compile(Source::Editor, "2026-09-27", Outcome::Accepted)
    };
    observe(&root, &id, &first).expect("first");
    let again = NewObservation {
        fixture: Some(root.join("fixtures").join(format!("{RUN26_SHA}.pine"))),
        ..compile(Source::Endpoint, "2026-09-06", Outcome::Accepted)
    };
    observe(&root, &id, &again).expect("by store path");
    let q = reload(&root, &id);
    for o in &q.observations {
        assert_eq!(
            o.fixture_name.as_deref(),
            Some("editor-export-outside-library.pine")
        );
    }

    // The same bytes captured under a second name: the names now disagree,
    // so a new observation by store path records none rather than picking.
    let elsewhere = scratch("fixture-name-copies");
    let copy = elsewhere.join("copy.pine");
    std::fs::copy(testdata("editor-export-outside-library.pine"), &copy).expect("copy");
    let keyed = NewObservation {
        key: Some("copy".into()),
        fixture: Some(copy),
        ..compile(Source::Editor, "2026-09-28", Outcome::Accepted)
    };
    observe(&root, &id, &keyed).expect("second name");
    observe(&root, &id, &again).expect("ambiguous by store path");
    assert_eq!(reload(&root, &id).observations[3].fixture_name, None);

    // A keyed rerun by store path carries its own observation's name, not
    // whichever another observation recorded.
    let rerun = observe(
        &root,
        &id,
        &NewObservation {
            fixture: Some(root.join("fixtures").join(format!("{RUN26_SHA}.pine"))),
            ..keyed.clone()
        },
    )
    .expect("keyed rerun by store path");
    assert!(rerun.existing);
    // Under its real name another file with the same bytes still differs.
    let other = elsewhere.join("other.pine");
    std::fs::copy(testdata("editor-export-outside-library.pine"), &other).expect("copy");
    refused(
        observe(
            &root,
            &id,
            &NewObservation {
                fixture: Some(other),
                ..keyed
            },
        ),
        "`fixture_name`: stored \"copy.pine\", given \"other.pine\"",
    );
}

/// A chart run of a compile question proves the script compiled: a chart
/// observation there records `accepted`, even one that does not count.
#[test]
fn chart_compile_observations_are_accepts() {
    let root = scratch("chart-compile");
    let id = add(&root, &question(Kind::Compile, &["export"]))
        .expect("add")
        .id;
    observe(
        &root,
        &id,
        &compile(Source::Chart, "2026-09-20", Outcome::Accepted),
    )
    .expect("chart accept");
    for o in [
        NewObservation {
            errors: vec![parse_diag("CE10099").expect("diag")],
            ..compile(Source::Chart, "2026-09-21", Outcome::Rejected)
        },
        NewObservation {
            inconclusive: Some("wrong symbol".into()),
            ..compile(Source::Chart, "2026-09-21", Outcome::Rejected)
        },
        NewObservation {
            crash: Some("chart froze".into()),
            ..compile(Source::Chart, "2026-09-21", Outcome::Crashed)
        },
    ] {
        refused(observe(&root, &id, &o), "records `accepted`");
    }
}

#[test]
fn undated_observations_take_an_upper_bound() {
    let root = scratch("date-before");
    let id = add(&root, &question(Kind::Runtime, &[])).expect("add").id;
    let bench = NewObservation {
        date: None,
        date_before: Some("2026-09-10".into()),
        ..runtime("")
    };
    observe(&root, &id, &bench).expect("bounded");
    observe(&root, &id, &runtime("2026-09-05")).expect("dated");
    let q = reload(&root, &id);
    let ranked = q.ranked();
    // The bound orders like a date: 2026-09-10 is newer than 2026-09-05.
    assert_eq!(ranked[0].observation.date_string(), "before 2026-09-10");
    assert_eq!(ranked[1].observation.date_string(), "2026-09-05");

    let both = NewObservation {
        date_before: Some("2026-09-10".into()),
        ..runtime("2026-09-05")
    };
    refused(observe(&root, &id, &both), "both `date` and `date_before`");
    let neither = NewObservation {
        date: None,
        ..runtime("")
    };
    refused(
        observe(&root, &id, &neither),
        "needs a `date` or a `date_before`",
    );
}

#[test]
fn diagnostics_without_message_text() {
    let root = scratch("no-message");
    let id = add(&root, &question(Kind::Compile, &["export"]))
        .expect("add")
        .id;
    let reject = NewObservation {
        errors: vec![parse_diag("CE10271").expect("code only")],
        ..compile(Source::Editor, "2026-09-27", Outcome::Rejected)
    };
    observe(&root, &id, &reject).expect("observe");
    let q = reload(&root, &id);
    assert_eq!(q.observations[0].errors[0].message, None);

    // An explicitly empty message in a hand-edited file is still an error.
    let path = record(&root, &id);
    let text = std::fs::read_to_string(&path).expect("read");
    std::fs::write(
        &path,
        text.replace("code = \"CE10271\"", "code = \"CE10271\"\nmessage = \"\""),
    )
    .expect("write");
    refused(load(&root), "has an empty message");
}

#[test]
fn quoted_ctx_values_round_trip() {
    let root = scratch("quoted-ctx");
    let id = add(&root, &question(Kind::Compile, &["export"]))
        .expect("add")
        .id;
    let reject = NewObservation {
        errors: vec![parse_diag(r#"CE10079|m|possibleValues="a, b""#).expect("quoted")],
        ..compile(Source::Editor, "2026-09-27", Outcome::Rejected)
    };
    observe(&root, &id, &reject).expect("observe");
    let q = reload(&root, &id);
    assert_eq!(
        q.observations[0].errors[0]
            .ctx
            .get("possibleValues")
            .map(String::as_str),
        Some("a, b")
    );
}

#[test]
fn a_candidate_name_means_one_model_per_question() {
    let root = scratch("candidate-models");
    let id = add(&root, &question(Kind::Runtime, &[])).expect("add").id;
    let first = NewObservation {
        candidates: vec![parse_candidate("A|gross").expect("A")],
        selected: vec!["A".into()],
        ..runtime("2026-09-20")
    };
    observe(&root, &id, &first).expect("first");
    let different_model = NewObservation {
        candidates: vec![parse_candidate("A|net").expect("A")],
        refuted: vec!["A".into()],
        ..runtime("2026-09-21")
    };
    refused(
        observe(&root, &id, &different_model),
        "names different models",
    );
}

#[test]
fn evidence_must_be_a_file() {
    let root = scratch("evidence-dir");
    let id = add(&root, &question(Kind::Compile, &["export"]))
        .expect("add")
        .id;
    std::fs::create_dir_all(root.join("evidence/run26")).expect("mkdir");
    let dir_evidence = NewObservation {
        evidence: vec![ev(root.join("evidence/run26"))],
        ..compile(Source::Editor, "2026-09-27", Outcome::Accepted)
    };
    refused(observe(&root, &id, &dir_evidence), "not a regular file");
    let missing = NewObservation {
        evidence: vec![ev(root.join("evidence/no-such-export.json"))],
        ..compile(Source::Editor, "2026-09-27", Outcome::Accepted)
    };
    refused(observe(&root, &id, &missing), "missing");
}

/// Evidence is cited only from inside the evidence directory: a file
/// elsewhere (the testdata itself, the records, a `..` escape spelled from
/// inside) is refused before anything is read or written, and so is the
/// directory itself.
#[test]
fn evidence_outside_the_evidence_tree_is_refused() {
    let root = scratch("evidence-outside");
    let id = add(&root, &question(Kind::Compile, &["export"]))
        .expect("add")
        .id;
    let before = std::fs::read(record(&root, &id)).expect("read");
    let escape = root.join("evidence/../records").join(format!("{id}.toml"));
    for path in [
        testdata("run26-export-outside-library.json"),
        record(&root, &id),
        escape,
        root.join("evidence"),
    ] {
        let o = NewObservation {
            evidence: vec![ev(path)],
            ..compile(Source::Editor, "2026-09-27", Outcome::Accepted)
        };
        refused(observe(&root, &id, &o), "is outside");
    }
    assert_eq!(std::fs::read(record(&root, &id)).expect("read"), before);
    // The same file once inside is accepted.
    let inside = NewObservation {
        evidence: vec![ev(evidence_copy(
            &root,
            "run26-export-outside-library.json",
        ))],
        ..compile(Source::Editor, "2026-09-27", Outcome::Accepted)
    };
    observe(&root, &id, &inside).expect("inside the tree");
    // Moving evidence out of the tree is refused the same way.
    refused(
        relocate(
            &root,
            &root.join("evidence/run26-export-outside-library.json"),
            &testdata("run26-export-outside-library.json"),
            false,
            "out",
        ),
        "is outside",
    );
    // So are `merge` and `attest` naming a file outside.
    let outside = testdata("run26-export-outside-library.json");
    let inside_path = root.join("evidence/run26-export-outside-library.json");
    refused(
        merge(&root, &id, 1, &outside, &inside_path, "r"),
        "is outside",
    );
    refused(attest(&root, &[outside], false, "r"), "is outside");
}

/// Evidence is never reached through a symbolic link: not the file, not a
/// directory below `evidence/`, not `evidence/` itself. Every verb that reads
/// a file refuses one, and a stored entry that became a link fails the audit.
#[cfg(unix)]
#[test]
fn evidence_through_a_link_is_refused() {
    use std::os::unix::fs::symlink;
    let (root, ev_dir, bytes) = evidence_store("evidence-links");
    let json = ev_dir.join("run26.json");
    let id = observed_with(&root, "k", vec![ev(json.clone())]);
    let outside = root
        .parent()
        .expect("parent")
        .join("evidence-links-outside");
    std::fs::create_dir_all(&outside).expect("mkdir");
    std::fs::write(outside.join("run26.json"), &bytes).expect("copy");

    // The file, and a directory on the way to it.
    symlink(outside.join("run26.json"), ev_dir.join("linked.json")).expect("file link");
    symlink(&outside, ev_dir.join("dir")).expect("dir link");
    for path in [ev_dir.join("linked.json"), ev_dir.join("dir/run26.json")] {
        let o = NewObservation {
            evidence: vec![ev(path.clone())],
            ..compile(Source::Editor, "2026-09-28", Outcome::Accepted)
        };
        refused(observe(&root, &id, &o), "is a symbolic link");
        refused(
            relocate(&root, &json, &path, false, "r"),
            "is a symbolic link",
        );
    }
    std::fs::remove_file(ev_dir.join("dir")).expect("unlink");

    // A recorded file replaced by a link fails the audit.
    std::fs::remove_file(&json).expect("rm");
    symlink(outside.join("run26.json"), &json).expect("link in place");
    let store = load(&root).expect("loads");
    let audit = audit(&store, &[]).expect("audit");
    assert_eq!(audit.failing.len(), 1, "{audit:?}");
    assert!(
        audit.failing[0]
            .inspection
            .to_string()
            .contains("is a symbolic link"),
        "{}",
        audit.failing[0].inspection
    );
    refused(
        attest(&root, std::slice::from_ref(&json), false, "r"),
        "is a symbolic link",
    );
    // A keyed rerun naming the recorded location, now a link, is refused,
    // not matched by location.
    let rerun = NewObservation {
        key: Some("k".into()),
        evidence: vec![ev(json.clone())],
        ..compile(Source::Editor, "2026-09-27", Outcome::Accepted)
    };
    refused(observe(&root, &id, &rerun), "is a symbolic link");
    std::fs::remove_file(&json).expect("unlink");
    std::fs::write(&json, &bytes).expect("restore");

    // `evidence/` itself as a link.
    let real = root.parent().expect("parent").join("evidence-links-real");
    if real.exists() {
        std::fs::remove_dir_all(&real).expect("clear");
    }
    std::fs::rename(&ev_dir, &real).expect("move evidence away");
    symlink(&real, &ev_dir).expect("evidence link");
    let o = NewObservation {
        evidence: vec![ev(ev_dir.join("run26.json"))],
        ..compile(Source::Editor, "2026-09-28", Outcome::Accepted)
    };
    refused(observe(&root, &id, &o), "is a symbolic link");
    std::fs::remove_file(&ev_dir).expect("unlink evidence");
    std::fs::rename(&real, &ev_dir).expect("restore evidence");

    // `verdicts/` itself as a link: evidence through it is refused.
    let linked_root = root.parent().expect("parent").join("evidence-links-alias");
    if linked_root.symlink_metadata().is_ok() {
        std::fs::remove_file(&linked_root).expect("clear");
    }
    symlink(&root, &linked_root).expect("root link");
    let o = NewObservation {
        evidence: vec![ev(linked_root.join("evidence/run26.json"))],
        ..compile(Source::Editor, "2026-09-28", Outcome::Accepted)
    };
    refused(observe(&linked_root, &id, &o), "is a symbolic link");
}

/// A directory renamed and replaced by a link to its new place after a read
/// keeps every file's fingerprint; the read is still reported changed.
#[cfg(unix)]
#[test]
fn a_directory_turned_link_after_a_read_is_a_change() {
    let (root, ev_dir, bytes) = evidence_store("evidence-dir-turned-link");
    std::fs::create_dir(ev_dir.join("run26")).expect("mkdir");
    let file = ev_dir.join("run26/export.json");
    std::fs::write(&file, &bytes).expect("write");
    let id = observed_with(&root, "k", vec![ev(file)]);
    let entry = reload(&root, &id).observations[0].evidence[0].clone();

    let mut inspector = Inspector::new();
    assert_eq!(
        inspector.inspect(&root, &entry).comparison,
        Comparison::Matches
    );
    inspector.stable().expect("nothing changed yet");
    std::fs::rename(ev_dir.join("run26"), ev_dir.join("moved")).expect("rename");
    std::os::unix::fs::symlink("moved", ev_dir.join("run26")).expect("link");
    refused(inspector.stable(), "evidence changed while it was read");
    // Inspected again, the remembered result is not reused.
    assert!(
        inspector
            .inspect(&root, &entry)
            .to_string()
            .contains("is a symbolic link")
    );
}

/// A write that is refused creates nothing in a repository that never held
/// verdicts beyond `verdicts/.lock`.
#[test]
fn a_refused_write_creates_no_store_directories() {
    let root = scratch("refused-write-dirs").join("verdicts");
    refused(
        observe(&root, "deadbeef", &runtime("2026-09-20")),
        "no question",
    );
    let entries: Vec<String> = std::fs::read_dir(&root)
        .expect("verdicts/ holds the lock")
        .map(|e| e.expect("entry").file_name().to_string_lossy().into_owned())
        .collect();
    assert_eq!(entries, vec![".lock".to_string()]);
}

/// The store is `verdicts/` beside the nearest `.git`, a directory or the
/// file a linked worktree or submodule has; none above is an error.
#[test]
fn locate_finds_the_nearest_repository_root() {
    let base = scratch("locate");
    let deep = base.join("repo/a/b");
    std::fs::create_dir_all(&deep).expect("mkdir");
    std::fs::create_dir(base.join("repo/.git")).expect("git dir");
    assert_eq!(
        locate_from(&deep).expect("found"),
        base.join("repo/verdicts")
    );
    std::fs::write(base.join("repo/a/.git"), "gitdir: elsewhere").expect("git file");
    assert_eq!(
        locate_from(&deep).expect("found"),
        base.join("repo/a/verdicts")
    );
    assert!(locate_from(Path::new("/")).is_err() || Path::new("/.git").exists());
}

/// A stored location that climbs out of the evidence directory fails the
/// load: no record may cite a file outside the tree.
#[test]
fn stored_evidence_outside_the_tree_fails_the_load() {
    let root = scratch("evidence-outside-stored");
    let id = add(&root, &question(Kind::Compile, &["export"]))
        .expect("add")
        .id;
    let o = NewObservation {
        evidence: vec![ev(evidence_copy(
            &root,
            "run26-export-outside-library.json",
        ))],
        ..compile(Source::Editor, "2026-09-27", Outcome::Accepted)
    };
    observe(&root, &id, &o).expect("observe");
    let path = record(&root, &id);
    let text = std::fs::read_to_string(&path).expect("read");
    std::fs::write(
        &path,
        text.replace(
            "path = \"run26-export-outside-library.json\"",
            "path = \"../run26-export-outside-library.json\"",
        ),
    )
    .expect("write");
    refused(load(&root), "climbs out of the evidence directory");
}

/// The verdicts directory holds only `records/`, `fixtures/`, `evidence/`,
/// notes and dot-files; a missing directory is an empty store, a dangling
/// link is an error, and the first write creates it.
#[test]
fn verdicts_directory_layout() {
    let base = scratch("verdicts-layout");
    let root = base.join("verdicts");
    assert!(load(&root).expect("absent loads").questions().is_empty());
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(base.join("gone"), &root).expect("link");
        refused(load(&root), "a link to nothing");
        std::fs::remove_file(&root).expect("unlink");
    }
    let id = add(&root, &question(Kind::Compile, &["export"]))
        .expect("add creates the directory")
        .id;
    assert!(record(&root, &id).is_file());
    // Anything else is refused, at the top and inside `records/`; evidence
    // contents are the caller's and never vetted.
    std::fs::create_dir(root.join("evidence")).expect("mkdir");
    std::fs::write(root.join("evidence/anything.bin"), "x").expect("write");
    std::fs::write(root.join("README.md"), "notes").expect("write");
    assert!(load(&root).is_ok());
    let stray = root.join("12345678.toml");
    std::fs::write(&stray, "").expect("write");
    refused(load(&root), "not `records/`");
    std::fs::remove_file(&stray).expect("rm");
    std::fs::write(root.join("records/notes.txt"), "").expect("write");
    refused(load(&root), "not a question file");
}

#[test]
fn search_treats_query_syntax_as_plain_text() {
    let root = scratch("search-syntax");
    let id = add(&root, &question(Kind::Runtime, &[])).expect("add").id;
    let run = NewObservation {
        settings: Some("NASDAQ:AAPL 1D, defaults".into()),
        ..runtime("2026-09-20")
    };
    observe(&root, &id, &run).expect("observe");
    let store = load(&root).expect("load");
    for q in ["NASDAQ:AAPL", "strategy.exit( NASDAQ", "?: NASDAQ"] {
        let hits = store.search(q, 5).expect("search");
        assert_eq!(
            hits.first().map(|h| h.id.as_str()),
            Some(id.as_str()),
            "{q}"
        );
    }
    assert!(store.search("?:", 5).expect("search").is_empty());
    // Uppercase operator words are plain words too: `NOT` excludes nothing.
    for q in ["TV did", "TV NOT did", "TV AND nothing", "OR did"] {
        let hits = store.search(q, 5).expect("search");
        assert_eq!(
            hits.first().map(|h| h.id.as_str()),
            Some(id.as_str()),
            "{q}"
        );
    }
}

#[test]
fn qualified_parameter_identifiers() {
    let root = scratch("param-identifiers");
    add(
        &root,
        &question(Kind::Runtime, &["strategy(process_orders_on_close)"]),
    )
    .expect("a real strategy() parameter");
    refused(
        add(
            &root,
            &question(Kind::Runtime, &["process_orders_on_close"]),
        ),
        "qualify a parameter",
    );
    refused(
        add(
            &root,
            &question(Kind::Runtime, &["strategy(no_such_param)"]),
        ),
        "is not a parameter of `strategy`",
    );
    refused(
        add(&root, &question(Kind::Runtime, &["close(length)"])),
        "is not a pine-data function",
    );
}

#[test]
fn non_counting_top_observations_do_not_cause_conflict() {
    let root = scratch("top-not-counting");
    let id = add(&root, &question(Kind::Compile, &["export"]))
        .expect("add")
        .id;
    observe(
        &root,
        &id,
        &compile(Source::Editor, "2026-09-27", Outcome::Accepted),
    )
    .expect("accept");
    let crash = NewObservation {
        crash: Some("editor threw".into()),
        ..compile(Source::Editor, "2026-09-27", Outcome::Crashed)
    };
    observe(&root, &id, &crash).expect("crash");
    let confounded = NewObservation {
        inconclusive: Some("confounded".into()),
        errors: vec![parse_diag("CE10099|m").expect("d")],
        ..compile(Source::Editor, "2026-09-27", Outcome::Rejected)
    };
    observe(&root, &id, &confounded).expect("inconclusive reject");
    assert_eq!(
        reload(&root, &id).own_status(),
        Status::Settled,
        "an editor crash and an inconclusive editor reject must not contradict an editor accept"
    );
}

#[test]
fn conflict_leaves_weaker_rows_unannotated_and_same_day_is_newest_first() {
    let root = scratch("conflict-annotations");
    let id = add(&root, &question(Kind::Compile, &["export"]))
        .expect("add")
        .id;
    observe(
        &root,
        &id,
        &compile(Source::Endpoint, "2026-09-01", Outcome::Accepted),
    )
    .expect("endpoint");
    observe(
        &root,
        &id,
        &compile(Source::Editor, "2026-09-27", Outcome::Accepted),
    )
    .expect("first editor");
    let reject = NewObservation {
        errors: vec![parse_diag("CE10099|m").expect("d")],
        ..compile(Source::Editor, "2026-09-27", Outcome::Rejected)
    };
    observe(&root, &id, &reject).expect("second editor");

    let q = reload(&root, &id);
    assert_eq!(q.own_status(), Status::Conflict);
    let ranked = q.ranked();
    // Same source strength and date: the later-recorded observation first.
    assert_eq!(ranked[0].observation.outcome, Some(Outcome::Rejected));
    assert_eq!(ranked[1].observation.outcome, Some(Outcome::Accepted));
    assert_eq!(ranked[2].observation.source, Source::Endpoint);
    assert_eq!(
        ranked[2].annotation, None,
        "no 'confirmed by editor' when editors disagree"
    );
}

#[test]
fn relation_cycles_are_rejected_per_relation() {
    for field in ["basis", "follow_up_to"] {
        let root = scratch(&format!("cycle-{field}"));
        let a = add(&root, &question(Kind::Compile, &["export"]))
            .expect("add")
            .id;
        let b = add(
            &root,
            &NewQuestion {
                basis: if field == "basis" {
                    vec![a.clone()]
                } else {
                    Vec::new()
                },
                follow_up_to: if field == "follow_up_to" {
                    vec![a.clone()]
                } else {
                    Vec::new()
                },
                ..question(Kind::Compile, &["export"])
            },
        )
        .expect("add")
        .id;
        let path = record(&root, &a);
        let text = std::fs::read_to_string(&path).expect("read");
        std::fs::write(
            &path,
            text.replace(&format!("{field} = []"), &format!("{field} = [\"{b}\"]")),
        )
        .expect("write");
        refused(load(&root), &format!("reaches itself through `{field}`"));
    }

    // Across relations is fine: a parent replaced by its own follow-up.
    let root = scratch("cycle-across");
    let parent = add(&root, &question(Kind::Compile, &["export"]))
        .expect("add")
        .id;
    let child = add(
        &root,
        &NewQuestion {
            follow_up_to: vec![parent.clone()],
            ..question(Kind::Compile, &["export"])
        },
    )
    .expect("add")
    .id;
    retire(&root, &parent, "split", &[child]).expect("replaced by its follow-up");
}

#[test]
fn truncated_orphan_fixture_blocks_until_removed() {
    let root = scratch("corrupt-fixture");
    let id = add(&root, &question(Kind::Compile, &["export"]))
        .expect("add")
        .id;
    // An interrupted earlier write: a truncated file under the right name
    // that no observation references. The strict load hashes every stored
    // fixture, so it is caught before a write could build on it.
    std::fs::create_dir_all(root.join("fixtures")).expect("mkdir");
    let path = root.join("fixtures").join(format!("{RUN26_SHA}.pine"));
    std::fs::write(&path, "//@vers").expect("truncated");
    refused(load(&root), "hashes to");
    let editor = NewObservation {
        fixture: Some(testdata("editor-export-outside-library.pine")),
        ..compile(Source::Editor, "2026-09-27", Outcome::Accepted)
    };
    refused(observe(&root, &id, &editor), "hashes to");

    std::fs::remove_file(&path).expect("remove");
    observe(&root, &id, &editor).expect("observe");
    let stored = std::fs::read(&path).expect("stored fixture");
    assert_eq!(
        stored,
        std::fs::read(testdata("editor-export-outside-library.pine")).expect("source")
    );
}

#[test]
fn strict_load_rejects_every_documented_problem() {
    let root = scratch("load-problems");
    let id = add(&root, &question(Kind::Runtime, &[])).expect("add").id;
    observe(&root, &id, &runtime("2026-09-20")).expect("observe");
    let path = record(&root, &id);
    let good = std::fs::read_to_string(&path).expect("read");

    for (edit, needle) in [
        (
            good.replace("date = 2026-09-20", "date = \"2026-09-20\""),
            "date",
        ),
        (
            good.replace("result = \"TV did X.\"", "result = \"  \""),
            "`result` is empty",
        ),
        (format!("{good}evidence_typo = 1\n"), "unknown field"),
        (
            good.replace("settings = ", "evidence = [\"/abs/x.json\"]\nsettings = "),
            "evidence `/abs/x.json` is absolute",
        ),
        (
            good.replace("settings = ", "evidence = [\"\"]\nsettings = "),
            "an empty evidence path",
        ),
        (
            good.replace(
                "settings = ",
                "evidence = [{ path = \"x.json\", attested = { at = 2026-01-01T00:00:00Z, \
                 reason = \"r\" } }]\nsettings = ",
            ),
            "need a `sha256`",
        ),
        (
            good.replace(
                "settings = ",
                "evidence = [{ path = \"x.json\", colour = 1 }]\nsettings = ",
            ),
            "unknown field",
        ),
        (
            good.replace(
                "settings = ",
                "evidence = [\"a.json\", { path = \"b.json\", sha256 = \"0000000000000000000000000000000000000000000000000000000000000000\", moved = [{ at = 2026-10-10T00:00:00Z, from = \"a.json\", reason = \"r\" }] }]\nsettings = ",
            ),
            "belongs to two entries",
        ),
        (
            format!("{good}\n[[observation.warning]]\ncode = \"CW10001\"\nmessage = \"m\"\n"),
            "carry errors only",
        ),
        (
            format!(
                "{good}\n[[observation.error]]\ncode = \"RE10044\"\nmessage = \"m\"\nspan = \"0:0-0:0\"\n"
            ),
            "ordered 1-based",
        ),
        (
            format!(
                "{good}\n[[observation.candidate]]\nname = \"A\"\nstate = \"selected\"\n\n[[observation.candidate]]\nname = \"A\"\nstate = \"refuted\"\n"
            ),
            "listed twice",
        ),
    ] {
        std::fs::write(&path, &edit).expect("write");
        refused(load(&root), needle);
    }
    std::fs::write(&path, &good).expect("restore");
    assert!(load(&root).is_ok());

    // Nothing in `records/` may be silently skipped.
    let records = root.join("records");
    for stray in ["ABCD1234.TOML", "notes.txt", "12345678.toml.bak"] {
        std::fs::write(records.join(stray), "").expect("stray");
        refused(load(&root), "not a question file");
        std::fs::remove_file(records.join(stray)).expect("remove");
    }
    std::fs::create_dir(records.join("sub")).expect("subdir");
    refused(load(&root), "not a question file");
    std::fs::remove_dir(records.join("sub")).expect("rmdir");
    // Nor at the top. A dot-directory is not a dot-file: it would hide its
    // contents.
    for dir in ["sub", ".hidden"] {
        std::fs::create_dir(root.join(dir)).expect("dir");
        refused(load(&root), "not `records/`");
        std::fs::remove_dir(root.join(dir)).expect("rmdir");
    }
    std::fs::create_dir_all(root.join("fixtures/.hidden")).expect("fixtures dot dir");
    refused(load(&root), "not a `<sha256>.pine` fixture");
    std::fs::remove_dir_all(root.join("fixtures")).expect("rmdir");
    // Markdown notes and dot-files (in-flight temp files) are allowed.
    std::fs::write(root.join("README.md"), "notes").expect("readme");
    std::fs::write(records.join(".x.toml.1.tmp"), "junk").expect("tmp");
    assert!(load(&root).is_ok());
}

#[test]
fn docs_example_matches_the_on_disk_shape() {
    let docs =
        std::fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("docs/verdict.md"))
            .expect("docs/verdict.md");
    let start = docs.find("```toml\n").expect("toml example") + "```toml\n".len();
    let len = docs[start..].find("```").expect("fence end");
    let q: Question = toml::from_str(&docs[start..start + len]).expect("example parses");
    assert_eq!(q.kind, Kind::Compile);
    assert_eq!(q.observations.len(), 2);
    assert_eq!(
        q.observations[0].errors[0].span.as_deref(),
        Some("9:1-10:5")
    );
    assert_eq!(q.observations[1].outcome, Some(Outcome::Accepted));
}

#[test]
fn strict_load_catches_tampering_and_unknown_fields() {
    let root = scratch("tamper");
    let id = add(&root, &question(Kind::Compile, &["export"]))
        .expect("add")
        .id;
    let editor = NewObservation {
        fixture: Some(testdata("editor-export-outside-library.pine")),
        ..compile(Source::Editor, "2026-09-27", Outcome::Accepted)
    };
    observe(&root, &id, &editor).expect("observe");
    assert!(load(&root).is_ok());

    let fixture = root.join("fixtures").join(format!("{RUN26_SHA}.pine"));
    std::fs::write(&fixture, "//@version=6\n").expect("tamper");
    let err = load(&root).expect_err("tampered fixture must fail the load");
    assert!(err.to_string().contains("hashes to"), "{err}");
    std::fs::copy(testdata("editor-export-outside-library.pine"), &fixture).expect("restore");

    let path = record(&root, &id);
    let text = std::fs::read_to_string(&path).expect("read");
    std::fs::write(&path, format!("sources = \"typo\"\n{text}")).expect("write");
    assert!(load(&root).is_err(), "unknown field must fail the load");
}

#[test]
fn search_finds_questions_by_prose_code_and_fixture_source() {
    let root = scratch("search");
    let id = add(
        &root,
        &NewQuestion {
            question: "Can an indicator script declare an exported function?".to_string(),
            answer: "No. Only libraries can contain exported functions.".to_string(),
            ..question(Kind::Compile, &["export"])
        },
    )
    .expect("add")
    .id;
    let editor = NewObservation {
        fixture: Some(testdata("editor-export-outside-library.pine")),
        errors: vec![
            parse_diag("CE10099|Only libraries can contain exported functions.").expect("d"),
        ],
        ..compile(Source::Editor, "2026-09-27", Outcome::Rejected)
    };
    observe(&root, &id, &editor).expect("observe");
    let store = load(&root).expect("load");
    for q in ["indicator exported function", "CE10099", "plot"] {
        let hits = store.search(q, 5).expect("search");
        assert_eq!(
            hits.first().map(|h| h.id.as_str()),
            Some(id.as_str()),
            "{q}"
        );
    }
    assert!(store.search("   ", 5).expect("search").is_empty());
}

/// A capture script rerun: the same keyed add and observe are no-ops that
/// hand back what the first run wrote; a changed payload under the same key
/// is refused with the differing field named.
#[test]
fn capture_keys_make_reruns_idempotent() {
    let root = scratch("capture-keys");
    let keyed = NewQuestion {
        key: Some("run26/export".into()),
        ..question(Kind::Compile, &["export"])
    };
    let first = add(&root, &keyed).expect("first add");
    assert!(!first.existing);
    let again = add(&root, &keyed).expect("rerun add");
    assert!(again.existing);
    assert_eq!(again.id, first.id);
    assert_eq!(load(&root).expect("load").questions().len(), 1);
    refused(
        add(
            &root,
            &NewQuestion {
                answer: "B.".into(),
                ..keyed.clone()
            },
        ),
        "`answer`: stored \"A.\", given \"B.\"",
    );

    let editor = NewObservation {
        key: Some("editor".into()),
        fixture: Some(testdata("editor-export-outside-library.pine")),
        ..compile(Source::Editor, "2026-09-27", Outcome::Accepted)
    };
    let o1 = observe(&root, &first.id, &editor).expect("first observe");
    assert_eq!((o1.number, o1.existing), (1, false));
    let o2 = observe(&root, &first.id, &editor).expect("rerun observe");
    assert_eq!((o2.number, o2.existing), (1, true));
    assert_eq!(reload(&root, &first.id).observations.len(), 1);
    refused(
        observe(
            &root,
            &first.id,
            &NewObservation {
                note: Some("new".into()),
                ..editor.clone()
            },
        ),
        "`note`: stored (absent)",
    );
    // An unkeyed observation is always appended: two identical runs are two
    // measurements.
    let plain = compile(Source::Editor, "2026-09-27", Outcome::Accepted);
    observe(&root, &first.id, &plain).expect("unkeyed");
    observe(&root, &first.id, &plain).expect("unkeyed again");
    assert_eq!(reload(&root, &first.id).observations.len(), 3);

    // A retired question does not hand its id back to a rerun.
    retire(&root, &first.id, "reworded", &[]).expect("retire");
    refused(add(&root, &keyed), "which is retired");

    // A key held twice (a hand edit) fails the load.
    let other = add(&root, &question(Kind::Compile, &["export"]))
        .expect("add")
        .id;
    let path = record(&root, &other);
    let text = std::fs::read_to_string(&path).expect("read");
    std::fs::write(&path, format!("key = \"run26/export\"\n{text}")).expect("write");
    refused(load(&root), "key `run26/export` is held by questions");
}

/// The 6293fd8b case: amending a keyed record's wording must not break the
/// landing script that wrote it. A rerun matches any value an amended field
/// has held; the stored value stands.
#[test]
fn amending_keeps_keyed_reruns_idempotent() {
    let root = scratch("amend-keyed");
    let keyed = NewQuestion {
        key: Some("grid".into()),
        ..question(Kind::Runtime, &[])
    };
    let id = add(&root, &keyed).expect("add").id;
    let run = NewObservation {
        key: Some("grid/1000".into()),
        result: Some("all but 0 off the 1000 grid".into()),
        ..runtime("2026-09-20")
    };
    observe(&root, &id, &run).expect("observe");

    let result = AmendTarget::Observation {
        number: 1,
        field: AmendField::Result,
    };
    let reworded = "every integral cell of the 1000 grid except x = 0";
    amend(&root, &id, result, Some(reworded), "unclear wording").expect("amend result");
    amend(&root, &id, AmendTarget::Answer, Some("B."), "sharper").expect("amend answer");

    // The original script, and one updated to the corrected values, rerun
    // as no-ops; the original is told its value is superseded.
    let rerun = observe(&root, &id, &run).expect("original rerun");
    assert_eq!((rerun.number, rerun.existing), (1, true));
    assert_eq!(
        rerun.warnings,
        vec![
            "the given `result` is a superseded value of observation #1; the amended value \
             stands. Update the script that emits it."
        ]
    );
    let updated = NewObservation {
        result: Some(reworded.into()),
        ..run.clone()
    };
    let rerun = observe(&root, &id, &updated).expect("updated rerun");
    assert!(rerun.existing && rerun.warnings.is_empty());
    let rerun = add(&root, &keyed).expect("original add rerun");
    assert!(rerun.existing);
    assert_eq!(
        rerun.warnings,
        vec![format!(
            "the given `answer` is a superseded value of question {id}; the amended value \
             stands. Update the script that emits it."
        )]
    );
    let updated_add = NewQuestion {
        answer: "B.".into(),
        ..keyed.clone()
    };
    let rerun = add(&root, &updated_add).expect("updated add rerun");
    assert!(rerun.existing && rerun.warnings.is_empty());

    // Optional fields: a note set from absent and an inconclusive reason
    // set then cleared both have an absent value in their history, which a
    // script that never passed them matches. Every step of a chain counts.
    let note = AmendTarget::Observation {
        number: 1,
        field: AmendField::Note,
    };
    amend(&root, &id, note, Some("n1"), "r").expect("note");
    amend(&root, &id, note, Some("n2"), "r").expect("note again");
    let inconclusive = AmendTarget::Observation {
        number: 1,
        field: AmendField::Inconclusive,
    };
    amend(&root, &id, inconclusive, Some("confounded"), "r").expect("inconclusive");
    for given in [None, Some("n1"), Some("n2")] {
        let o = NewObservation {
            note: given.map(Into::into),
            ..run.clone()
        };
        assert!(observe(&root, &id, &o).expect("note rerun").existing);
    }
    // The stored inconclusive reason stands: the original rerun cannot
    // make the observation count again.
    assert!(reload(&root, &id).observations[0].inconclusive.is_some());
    amend(&root, &id, inconclusive, None, "r").expect("clear");
    let o = NewObservation {
        inconclusive: Some("confounded".into()),
        ..run.clone()
    };
    assert!(
        observe(&root, &id, &o)
            .expect("inconclusive rerun")
            .existing
    );
    assert!(reload(&root, &id).observations[0].inconclusive.is_none());
    let q = reload(&root, &id);
    assert_eq!(q.answer, "B.");
    assert_eq!(q.observations[0].result.as_deref(), Some(reworded));

    // A value the field never held still differs, and an unamended field
    // is compared as before.
    refused(
        observe(
            &root,
            &id,
            &NewObservation {
                result: Some("something else".into()),
                ..run.clone()
            },
        ),
        "`result`: stored",
    );
    refused(
        observe(
            &root,
            &id,
            &NewObservation {
                note: Some("n3".into()),
                ..run.clone()
            },
        ),
        "`note`: stored \"n2\", given \"n3\"",
    );
    refused(
        observe(
            &root,
            &id,
            &NewObservation {
                environment: Some("new".into()),
                ..run
            },
        ),
        "`environment`: stored (absent)",
    );
    refused(
        add(
            &root,
            &NewQuestion {
                answer: "C.".into(),
                ..keyed
            },
        ),
        "`answer`: stored \"B.\", given \"C.\"",
    );
}

/// The piners cases: a note with wrong counts, and an answer that claimed
/// more than was measured, corrected in place with the history kept.
#[test]
fn amendments_keep_history_and_chain() {
    let root = scratch("amend");
    let id = add(&root, &question(Kind::Runtime, &[])).expect("add").id;
    let run = NewObservation {
        note: Some("35 long round trips over 5602 bars.".into()),
        ..runtime("2026-09-20")
    };
    observe(&root, &id, &run).expect("observe");

    let note = AmendTarget::Observation {
        number: 1,
        field: AmendField::Note,
    };
    amend(
        &root,
        &id,
        note,
        Some("70 long round trips over about 20,240 bars."),
        "the export lists 70; 5602 was the loaded chart bars",
    )
    .expect("amend note");
    amend(
        &root,
        &id,
        AmendTarget::Answer,
        Some("B."),
        "claimed more than measured",
    )
    .expect("amend answer");
    let q = reload(&root, &id);
    assert_eq!(q.answer, "B.");
    assert_eq!(q.amendments[0].was.as_deref(), Some("A."));
    let o = &q.observations[0];
    assert_eq!(
        o.note.as_deref(),
        Some("70 long round trips over about 20,240 bars.")
    );
    assert_eq!(
        o.amendments[0].was.as_deref(),
        Some("35 long round trips over 5602 bars.")
    );

    // A second correction chains onto the first; clearing leaves `now` out.
    amend(&root, &id, note, None, "the note restates the result").expect("clear note");
    let o = reload(&root, &id).observations[0].clone();
    assert_eq!(o.note, None);
    assert_eq!(o.amendments[1].was, o.amendments[0].now);
    assert_eq!(o.amendments[1].now, None);

    // Search sees current wording only.
    let store = load(&root).expect("load");
    assert!(store.search("5602", 5).expect("search").is_empty());

    // Refusals: a no-op, a missing observation, a required runtime field
    // cleared, an answer cleared.
    refused(
        amend(&root, &id, AmendTarget::Answer, Some("B."), "r"),
        "already reads that",
    );
    let missing = AmendTarget::Observation {
        number: 2,
        field: AmendField::Note,
    };
    refused(
        amend(&root, &id, missing, Some("x"), "r"),
        "has no observation #2 (it has 1)",
    );
    let result = AmendTarget::Observation {
        number: 1,
        field: AmendField::Result,
    };
    refused(amend(&root, &id, result, None, "r"), "needs a `result`");
    refused(
        amend(&root, &id, AmendTarget::Answer, None, "r"),
        "cannot be cleared",
    );

    // An amended field edited by hand, without its history, fails the load.
    let path = record(&root, &id);
    let text = std::fs::read_to_string(&path).expect("read");
    std::fs::write(&path, text.replace("answer = \"B.\"", "answer = \"C.\"")).expect("write");
    refused(load(&root), "amend it rather than editing it by hand");
    std::fs::write(&path, &text).expect("restore");

    // Retired questions take corrections too.
    retire(&root, &id, "reworded", &[]).expect("retire");
    amend(
        &root,
        &id,
        AmendTarget::Answer,
        Some("D."),
        "typo in the former answer",
    )
    .expect("amend retired");
}

#[test]
fn inconclusive_and_void_stop_an_observation_counting() {
    let root = scratch("void");
    let id = add(&root, &question(Kind::Compile, &["export"]))
        .expect("add")
        .id;
    let accept = NewObservation {
        key: Some("a".into()),
        ..compile(Source::Editor, "2026-09-27", Outcome::Accepted)
    };
    observe(&root, &id, &accept).expect("accept");
    let reject = NewObservation {
        errors: vec![parse_diag("CE10099").expect("d")],
        ..compile(Source::Editor, "2026-09-27", Outcome::Rejected)
    };
    observe(&root, &id, &reject).expect("reject");
    assert_eq!(reload(&root, &id).own_status(), Status::Conflict);

    // Marked inconclusive after the fact, the reject stops counting; cleared,
    // it counts again.
    let inconclusive = AmendTarget::Observation {
        number: 2,
        field: AmendField::Inconclusive,
    };
    amend(
        &root,
        &id,
        inconclusive,
        Some("fixture tested a different claim"),
        "reviewed",
    )
    .expect("mark");
    assert_eq!(reload(&root, &id).own_status(), Status::Settled);
    amend(
        &root,
        &id,
        inconclusive,
        None,
        "the claim was the same after all",
    )
    .expect("clear");
    assert_eq!(reload(&root, &id).own_status(), Status::Conflict);

    // A mis-transcribed accept: void it, the reject stands alone.
    void(
        &root,
        &id,
        VoidTarget::Observation(1),
        "the capture was a reject; transcribed wrong",
    )
    .expect("void");
    let q = reload(&root, &id);
    assert_eq!(q.own_status(), Status::Settled);
    let voided = q.ranked().into_iter().find(|r| r.number == 1).expect("#1");
    assert_eq!(
        voided.annotation.as_deref(),
        Some("void, does not count: the capture was a reject; transcribed wrong")
    );
    refused(
        void(&root, &id, VoidTarget::Observation(1), "again"),
        "already void",
    );
    refused(
        void(&root, &id, VoidTarget::Observation(3), "r"),
        "has no observation #3",
    );
    // A rerun script does not quietly re-confirm a voided capture.
    refused(observe(&root, &id, &accept), "which is void");

    // A void observation's codes no longer find the question.
    let by_code = |root: &Path| {
        ids_of(
            &load(root).expect("load"),
            &ListFilter {
                code: Some("CE10099"),
                ..Default::default()
            },
        )
    };
    assert_eq!(by_code(&root), vec![id.clone()]);
    void(&root, &id, VoidTarget::Observation(2), "wrong fixture").expect("void reject");
    assert!(by_code(&root).is_empty());
    assert!(
        load(&root)
            .expect("load")
            .search("CE10099", 5)
            .expect("search")
            .is_empty()
    );

    // A voided mis-recorded candidate model does not block the corrected one.
    let rt = add(&root, &question(Kind::Runtime, &[])).expect("add").id;
    let wrong = NewObservation {
        candidates: vec![parse_candidate("A|gross").expect("A")],
        selected: vec!["A".into()],
        ..runtime("2026-09-20")
    };
    observe(&root, &rt, &wrong).expect("wrong model");
    void(
        &root,
        &rt,
        VoidTarget::Observation(1),
        "model transcribed wrong",
    )
    .expect("void");
    let right = NewObservation {
        candidates: vec![parse_candidate("A|net").expect("A")],
        selected: vec!["A".into()],
        ..runtime("2026-09-20")
    };
    observe(&root, &rt, &right).expect("corrected model under the same name");
}

#[test]
fn answer_support_is_reported() {
    let root = scratch("answer-support");
    let id = add(&root, &question(Kind::Compile, &["export"]))
        .expect("add")
        .id;
    observe(
        &root,
        &id,
        &compile(Source::Endpoint, "2026-09-10", Outcome::Accepted),
    )
    .expect("endpoint");
    observe(
        &root,
        &id,
        &compile(Source::Editor, "2026-09-27", Outcome::Accepted),
    )
    .expect("editor");
    let support = amend(&root, &id, AmendTarget::Answer, Some("B."), "r")
        .expect("amend")
        .support
        .expect("answer support");
    assert_eq!(
        support,
        "the answer rests on counting editor/chart observations #2 editor; weaker counting observations #1 endpoint; status settled"
    );
    let note = AmendTarget::Observation {
        number: 1,
        field: AmendField::Note,
    };
    assert_eq!(
        amend(&root, &id, note, Some("n"), "r")
            .expect("amend note")
            .support,
        None
    );
    retire(&root, &id, "reworded", &[]).expect("retire");
    let support = amend(&root, &id, AmendTarget::Answer, Some("C."), "r")
        .expect("amend")
        .support
        .expect("answer support");
    assert!(support.contains("no active disposition"), "{support}");

    // A retired replacement still names what it replaced.
    let successor = add(&root, &question(Kind::Compile, &["export"]))
        .expect("add")
        .id;
    let predecessor = add(&root, &question(Kind::Compile, &["export"]))
        .expect("add")
        .id;
    retire(
        &root,
        &predecessor,
        "reworded",
        std::slice::from_ref(&successor),
    )
    .expect("retire predecessor");
    retire(&root, &successor, "split", &[]).expect("retire successor");
    let support = amend(&root, &successor, AmendTarget::Answer, Some("D."), "r")
        .expect("amend")
        .support
        .expect("answer support");
    assert!(
        support.ends_with(&format!("no active disposition; replaces {predecessor}")),
        "{support}"
    );
}

/// The manual section behind piners verdict 127addd2 (`po search` id), with
/// both bullets as `po show` prints them: emphasis as asterisks, link text
/// without its URL.
const LOWER_TF: &str = "b3541569";
const LOOKAHEAD_ON: &str = "barmerge.lookahead_on returns the *first* intrabar of the period historically, but the *last* intrabar in realtime.";
const LOOKAHEAD_OFF: &str = "barmerge.lookahead_off always returns the last intrabar for both historical and realtime data.";

fn quotes(q: &[&str]) -> Vec<String> {
    q.iter().map(ToString::to_string).collect()
}

/// The 127addd2 case: a runtime question the manual states outright, with
/// no practical live capture. Cited, it is `documented` - owed no
/// measurement, but never `settled` - until a measurement decides it.
#[test]
fn manual_citations_document_a_question() {
    let root = scratch("cite");
    let id = add(&root, &question(Kind::Runtime, &["request.security"]))
        .expect("add")
        .id;
    assert_eq!(resolved(&root, &id), Resolved::Measured(Status::Open));

    let both = quotes(&[LOOKAHEAD_ON, LOOKAHEAD_OFF]);
    let cited = cite(&root, &id, LOWER_TF, &both, None).expect("cite");
    assert_eq!((cited.number, cited.existing), (1, false));
    assert_eq!(resolved(&root, &id), Resolved::Documented);
    let q = reload(&root, &id);
    assert_eq!(
        q.citations[0].section,
        "faq/other-data-and-timeframes#lower-timeframes"
    );
    // The same claim again is a no-op; the page#anchor spelling resolves too.
    let again = cite(
        &root,
        &id,
        "faq/other-data-and-timeframes#lower-timeframes",
        &both,
        None,
    )
    .expect("cite again");
    assert_eq!((again.number, again.existing), (1, true));

    // `list --status open` no longer owes it a measurement.
    let store = load(&root).expect("load");
    let by_status = |status| {
        ids_of(
            &store,
            &ListFilter {
                status: Some(status),
                ..Default::default()
            },
        )
    };
    assert_eq!(by_status(Status::Documented), vec![id.clone()]);
    assert!(by_status(Status::Open).is_empty());
    assert!(by_status(Status::Settled).is_empty());
    assert_eq!(
        store
            .search("first intrabar historically", 5)
            .expect("search")[0]
            .id,
        id
    );

    // An inference resting on it is documented, naming the premise.
    let inferred = add(
        &root,
        &NewQuestion {
            basis: vec![id.clone()],
            ..question(Kind::Runtime, &["request.security"])
        },
    )
    .expect("add inferred")
    .id;
    assert_eq!(
        resolved(&root, &inferred).to_string(),
        format!("inferred (documented via {id})")
    );
    refused(
        cite(&root, &inferred, LOWER_TF, &both, None),
        "carries an active citation",
    );

    // A measurement that decides wins over the manual.
    observe(&root, &id, &runtime("2026-09-29")).expect("chart run");
    assert_eq!(resolved(&root, &id), Resolved::Measured(Status::Settled));
}

#[test]
fn citation_quotes_must_be_in_one_block_of_the_section() {
    let root = scratch("cite-quotes");
    let id = add(&root, &question(Kind::Runtime, &["request.security"]))
        .expect("add")
        .id;
    refused(
        cite(
            &root,
            &id,
            LOWER_TF,
            &quotes(&["returns the first intrabar in realtime"]),
            None,
        ),
        "does not say",
    );
    // Both halves are real, but they sit in two list items.
    refused(
        cite(
            &root,
            &id,
            LOWER_TF,
            &quotes(&["intrabar in realtime. barmerge.lookahead_off always"]),
            None,
        ),
        "does not say",
    );
    refused(
        cite(&root, &id, LOWER_TF, &quotes(&[LOOKAHEAD_ON, "  "]), None),
        "an empty --quote",
    );
    // Nor may the quote itself be two paragraphs joined into one.
    refused(
        cite(
            &root,
            &id,
            LOWER_TF,
            &quotes(&["the first\n\nintrabar of the period"]),
            None,
        ),
        "does not say",
    );
    refused(
        cite(&root, &id, "ffffffff", &quotes(&[LOOKAHEAD_ON]), None),
        "no manual section with id",
    );
    refused(
        cite(&root, &id, "faq/nope#nope", &quotes(&[LOOKAHEAD_ON]), None),
        "no manual section `faq/nope#nope`",
    );
    // Pine generics parse as inline HTML; they are quoted text, never
    // dropped, so an invented type spelling does not shrink to `array`, and
    // the manual's own declaration syntax matches as written.
    const ARRAYS: &str = "language/arrays#declaring-arrays";
    refused(
        cite(
            &root,
            &id,
            ARRAYS,
            &quotes(&["array<definitely_not_a_pine_type>"]),
            None,
        ),
        "does not say",
    );
    cite(
        &root,
        &id,
        ARRAYS,
        &quotes(&["[var/varip ][array<type> ]<identifier> = <expression>"]),
        None,
    )
    .expect("declaration syntax is quotable");
    retire(&root, &id, "reworded", &[]).expect("retire");
    refused(
        cite(&root, &id, LOWER_TF, &quotes(&[LOOKAHEAD_ON]), None),
        "is retired",
    );
}

/// A re-vendored manual that changed the section: the citation goes stale,
/// the load still succeeds (so the repair writes work), and the question is
/// owed work again until the stale citation is voided and the passage cited
/// anew.
#[test]
fn stale_citations_stop_counting_without_failing_the_load() {
    let root = scratch("cite-stale");
    let id = add(&root, &question(Kind::Runtime, &["request.security"]))
        .expect("add")
        .id;
    cite(&root, &id, LOWER_TF, &quotes(&[LOOKAHEAD_ON]), None).expect("cite");
    // Simulate drift: the digest recorded at cite time no longer matches.
    let path = record(&root, &id);
    let text = std::fs::read_to_string(&path).expect("read");
    let digest = reload(&root, &id).citations[0].digest.clone();
    std::fs::write(&path, text.replace(&digest, &"0".repeat(64))).expect("write");

    assert_eq!(resolved(&root, &id), Resolved::Measured(Status::Open));
    let stale = reload(&root, &id).citations[0].not_counting();
    assert!(
        stale
            .as_deref()
            .is_some_and(|s| s.starts_with("stale, does not count")),
        "{stale:?}"
    );
    // One stale member stops the whole support set, and the caller is told
    // which one to review.
    let other = cite(&root, &id, LOWER_TF, &quotes(&[LOOKAHEAD_OFF]), None).expect("cite other");
    assert_eq!(resolved(&root, &id), Resolved::Measured(Status::Open));
    assert_eq!(other.outcome[0], format!("question {id} is open"));
    assert!(
        other.outcome[1].starts_with("citation #1 is stale"),
        "{:?}",
        other.outcome
    );
    // Re-citing the stale passage appends a fresh citation rather than
    // confirming the stale one; the stale one still blocks until voided.
    let recited = cite(&root, &id, LOWER_TF, &quotes(&[LOOKAHEAD_ON]), None).expect("re-cite");
    assert_eq!((recited.number, recited.existing), (3, false));
    assert_eq!(resolved(&root, &id), Resolved::Measured(Status::Open));

    void(
        &root,
        &id,
        VoidTarget::Citation(1),
        "section reworded; re-cited",
    )
    .expect("void stale");
    assert_eq!(resolved(&root, &id), Resolved::Documented);
    refused(
        void(&root, &id, VoidTarget::Citation(4), "r"),
        "has no citation #4",
    );

    // A section the manual dropped altogether.
    let gone = add(&root, &question(Kind::Runtime, &["request.security"]))
        .expect("add")
        .id;
    cite(&root, &gone, LOWER_TF, &quotes(&[LOOKAHEAD_ON]), None).expect("cite");
    let path = record(&root, &gone);
    let text = std::fs::read_to_string(&path).expect("read");
    std::fs::write(
        &path,
        text.replace(
            "faq/other-data-and-timeframes#lower-timeframes",
            "faq/removed-page#removed-anchor",
        ),
    )
    .expect("write");
    assert_eq!(resolved(&root, &gone), Resolved::Measured(Status::Open));
    assert_eq!(
        reload(&root, &gone).citations[0].not_counting().as_deref(),
        Some("stale, does not count: the section is no longer in the manual")
    );
}

/// A pine-data refresh that drops or renames a name must not brick the
/// records: stored identifiers are held to shape rules only, and pine-data
/// membership is checked when `add` creates a question.
#[test]
fn stored_identifiers_survive_a_pine_data_refresh() {
    let root = scratch("identifier-refresh");
    let keyed = NewQuestion {
        key: Some("k".into()),
        ..question(Kind::Compile, &["export"])
    };
    let id = add(&root, &keyed).expect("add").id;
    // Simulate the refresh: the stored name no longer resolves.
    let path = record(&root, &id);
    let text = std::fs::read_to_string(&path).expect("read");
    std::fs::write(&path, text.replace("\"export\"", "\"export_gone\"")).expect("write");
    assert!(!identifier_known("export_gone"));

    load(&root).expect("an unknown stored identifier loads");
    observe(
        &root,
        &id,
        &compile(Source::Editor, "2026-09-27", Outcome::Accepted),
    )
    .expect("writes to the question still work");
    let rerun = NewQuestion {
        key: Some("k".into()),
        ..question(Kind::Compile, &["export_gone"])
    };
    let again = add(&root, &rerun).expect("a keyed rerun stays a no-op");
    assert!(again.existing);
    // A new question may not introduce an unknown name.
    refused(
        add(&root, &question(Kind::Compile, &["export_gone"])),
        "is not in pine-data",
    );

    // Shape rules still fail the load.
    for bad in ["f(", "f(a(b))", " export", ""] {
        std::fs::write(&path, text.replace("\"export\"", &format!("{bad:?}"))).expect("write");
        assert!(load(&root).is_err(), "`{bad}` must fail the load");
    }
}

#[test]
fn list_filters() {
    let root = scratch("list");
    let a = add(&root, &question(Kind::Compile, &["export"]))
        .expect("add")
        .id;
    let b = add(&root, &question(Kind::Runtime, &[])).expect("add").id;
    let rejected = NewObservation {
        errors: vec![parse_diag("CE10099|m").expect("d")],
        ..compile(Source::Editor, "2026-09-27", Outcome::Rejected)
    };
    observe(&root, &a, &rejected).expect("observe");
    let store = load(&root).expect("load");
    let ids = |f: ListFilter<'_>| -> Vec<String> {
        store.list(&f).into_iter().map(|q| q.id.clone()).collect()
    };
    assert_eq!(
        ids(ListFilter {
            identifier: Some("EXPORT"),
            ..Default::default()
        }),
        vec![a.clone()]
    );
    assert_eq!(
        ids(ListFilter {
            code: Some("ce10099"),
            ..Default::default()
        }),
        vec![a.clone()]
    );
    assert_eq!(
        ids(ListFilter {
            status: Some(Status::Open),
            ..Default::default()
        }),
        vec![b.clone()]
    );
    assert_eq!(
        ids(ListFilter {
            kind: Some(Kind::Compile),
            ..Default::default()
        }),
        vec![a]
    );
}

fn disposition(root: &Path, id: &str) -> String {
    let store = load(root).expect("load");
    let q = store.get(id).expect("question").clone();
    store.disposition(&q).to_string()
}

/// The piners 0ac37cd3 shape: every tested model refuted, none selected.
/// Status stays settled (the runs agree); the authored negative mark is
/// what tells a reader the answer identifies nothing.
#[test]
fn negative_marks_the_answer_without_changing_status() {
    let root = scratch("negative");
    let refuting = NewObservation {
        candidates: vec![
            parse_candidate("r1|running sum").expect("r1"),
            parse_candidate("f1|fresh sum").expect("f1"),
        ],
        refuted: vec!["r1".into(), "f1".into()],
        ..runtime("2026-10-05")
    };
    let id = add(
        &root,
        &NewQuestion {
            key: Some("sma".into()),
            ..question(Kind::Runtime, &["ta.sma"])
        },
    )
    .expect("add")
    .id;
    observe(&root, &id, &refuting).expect("observe");
    let q = reload(&root, &id);
    assert_eq!(q.own_status(), Status::Settled);
    assert!(q.every_candidate_refuted());
    assert_eq!(disposition(&root, &id), "settled");

    amend(
        &root,
        &id,
        AmendTarget::Negative,
        Some("true"),
        "identifies nothing",
    )
    .expect("mark negative");
    assert_eq!(disposition(&root, &id), "settled, negative");
    refused(
        amend(&root, &id, AmendTarget::Negative, Some("true"), "again"),
        "already marked negative",
    );
    let store = load(&root).expect("load");
    let negative = ListFilter {
        negative: true,
        ..Default::default()
    };
    assert_eq!(ids_of(&store, &negative), vec![id.clone()]);

    // The keyed add rerun that landed it unmarked still matches, with a
    // superseded-value warning; one emitting the mark matches silently.
    let rerun = NewQuestion {
        key: Some("sma".into()),
        ..question(Kind::Runtime, &["ta.sma"])
    };
    let again = add(&root, &rerun).expect("rerun");
    assert!(again.existing);
    assert_eq!(again.warnings.len(), 1, "{:?}", again.warnings);
    let marked = add(
        &root,
        &NewQuestion {
            negative: true,
            ..rerun
        },
    )
    .expect("marked rerun");
    assert!(marked.warnings.is_empty(), "{:?}", marked.warnings);

    // An answer and its classification change in one write.
    amend_all(
        &root,
        &id,
        &[
            (AmendTarget::Answer, Some("r1, after all.")),
            (AmendTarget::Negative, None),
        ],
        "a later fit",
    )
    .expect("amend both");
    let q = reload(&root, &id);
    assert!(!q.negative);
    let fields: Vec<AmendField> = q.amendments.iter().map(|a| a.field).collect();
    assert_eq!(
        fields,
        vec![
            AmendField::Negative,
            AmendField::Answer,
            AmendField::Negative
        ]
    );

    // A negative history value other than "true" is refused.
    let path = record(&root, &id);
    let text = std::fs::read_to_string(&path).expect("read");
    std::fs::write(&path, text.replacen("now = \"true\"", "now = \"yes\"", 1)).expect("write");
    let err = load(&root).expect_err("tampered").to_string();
    assert!(err.contains("must be \"true\" or absent"), "{err}");
}

/// A retired question's classification and coverage are history: no
/// `negative` qualifier, no coverage summary over withdrawn comparisons.
#[test]
fn retired_questions_carry_no_negative_qualifier_or_coverage() {
    let root = scratch("negative-retired");
    let id = add(
        &root,
        &NewQuestion {
            negative: true,
            ..question(Kind::Runtime, &[])
        },
    )
    .expect("add")
    .id;
    let refuting = NewObservation {
        candidates: vec![parse_candidate("A").expect("A")],
        refuted: vec!["A".into()],
        ..runtime("2026-10-05")
    };
    observe(&root, &id, &refuting).expect("observe");
    assert_eq!(disposition(&root, &id), "settled, negative");
    retire(&root, &id, "mis-filed", &[]).expect("retire");
    assert_eq!(disposition(&root, &id), "retired");
    assert!(!reload(&root, &id).every_candidate_refuted());
    let store = load(&root).expect("load");
    let negative = ListFilter {
        negative: true,
        ..Default::default()
    };
    assert!(ids_of(&store, &negative).is_empty());
}

#[test]
fn partial_refutation_is_not_full_coverage() {
    let root = scratch("partial-refutation");
    let id = add(&root, &question(Kind::Runtime, &[])).expect("add").id;
    let partial = NewObservation {
        candidates: vec![
            parse_candidate("A").expect("A"),
            parse_candidate("B").expect("B"),
        ],
        refuted: vec!["A".into()],
        ..runtime("2026-10-05")
    };
    observe(&root, &id, &partial).expect("observe");
    let q = reload(&root, &id);
    assert_eq!(q.own_status(), Status::Settled);
    assert!(!q.every_candidate_refuted());
}

/// The piners 0f412442 shape: every run inconclusive, the investigation
/// stopped. The question stays open; closure is a workflow fact.
#[test]
fn closing_an_open_question_and_ending_the_closure() {
    let root = scratch("closure");
    let id = add(&root, &question(Kind::Runtime, &[])).expect("add").id;
    let misfire = NewObservation {
        inconclusive: Some("the terminal cannot be isolated".into()),
        ..runtime("2026-10-02")
    };
    observe(&root, &id, &misfire).expect("inconclusive");
    refused(reopen(&root, &id, "r"), "is not closed");
    close(&root, &id, "no basis reproduces every cycle").expect("close");
    assert_eq!(disposition(&root, &id), "open, closed");
    refused(close(&root, &id, "again"), "already closed");

    let store = load(&root).expect("load");
    let open = ListFilter {
        status: Some(Status::Open),
        ..Default::default()
    };
    let closed = ListFilter {
        closed: true,
        ..Default::default()
    };
    assert!(ids_of(&store, &open).is_empty(), "closed is not owed work");
    assert_eq!(ids_of(&store, &closed), vec![id.clone()]);

    // A dry run predicts the automatic reopen; the write records it.
    let dry = observe_dry_run(&root, &id, &runtime("2026-10-06")).expect("dry");
    assert_eq!(dry.was.to_string(), "open, closed");
    assert_eq!(dry.now.to_string(), "settled");
    observe(&root, &id, &runtime("2026-10-06")).expect("deciding run");
    let q = reload(&root, &id);
    assert!(!q.is_closed());
    let last = q.closures.last().expect("reopen");
    assert!(last.automatic);
    assert_eq!(
        last.reason,
        "the question is now settled after adding observation #2"
    );
    // Voiding what decided it leaves it open, owed work again: the closure
    // does not silently resume.
    void(&root, &id, VoidTarget::Observation(2), "mis-recorded").expect("void");
    assert_eq!(disposition(&root, &id), "open");

    // Only open, measured, active questions close.
    let settled = add(&root, &question(Kind::Runtime, &[])).expect("add").id;
    observe(&root, &settled, &runtime("2026-10-06")).expect("run");
    refused(close(&root, &settled, "r"), "only an open question");
    let inferred = add(
        &root,
        &NewQuestion {
            basis: vec![id.clone()],
            ..question(Kind::Runtime, &[])
        },
    )
    .expect("add")
    .id;
    refused(close(&root, &inferred, "r"), "is inferred");

    // A retired question's closure is history: amending it appends nothing,
    // and it cannot be reopened.
    let retiring = add(&root, &question(Kind::Runtime, &[])).expect("add").id;
    close(&root, &retiring, "stopped").expect("close");
    retire(&root, &retiring, "reworded", &[]).expect("retire");
    amend(&root, &retiring, AmendTarget::Answer, Some("B."), "r").expect("amend");
    assert_eq!(reload(&root, &retiring).closures.len(), 1);
    refused(reopen(&root, &retiring, "r"), "retired");
    refused(close(&root, &settled, "r"), "only an open question");

    // A hand-edited history that does not alternate fails the load.
    let path = record(&root, &id);
    let text = std::fs::read_to_string(&path).expect("read");
    std::fs::write(
        &path,
        text.replace("action = \"reopen\"", "action = \"close\""),
    )
    .expect("w");
    let err = load(&root).expect_err("tampered").to_string();
    assert!(err.contains("expected a reopen"), "{err}");
}

/// A run that cannot separate A from B marks both consistent: it stays open,
/// and a later run that separates them settles the question without a
/// conflict (marking both selected made exactly this a conflict).
#[test]
fn consistent_candidates_never_decide_or_conflict() {
    let root = scratch("consistent");
    let id = add(&root, &question(Kind::Runtime, &["strategy.entry"]))
        .expect("add")
        .id;
    let candidates = vec![
        parse_candidate("A|left to right").expect("A"),
        parse_candidate("B|skips na").expect("B"),
    ];
    let fits_both = observe(
        &root,
        &id,
        &NewObservation {
            candidates: candidates.clone(),
            consistent: vec!["A".into(), "B".into()],
            ..runtime("2026-09-20")
        },
    )
    .expect("non-separating run");
    assert!(fits_both.warnings.is_empty(), "{:?}", fits_both.warnings);
    let q = reload(&root, &id);
    assert_eq!(q.own_status(), Status::Open);
    assert_eq!(
        q.observations[0].verdict_summary(Kind::Runtime),
        "runs clean; consistent with A, B"
    );
    let separates = observe(
        &root,
        &id,
        &NewObservation {
            candidates: candidates.clone(),
            selected: vec!["A".into()],
            refuted: vec!["B".into()],
            ..runtime("2026-09-21")
        },
    )
    .expect("separating run");
    assert!(separates.warnings.is_empty(), "{:?}", separates.warnings);
    assert_eq!(reload(&root, &id).own_status(), Status::Settled);
    // Differing halts still contradict: a consistent run counts.
    let halts = observe(
        &root,
        &id,
        &NewObservation {
            candidates: candidates.clone(),
            consistent: vec!["A".into()],
            errors: vec![parse_diag("RE10044").expect("diag")],
            ..runtime("2026-09-22")
        },
    )
    .expect("halting run");
    assert_eq!(reload(&root, &id).own_status(), Status::Conflict);
    assert!(
        halts
            .warnings
            .iter()
            .any(|w| w.contains("error codes differ")),
        "{:?}",
        halts.warnings
    );
    refused(
        observe(
            &root,
            &id,
            &NewObservation {
                candidates,
                selected: vec!["A".into()],
                consistent: vec!["A".into()],
                ..runtime("2026-09-23")
            },
        ),
        "cannot be both selected and consistent",
    );
}

/// Clearing `inconclusive` makes a run count again, so it gets the same
/// contradiction warning as a new observation, naming the run it revived and
/// the codes that differ.
#[test]
fn clearing_inconclusive_warns_of_the_contradiction_it_creates() {
    let root = scratch("clear-inconclusive");
    let id = add(&root, &question(Kind::Compile, &["export"]))
        .expect("add")
        .id;
    let reject = |code: &str, date: &str| NewObservation {
        errors: vec![parse_diag(code).expect("diag")],
        ..compile(Source::Editor, date, Outcome::Rejected)
    };
    observe(&root, &id, &reject("CE10099", "2026-09-20")).expect("first");
    observe(
        &root,
        &id,
        &NewObservation {
            inconclusive: Some("fixture bug".into()),
            ..reject("CE10147", "2026-09-21")
        },
    )
    .expect("second");
    observe(&root, &id, &reject("CE10099", "2026-09-22")).expect("third");
    let clear = AmendTarget::Observation {
        number: 2,
        field: AmendField::Inconclusive,
    };
    let amended = amend(&root, &id, clear, None, "the fixture was fine").expect("amend");
    assert_eq!(amended.warnings.len(), 1, "{:?}", amended.warnings);
    let w = &amended.warnings[0];
    assert!(
        w.starts_with(&format!(
            "after clearing inconclusive on #2, observations newly contradict: #1 editor \
             rejected (CE10099) and #2 editor rejected (CE10147) [error codes differ (only \
             in #1: CE10099; only in #2: CE10147)], #2 editor rejected (CE10147) and #3 \
             editor rejected (CE10099) [error codes differ (only in #2: CE10147; only in \
             #3: CE10099)]: question {id} is now conflict (was settled)."
        )),
        "{w}"
    );
    assert!(w.contains("or mark it inconclusive again"), "{w}");
    // A prose fix of an already counting run warns of nothing.
    let note = AmendTarget::Observation {
        number: 2,
        field: AmendField::Note,
    };
    let amended = amend(&root, &id, note, Some("n"), "r").expect("note");
    assert!(amended.warnings.is_empty(), "{:?}", amended.warnings);
}

/// The question text is amendable as a wording correction with history; a
/// keyed rerun giving the superseded text fails (it would hand a script for
/// the old claim the reworded question), and a retired question's text is
/// what was replaced, so it stays.
#[test]
fn question_text_amends_but_superseded_text_breaks_keyed_reruns() {
    let root = scratch("amend-question");
    let both = NewQuestion {
        key: Some("split".into()),
        question: "Do X and Y reject?".into(),
        ..question(Kind::Compile, &["export"])
    };
    let id = add(&root, &both).expect("add").id;
    let premise_of = add(
        &root,
        &NewQuestion {
            basis: vec![id.clone()],
            ..question(Kind::Compile, &[])
        },
    )
    .expect("inferred")
    .id;
    let amended = amend(
        &root,
        &id,
        AmendTarget::Question,
        Some("Does X reject?"),
        "Y split out",
    )
    .expect("amend");
    let support = amended.support.expect("support");
    assert!(
        support.contains(&format!("; premise of {premise_of} (")),
        "{support}"
    );
    let q = reload(&root, &id);
    assert_eq!(q.question, "Does X reject?");
    assert_eq!(q.amendments[0].field, AmendField::Question);
    refused(add(&root, &both), "was reworded from this text");
    let current = NewQuestion {
        question: "Does X reject?".into(),
        ..both.clone()
    };
    assert!(add(&root, &current).expect("rerun").existing);
    let found = load(&root)
        .expect("load")
        .search("X reject", 5)
        .expect("search");
    assert_eq!(found.first().map(|h| h.id.as_str()), Some(id.as_str()));

    let retired = add(&root, &question(Kind::Compile, &[])).expect("add").id;
    retire(&root, &retired, "mis-filed", &[]).expect("retire");
    refused(
        amend(&root, &retired, AmendTarget::Question, Some("Q2?"), "r"),
        "its text is what was replaced",
    );
    // The whole write is refused, not just the question part.
    refused(
        amend_all(
            &root,
            &retired,
            &[
                (AmendTarget::Answer, Some("B.")),
                (AmendTarget::Question, Some("Q2?")),
            ],
            "r",
        ),
        "its text is what was replaced",
    );
    assert_eq!(reload(&root, &retired).answer, "A.");
}

/// An id is what agents reach for first: a query that is exactly a loaded
/// question's id puts that question first, in one slot of the limit and
/// never twice; an unknown 8-hex token is plain words.
#[test]
fn search_by_exact_id() {
    let root = scratch("search-id");
    let a = add(&root, &question(Kind::Compile, &["export"]))
        .expect("add")
        .id;
    let mentions = add(
        &root,
        &NewQuestion {
            question: format!("Follow-up to {a}?"),
            ..question(Kind::Compile, &[])
        },
    )
    .expect("add")
    .id;
    let store = load(&root).expect("load");
    let hits = store.search(&a.to_uppercase(), 5).expect("search");
    let ids: Vec<&str> = hits.iter().map(|h| h.id.as_str()).collect();
    assert_eq!(ids, [a.as_str(), mentions.as_str()]);
    assert!(hits[0].exact && !hits[1].exact);
    let one = store.search(&a, 1).expect("search");
    assert_eq!(one.len(), 1);
    assert_eq!(one[0].id, a);
    assert!(store.search(&a, 0).expect("search").is_empty());
    assert!(store.search("0badf00d", 5).expect("search").is_empty());
}

fn rival_candidates() -> Vec<Candidate> {
    vec![
        parse_candidate("A|stamps the bar open").expect("A"),
        parse_candidate("B|stamps the bar close").expect("B"),
    ]
}

/// Which bar stamp rule TradingView uses is a choice between alternatives:
/// under an exclusive question two runs selecting different rules conflict,
/// where a non-exclusive one settles them silently.
#[test]
fn exclusive_candidates_rival_across_runs() {
    let root = scratch("exclusive-rival");
    let select = |id: &str, name: &str, date: &str| {
        observe(
            &root,
            id,
            &NewObservation {
                candidates: rival_candidates(),
                selected: vec![name.into()],
                ..runtime(date)
            },
        )
        .expect("observe")
    };
    let open = add(&root, &question(Kind::Runtime, &["strategy.entry"]))
        .expect("add")
        .id;
    select(&open, "A", "2026-09-20");
    select(&open, "B", "2026-09-21");
    assert_eq!(reload(&root, &open).own_status(), Status::Settled);

    let exclusive = add(
        &root,
        &NewQuestion {
            exclusive: true,
            ..question(Kind::Runtime, &["strategy.entry"])
        },
    )
    .expect("add")
    .id;
    select(&exclusive, "A", "2026-09-20");
    let second = select(&exclusive, "B", "2026-09-21");
    assert_eq!(reload(&root, &exclusive).own_status(), Status::Conflict);
    assert!(
        second.warnings.iter().any(|w| w.contains(
            "#1 chart runs clean; selects A [#2 selects B where #1 selects A (exclusive)]"
        )),
        "{:?}",
        second.warnings
    );
    // A name the other run never declared still rivals it.
    let undeclared = add(
        &root,
        &NewQuestion {
            exclusive: true,
            ..question(Kind::Runtime, &["strategy.entry"])
        },
    )
    .expect("add")
    .id;
    select(&undeclared, "A", "2026-09-20");
    observe(
        &root,
        &undeclared,
        &NewObservation {
            candidates: vec![
                parse_candidate("A|stamps the bar open").expect("A"),
                parse_candidate("C|stamps the tick").expect("C"),
            ],
            selected: vec!["C".into()],
            ..runtime("2026-09-21")
        },
    )
    .expect("observe C");
    assert_eq!(reload(&root, &undeclared).own_status(), Status::Conflict);
    // One counting run may establish only one alternative.
    refused(
        observe(
            &root,
            &exclusive,
            &NewObservation {
                candidates: rival_candidates(),
                selected: vec!["A".into(), "B".into()],
                ..runtime("2026-09-22")
            },
        ),
        "a counting run may select at most one",
    );
    refused(
        add(
            &root,
            &NewQuestion {
                exclusive: true,
                ..question(Kind::Compile, &["export"])
            },
        ),
        "`exclusive` on a compile question",
    );
}

/// Opting an existing question in: retained void or inconclusive history
/// never blocks it, a run selecting two then cannot be revived, a keyed
/// rerun of that run stays a no-op, and a pair already conflicting gains
/// the rival axis in the warning.
#[test]
fn amend_exclusive_opts_a_question_in() {
    let root = scratch("exclusive-amend");
    let id = add(&root, &question(Kind::Runtime, &["strategy.entry"]))
        .expect("add")
        .id;
    let both = NewObservation {
        key: Some("both".into()),
        candidates: rival_candidates(),
        selected: vec!["A".into(), "B".into()],
        ..runtime("2026-09-20")
    };
    observe(&root, &id, &both).expect("both selected");
    refused(
        amend(
            &root,
            &id,
            AmendTarget::Exclusive,
            Some("true"),
            "alternatives",
        ),
        "a counting run may select at most one",
    );
    let inconclusive = AmendTarget::Observation {
        number: 1,
        field: AmendField::Inconclusive,
    };
    amend(&root, &id, inconclusive, Some("did not separate"), "r").expect("inconclusive");
    amend(
        &root,
        &id,
        AmendTarget::Exclusive,
        Some("true"),
        "alternatives",
    )
    .expect("exclusive with the run inconclusive");
    assert!(
        observe(&root, &id, &both).expect("keyed rerun").existing,
        "the rerun of the original payload is a no-op"
    );
    refused(
        amend(&root, &id, inconclusive, None, "r"),
        "a counting run may select at most one",
    );

    // #1 selects A and refutes B, #2 selects B: already a conflict; marking
    // the question exclusive adds the rival axis, reported on its own.
    let conflicting = add(&root, &question(Kind::Runtime, &["strategy.entry"]))
        .expect("add")
        .id;
    observe(
        &root,
        &conflicting,
        &NewObservation {
            candidates: rival_candidates(),
            selected: vec!["A".into()],
            refuted: vec!["B".into()],
            ..runtime("2026-09-20")
        },
    )
    .expect("first");
    observe(
        &root,
        &conflicting,
        &NewObservation {
            candidates: rival_candidates(),
            selected: vec!["B".into()],
            ..runtime("2026-09-21")
        },
    )
    .expect("second");
    let amended = amend(
        &root,
        &conflicting,
        AmendTarget::Exclusive,
        Some("true"),
        "alternatives",
    )
    .expect("exclusive");
    let w = amended.warnings.join("\n");
    assert!(
        w.starts_with(&format!(
            "after marking the question exclusive, observations newly contradict: #1 chart \
             runs clean; selects A; refutes B and #2 chart runs clean; selects B [#1 selects \
             A where #2 selects B (exclusive)]: question {conflicting} stays conflict."
        )),
        "{w}"
    );
    assert!(w.contains("`amend --clear exclusive`"), "{w}");
}

/// Exclusivity changes how a capture script's future runs are compared, so
/// a keyed `add` rerun matches only its current value: after absent, true,
/// absent, the rerun without it succeeds and the one with it fails.
#[test]
fn keyed_add_matches_only_the_current_exclusivity() {
    let root = scratch("exclusive-key");
    let plain = NewQuestion {
        key: Some("stamp".into()),
        ..question(Kind::Runtime, &["strategy.entry"])
    };
    let id = add(&root, &plain).expect("add").id;
    amend(
        &root,
        &id,
        AmendTarget::Exclusive,
        Some("true"),
        "alternatives",
    )
    .expect("on");
    refused(add(&root, &plain), "which is exclusive since an amendment");
    amend(&root, &id, AmendTarget::Exclusive, None, "not alternatives").expect("off");
    assert!(add(&root, &plain).expect("current value").existing);
    // The error cites the amendment that set the current state, not the
    // first one that held the given value.
    let marked = NewQuestion {
        exclusive: true,
        ..plain.clone()
    };
    refused(
        add(&root, &marked),
        "which is no longer exclusive since an amendment",
    );
    refused(add(&root, &marked), ": not alternatives)");
}

/// One amendment can revive runs of both tiers; an endpoint pair is marked
/// as never deciding status, per pair.
#[test]
fn amend_warning_marks_weaker_pairs() {
    let root = scratch("amend-weaker");
    let id = add(&root, &question(Kind::Compile, &["export"]))
        .expect("add")
        .id;
    observe(
        &root,
        &id,
        &compile(Source::Editor, "2026-09-20", Outcome::Accepted),
    )
    .expect("editor");
    observe(
        &root,
        &id,
        &compile(Source::Endpoint, "2026-09-20", Outcome::Accepted),
    )
    .expect("endpoint accept");
    observe(
        &root,
        &id,
        &NewObservation {
            inconclusive: Some("stale session".into()),
            errors: vec![parse_diag("CE10099").expect("diag")],
            ..compile(Source::Endpoint, "2026-09-21", Outcome::Rejected)
        },
    )
    .expect("endpoint reject");
    let clear = AmendTarget::Observation {
        number: 3,
        field: AmendField::Inconclusive,
    };
    let amended = amend(&root, &id, clear, None, "r").expect("amend");
    let w = amended.warnings.join("\n");
    assert!(
        w.contains(&format!(
            "observations newly contradict: weaker source only, #2 endpoint accepted and #3 \
             endpoint rejected (CE10099) [the outcome differs]: question {id} stays settled."
        )),
        "{w}"
    );
}

/// A verdicts directory whose `evidence/` holds a copy of the run 26 probe
/// export as `run26.json`.
fn evidence_store(test: &str) -> (PathBuf, PathBuf, Vec<u8>) {
    let root = scratch(test);
    let ev_dir = root.join("evidence");
    let bytes = std::fs::read(testdata("run26-export-outside-library.json")).expect("export");
    std::fs::write(ev_dir.join("run26.json"), &bytes).expect("copy");
    (root, ev_dir, bytes)
}

fn gzip(bytes: &[u8]) -> Vec<u8> {
    use std::io::Write as _;
    let mut enc = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::best());
    enc.write_all(bytes).expect("gzip");
    enc.finish().expect("gzip")
}

fn zstd_of(bytes: &[u8]) -> Vec<u8> {
    zstd::encode_all(bytes, 19).expect("zstd")
}

/// An editor accept under a fresh question, its evidence `paths`.
fn observed_with(root: &Path, key: &str, evidence: Vec<NewEvidence>) -> String {
    let id = add(root, &question(Kind::Compile, &["export"]))
        .expect("add")
        .id;
    let o = NewObservation {
        key: Some(key.to_string()),
        evidence,
        ..compile(Source::Editor, "2026-09-27", Outcome::Accepted)
    };
    observe(root, &id, &o).expect("observe");
    id
}

#[test]
fn compressed_evidence_keeps_its_identity_through_a_move() {
    let (root, ev_dir, bytes) = evidence_store("evidence-move");
    let json = ev_dir.join("run26.json");
    let id = observed_with(&root, "k", vec![ev(json.clone())]);
    // A second question citing the same file: a move is store-wide.
    let other = observed_with(&root, "k", vec![ev(json.clone())]);
    let digest = validate::sha256_hex(&bytes);

    // Compacting the JSON is a different document: refused.
    let compact = ev_dir.join("run26.compact.json");
    let value: serde_json::Value = serde_json::from_slice(&bytes).expect("json");
    std::fs::write(&compact, serde_json::to_vec(&value).expect("compact")).expect("write");
    refused(
        relocate(&root, &json, &compact, false, "compact"),
        "but the entry's content is",
    );

    let zst = ev_dir.join("run26.json.zst");
    std::fs::write(&zst, zstd_of(&bytes)).expect("zst");
    let moved = relocate(&root, &json, &zst, false, "compressed for LFS").expect("move");
    assert_eq!((moved.entries, moved.questions, moved.attested), (2, 2, 0));
    for q in [&id, &other] {
        let e = reload(&root, q).observations[0].evidence[0].clone();
        assert_eq!(e.path, "run26.json.zst");
        assert_eq!(e.sha256.as_deref(), Some(digest.as_str()));
        assert_eq!(e.moves.len(), 1);
        assert_eq!(e.moves[0].from, "run26.json");
    }
    // Rerunning resumes: nothing is left at the old location.
    let again = relocate(&root, &json, &zst, false, "compressed for LFS").expect("again");
    assert_eq!(again.entries, 0);

    std::fs::remove_file(&json).expect("delete the original");
    let store = load(&root).expect("a moved store loads");
    let audit = audit(&store, &[]).expect("audit");
    assert_eq!((audit.entries, audit.matching), (2, 2), "{audit:?}");

    // The landing script still names the old, now deleted, location: a
    // match by location, with a note pointing at the new one.
    let rerun = NewObservation {
        key: Some("k".into()),
        evidence: vec![ev(json.clone())],
        ..compile(Source::Editor, "2026-09-27", Outcome::Accepted)
    };
    let observed = observe(&root, &id, &rerun).expect("old location matches");
    assert!(observed.existing);
    assert!(
        observed.warnings.join("\n").contains("has moved to"),
        "{:?}",
        observed.warnings
    );
    // The new location matches silently.
    let current = NewObservation {
        evidence: vec![ev(zst.clone())],
        ..rerun.clone()
    };
    assert!(
        observe(&root, &id, &current)
            .expect("new")
            .warnings
            .is_empty()
    );

    // A file at the old location with other content is not the evidence.
    std::fs::write(&json, b"{\"other\": 1}").expect("reuse the name");
    refused(observe(&root, &id, &rerun), "not the stored");
}

/// The piners migration: one capture cited twice in an observation, once
/// directly and once through a probe's copy. The record is compressed and
/// moved, the probe's redundant copy is deleted, and its citation merges
/// into the other (no file is read to merge).
#[test]
fn a_redundant_copy_of_compressed_evidence_merges() {
    let (root, ev_dir, bytes) = evidence_store("evidence-redundant-copy");
    let capture = ev_dir.join("run26.json");
    let probe = ev_dir.join("tv_record.json");
    std::fs::write(&probe, &bytes).expect("probe copy");
    let id = observed_with(&root, "k", vec![ev(capture.clone()), ev(probe.clone())]);

    let zst = ev_dir.join("run26.json.zst");
    std::fs::write(&zst, zstd_of(&bytes)).expect("zst");
    relocate(&root, &capture, &zst, false, "compressed").expect("move");
    std::fs::remove_file(&capture).expect("delete the original");
    std::fs::remove_file(&probe).expect("drop the copy");

    // A record written while the reading was stored: the legacy key loads,
    // decides nothing, and is gone on the next rewrite.
    let file = record(&root, &id);
    let text = std::fs::read_to_string(&file).expect("read");
    let legacy = text.replace(
        "path = \"tv_record.json\"",
        "path = \"tv_record.json\"\nencoding = \"gzip\"",
    );
    assert_ne!(legacy, text);
    std::fs::write(&file, legacy).expect("legacy");

    let store = load(&root).expect("loads");
    let audit = audit(&store, &[]).expect("audit");
    assert_eq!((audit.entries, audit.matching), (2, 1), "{audit:?}");
    let row = store.get(&id).expect("question").observations[0].evidence[1].clone();
    assert_eq!(
        Inspector::new().inspect(&root, &row).fault,
        Some(Fault::Missing)
    );

    // The landing script naming both locations is still a no-op rerun: an
    // absent file matches by its location.
    let rerun = NewObservation {
        key: Some("k".into()),
        evidence: vec![ev(zst.clone()), ev(probe.clone())],
        ..compile(Source::Editor, "2026-09-27", Outcome::Accepted)
    };
    assert!(observe(&root, &id, &rerun).expect("rerun").existing);

    // Merging needs the current locations, equal digests, a reason.
    refused(merge(&root, &id, 1, &probe, &zst, " "), "needs a --reason");
    refused(merge(&root, &id, 2, &probe, &zst, "r"), "no observation #2");
    refused(
        merge(&root, &id, 1, &probe, &capture, "r"),
        "former or merged location",
    );
    assert_eq!(
        merge(&root, &id, 1, &probe, &zst, "one capture, cited twice").expect("merge"),
        Merged::Merged
    );
    assert!(
        !std::fs::read_to_string(&file)
            .expect("read")
            .contains("encoding"),
        "the legacy key is dropped on rewrite"
    );
    assert_eq!(
        merge(&root, &id, 1, &probe, &zst, "again").expect("rerun"),
        Merged::Already
    );
    let e = reload(&root, &id).observations[0].evidence.clone();
    assert_eq!(e.len(), 1);
    assert_eq!(e[0].merged[0].entry.path, "tv_record.json");
    let store = load(&root).expect("loads");
    let audit = super::audit(&store, &[]).expect("audit");
    assert_eq!((audit.entries, audit.matching), (1, 1), "{audit:?}");

    // The landing script naming both is still the one entry; naming either
    // alone covers it too; naming one location twice does not.
    assert!(observe(&root, &id, &rerun).expect("merged rerun").existing);
    let probe_only = NewObservation {
        evidence: vec![ev(probe.clone())],
        ..rerun.clone()
    };
    let observed = observe(&root, &id, &probe_only).expect("alias alone");
    assert!(
        observed.warnings.join("\n").contains("was merged into"),
        "{:?}",
        observed.warnings
    );
    let twice = NewObservation {
        evidence: vec![ev(zst.clone()), ev(zst.clone())],
        ..rerun.clone()
    };
    refused(observe(&root, &id, &twice), "given twice");

    // A move of the survivor keeps what was merged into it.
    let elsewhere = ev_dir.join("elsewhere.json");
    std::fs::write(&elsewhere, &bytes).expect("copy");
    relocate(&root, &zst, &elsewhere, false, "rename").expect("move survivor");
    let e = reload(&root, &id).observations[0].evidence[0].clone();
    assert_eq!(e.merged.len(), 1, "a move keeps the merged history");

    // Other content behind the survivor is still a change.
    std::fs::write(&elsewhere, zstd_of(b"{\"other\": 1}")).expect("other");
    let store = load(&root).expect("loads");
    let audit = super::audit(&store, &[]).expect("audit");
    assert_eq!(audit.failing.len(), 1, "{audit:?}");
    assert!(
        audit.failing[0].inspection.to_string().contains("raw "),
        "every reading tried is named: {}",
        audit.failing[0].inspection
    );
}

#[test]
fn merging_needs_one_content() {
    let (root, ev_dir, bytes) = evidence_store("evidence-merge-digests");
    let a = ev_dir.join("run26.json");
    let b = ev_dir.join("other.json");
    std::fs::write(&b, b"{\"other\": 1}").expect("other");
    let id = observed_with(&root, "k", vec![ev(a.clone()), ev(b.clone())]);
    refused(
        merge(&root, &id, 1, &b, &a, "r"),
        "only the same content merges",
    );
    // Equal digests merge even with no file checked out.
    std::fs::write(&b, &bytes).expect("same");
    let id = observed_with(&root, "k2", vec![ev(a.clone()), ev(b.clone())]);
    std::fs::remove_file(&a).expect("gone");
    std::fs::remove_file(&b).expect("gone");
    assert_eq!(
        merge(&root, &id, 1, &b, &a, "r").expect("merge"),
        Merged::Merged
    );
    // A hand-edited merged entry with another digest is caught on load.
    let file = record(&root, &id);
    let text = std::fs::read_to_string(&file).expect("read");
    let digest = validate::sha256_hex(&bytes);
    let n = text.matches(digest.as_str()).count();
    assert_eq!(n, 2, "{text}");
    let tampered = text
        .replacen(&digest, &"0".repeat(64), 2)
        .replacen(&"0".repeat(64), &digest, 1);
    std::fs::write(&file, tampered).expect("tamper");
    refused(load(&root), "has another digest");
}

#[test]
fn merged_locations_stay_reserved_and_merges_rerun_after_moves() {
    let (root, ev_dir, bytes) = evidence_store("evidence-merge-reserve");
    let [a, b, c, d] = ["a.json", "b.json", "c.json", "d.json"].map(|n| {
        let p = ev_dir.join(n);
        std::fs::write(&p, &bytes).expect("copy");
        p
    });
    let other = ev_dir.join("other.json");
    std::fs::write(&other, b"{\"other\": 1}").expect("other");
    let id = observed_with(
        &root,
        "k",
        vec![
            ev(a.clone()),
            ev(b.clone()),
            ev(c.clone()),
            ev(other.clone()),
        ],
    );
    // B into A (B after A in the list), then A into C (A before C): nested.
    assert_eq!(
        merge(&root, &id, 1, &b, &a, "r").expect("b"),
        Merged::Merged
    );
    assert_eq!(
        merge(&root, &id, 1, &a, &c, "r").expect("a"),
        Merged::Merged
    );
    let e = reload(&root, &id).observations[0].evidence.clone();
    assert_eq!(e.len(), 2);
    assert_eq!(e[0].path, "c.json");
    assert_eq!(e[0].merged[0].entry.merged[0].entry.path, "b.json");
    // Both earlier merges rerun as no-ops, though A is no longer current.
    assert_eq!(
        merge(&root, &id, 1, &b, &a, "r").expect("b again"),
        Merged::Already
    );
    assert_eq!(
        merge(&root, &id, 1, &a, &c, "r").expect("a again"),
        Merged::Already
    );
    // ... and after the survivor moves.
    relocate(&root, &c, &d, false, "rename").expect("move c");
    assert_eq!(
        merge(&root, &id, 1, &b, &a, "r").expect("after move"),
        Merged::Already
    );

    // Another entry cannot move onto a nested merged location ...
    std::fs::write(&b, b"{\"other\": 1}").expect("other content at b");
    refused(
        relocate(&root, &other, &b, false, "r"),
        "is a location of its other evidence",
    );
    // ... while the survivor may return to one of its own.
    std::fs::write(&b, &bytes).expect("same content at b");
    relocate(&root, &d, &b, false, "back").expect("own merged location");
    load(&root).expect("still valid");
    // The merge reruns as a no-op even with the survivor now at B.
    assert_eq!(
        merge(&root, &id, 1, &b, &a, "r").expect("survivor at the alias"),
        Merged::Already
    );
    // A location only a merged entry left still counts as moved from.
    let moved = relocate(&root, &a, &d, false, "r").expect("finished");
    assert_eq!(moved.entries, 0);
}

/// A raw-captured artifact that merely starts like a gzip stream, wrapped in
/// zstd and unwrapped again: its raw bytes are its identity, and the
/// verifier tries them whatever the file once was.
#[test]
fn a_raw_artifact_survives_a_round_trip_through_compression() {
    let (root, ev_dir, _) = evidence_store("evidence-raw-roundtrip");
    let odd = ev_dir.join("odd.bin");
    let raw_bytes = [0x1f, 0x8b, b'n', b'o', b't', b' ', b'g', b'z'];
    std::fs::write(&odd, raw_bytes).expect("odd");
    let id = observed_with(
        &root,
        "k",
        vec![NewEvidence {
            path: odd.clone(),
            raw: true,
        }],
    );
    let zst = ev_dir.join("odd.bin.zst");
    std::fs::write(&zst, zstd_of(&raw_bytes)).expect("zst");
    relocate(&root, &odd, &zst, false, "wrap").expect("wrap");
    std::fs::remove_file(&odd).expect("rm");
    // Unwrapped in place under the old name, and moved back.
    std::fs::write(&odd, raw_bytes).expect("unwrap");
    relocate(&root, &zst, &odd, false, "unwrap").expect("back to a former location");
    let store = load(&root).expect("loads");
    let audit = audit(&store, &[]).expect("audit");
    assert_eq!((audit.entries, audit.matching), (1, 1), "{audit:?}");
    let e = &reload(&root, &id).observations[0].evidence[0];
    assert_eq!(e.moves.len(), 2);
}

/// A compressed artifact captured raw identifies the stream's bytes; a
/// decoded LFS pointer inside it does not override that match.
#[test]
fn a_raw_match_outranks_a_decoded_pointer() {
    let (root, ev_dir, _) = evidence_store("evidence-raw-pointer");
    let pointer = format!(
        "version https://git-lfs.github.com/spec/v1\noid sha256:{}\nsize 12\n",
        "a".repeat(64)
    );
    let gz = ev_dir.join("pointer.gz");
    std::fs::write(&gz, gzip(pointer.as_bytes())).expect("gz");
    observed_with(
        &root,
        "k",
        vec![NewEvidence {
            path: gz.clone(),
            raw: true,
        }],
    );
    let store = load(&root).expect("loads");
    let audit = audit(&store, &[]).expect("audit");
    assert_eq!((audit.entries, audit.matching), (1, 1), "{audit:?}");
    // A different stream around the same pointer matches nothing: the
    // pointer is the reason.
    std::fs::write(&gz, zstd_of(pointer.as_bytes())).expect("other wrapper");
    let store = load(&root).expect("loads");
    let audit = super::audit(&store, &[]).expect("audit");
    assert_eq!(audit.failing[0].inspection.fault, Some(Fault::Pointer));
}

#[test]
fn slimmed_evidence_is_detected_without_changing_status() {
    let (root, ev_dir, _) = evidence_store("evidence-slimmed");
    let json = ev_dir.join("run26.json");
    let id = observed_with(&root, "k", vec![ev(json.clone())]);
    let before = disposition(&root, &id);
    std::fs::write(&json, b"{\"slimmed\": true}").expect("slim");
    let store = load(&root).expect("loads");
    let audit = audit(&store, &[]).expect("audit");
    assert_eq!(audit.failing.len(), 1);
    assert!(matches!(
        audit.failing[0].inspection.comparison,
        Comparison::Changed { .. }
    ));
    assert_eq!(
        disposition(&root, &id),
        before,
        "evidence never decides status"
    );

    std::fs::remove_file(&json).expect("delete");
    let store = load(&root).expect("missing evidence still loads");
    let audit = super::audit(&store, &[]).expect("audit");
    assert_eq!(audit.failing[0].inspection.fault, Some(Fault::Missing));
}

#[test]
fn legacy_entries_gain_only_attested_digests() {
    let (root, ev_dir, bytes) = evidence_store("evidence-legacy");
    let id = add(&root, &question(Kind::Compile, &["export"]))
        .expect("add")
        .id;
    observe(
        &root,
        &id,
        &compile(Source::Editor, "2026-09-27", Outcome::Accepted),
    )
    .expect("observe");
    // A record written before digests: bare path strings.
    let path = record(&root, &id);
    let text = std::fs::read_to_string(&path).expect("read");
    let gz = ev_dir.join("other.json.gz");
    std::fs::write(&gz, gzip(b"{\"other\": true}\n")).expect("gz");
    std::fs::write(
        &path,
        text.replace(
            "outcome = ",
            "evidence = [\"run26.json\", \"other.json.gz\"]\noutcome = ",
        ),
    )
    .expect("legacy");
    let store = load(&root).expect("legacy loads");
    let audit = super::audit(&store, &[]).expect("audit");
    assert_eq!((audit.unhashed.len(), audit.failing.len()), (2, 0));

    // Attesting one path leaves the other unhashed.
    let done = attest(
        &root,
        std::slice::from_ref(&gz),
        false,
        "LFS objects verified",
    )
    .expect("attest");
    assert_eq!((done.entries, done.files), (1, 1));
    let e = reload(&root, &id).observations[0].evidence.clone();
    assert_eq!(e[0].sha256, None);
    assert_eq!(
        e[1].sha256.as_deref(),
        Some(validate::sha256_hex(b"{\"other\": true}\n").as_str())
    );
    assert!(e[1].attested.is_some());
    // Rerunning skips (and re-verifies) the hashed entry.
    assert_eq!(
        attest(&root, &[gz], false, "again").expect("rerun").entries,
        0
    );

    // Moving the unhashed one compares NEW with OLD as it reads now and
    // establishes the digest, marked attested by the move.
    let zst = ev_dir.join("run26.json.zst");
    std::fs::write(&zst, zstd_of(&bytes)).expect("zst");
    refused(
        relocate(&root, &ev_dir.join("run26.json"), &zst, false, ""),
        "needs a --reason",
    );
    let moved = relocate(&root, &ev_dir.join("run26.json"), &zst, false, "compress").expect("move");
    assert_eq!(moved.attested, 1);
    let e = &reload(&root, &id).observations[0].evidence[0];
    assert_eq!(
        e.sha256.as_deref(),
        Some(validate::sha256_hex(&bytes).as_str())
    );
    assert_eq!(
        e.attested.as_ref().map(|a| a.reason.as_str()),
        Some("compress")
    );
    refused(
        attest(&root, &[ev_dir.join("nothing.json")], false, "r"),
        "no evidence entry is at",
    );
}

#[test]
fn decoding_is_complete_or_refused() {
    let (root, ev_dir, bytes) = evidence_store("evidence-decoding");
    let write = |name: &str, data: &[u8]| {
        let p = ev_dir.join(name);
        std::fs::write(&p, data).expect("write");
        p
    };
    // Concatenated gzip members decode to the concatenation.
    let mut two = gzip(&bytes[..100]);
    two.extend(gzip(&bytes[100..]));
    let members = write("members.json.gz", &two);
    let id = observed_with(&root, "k", vec![ev(members)]);
    assert_eq!(
        reload(&root, &id).observations[0].evidence[0]
            .sha256
            .as_deref(),
        Some(validate::sha256_hex(&bytes).as_str())
    );
    // Several zstd frames, led by a skippable one.
    let mut frames = vec![0x50, 0x2a, 0x4d, 0x18, 3, 0, 0, 0, 1, 2, 3];
    frames.extend(zstd_of(&bytes[..50]));
    frames.extend(zstd_of(&bytes[50..]));
    let framed = write("frames.json.zst", &frames);
    let id = observed_with(&root, "k", vec![ev(framed)]);
    assert_eq!(
        reload(&root, &id).observations[0].evidence[0]
            .sha256
            .as_deref(),
        Some(validate::sha256_hex(&bytes).as_str())
    );

    let z = zstd_of(&bytes);
    let g = gzip(&bytes);
    let mut trailing = g.clone();
    trailing.extend(b"junk");
    let pointer = "version https://git-lfs.github.com/spec/v1\noid sha256:\
        4d7a214614ab2935c943f9e0ff69d22eadbb8f32b1258daaa5e2ca24d17e2393\nsize 12345\n";
    for (name, data, needle) in [
        ("cut.zst", &z[..z.len() - 3], "does not decode as zstd"),
        ("cut.gz", &g[..g.len() - 3], "does not decode as gzip"),
        ("trailing.gz", &trailing[..], "does not decode as gzip"),
        (
            "skip-only.zst",
            &[0x50, 0x2a, 0x4d, 0x18, 0, 0, 0, 0][..],
            "decodes to nothing",
        ),
        ("pointer.json", pointer.as_bytes(), "Git LFS pointer"),
        (
            "pointer.json.gz",
            &gzip(pointer.as_bytes())[..],
            "Git LFS pointer",
        ),
    ] {
        let p = write(name, data);
        let o = NewObservation {
            evidence: vec![ev(p)],
            ..compile(Source::Editor, "2026-09-27", Outcome::Accepted)
        };
        refused(observe(&root, &id, &o), needle);
    }
    // An artifact that merely starts like gzip is recorded raw on request.
    let fake = write("starts-like-gzip.bin", &[0x1f, 0x8b, b'x', b'y']);
    let o = NewObservation {
        evidence: vec![NewEvidence {
            path: fake.clone(),
            raw: false,
        }],
        ..compile(Source::Editor, "2026-09-27", Outcome::Accepted)
    };
    refused(observe(&root, &id, &o), "does not decode as gzip");
    let raw = NewObservation {
        evidence: vec![NewEvidence {
            path: fake,
            raw: true,
        }],
        ..o
    };
    observe(&root, &id, &raw).expect("raw");
    let e = &reload(&root, &id).observations[1].evidence[0];
    assert_eq!(
        e.sha256.as_deref(),
        Some(validate::sha256_hex(&[0x1f, 0x8b, b'x', b'y']).as_str())
    );
}

#[test]
fn keyed_reruns_must_name_every_entry_once() {
    let (root, ev_dir, _) = evidence_store("evidence-rerun");
    let a = ev_dir.join("run26.json");
    let b = ev_dir.join("b.json");
    std::fs::write(&b, b"{\"b\": 1}").expect("b");
    let id = observed_with(&root, "k", vec![ev(a.clone()), ev(b.clone())]);
    let rerun = |evidence: Vec<NewEvidence>| NewObservation {
        key: Some("k".into()),
        evidence,
        ..compile(Source::Editor, "2026-09-27", Outcome::Accepted)
    };
    // The order is not part of the payload.
    observe(&root, &id, &rerun(vec![ev(b.clone()), ev(a.clone())])).expect("same set");
    refused(
        observe(&root, &id, &rerun(vec![ev(a.clone())])),
        "is not given",
    );
    refused(
        observe(
            &root,
            &id,
            &rerun(vec![ev(a.clone()), ev(a.clone()), ev(b.clone())]),
        ),
        "is given twice",
    );
    let c = ev_dir.join("c.json");
    refused(
        observe(&root, &id, &rerun(vec![ev(a), ev(b), ev(c)])),
        "is not a location of any stored evidence",
    );
}

#[test]
fn a_move_never_lands_on_another_entry() {
    let (root, ev_dir, _) = evidence_store("evidence-collide");
    let a = ev_dir.join("run26.json");
    let b = ev_dir.join("b.json");
    std::fs::copy(&a, &b).expect("identical copy");
    observed_with(&root, "k", vec![ev(a.clone()), ev(b.clone())]);
    // Same content, but `b` is the observation's other entry: merging them
    // would drop an independently recorded artifact.
    refused(
        relocate(&root, &a, &b, false, "dedupe"),
        "is a location of its other evidence",
    );
    // Nor onto a location the other entry has left: an old landing script
    // naming it would then match two entries.
    let b2 = ev_dir.join("b2.json");
    std::fs::copy(&b, &b2).expect("copy");
    relocate(&root, &b, &b2, false, "rename").expect("move b");
    refused(
        relocate(&root, &a, &b, false, "reuse"),
        "is a location of its other evidence",
    );
}

#[test]
fn interrupted_maintenance_resumes() {
    let (root, ev_dir, bytes) = evidence_store("evidence-resume");
    let json = ev_dir.join("run26.json");
    let zst = ev_dir.join("run26.json.zst");
    std::fs::write(&zst, zstd_of(&bytes)).expect("zst");
    let first = observed_with(&root, "k", vec![ev(json.clone())]);
    relocate(&root, &json, &zst, false, "compress").expect("move");
    // The state an interrupted move leaves: some references moved, one
    // still at the old location.
    let second = observed_with(&root, "k", vec![ev(json.clone())]);
    let moved = relocate(&root, &json, &zst, false, "compress").expect("resume");
    assert_eq!((moved.entries, moved.questions), (1, 1));
    for id in [&first, &second] {
        assert_eq!(
            reload(&root, id).observations[0].evidence[0].path,
            "run26.json.zst"
        );
    }

    // Attesting two paths after only one was done attests the other and
    // verifies the first.
    let (a, b) = (ev_dir.join("a.json"), ev_dir.join("b.json"));
    std::fs::write(&a, b"{\"a\": 1}").expect("a");
    std::fs::write(&b, b"{\"b\": 1}").expect("b");
    let id = add(&root, &question(Kind::Compile, &["export"]))
        .expect("add")
        .id;
    observe(
        &root,
        &id,
        &compile(Source::Editor, "2026-09-27", Outcome::Accepted),
    )
    .expect("observe");
    let path = record(&root, &id);
    let text = std::fs::read_to_string(&path).expect("read");
    std::fs::write(
        &path,
        text.replace(
            "outcome = ",
            "evidence = [\"a.json\", \"b.json\"]\noutcome = ",
        ),
    )
    .expect("legacy");
    attest(&root, std::slice::from_ref(&a), false, "r").expect("a only");
    let done = attest(&root, &[a.clone(), b], false, "r").expect("resume");
    assert_eq!(done.entries, 1);
    // A file that changed since its digest blocks attesting anything there.
    std::fs::write(&a, b"{\"a\": 2}").expect("change a");
    refused(attest(&root, &[a], false, "r"), "not its digest");
}

#[test]
fn legacy_entries_rerun_by_location_and_rewrite_as_tables() {
    let (root, ev_dir, _) = evidence_store("evidence-legacy-rerun");
    let id = add(&root, &question(Kind::Compile, &["export"]))
        .expect("add")
        .id;
    let landed = NewObservation {
        key: Some("k".into()),
        ..compile(Source::Editor, "2026-09-27", Outcome::Accepted)
    };
    observe(&root, &id, &landed).expect("observe");
    let path = record(&root, &id);
    let text = std::fs::read_to_string(&path).expect("read");
    std::fs::write(
        &path,
        text.replace("outcome = ", "evidence = [\"run26.json\"]\noutcome = "),
    )
    .expect("legacy");
    let json = ev_dir.join("run26.json");
    let rerun = NewObservation {
        evidence: vec![ev(json.clone())],
        ..landed
    };
    // Matched by location, present or not, and never stamped.
    observe(&root, &id, &rerun).expect("present");
    std::fs::rename(&json, ev_dir.join("away.json")).expect("hide");
    observe(&root, &id, &rerun).expect("missing");
    assert_eq!(reload(&root, &id).observations[0].evidence[0].sha256, None);

    // Any write rewrites the bare string as a table, still without a digest.
    amend(
        &root,
        &id,
        AmendTarget::Observation {
            number: 1,
            field: AmendField::Note,
        },
        Some("n"),
        "r",
    )
    .expect("amend");
    let text = std::fs::read_to_string(&path).expect("read");
    assert!(
        text.contains("[[observation.evidence]]\npath = \"run26.json\"\n"),
        "{text}"
    );
    assert!(!text.contains("sha256"), "{text}");
}

#[test]
fn unfetched_lfs_objects_and_typos() {
    let (root, ev_dir, _) = evidence_store("evidence-lfs-rerun");
    let json = ev_dir.join("run26.json");
    let id = observed_with(&root, "k", vec![ev(json.clone())]);
    // A fresh clone without `git lfs pull`: the pointer stands in for the
    // file, and the landing script's rerun still matches by location.
    let pointer = "version https://git-lfs.github.com/spec/v1\noid sha256:\
        4d7a214614ab2935c943f9e0ff69d22eadbb8f32b1258daaa5e2ca24d17e2393\nsize 12345\n";
    std::fs::write(&json, pointer).expect("pointer");
    let rerun = NewObservation {
        key: Some("k".into()),
        evidence: vec![ev(json.clone())],
        ..compile(Source::Editor, "2026-09-27", Outcome::Accepted)
    };
    observe(&root, &id, &rerun).expect("pointer rerun");
    let store = load(&root).expect("loads");
    assert_eq!(
        audit(&store, &[]).expect("audit").failing[0]
            .inspection
            .fault,
        Some(Fault::Pointer)
    );
    refused(
        relocate(&root, &ev_dir.join("run62.json"), &json, false, "r"),
        "no evidence entry is or was at",
    );
}

#[test]
fn bare_attest_verifies_digests_sharing_a_location() {
    let (root, ev_dir, _) = evidence_store("evidence-attest-shared");
    let json = ev_dir.join("run26.json");
    observed_with(&root, "k", vec![ev(json.clone())]);
    let id = add(&root, &question(Kind::Compile, &["export"]))
        .expect("add")
        .id;
    observe(
        &root,
        &id,
        &compile(Source::Editor, "2026-09-27", Outcome::Accepted),
    )
    .expect("observe");
    let path = record(&root, &id);
    let text = std::fs::read_to_string(&path).expect("read");
    std::fs::write(
        &path,
        text.replace("outcome = ", "evidence = [\"run26.json\"]\noutcome = "),
    )
    .expect("legacy");
    // The file changed after the hashed reference recorded it: attesting
    // the legacy one would stamp the changed bytes.
    std::fs::write(&json, b"{\"slimmed\": true}").expect("slim");
    refused(attest(&root, &[], false, "r"), "not its digest");
}

#[test]
fn moves_refuse_disagreeing_references() {
    let (root, ev_dir, bytes) = evidence_store("evidence-disagree");
    let json = ev_dir.join("run26.json");
    let zst = ev_dir.join("run26.json.zst");
    std::fs::write(&zst, zstd_of(&bytes)).expect("zst");
    observed_with(&root, "k", vec![ev(json.clone())]);
    // The file was rewritten between two captures citing it: no one NEW
    // can be both.
    std::fs::write(&json, b"{\"rewritten\": true}").expect("rewrite");
    observed_with(&root, "k", vec![ev(json.clone())]);
    refused(
        relocate(&root, &json, &zst, false, "compress"),
        "but the entry's content is",
    );

    // A legacy reference is compared with OLD as it reads now, which is
    // not what NEW holds, even though NEW matches the hashed reference.
    let (root, ev_dir, bytes) = evidence_store("evidence-mixed");
    let json = ev_dir.join("run26.json");
    let zst = ev_dir.join("run26.json.zst");
    std::fs::write(&zst, zstd_of(&bytes)).expect("zst");
    observed_with(&root, "k", vec![ev(json.clone())]);
    let id = add(&root, &question(Kind::Compile, &["export"]))
        .expect("add")
        .id;
    observe(
        &root,
        &id,
        &compile(Source::Editor, "2026-09-27", Outcome::Accepted),
    )
    .expect("observe");
    let path = record(&root, &id);
    let text = std::fs::read_to_string(&path).expect("read");
    std::fs::write(
        &path,
        text.replace("outcome = ", "evidence = [\"run26.json\"]\noutcome = "),
    )
    .expect("legacy");
    std::fs::write(&json, b"{\"slimmed\": true}").expect("slim");
    refused(
        relocate(&root, &json, &zst, false, "compress"),
        "but the entry's content is",
    );
    refused(
        relocate(&root, &json, &zst, true, "compress"),
        "but the entry's content is",
    );
}

#[test]
fn revisiting_a_location_needs_only_the_same_content() {
    let (root, ev_dir, bytes) = evidence_store("evidence-revisit");
    let json = ev_dir.join("run26.json");
    let zst = ev_dir.join("run26.json.zst");
    std::fs::write(&zst, zstd_of(&bytes)).expect("zst");
    let id = observed_with(&root, "k", vec![ev(json.clone())]);
    relocate(&root, &json, &zst, false, "compress").expect("there");
    relocate(&root, &zst, &json, false, "decompress").expect("and back");
    let e = &reload(&root, &id).observations[0].evidence[0];
    assert_eq!((e.path.as_str(), e.moves.len()), ("run26.json", 2));

    relocate(&root, &json, &zst, false, "compress").expect("there again");
    // The original name now holds gzip of the same content: no reading is
    // pinned to a location, so it is the same evidence.
    std::fs::write(&json, gzip(&bytes)).expect("gzip in place");
    relocate(&root, &zst, &json, false, "back").expect("same content");
    // Other content there is not.
    relocate(&root, &json, &zst, false, "compress").expect("once more");
    std::fs::write(&json, gzip(b"{\"other\": 1}")).expect("other");
    refused(
        relocate(&root, &zst, &json, false, "back"),
        "but the entry's content is",
    );
}

#[test]
fn attest_takes_only_the_identity_already_at_a_location() {
    let (root, ev_dir, bytes) = evidence_store("evidence-attest-conflict");
    let zst = ev_dir.join("run26.json.zst");
    std::fs::write(&zst, zstd_of(&bytes)).expect("zst");
    // A hashed entry identifying the decoded content ...
    observed_with(&root, "k", vec![ev(zst.clone())]);
    // ... and a legacy one at the same file.
    let id = add(&root, &question(Kind::Compile, &["export"]))
        .expect("add")
        .id;
    observe(
        &root,
        &id,
        &compile(Source::Editor, "2026-09-27", Outcome::Accepted),
    )
    .expect("observe");
    let path = record(&root, &id);
    let text = std::fs::read_to_string(&path).expect("read");
    std::fs::write(
        &path,
        text.replace("outcome = ", "evidence = [\"run26.json.zst\"]\noutcome = "),
    )
    .expect("legacy");
    // Raw would identify the stream's bytes: two records of one file would
    // name different evidence.
    refused(
        attest(&root, std::slice::from_ref(&zst), true, "r"),
        "but entries already there identify",
    );
    let done = attest(&root, &[zst], false, "r").expect("decoded agrees");
    assert_eq!(done.entries, 1);
    assert_eq!(
        reload(&root, &id).observations[0].evidence[0]
            .sha256
            .as_deref(),
        Some(validate::sha256_hex(&bytes).as_str())
    );
}

#[test]
fn corrupt_streams_are_refused() {
    let (root, ev_dir, bytes) = evidence_store("evidence-corrupt");
    let id = add(&root, &question(Kind::Compile, &["export"]))
        .expect("add")
        .id;
    let mut g = gzip(&bytes);
    let n = g.len();
    g[n - 6] ^= 0xff; // the CRC32 in the trailer
    let checked = {
        let mut enc = zstd::stream::write::Encoder::new(Vec::new(), 3).expect("enc");
        enc.include_checksum(true).expect("checksum");
        std::io::Write::write_all(&mut enc, &bytes).expect("write");
        enc.finish().expect("finish")
    };
    let mut bad_sum = checked.clone();
    let n = bad_sum.len();
    bad_sum[n - 1] ^= 0xff;
    let mut z_trailing = zstd_of(&bytes);
    z_trailing.extend(b"junk");
    // A skippable frame between two data frames decodes fine.
    let mut interleaved = zstd_of(&bytes[..10]);
    interleaved.extend([0x53, 0x2a, 0x4d, 0x18, 2, 0, 0, 0, 9, 9]);
    interleaved.extend(zstd_of(&bytes[10..]));
    let p = ev_dir.join("interleaved.zst");
    std::fs::write(&p, &interleaved).expect("write");
    let o = NewObservation {
        evidence: vec![ev(p)],
        ..compile(Source::Editor, "2026-09-27", Outcome::Accepted)
    };
    observe(&root, &id, &o).expect("interleaved skippable frame");
    assert_eq!(
        reload(&root, &id).observations[0].evidence[0]
            .sha256
            .as_deref(),
        Some(validate::sha256_hex(&bytes).as_str())
    );
    for (name, data, needle) in [
        ("bad-crc.gz", g, "does not decode as gzip"),
        ("bad-sum.zst", bad_sum, "does not decode as zstd"),
        ("trailing.zst", z_trailing, "does not decode as zstd"),
    ] {
        let p = ev_dir.join(name);
        std::fs::write(&p, data).expect("write");
        let o = NewObservation {
            evidence: vec![ev(p)],
            ..compile(Source::Editor, "2026-09-27", Outcome::Accepted)
        };
        refused(observe(&root, &id, &o), needle);
    }
}
