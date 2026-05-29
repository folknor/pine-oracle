use super::*;

// --- IndicatorGeneratedExpect round-trips as ExpectFile (test gap) ---

#[test]
fn runner_expect_round_trips_as_expect_file() {
    // Run a real fixture to produce a runner_expect
    let report = run_actual("smoke-close").expect("actual run");
    let generated = &report.runner_expect;

    // Serialize to JSON, deserialize as ExpectFile
    let json = serde_json::to_string(generated).expect("serialize runner_expect");
    let reparsed = parse_expect("smoke-close", &json)
        .expect("runner_expect JSON must be valid as an ExpectFile");
    assert_eq!(
        reparsed.outputs, generated.outputs,
        "outputs must round-trip"
    );
    assert_eq!(
        reparsed.tolerance, generated.tolerance,
        "tolerance must round-trip"
    );
    assert_eq!(
        reparsed.schema_version, generated.schema_version,
        "schema_version must round-trip"
    );
    assert_eq!(
        reparsed.indicator_slug, generated.indicator_slug,
        "indicator_slug must round-trip"
    );
}
