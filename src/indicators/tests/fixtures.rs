use super::*;

#[test]
fn strict_fixture_matches_runner_outputs() {
    let fixture = parse_fixture(
        "close-plus-one",
        "indicator(\"fixture\")\nplot(close + 1)\n".to_string(),
        BARS,
        r#"{
            "schema_version": 1,
            "indicator_slug": "close-plus-one",
            "pine_version": "6.0.0",
            "outputs": {"plot": [11.0, 12.0]}
        }"#,
        None,
    )
    .expect("fixture");
    let report = run_fixture(&fixture).expect("run");
    assert!(report.ok, "{:?}", report.mismatches);
    assert_eq!(report.output_count, 1);
    assert_eq!(report.bar_count, 2);
    assert_eq!(report.expected_output_keys, vec!["plot"]);
    assert_eq!(report.actual_output_keys, vec!["plot"]);
}

#[test]
fn strict_fixture_matches_named_plot_titles() {
    let fixture = parse_fixture(
        "named-titles",
        "indicator(\"fixture\")\nplot(close, title=\"Close Line\")\nplotshape(close > open, title=\"Up Shape\")\n".to_string(),
        BARS,
        r#"{
            "schema_version": 1,
            "indicator_slug": "named-titles",
            "outputs": {
                "Close Line": [10.0, 11.0],
                "Up Shape": [false, true]
            }
        }"#,
        None,
    )
    .expect("fixture");
    let report = run_fixture(&fixture).expect("run");
    assert!(report.ok, "{:?}", report.mismatches);
    assert_eq!(report.expected_output_keys, vec!["Close Line", "Up Shape"]);
    assert_eq!(report.actual_output_keys, vec!["Close Line", "Up Shape"]);
}

#[test]
fn output_keys_do_not_collide_with_literal_hash_titles() {
    let mut keys = HashMap::new();
    assert_eq!(
        indicator_output_key(&mut keys, "Signal#1", Some(10)),
        "Signal#1"
    );
    assert_eq!(
        indicator_output_key(&mut keys, "Signal", Some(20)),
        "Signal"
    );
    assert_eq!(
        indicator_output_key(&mut keys, "Signal", Some(30)),
        "Signal#2"
    );
}

#[test]
fn duplicate_output_keys_reuse_call_site_identity() {
    let mut keys = HashMap::new();
    assert_eq!(
        indicator_output_key(&mut keys, "Signal", Some(10)),
        "Signal"
    );
    assert_eq!(
        indicator_output_key(&mut keys, "Signal", Some(20)),
        "Signal#1"
    );
    assert_eq!(
        indicator_output_key(&mut keys, "Signal", Some(10)),
        "Signal"
    );
}

#[test]
fn baked_fixtures_validate_strictly() {
    let entries = INDICATORS
        .find("**/source.pine")
        .expect("walking baked fixtures");
    let mut count = 0;
    for entry in entries {
        let Some(file) = entry.as_file() else {
            continue;
        };
        let Some(parent) = file.path().parent() else {
            continue;
        };
        let Some(slug) = parent.to_str() else {
            continue;
        };
        if slug.is_empty() {
            continue;
        }
        let fixture = load_fixture(slug)
            .unwrap_or_else(|err| panic!("fixture {slug} failed strict validation: {err}"));
        assert_eq!(fixture.slug, slug);
        count += 1;
    }
    assert!(count >= 10, "expected baked smoke fixtures, found {count}");
}

#[test]
fn baked_smoke_fixtures_run_cleanly() {
    for slug in [
        "smoke-close",
        "smoke-close-plus-one",
        "smoke-duplicate-titles",
        "smoke-na-output",
        "smoke-plotshape-bool",
        "smoke-request-security-current",
        "smoke-titled-outputs",
        "smoke-two-plots",
        "smoke-sma-warmup",
        "smoke-test-range",
    ] {
        let report = run_strict(slug).expect("strict run");
        assert!(report.ok, "{slug}: {:?}", report.mismatches);
        assert_eq!(report.baseline, BaselineKind::Smoke);
    }
}

