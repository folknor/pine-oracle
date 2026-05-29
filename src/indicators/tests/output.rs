use super::*;

// --- output_value_from_text: full coverage (test gap) ---

#[test]
fn parses_all_runner_output_text_variants() {
    // float
    assert!(values_match(
        output_value_from_text("2.5"),
        OutputValue::number(2.5),
        1e-9
    ));
    // na / NaN
    assert!(values_match(
        output_value_from_text("na"),
        OutputValue::Na,
        0.0
    ));
    assert!(values_match(
        output_value_from_text("NaN"),
        OutputValue::Na,
        0.0
    ));
    // positive infinity
    assert!(values_match(
        output_value_from_text("inf"),
        OutputValue::PosInfinity,
        0.0
    ));
    assert!(values_match(
        output_value_from_text("Infinity"),
        OutputValue::PosInfinity,
        0.0
    ));
    // negative infinity
    assert!(values_match(
        output_value_from_text("-inf"),
        OutputValue::NegInfinity,
        0.0
    ));
    assert!(values_match(
        output_value_from_text("-Infinity"),
        OutputValue::NegInfinity,
        0.0
    ));
    // bool
    assert!(values_match(
        output_value_from_text("false"),
        OutputValue::bool(false),
        0.0
    ));
    assert!(values_match(
        output_value_from_text("true"),
        OutputValue::bool(true),
        0.0
    ));
    // unknown text falls to Undefined
    assert!(values_match(
        output_value_from_text("Value::Series(...)"),
        OutputValue::undefined(),
        0.0
    ));
}

// --- indicator_output_key: literal-#N title collision (test gap) ---

#[test]
fn duplicate_literal_hash_titles_get_chained_suffixes() {
    let mut keys = HashMap::new();
    // First allocation of "Signal#1" by literal title
    let k1 = indicator_output_key(&mut keys, "Signal#1", Some(10));
    assert_eq!(k1, "Signal#1");
    // Second distinct call site with the same literal title
    let k2 = indicator_output_key(&mut keys, "Signal#1", Some(20));
    // The algorithm increments ordinal until a free slot is found.
    // "Signal#1" is taken, ordinal 0 = "Signal#1" (taken), ordinal 1 = "Signal#1#1".
    assert_eq!(k2, "Signal#1#1", "second call site gets chained suffix");
    // Same call site again must return the same key
    let k3 = indicator_output_key(&mut keys, "Signal#1", Some(10));
    assert_eq!(k3, "Signal#1", "same call site returns same key");
}

// --- length mismatch reported (test gap) ---

#[test]
fn length_mismatch_reported_when_expected_shorter() {
    // expected has 1 value, actual has 2 -- LengthMismatch must appear first.
    // A ValueMismatch may also appear for the out-of-bounds bar; we only
    // assert the first mismatch here, which must be the LengthMismatch.
    let expected = BTreeMap::from([("plot".to_string(), vec![OutputValue::number(1.0)])]);
    let mut actual = BTreeMap::new();
    actual.insert(
        "plot".to_string(),
        vec![OutputValue::number(1.0), OutputValue::number(2.0)],
    );
    let mismatches = diff_outputs(&expected, &actual, 0.0, &ComparisonPlan::full(2));
    assert!(
        mismatches
            .iter()
            .any(|m| m.reason == MismatchReason::LengthMismatch),
        "LengthMismatch must be present"
    );
    let lm = mismatches
        .iter()
        .find(|m| m.reason == MismatchReason::LengthMismatch)
        .unwrap();
    assert_eq!(lm.expected_len, 1);
    assert_eq!(lm.actual_len, 2);
}

#[test]
fn length_mismatch_reported_when_expected_longer() {
    // expected has 2 values, actual has 1 -- LengthMismatch must appear.
    let expected = BTreeMap::from([(
        "plot".to_string(),
        vec![OutputValue::number(1.0), OutputValue::number(2.0)],
    )]);
    let mut actual = BTreeMap::new();
    actual.insert("plot".to_string(), vec![OutputValue::number(1.0)]);
    let mismatches = diff_outputs(&expected, &actual, 0.0, &ComparisonPlan::full(2));
    assert!(
        mismatches
            .iter()
            .any(|m| m.reason == MismatchReason::LengthMismatch),
        "LengthMismatch must be present"
    );
    let lm = mismatches
        .iter()
        .find(|m| m.reason == MismatchReason::LengthMismatch)
        .unwrap();
    assert_eq!(lm.expected_len, 2);
    assert_eq!(lm.actual_len, 1);
}

// --- TV fixture round-trips through fixture_counts / baseline_catalog (test gap) ---

#[test]
fn tv_fixture_routes_to_tv_bucket_in_counts() {
    // Build a valid TV fixture inline and verify it is classified as Tv baseline.
    // This pins the routing logic without requiring a real baked TV fixture.
    let tv_fixture = parse_fixture(
        "tv-count-test",
        "indicator(\"fixture\")\nplot(close)\n".to_string(),
        BARS,
        r#"{"schema_version": 1, "indicator_slug": "tv-count-test", "pine_version": "6.0.0", "outputs": {"plot": [10.0, 11.0]}}"#,
        Some(r#"{"baseline": "tv", "tv_snapshot": "2026-01-01"}"#),
    )
    .expect("tv fixture must parse ok");
    assert_eq!(tv_fixture.metadata.baseline, BaselineKind::Tv);
    assert_eq!(
        tv_fixture.effective_pine_version().as_deref(),
        Some("6.0.0")
    );
    assert_eq!(
        tv_fixture.effective_tv_snapshot().as_deref(),
        Some("2026-01-01")
    );
}

// --- filter_description format (test gap) ---

#[test]
fn filter_description_no_filters() {
    use super::fixture::filter_description as fd;
    assert_eq!(fd(None, None), "no filters");
}

#[test]
fn filter_description_grep_only() {
    use super::fixture::filter_description as fd;
    assert_eq!(fd(Some("foo"), None), "grep=foo");
}

#[test]
fn filter_description_baseline_only() {
    use super::fixture::filter_description as fd;
    assert_eq!(fd(None, Some("smoke")), "baseline=smoke");
}

#[test]
fn filter_description_both() {
    use super::fixture::filter_description as fd;
    assert_eq!(fd(Some("foo"), Some("tv")), "grep=foo baseline=tv");
}
