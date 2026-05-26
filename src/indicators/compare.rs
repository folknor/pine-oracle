// SPDX-License-Identifier: MPL-2.0

use std::collections::BTreeMap;

use anyhow::{Context, Result, bail};
use chrono::{DateTime, Utc};

use super::fixture::IndicatorFixture;
use super::types::{IndicatorMismatch, MismatchReason, OutputValue, TestRange};

#[derive(Debug, Clone)]
pub(super) struct ComparisonPlan {
    bar_indices: Vec<usize>,
    expected_shape: ExpectedShape,
    windowed: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ExpectedShape {
    FullSeries,
    WindowSeries,
}

pub(super) fn comparison_plan(fixture: &IndicatorFixture) -> Result<ComparisonPlan> {
    let bar_count = fixture.bars.bars.len();
    let Some(range) = &fixture.expect.test_range else {
        return Ok(ComparisonPlan::full(bar_count));
    };
    let (start, end) = parse_test_range(&fixture.slug, range)?;
    let bar_indices = fixture
        .bars
        .bars
        .iter()
        .enumerate()
        .filter_map(|(index, bar)| {
            (bar.timestamp >= start && bar.timestamp <= end).then_some(index)
        })
        .collect::<Vec<_>>();
    if bar_indices.is_empty() {
        bail!(
            "{}/expect.json test_range selects no bars from bars.json",
            fixture.slug
        );
    }
    let expected_is_full_series = fixture
        .expect
        .outputs
        .values()
        .all(|values| values.len() == bar_count);
    let expected_is_window_series = fixture
        .expect
        .outputs
        .values()
        .all(|values| values.len() == bar_indices.len());
    // When the range covers every bar, full-series and window-series are
    // observationally identical (projecting through bar_indices yields the same
    // sequence as indexing the sliced array). Prefer FullSeries so the chosen
    // shape stays stable as the range narrows.
    let expected_shape = match (expected_is_full_series, expected_is_window_series) {
        (true, _) => ExpectedShape::FullSeries,
        (false, true) => ExpectedShape::WindowSeries,
        (false, false) => {
            bail!(
                "{}/expect.json output lengths must match either bars.json length or test_range length",
                fixture.slug
            );
        }
    };
    Ok(ComparisonPlan::window(bar_indices, expected_shape))
}

fn parse_test_range(slug: &str, range: &TestRange) -> Result<(i64, i64)> {
    let start = parse_range_endpoint(slug, "start", &range.start)?;
    let end = parse_range_endpoint(slug, "end", &range.end)?;
    if start > end {
        bail!("{slug}/expect.json test_range start must be before or equal to end");
    }
    Ok((start, end))
}

fn parse_range_endpoint(slug: &str, label: &str, value: &str) -> Result<i64> {
    let dt = DateTime::parse_from_rfc3339(value)
        .with_context(|| format!("parsing {slug}/expect.json test_range.{label}"))?;
    Ok(dt.with_timezone(&Utc).timestamp())
}

pub(super) fn diff_outputs(
    expected: &BTreeMap<String, Vec<OutputValue>>,
    actual: &BTreeMap<String, Vec<OutputValue>>,
    tolerance: f64,
    comparison: &ComparisonPlan,
) -> Vec<IndicatorMismatch> {
    let mut mismatches = Vec::new();
    for (name, expected_values) in expected {
        let actual_values = actual.get(name);
        match actual_values {
            Some(actual_values) => {
                let expected_len = comparison.expected_len(expected_values);
                let actual_len = comparison.actual_len(actual_values);
                if expected_len != actual_len {
                    mismatches.push(IndicatorMismatch {
                        output: name.clone(),
                        bar_index: None,
                        reason: MismatchReason::LengthMismatch,
                        expected: None,
                        actual: None,
                        expected_len,
                        actual_len,
                    });
                }
                let len = expected_len.max(actual_len);
                for index in 0..len {
                    let expected_value = comparison
                        .expected_value(expected_values, index)
                        .unwrap_or_else(OutputValue::undefined);
                    let actual_value = comparison
                        .actual_value(actual_values, index)
                        .unwrap_or_else(OutputValue::undefined);
                    if !values_match(expected_value, actual_value, tolerance) {
                        mismatches.push(IndicatorMismatch {
                            output: name.clone(),
                            bar_index: comparison.bar_index(index),
                            reason: MismatchReason::ValueMismatch,
                            expected: Some(expected_value),
                            actual: Some(actual_value),
                            expected_len,
                            actual_len,
                        });
                    }
                }
            }
            None => mismatches.push(IndicatorMismatch {
                output: name.clone(),
                bar_index: None,
                reason: MismatchReason::MissingOutput,
                expected: None,
                actual: None,
                expected_len: comparison.expected_len(expected_values),
                actual_len: 0,
            }),
        }
    }
    for name in actual.keys() {
        if !expected.contains_key(name) {
            mismatches.push(IndicatorMismatch {
                output: name.clone(),
                bar_index: None,
                reason: MismatchReason::UnexpectedOutput,
                expected: None,
                actual: None,
                expected_len: 0,
                actual_len: actual.get(name).map_or(0, Vec::len),
            });
        }
    }
    mismatches
}

impl ComparisonPlan {
    pub(super) fn full(bar_count: usize) -> Self {
        Self {
            bar_indices: (0..bar_count).collect(),
            expected_shape: ExpectedShape::FullSeries,
            windowed: false,
        }
    }