#[test]
fn baked_fixture_list_contains_smoke_fixtures() {
    let fixtures = list_fixtures().expect("fixtures");
    let slugs = fixtures
        .iter()
        .map(|fixture| fixture.slug.as_str())
        .collect::<Vec<_>>();
    assert!(slugs.contains(&"smoke-close"));
    assert!(slugs.contains(&"smoke-close-plus-one"));
    assert!(slugs.contains(&"smoke-duplicate-titles"));
    assert!(slugs.contains(&"smoke-na-output"));
    assert!(slugs.contains(&"smoke-plotshape-bool"));
    assert!(slugs.contains(&"smoke-request-security-current"));
    assert!(slugs.contains(&"smoke-titled-outputs"));
    assert!(slugs.contains(&"smoke-two-plots"));
    assert!(slugs.contains(&"smoke-sma-warmup"));
    assert!(slugs.contains(&"smoke-test-range"));
    for fixture in fixtures
        .iter()
        .filter(|fixture| fixture.slug.starts_with("smoke-"))
    {
        assert_eq!(fixture.baseline, BaselineKind::Smoke);
    }
    let counts = fixture_counts().expect("counts");
    assert_eq!(counts.total, fixtures.len());
    assert!(counts.smoke >= 10);
}

#[test]
fn fixture_listings_include_shape_counts_and_range() {
    let fixtures = list_fixtures().expect("fixtures");
    let close = fixtures
        .iter()
        .find(|fixture| fixture.slug == "smoke-close")
        .expect("smoke-close listing");
    assert_eq!(close.symbol.as_deref(), Some("SMOKE:FIXTURE"));
    assert_eq!(close.timeframe.as_deref(), Some("1D"));
    assert_eq!(close.bar_count, Some(4));
    assert_eq!(close.output_count, Some(1));
    assert!(close.test_range.is_none());

    let ranged = fixtures
        .iter()
        .find(|fixture| fixture.slug == "smoke-test-range")
        .expect("smoke-test-range listing");
    assert!(ranged.test_range.is_some());
}

#[test]
fn fixture_detail_includes_source_bars_and_expected_outputs() {
    let detail = load_fixture_detail_with_actual("smoke-titled-outputs").expect("detail");
    assert_eq!(detail.slug, "smoke-titled-outputs");
    assert_eq!(detail.baseline, BaselineKind::Smoke);
    assert!(detail.source_pine.contains("plot(close, \"Close Line\")"));
    assert_eq!(detail.symbol.as_deref(), Some("SMOKE:FIXTURE"));
    assert_eq!(detail.timeframe.as_deref(), Some("1D"));
    assert_eq!(detail.bar_count, 4);
    assert_eq!(detail.first_bar_timestamp, Some(1735689600));
    assert_eq!(detail.last_bar_timestamp, Some(1735948800));
    assert_eq!(detail.output_count, 2);
    assert!(detail.actual_outputs_checked);
    assert_eq!(detail.actual_output_keys, vec!["Close Line", "Up Shape"]);
    assert!(detail.missing_expected_output_keys.is_empty());
    assert!(detail.unexpected_actual_output_keys.is_empty());
    assert_eq!(detail.expected_outputs[0].key, "Close Line");
    assert_eq!(detail.expected_outputs[0].value_count, 4);
    assert_eq!(
        detail.expected_outputs[0].first_value,
        Some(OutputValue::number(9.0))
    );
    assert_eq!(
        detail.expected_outputs[0].last_value,
        Some(OutputValue::number(13.0))
    );
    assert_eq!(detail.expected_outputs[1].key, "Up Shape");
    assert_eq!(detail.expected_outputs[1].value_count, 4);
    assert_eq!(
        detail.expected_outputs[1].first_value,
        Some(OutputValue::bool(false))
    );
    assert_eq!(
        detail.expected_outputs[1].last_value,
        Some(OutputValue::bool(true))
    );
    assert!(
        detail
            .notes
            .as_deref()
            .is_some_and(|notes| notes.contains("title-based output matching"))
    );
    assert!(detail.runtime_error.is_none());
    assert!(detail.stub_dependencies.is_empty());
}

#[test]
fn fixture_detail_can_skip_runner_output_check() {
    let detail = load_fixture_detail("smoke-titled-outputs").expect("detail");
    assert_eq!(detail.slug, "smoke-titled-outputs");
    assert!(!detail.actual_outputs_checked);
    assert!(detail.actual_output_keys.is_empty());
    assert!(detail.missing_expected_output_keys.is_empty());
    assert!(detail.unexpected_actual_output_keys.is_empty());
    assert_eq!(detail.expected_outputs.len(), 2);
}

