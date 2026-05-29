use super::*;

// --- script_kind != Indicator rejection (test gap) ---

#[test]
fn strategy_source_is_rejected_as_non_indicator_fixture() {
    let fixture = parse_fixture(
        "not-an-indicator",
        // strategy() causes piners-runner to classify as Strategy, not Indicator
        "strategy(\"Foo\")\nplot(close)\n".to_string(),
        BARS,
        r#"{"schema_version": 1, "indicator_slug": "not-an-indicator", "outputs": {"plot": [10.0, 11.0]}}"#,
        None,
    )
    .expect("parse_fixture should not reject at the fixture stage");
    let result = run_fixture_actual(&fixture);
    // piners-runner bails because script_kind != Indicator.
    // If compilation also fails (hypothetically), the test still passes -- the
    // important invariant is that a non-indicator source is never silently
    // accepted as a fixture.
    assert!(
        result.is_err(),
        "strategy source must be rejected by the runner"
    );
    let msg = result.unwrap_err().to_string();
    assert!(
        msg.contains("is not an indicator fixture") || msg.contains("not-an-indicator"),
        "error must identify the fixture or reason: {msg}"
    );
}

// --- validate_bars: empty bars rejection (correctness finding) ---

#[test]
fn validate_bars_rejects_empty_bars() {
    let err = parse_fixture(
        "empty-bars",
        "indicator(\"fixture\")\nplot(close)\n".to_string(),
        r#"{"symbol": "TEST:SYM", "timeframe": "1D", "source": "test", "bars": []}"#,
        r#"{"schema_version": 1, "indicator_slug": "empty-bars", "outputs": {"plot": [1.0]}}"#,
        None,
    )
    .expect_err("must reject empty bars");
    assert!(
        err.to_string().contains("must contain at least one bar"),
        "unexpected error: {err}"
    );
}

// --- TV baseline validation tests (test gap findings) ---

#[test]
fn tv_baseline_requires_pine_version() {
    let err = parse_fixture(
        "tv-no-pine-version",
        "indicator(\"fixture\")\nplot(close)\n".to_string(),
        BARS,
        r#"{"schema_version": 1, "indicator_slug": "tv-no-pine-version", "tv_snapshot": "2026-01-01", "outputs": {"plot": [10.0, 11.0]}}"#,
        Some(r#"{"baseline": "tv"}"#),
    )
    .expect_err("must require pine_version for tv baseline");
    assert!(
        err.to_string().contains("pine_version"),
        "unexpected error: {err}"
    );
}

#[test]
fn tv_baseline_requires_tv_snapshot() {
    let err = parse_fixture(
        "tv-no-snapshot",
        "indicator(\"fixture\")\nplot(close)\n".to_string(),
        BARS,
        r#"{"schema_version": 1, "indicator_slug": "tv-no-snapshot", "pine_version": "6.0.0", "outputs": {"plot": [10.0, 11.0]}}"#,
        Some(r#"{"baseline": "tv"}"#),
    )
    .expect_err("must require tv_snapshot for tv baseline");
    assert!(
        err.to_string().contains("tv_snapshot"),
        "unexpected error: {err}"
    );
}

#[test]
fn tv_baseline_accepts_pine_version_from_metadata_only() {
    // pine_version in metadata, not in expect.json -- should succeed
    let result = parse_fixture(
        "tv-meta-pine-version",
        "indicator(\"fixture\")\nplot(close)\n".to_string(),
        BARS,
        r#"{"schema_version": 1, "indicator_slug": "tv-meta-pine-version", "outputs": {"plot": [10.0, 11.0]}}"#,
        Some(r#"{"baseline": "tv", "pine_version": "6.0.0", "tv_snapshot": "2026-01-01"}"#),
    );
    assert!(result.is_ok(), "expected ok, got: {:?}", result.err());
    let fixture = result.unwrap();
    assert_eq!(fixture.effective_pine_version().as_deref(), Some("6.0.0"));
    assert_eq!(
        fixture.effective_tv_snapshot().as_deref(),
        Some("2026-01-01")
    );
}

// --- parse_test_range validation tests (test gap findings) ---

#[test]
fn parse_test_range_rejects_start_after_end() {
    // parse_fixture succeeds; the test_range is validated by comparison_plan inside run_fixture
    let fixture = parse_fixture(
        "bad-range",
        "indicator(\"fixture\")\nplot(close)\n".to_string(),
        RANGE_BARS,
        r#"{
            "schema_version": 1,
            "indicator_slug": "bad-range",
            "outputs": {"plot": [10.0]},
            "test_range": {
                "start": "2025-01-03T00:00:00Z",
                "end": "2025-01-02T00:00:00Z"
            }
        }"#,
        None,
    )
    .expect("parse ok");
    let result = run_fixture(&fixture);
    assert!(
        result.is_err(),
        "expected run_fixture to fail for start > end"
    );
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("start must be before or equal to end"),
        "error must mention start-before-end constraint"
    );
}

#[test]
fn parse_test_range_rejects_malformed_rfc3339() {
    let fixture = parse_fixture(
        "bad-date",
        "indicator(\"fixture\")\nplot(close)\n".to_string(),
        RANGE_BARS,
        r#"{
            "schema_version": 1,
            "indicator_slug": "bad-date",
            "outputs": {"plot": [10.0]},
            "test_range": {
                "start": "not a date",
                "end": "2025-01-03T00:00:00Z"
            }
        }"#,
        None,
    )
    .expect("parse ok");
    let result = run_fixture(&fixture);
    assert!(result.is_err(), "expected run_fixture to fail for bad date");
    assert!(
        result.unwrap_err().to_string().contains("test_range.start"),
        "error must mention test_range.start"
    );
}