    fn window(bar_indices: Vec<usize>, expected_shape: ExpectedShape) -> Self {
        Self {
            bar_indices,
            expected_shape,
            windowed: true,
        }
    }

    pub(super) fn compared_bar_count(&self) -> usize {
        self.bar_indices.len()
    }

    fn expected_len(&self, values: &[OutputValue]) -> usize {
        match (self.windowed, self.expected_shape) {
            (true, ExpectedShape::FullSeries) => self.accessible_window_values(values),
            _ => values.len(),
        }
    }

    fn actual_len(&self, values: &[OutputValue]) -> usize {
        if self.windowed {
            self.accessible_window_values(values)
        } else {
            values.len()
        }
    }

    fn expected_value(&self, values: &[OutputValue], index: usize) -> Option<OutputValue> {
        match (self.windowed, self.expected_shape) {
            (true, ExpectedShape::FullSeries) => self.window_value(values, index),
            _ => values.get(index).copied(),
        }
    }

    fn actual_value(&self, values: &[OutputValue], index: usize) -> Option<OutputValue> {
        if self.windowed {
            self.window_value(values, index)
        } else {
            values.get(index).copied()
        }
    }

    fn bar_index(&self, index: usize) -> Option<usize> {
        if self.windowed {
            self.bar_indices.get(index).copied()
        } else {
            Some(index)
        }
    }

    // Count the number of bar_indices that fall within `values`.
    // For actual outputs, `actual_plot_outputs` pre-pads with `OutputValue::undefined()`
    // up to `bar_count`, so `values.len() == bar_count` is an invariant for actuals
    // and this filter always returns `bar_indices.len()`. For full-series expected
    // values the same holds because the fixture was validated to have the right length.
    // The filter guards against hypothetical shorter slices, e.g. a WindowSeries
    // expected array that is shorter than bar_indices.len() -- which is caught by
    // `comparison_plan` before we get here, so this is belt-and-suspenders.
    fn accessible_window_values(&self, values: &[OutputValue]) -> usize {
        self.bar_indices
            .iter()
            .filter(|index| values.get(**index).is_some())
            .count()
    }

    fn window_value(&self, values: &[OutputValue], index: usize) -> Option<OutputValue> {
        self.bar_indices
            .get(index)
            .and_then(|bar_index| values.get(*bar_index))
            .copied()
    }
}

pub(super) fn values_match(expected: OutputValue, actual: OutputValue, tolerance: f64) -> bool {
    match (expected, actual) {
        (OutputValue::Number(e), OutputValue::Number(a)) => (e - a).abs() <= tolerance,
        (OutputValue::Bool(e), OutputValue::Bool(a)) => e == a,
        (OutputValue::Na, OutputValue::Na)
        | (OutputValue::PosInfinity, OutputValue::PosInfinity)
        | (OutputValue::NegInfinity, OutputValue::NegInfinity)
        | (OutputValue::Undefined, OutputValue::Undefined) => true,
        _ => false,
    }
}