#[test]
fn actual_report_returns_runner_outputs_without_diffing() {
    let report = run_actual("smoke-titled-outputs").expect("actual report");
    assert_eq!(report.slug, "smoke-titled-outputs");
    assert_eq!(report.baseline, BaselineKind::Smoke);
    assert_eq!(report.bar_count, 4);
    assert_eq!(report.output_count, 2);
    assert_eq!(report.output_keys, vec!["Close Line", "Up Shape"]);
    assert_eq!(report.runner_expect.schema_version, EXPECT_SCHEMA_VERSION);
    assert_eq!(report.runner_expect.indicator_slug, "smoke-titled-outputs");
    assert_eq!(report.runner_expect.pine_version, None);
    assert_eq!(
        report.runner_expect.tolerance,
        DEFAULT_RUNNER_EXPECT_TOLERANCE
    );
    assert_eq!(
        report.outputs["Close Line"],
        vec![
            OutputValue::number(9.0),
            OutputValue::number(11.0),
            OutputValue::number(12.0),
            OutputValue::number(13.0),
        ]
    );
    assert_eq!(
        report.outputs["Up Shape"],
        vec![
            OutputValue::bool(false),
            OutputValue::bool(true),
            OutputValue::bool(false),
            OutputValue::bool(true),
        ]
    );
    assert_eq!(report.runner_expect.outputs, report.outputs);
    assert_eq!(report.runner_expect.test_range, None);
    assert!(report.runtime_error.is_none());
    assert!(report.stub_dependencies.is_empty());
}

#[test]
fn actual_runner_expect_does_not_inherit_fixture_metadata() {
    let fixture = parse_fixture(
        "actual-metadata",
        "indicator(\"fixture\")\nplot(close)\n".to_string(),
        RANGE_BARS,
        r#"{
            "schema_version": 1,
            "indicator_slug": "actual-metadata",
            "pine_version": "6.0.0",
            "tolerance": 0.0001,
            "outputs": {"plot": [11.0, 12.0]},
            "test_range": {
                "start": "2025-01-02T00:00:00Z",
                "end": "2025-01-03T00:00:00Z"
            }
        }"#,
        None,
    )
    .expect("fixture");
    let report = run_fixture_actual(&fixture).expect("actual report");
    assert_eq!(report.runner_expect.pine_version, None);
    assert_eq!(
        report.runner_expect.tolerance,
        DEFAULT_RUNNER_EXPECT_TOLERANCE
    );
    assert_eq!(report.runner_expect.test_range, None);
}

#[test]
fn fixture_detail_reports_output_key_drift() {
    let fixture = parse_fixture(
        "title-drift",
        "indicator(\"fixture\")\nplot(close, \"Close line\")\n".to_string(),
        BARS,
        r#"{
            "schema_version": 1,
            "indicator_slug": "title-drift",
            "outputs": {"Close Line": [10.0, 11.0]}
        }"#,
        None,
    )
    .expect("fixture");
    let actual = run_fixture_actual(&fixture).expect("actual report");
    let detail = fixture_detail(&fixture, Some(&actual));
    assert!(detail.actual_outputs_checked);
    assert_eq!(detail.actual_output_keys, vec!["Close line"]);
    assert_eq!(detail.missing_expected_output_keys, vec!["Close Line"]);
    assert_eq!(detail.unexpected_actual_output_keys, vec!["Close line"]);
}

#[test]
fn fixture_list_filters_by_grep_and_baseline() {
    let fixtures =
        list_fixtures_filtered(Some("request"), Some("SMOKE")).expect("filtered fixtures");
    assert!(
        fixtures
            .iter()
            .any(|fixture| fixture.slug == "smoke-request-security-current")
    );
    assert!(
        fixtures
            .iter()
            .all(|fixture| fixture.baseline == BaselineKind::Smoke)
    );
}

#[test]
fn indicator_baseline_catalog_counts_fixtures() {
    let counts = fixture_counts().expect("counts");
    let catalog = baseline_catalog().expect("catalog");
    let catalog_total = catalog.iter().map(|entry| entry.count).sum::<usize>();
    assert_eq!(catalog_total, counts.total);
    assert!(
        catalog
            .iter()
            .any(|entry| entry.baseline == BaselineKind::Smoke && entry.count == counts.smoke)
    );
    assert!(
        catalog
            .iter()
            .any(|entry| entry.baseline == BaselineKind::Tv && entry.count == counts.tv)
    );
}

