// SPDX-License-Identifier: MPL-2.0

use std::collections::{BTreeMap, HashMap};
use std::sync::Arc;

use anyhow::{Context, Result, bail};
use piners_runner::{
    BarSeries, Engine, PineOutput, PineOutputChannel, PineOutputEventKind, RunConfig, ScriptKind,
    run_single,
};

use super::compare::{comparison_plan, diff_outputs};
use super::fixture::{BarsFile, IndicatorFixture};
use super::types::{
    DEFAULT_RUNNER_EXPECT_TOLERANCE, EXPECT_SCHEMA_VERSION, IndicatorActualReport,
    IndicatorGeneratedExpect, IndicatorReport, OutputValue,
};

pub(super) fn run_fixture(fixture: &IndicatorFixture) -> Result<IndicatorReport> {
    // Validate the comparison plan first so a malformed test_range does not
    // waste a full runner pass before the error is surfaced.
    let comparison = comparison_plan(fixture)?;
    let actual = run_fixture_actual(fixture)?;
    let expected_output_keys = fixture.expect.outputs.keys().cloned().collect::<Vec<_>>();
    let mismatches = diff_outputs(
        &fixture.expect.outputs,
        &actual.outputs,
        fixture.expect.tolerance,
        &comparison,
    );
    let mismatch_count = mismatches.len();
    // A runtime error is a first-class failure already surfaced on
    // `IndicatorReport.runtime_error`. Do not synthesise a fake mismatch
    // row -- mismatches are value-level diffs only. The `ok` flag must
    // still be false when the runner crashed even if no value-level
    // mismatches were recorded (e.g. when outputs were empty / unreachable).
    let ok = mismatch_count == 0 && actual.runtime_error.is_none();
    Ok(IndicatorReport {
        slug: fixture.slug.clone(),
        baseline: fixture.metadata.baseline,
        ok,
        bar_count: fixture.bars.bars.len(),
        compared_bar_count: comparison.compared_bar_count(),
        output_count: fixture.expect.outputs.len(),
        expected_output_keys,
        actual_output_keys: actual.output_keys,
        mismatch_count,
        tolerance: fixture.expect.tolerance,
        test_range: fixture.expect.test_range.clone(),
        pine_version: fixture.effective_pine_version(),
        tv_snapshot: fixture.effective_tv_snapshot(),
        runtime_error: actual.runtime_error,
        stub_dependencies: actual.stub_dependencies,
        mismatches,
    })
}

pub(super) fn run_fixture_actual(fixture: &IndicatorFixture) -> Result<IndicatorActualReport> {
    if fixture.bars.bars.is_empty() {
        bail!("{}/bars.json contains no bars", fixture.slug);
    }
    let engine = Engine::new();
    let program = Arc::new(
        engine
            .compile_source(&fixture.source, None)
            .with_context(|| format!("compiling {}/source.pine", fixture.slug))?,
    );
    if !matches!(program.script_kind, ScriptKind::Indicator) {
        bail!("{} is not an indicator fixture", fixture.slug);
    }
    let bars = Arc::new(bars_to_series(&fixture.bars));
    let result = run_single(RunConfig::new(program, bars));
    let outputs = actual_plot_outputs(&result.pine_outputs, fixture.bars.bars.len());
    let output_keys = outputs.keys().cloned().collect::<Vec<_>>();
    let runner_expect = IndicatorGeneratedExpect {
        schema_version: EXPECT_SCHEMA_VERSION,
        indicator_slug: fixture.slug.clone(),
        pine_version: None,
        tolerance: DEFAULT_RUNNER_EXPECT_TOLERANCE,
        outputs: outputs.clone(),
        test_range: None,
    };
    Ok(IndicatorActualReport {
        slug: fixture.slug.clone(),
        baseline: fixture.metadata.baseline,
        bar_count: fixture.bars.bars.len(),
        output_count: outputs.len(),
        output_keys,
        outputs,
        runner_expect,
        runtime_error: result.runtime_error.map(|e| e.to_string()),
        stub_dependencies: result.stub_dependencies,
    })
}

fn bars_to_series(bars: &BarsFile) -> BarSeries {
    let mut series = BarSeries::new(
        bars.bars.clone(),
        bars.symbol.as_deref().unwrap_or("fixture"),
        bars.timeframe.as_deref().unwrap_or("1D"),
        bars.source.as_deref().unwrap_or("indicator-fixture"),
    );
    if let Some(context) = &bars.context {
        series = series.with_context(context.clone());
    }
    series
}

fn actual_plot_outputs(
    outputs: &[PineOutput],
    bar_count: usize,
) -> BTreeMap<String, Vec<OutputValue>> {
    let mut keys = HashMap::new();
    let mut actual = BTreeMap::new();
    for output in outputs {
        if output.channel != PineOutputChannel::Plot || output.event != PineOutputEventKind::Emit {
            continue;
        }
        let key_base = output_title(output).unwrap_or(&output.name);
        let key = indicator_output_key(&mut keys, key_base, output.call_site);
        let values = actual
            .entry(key)
            .or_insert_with(|| vec![OutputValue::undefined(); bar_count]);
        if let Some(slot) = values.get_mut(output.bar_index) {
            *slot = output
                .args
                .first()
                .map_or_else(OutputValue::undefined, |value| {
                    output_value_from_text(value)
                });
        }
    }
    actual
}

pub(super) fn indicator_output_key(
    keys: &mut HashMap<(String, Option<usize>), String>,
    name: &str,
    call_site: Option<usize>,
) -> String {
    let identity = (name.to_string(), call_site);
    if let Some(key) = keys.get(&identity) {
        return key.clone();
    }
    let mut ordinal = 0;
    let key = loop {
        let candidate = if ordinal == 0 {
            name.to_string()
        } else {
            format!("{name}#{ordinal}")
        };
        if !keys.values().any(|key| key == &candidate) {
            break candidate;
        }
        ordinal += 1;
    };
    keys.insert(identity, key.clone());
    key
}

fn output_title(output: &PineOutput) -> Option<&str> {
    if !matches!(
        output.name.as_str(),
        "plot" | "plotarrow" | "plotchar" | "plotshape"
    ) {
        return None;
    }
    // piners-runner stores the title at args[1] for both positional and
    // named-argument calls; the named-title fixture pins that contract.
    output.args.get(1).and_then(|title| {
        let title = title.trim();
        (!title.is_empty() && title != "na").then_some(title)
    })
}

pub(super) fn output_value_from_text(text: &str) -> OutputValue {
    match text {
        "na" | "NaN" => OutputValue::Na,
        "inf" | "Infinity" => OutputValue::PosInfinity,
        "-inf" | "-Infinity" => OutputValue::NegInfinity,
        "true" => OutputValue::Bool(true),
        "false" => OutputValue::Bool(false),
        _ => text
            .parse::<f64>()
            .map_or_else(|_| OutputValue::undefined(), OutputValue::from_f64),
    }
}