#[test]
fn parse_test_range_rejects_no_bars_selected() {
    let fixture = parse_fixture(
        "far-future-range",
        "indicator(\"fixture\")\nplot(close)\n".to_string(),
        RANGE_BARS,
        r#"{
            "schema_version": 1,
            "indicator_slug": "far-future-range",
            "outputs": {"plot": [10.0]},
            "test_range": {
                "start": "1900-01-01T00:00:00Z",
                "end": "1900-01-02T00:00:00Z"
            }
        }"#,
        None,
    )
    .expect("parse ok");
    let result = run_fixture(&fixture);
    assert!(
        result.is_err(),
        "expected run_fixture to fail when no bars match"
    );
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("test_range selects no bars"),
        "error must mention no-bars selection"
    );
}

// --- comparison_plan: output length neither full nor window (test gap) ---

#[test]
fn comparison_plan_rejects_output_length_mismatch() {
    // 3 bars, range covers 2, expected has 1 value: neither full (3) nor window (2)
    let fixture = parse_fixture(
        "wrong-len",
        "indicator(\"fixture\")\nplot(close)\n".to_string(),
        RANGE_BARS,
        r#"{
            "schema_version": 1,
            "indicator_slug": "wrong-len",
            "outputs": {"plot": [10.0]},
            "test_range": {
                "start": "2025-01-02T00:00:00Z",
                "end": "2025-01-03T00:00:00Z"
            }
        }"#,
        None,
    )
    .expect("parse ok");
    let result = run_fixture(&fixture);
    assert!(
        result.is_err(),
        "expected run_fixture to fail for wrong output length"
    );
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("output lengths must match"),
        "error must mention output lengths"
    );
}

// --- tolerance validation tests (test gap) ---

#[test]
fn rejects_negative_tolerance() {
    let err = parse_expect(
        "neg-tolerance",
        r#"{"schema_version": 1, "indicator_slug": "neg-tolerance", "tolerance": -0.001, "outputs": {"plot": [1.0]}}"#,
    )
    .expect_err("must reject negative tolerance");
    assert!(
        err.to_string().contains("finite non-negative"),
        "unexpected error: {err}"
    );
}