#[test]
fn invalid_baseline_filter_errors() {
    let err = list_fixtures_filtered(None, Some("paper")).expect_err("must reject");
    assert!(err.to_string().contains("unknown indicator baseline"));
}

#[test]
fn rejects_unsupported_expect_schema_version() {
    let err = parse_expect(
        "bad-schema",
        r#"{
            "schema_version": 2,
            "indicator_slug": "bad-schema",
            "outputs": {"plot": [1.0]}
        }"#,
    )
    .expect_err("must reject unsupported schema version");
    assert!(err.to_string().contains("unsupported schema_version 2"));
    assert!(
        err.to_string()
            .contains(&format!("expected {EXPECT_SCHEMA_VERSION}"))
    );
}

#[test]
fn rejects_loose_expect_tolerance() {
    let err = parse_expect(
        "loose-tolerance",
        r#"{
            "schema_version": 1,
            "indicator_slug": "loose-tolerance",
            "tolerance": 1.0,
            "outputs": {"plot": [1.0]}
        }"#,
    )
    .expect_err("must reject loose tolerance");
    assert!(err.to_string().contains("tolerance 1"));
    assert!(err.to_string().contains("too loose"));
}

#[test]
fn rejects_empty_expected_output_key() {
    let err = parse_expect(
        "empty-key",
        r#"{
            "schema_version": 1,
            "indicator_slug": "empty-key",
            "outputs": {"": [1.0]}
        }"#,
    )
    .expect_err("must reject empty output key");
    assert!(err.to_string().contains("output keys must not be empty"));
}

#[test]
fn rejects_empty_expected_output_series() {
    let err = parse_expect(
        "empty-series",
        r#"{
            "schema_version": 1,
            "indicator_slug": "empty-series",
            "outputs": {"plot": []}
        }"#,
    )
    .expect_err("must reject empty output series");
    assert!(
        err.to_string()
            .contains("output `plot` must define at least one value")
    );
}

#[test]
fn smoke_fixture_rejects_tv_snapshot_metadata() {
    let err = parse_fixture(
        "smoke-tv-copy",
        "indicator(\"fixture\")\nplot(close)\n".to_string(),
        BARS,
        r#"{
            "schema_version": 1,
            "indicator_slug": "smoke-tv-copy",
            "tv_snapshot": "2026-05-20",
            "outputs": {"plot": [10.0, 11.0]}
        }"#,
        None,
    )
    .expect_err("must reject stale tv metadata");
    assert!(
        err.to_string()
            .contains("smoke fixtures must not define tv_snapshot")
    );
}

#[test]
fn rejects_bar_spacing_shorter_than_timeframe() {
    let hourly_bars_marked_daily = r#"{
        "symbol": "NASDAQ:SPY",
        "timeframe": "1D",
        "source": "test",
        "bars": [
            {"timestamp": 1735689600, "open": 10.0, "high": 11.0, "low": 9.0, "close": 10.0, "volume": 100.0},
            {"timestamp": 1735693200, "open": 10.0, "high": 12.0, "low": 9.0, "close": 11.0, "volume": 110.0}
        ]
    }"#;
    let err = parse_fixture(
        "bad-spacing",
        "indicator(\"fixture\")\nplot(close)\n".to_string(),
        hourly_bars_marked_daily,
        r#"{
            "schema_version": 1,
            "indicator_slug": "bad-spacing",
            "outputs": {"plot": [10.0, 11.0]}
        }"#,
        None,
    )
    .expect_err("must reject bars shorter than timeframe");
    assert!(err.to_string().contains("shorter than timeframe `1D`"));
}

#[test]
fn rejects_monthly_bar_spacing_shorter_than_timeframe() {
    let hourly_bars_marked_monthly = r#"{
        "symbol": "NASDAQ:SPY",
        "timeframe": "M",
        "source": "test",
        "bars": [
            {"timestamp": 1735689600, "open": 10.0, "high": 11.0, "low": 9.0, "close": 10.0, "volume": 100.0},
            {"timestamp": 1735693200, "open": 10.0, "high": 12.0, "low": 9.0, "close": 11.0, "volume": 110.0}
        ]
    }"#;
    let err = parse_fixture(
        "bad-month-spacing",
        "indicator(\"fixture\")\nplot(close)\n".to_string(),
        hourly_bars_marked_monthly,
        r#"{
            "schema_version": 1,
            "indicator_slug": "bad-month-spacing",
            "outputs": {"plot": [10.0, 11.0]}
        }"#,
        None,
    )
    .expect_err("must reject bars shorter than monthly timeframe");
    assert!(err.to_string().contains("shorter than timeframe `M`"));
}

