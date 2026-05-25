// SPDX-License-Identifier: MPL-2.0

use std::collections::BTreeSet;

use super::fixture::IndicatorFixture;
use super::types::{IndicatorActualReport, IndicatorExpectedOutput, IndicatorFixtureDetail};

pub(super) fn fixture_detail(
    fixture: &IndicatorFixture,
    actual: Option<&IndicatorActualReport>,
) -> IndicatorFixtureDetail {
    let expected_outputs = fixture
        .expect
        .outputs
        .iter()
        .map(|(key, values)| IndicatorExpectedOutput {
            key: key.clone(),
            value_count: values.len(),
            first_value: values.first().copied(),
            last_value: values.last().copied(),
        })
        .collect();
    let actual_output_keys = actual
        .map(|actual| actual.output_keys.clone())
        .unwrap_or_default();
    let expected_keys = fixture
        .expect
        .outputs
        .keys()
        .cloned()
        .collect::<BTreeSet<_>>();
    let actual_keys = actual_output_keys.iter().cloned().collect::<BTreeSet<_>>();
    let missing_expected_output_keys = actual
        .map(|_| expected_keys.difference(&actual_keys).cloned().collect())
        .unwrap_or_default();
    let unexpected_actual_output_keys = actual
        .map(|_| actual_keys.difference(&expected_keys).cloned().collect())
        .unwrap_or_default();
    IndicatorFixtureDetail {
        slug: fixture.slug.clone(),
        baseline: fixture.metadata.baseline,
        source_pine: fixture.source.clone(),
        bar_count: fixture.bars.bars.len(),
        symbol: fixture.bars.symbol.clone(),
        timeframe: fixture.bars.timeframe.clone(),
        data_source: fixture.bars.source.clone(),
        first_bar_timestamp: fixture.bars.bars.first().map(|bar| bar.timestamp),
        last_bar_timestamp: fixture.bars.bars.last().map(|bar| bar.timestamp),
        output_count: fixture.expect.outputs.len(),
        expected_outputs,
        actual_outputs_checked: actual.is_some(),
        actual_output_keys,
        missing_expected_output_keys,
        unexpected_actual_output_keys,
        tolerance: fixture.expect.tolerance,
        test_range: fixture.expect.test_range.clone(),
        pine_version: fixture.effective_pine_version(),
        tv_snapshot: fixture.effective_tv_snapshot(),
        notes: fixture.metadata.notes.clone(),
        runtime_error: actual.and_then(|actual| actual.runtime_error.clone()),
        stub_dependencies: actual
            .map(|actual| actual.stub_dependencies.clone())
            .unwrap_or_default(),
    }
}
