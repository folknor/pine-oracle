// End-to-end verdict tests: write through `add` / `observe` into a scratch
// records directory under `target/`, read back through the strict `load`.
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

/// A fresh, empty records directory unique to one test.
fn scratch(test: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("target/verdict-tests")
        .join(test);
    if dir.exists() {
        std::fs::remove_dir_all(&dir).expect("clear scratch dir");
    }
    std::fs::create_dir_all(&dir).expect("create scratch dir");
    dir
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
    load(&[root.to_path_buf()])
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
        evidence: vec![testdata("run26-export-outside-library.json")],
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
    // Evidence is stored relative to the records directory.
    assert!(editor_obs.evidence[0].starts_with("../"));
    assert!(root.join(&editor_obs.evidence[0]).is_file());
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
        evidence: vec![testdata("run24-risk-drawdown-na.json")],
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
    let before_c = std::fs::read(root.join(format!("{c}.toml"))).expect("read");

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
    assert_eq!(
        std::fs::read(root.join(format!("{c}.toml"))).expect("read"),
        before_c
    );
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
    let before_r = std::fs::read(root.join(format!("{r}.toml"))).expect("read");
    for (o, needle) in [
        (&ce_on_runtime, "must be RE followed by five digits"),
        (&runtime_from_endpoint, "come from a chart run"),
        (&no_settings, "needs `settings`"),
        (&undeclared_candidate, "is not a declared candidate"),
        (&both_states, "cannot be both selected and refuted"),
    ] {
        refused(observe(&root, &r, o), needle);
    }
    assert_eq!(
        std::fs::read(root.join(format!("{r}.toml"))).expect("read"),
        before_r
    );

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
    let store = load(&[root.to_path_buf()]).expect("load");
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
    let store = load(std::slice::from_ref(&root)).expect("load");
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
    let store = load(std::slice::from_ref(&root)).expect("load");
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
    let store = load(std::slice::from_ref(&root)).expect("load");
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
    let store = load(std::slice::from_ref(&root)).expect("load");
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
    let path = root.join(format!("{b}.toml"));
    let text = std::fs::read_to_string(&path).expect("read");

    // The empty list every record written before the split carries is inert,
    // and the next write drops it.
    std::fs::write(&path, format!("derived_from = []\n{text}")).expect("write");
    load(std::slice::from_ref(&root)).expect("an empty legacy list claims nothing");
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
    refused(load(&[root]), "`derived_from` was split");
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
    let path = root.join(format!("{id}.toml"));
    let text = std::fs::read_to_string(&path).expect("read");
    std::fs::write(
        &path,
        text.replace("code = \"CE10271\"", "code = \"CE10271\"\nmessage = \"\""),
    )
    .expect("write");
    refused(load(&[root]), "has an empty message");
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
    let dir_evidence = NewObservation {
        evidence: vec![testdata("")],
        ..compile(Source::Editor, "2026-09-27", Outcome::Accepted)
    };
    refused(
        observe(&root, &id, &dir_evidence),
        "is not an existing file",
    );
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
    let store = load(&[root]).expect("load");
    for q in ["NASDAQ:AAPL", "strategy.exit( NASDAQ", "?: NASDAQ"] {
        let hits = store.search(q, 5).expect("search");
        assert_eq!(
            hits.first().map(|h| h.id.as_str()),
            Some(id.as_str()),
            "{q}"
        );
    }
    assert!(store.search("?:", 5).expect("search").is_empty());
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
        let path = root.join(format!("{a}.toml"));
        let text = std::fs::read_to_string(&path).expect("read");
        std::fs::write(
            &path,
            text.replace(&format!("{field} = []"), &format!("{field} = [\"{b}\"]")),
        )
        .expect("write");
        refused(load(&[root]), &format!("reaches itself through `{field}`"));
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
fn relations_resolve_within_their_own_directory() {
    let a = scratch("cross-root-a");
    let b = scratch("cross-root-b");
    let in_a = add(&a, &question(Kind::Compile, &["export"]))
        .expect("add")
        .id;
    let in_b = add(&b, &question(Kind::Compile, &["export"]))
        .expect("add")
        .id;
    // Hand-edit a cross-directory follow_up_to into b's question.
    let path = b.join(format!("{in_b}.toml"));
    let text = std::fs::read_to_string(&path).expect("read");
    std::fs::write(
        &path,
        text.replace("follow_up_to = []", &format!("follow_up_to = [\"{in_a}\"]")),
    )
    .expect("write");
    refused(load(&[a, b]), "names unknown question");
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
    refused(load(std::slice::from_ref(&root)), "hashes to");
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
    let path = root.join(format!("{id}.toml"));
    let good = std::fs::read_to_string(&path).expect("read");
    let roots = [root.clone()];

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
            good.replace("settings = ", "evidence = [\"missing.json\"]\nsettings = "),
            "evidence `missing.json` is not an existing file",
        ),
        (
            good.replace("settings = ", "evidence = [\"\"]\nsettings = "),
            "empty evidence path",
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
        refused(load(&roots), needle);
    }
    std::fs::write(&path, &good).expect("restore");
    assert!(load(&roots).is_ok());

    // Nothing in a records directory may be silently skipped.
    for stray in ["ABCD1234.TOML", "notes.txt", "12345678.toml.bak"] {
        std::fs::write(root.join(stray), "").expect("stray");
        refused(load(&roots), "not a question file");
        std::fs::remove_file(root.join(stray)).expect("remove");
    }
    std::fs::create_dir(root.join("sub")).expect("subdir");
    refused(load(&roots), "not a question file");
    std::fs::remove_dir(root.join("sub")).expect("rmdir");
    // Markdown notes and dot-files (in-flight temp files) are allowed.
    std::fs::write(root.join("README.md"), "notes").expect("readme");
    std::fs::write(root.join(".x.toml.1.tmp"), "junk").expect("tmp");
    assert!(load(&roots).is_ok());

    // The same id in two directories.
    let other = scratch("load-problems-other");
    std::fs::copy(&path, other.join(format!("{id}.toml"))).expect("copy");
    refused(load(&[root, other]), "more than one records directory");
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
    let roots = [root.clone()];
    assert!(load(&roots).is_ok());

    let fixture = root.join("fixtures").join(format!("{RUN26_SHA}.pine"));
    std::fs::write(&fixture, "//@version=6\n").expect("tamper");
    let err = load(&roots).expect_err("tampered fixture must fail the load");
    assert!(err.to_string().contains("hashes to"), "{err}");
    std::fs::copy(testdata("editor-export-outside-library.pine"), &fixture).expect("restore");

    let path = root.join(format!("{id}.toml"));
    let text = std::fs::read_to_string(&path).expect("read");
    std::fs::write(&path, format!("sources = \"typo\"\n{text}")).expect("write");
    assert!(load(&roots).is_err(), "unknown field must fail the load");
}

#[test]
fn load_requires_an_existing_named_directory() {
    assert!(load(&[]).is_err());
    let root = scratch("same-root-twice");
    add(&root, &question(Kind::Compile, &["export"])).expect("add");
    let twice = [root.clone(), root.join(".")];
    assert_eq!(
        load(&twice)
            .expect("one root named twice")
            .questions()
            .len(),
        1
    );
    let missing = Path::new(env!("CARGO_MANIFEST_DIR")).join("target/verdict-tests/missing");
    assert!(load(&[missing]).is_err());
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
    let store = load(&[root]).expect("load");
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
    assert_eq!(
        load(std::slice::from_ref(&root))
            .expect("load")
            .questions()
            .len(),
        1
    );
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
    let path = root.join(format!("{other}.toml"));
    let text = std::fs::read_to_string(&path).expect("read");
    std::fs::write(&path, format!("key = \"run26/export\"\n{text}")).expect("write");
    refused(load(&[root]), "key `run26/export` is held by questions");
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
    let store = load(&[root]).expect("load");
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