#[test]
fn strict_batch_runs_matching_fixtures() {
    let report = run_strict_filtered(Some("request-security"), Some("smoke")).expect("batch run");
    assert!(report.ok, "{:?}", report.reports);
    assert_eq!(report.fixture_count, 1);
    assert_eq!(report.passed_count, 1);
    assert_eq!(report.failed_count, 0);
    assert_eq!(report.reports[0].slug, "smoke-request-security-current");
}

#[test]
fn strict_batch_rejects_empty_filter_result() {
    let err = run_strict_filtered(Some("definitely-not-a-fixture"), Some("smoke"))
        .expect_err("must reject empty batch");
    assert!(err.to_string().contains("no indicator fixtures matched"));
    assert!(err.to_string().contains("grep=definitely-not-a-fixture"));
    assert!(err.to_string().contains("baseline=smoke"));
}

#[test]
fn strict_batch_report_count_matches_fixture_count() {
    // Runs all smoke fixtures in one batch and verifies that the number of
    // IndicatorReport entries equals the number of fixtures selected, even
    // when multiple fixtures are involved.  This pins the non-aborting batch
    // behavior: a broken fixture must produce a report entry rather than
    // collapsing the entire batch into an Err.
    let fixtures = list_fixtures_filtered(None, Some("smoke")).expect("fixture list");
    let fixture_count = fixtures.len();
    assert!(fixture_count >= 10, "expected at least 10 smoke fixtures");

    let report = run_strict_filtered(None, Some("smoke")).expect("batch run");
    assert_eq!(
        report.reports.len(),
        fixture_count,
        "report count must equal fixture count: each fixture must produce exactly one entry"
    );
    assert_eq!(report.fixture_count, fixture_count);
    let counted_passed = report.reports.iter().filter(|r| r.ok).count();
    let counted_failed = report.reports.iter().filter(|r| !r.ok).count();
    assert_eq!(counted_passed, report.passed_count);
    assert_eq!(counted_failed, report.failed_count);
    assert_eq!(report.passed_count + report.failed_count, fixture_count);
}

#[test]
fn strict_fixture_reports_value_mismatch() {
    let fixture = parse_fixture(
        "close-plus-one",
        "indicator(\"fixture\")\nplot(close + 1)\n".to_string(),
        BARS,
        r#"{
            "schema_version": 1,
            "indicator_slug": "close-plus-one",
            "outputs": {"plot": [11.0, 99.0]}
        }"#,
        None,
    )
    .expect("fixture");
    let report = run_fixture(&fixture).expect("run");
    assert!(!report.ok);
    assert_eq!(report.mismatch_count, 1);
    assert_eq!(report.mismatches[0].bar_index, Some(1));
    assert_eq!(report.mismatches[0].reason, MismatchReason::ValueMismatch);
}

// Fix 1 regression pin: a fixture whose piners-runner crashes must still
// report ok=false via runtime_error, not via a synthetic mismatch row.
// Real "output never emitted" mismatches can still appear (MissingOutput),
// but no synthetic "<runtime>" row with reason ValueMismatch may exist.
#[test]
fn runtime_error_sets_ok_false_without_synthetic_runtime_mismatch() {
    let fixture = parse_fixture(
        "runtime-error",
        // runtime.error() causes a runtime crash; plot() output still emits
        // so diff_outputs has something to compare against.
        "indicator(\"fixture\")\nruntime.error(\"intentional crash\")\nplot(close)\n".to_string(),
        BARS,
        r#"{
            "schema_version": 1,
            "indicator_slug": "runtime-error",
            "outputs": {"plot": [10.0, 11.0]}
        }"#,
        None,
    )
    .expect("fixture");
    let report = run_fixture(&fixture).expect("run_fixture must not error at the Rust level");
    assert!(!report.ok, "runtime error must make ok=false");
    assert!(
        report.runtime_error.is_some(),
        "runtime_error must be populated: {report:?}"
    );
    // The synthetic `<runtime>` mismatch (reason=ValueMismatch) introduced by
    // the old code is gone: any mismatch present must be a real value diff
    // or output-missing, not a re-coding of the runtime error.
    assert!(
        !report
            .mismatches
            .iter()
            .any(|m| m.output == "<runtime>" && m.reason == MismatchReason::ValueMismatch),
        "synthetic <runtime> mismatch must not be present: {report:?}"
    );
}

#[test]
fn test_range_compares_sliced_expected_values() {
    let fixture = parse_fixture(
        "range-close",
        "indicator(\"fixture\")\nplot(close)\n".to_string(),
        RANGE_BARS,
        r#"{
            "schema_version": 1,
            "indicator_slug": "range-close",
            "outputs": {"plot": [11.0, 12.0]},
            "test_range": {
                "start": "2025-01-02T00:00:00Z",
                "end": "2025-01-03T00:00:00Z"
            }
        }"#,
        None,
    )
    .expect("fixture");
    let report = run_fixture(&fixture).expect("run");
    assert!(report.ok, "{:?}", report.mismatches);
    assert_eq!(report.bar_count, 3);
    assert_eq!(report.compared_bar_count, 2);
}

#[test]
fn test_range_accepts_full_series_expected() {
    let fixture = parse_fixture(
        "range-close-full",
        "indicator(\"fixture\")\nplot(close)\n".to_string(),
        RANGE_BARS,
        r#"{
            "schema_version": 1,
            "indicator_slug": "range-close-full",
            "outputs": {"plot": ["__NaN__", 11.0, "__NaN__"]},
            "test_range": {
                "start": "2025-01-02T00:00:00Z",
                "end": "2025-01-02T00:00:00Z"
            }
        }"#,
        None,
    )
    .expect("fixture");
    let report = run_fixture(&fixture).expect("run");
    assert!(report.ok, "{:?}", report.mismatches);
    assert_eq!(report.bar_count, 3);
    assert_eq!(report.compared_bar_count, 1);
}

#[test]
fn test_range_reports_original_bar_index() {
    let fixture = parse_fixture(
        "range-close",
        "indicator(\"fixture\")\nplot(close)\n".to_string(),
        RANGE_BARS,
        r#"{
            "schema_version": 1,
            "indicator_slug": "range-close",
            "outputs": {"plot": [99.0]},
            "test_range": {
                "start": "2025-01-02T00:00:00Z",
                "end": "2025-01-02T00:00:00Z"
            }
        }"#,
        None,
    )
    .expect("fixture");
    let report = run_fixture(&fixture).expect("run");
    assert!(!report.ok);
    assert_eq!(report.mismatch_count, 1);
    assert_eq!(report.mismatches[0].bar_index, Some(1));
}

#[test]
fn missing_output_is_reported_once() {
    let mut actual = BTreeMap::new();
    actual.insert("plot".to_string(), vec![OutputValue::number(1.0)]);
    let expected = BTreeMap::from([("plot#1".to_string(), vec![OutputValue::number(2.0)])]);
    let mismatches = diff_outputs(&expected, &actual, 0.0, &ComparisonPlan::full(1));
    assert_eq!(mismatches.len(), 2);
    assert_eq!(mismatches[0].reason, MismatchReason::MissingOutput);
    assert_eq!(mismatches[1].reason, MismatchReason::UnexpectedOutput);
}

#[test]
fn parses_runner_output_text_values() {
    assert!(values_match(
        output_value_from_text("12.5"),
        OutputValue::number(12.5),
        0.0
    ));
    assert!(values_match(
        output_value_from_text("na"),
        OutputValue::Na,
        0.0
    ));
    assert!(values_match(
        output_value_from_text("true"),
        OutputValue::bool(true),
        0.0
    ));
}

#[test]
fn special_tokens_match_non_finite_values() {
    assert!(values_match(
        OutputValue::Na,
        OutputValue::from_f64(f64::NAN),
        0.0
    ));
    assert!(values_match(
        OutputValue::PosInfinity,
        OutputValue::from_f64(f64::INFINITY),
        0.0
    ));
    assert!(values_match(
        OutputValue::NegInfinity,
        OutputValue::from_f64(f64::NEG_INFINITY),
        0.0
    ));
}

#[test]
fn rejects_traversal_slug() {
    let err = sanitise_slug("../x").expect_err("must reject");
    assert!(err.to_string().contains("invalid indicator slug"));
}

#[test]
fn output_value_serializes_tokens() {
    assert_eq!(
        serde_json::to_string(&OutputValue::undefined()).expect("json"),
        r#""__undefined__""#
    );
}
