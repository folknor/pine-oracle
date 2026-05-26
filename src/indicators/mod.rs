// SPDX-License-Identifier: MPL-2.0
//
// Per-bar indicator parity fixtures.
//
// The `indicators/` tree is embedded into the binary. Each fixture provides
// Pine source, OHLCV bars, and expected per-output values captured from a
// baseline. The current command substrate runs the source through piners'
// runner and diffs runner outputs against the baked expectation.

mod compare;
mod detail;
mod fixture;
mod runner;
mod types;

#[cfg(test)]
mod tests;

use anyhow::{Result, bail};

pub use fixture::{
    baseline_catalog, fixture_counts, is_baseline_catalog_request, list_fixtures,
    list_fixtures_filtered,
};
pub use types::{
    BaselineKind, DEFAULT_RUNNER_EXPECT_TOLERANCE, EXPECT_SCHEMA_VERSION, IndicatorActualReport,
    IndicatorBaselineInfo, IndicatorBatchReport, IndicatorExpectedOutput, IndicatorFixtureCounts,
    IndicatorFixtureDetail, IndicatorGeneratedExpect, IndicatorListing, IndicatorMismatch,
    IndicatorReport, MismatchReason, OutputValue, TestRange,
};

/// Run one indicator fixture through the piners runner and compare bar-by-bar
/// against the baked `expect.json` baseline. Returns `IndicatorReport.ok = false`
/// when any value mismatch or runtime error is detected.
pub fn run_strict(slug: &str) -> Result<IndicatorReport> {
    let fixture = fixture::load_fixture(slug)?;
    runner::run_fixture(&fixture)
}

/// Load fixture metadata and expected-output summary for `<slug>` without
/// running the piners runner. Suitable for `--metadata-only` authoring views.
/// `actual_outputs_checked` is `false` on the returned detail.
pub fn load_fixture_detail(slug: &str) -> Result<IndicatorFixtureDetail> {
    let fixture = fixture::load_fixture(slug)?;
    Ok(detail::fixture_detail(&fixture, None))
}

/// Load fixture metadata and expected-output summary for `<slug>`, also
/// running the piners runner to collect actual output keys. Suitable for the
/// default `po indicator <slug>` authoring view. `actual_outputs_checked` is
/// `true` on the returned detail.
pub fn load_fixture_detail_with_actual(slug: &str) -> Result<IndicatorFixtureDetail> {
    let fixture = fixture::load_fixture(slug)?;
    let actual = runner::run_fixture_actual(&fixture)?;
    Ok(detail::fixture_detail(&fixture, Some(&actual)))
}

/// Run one indicator fixture and return the raw runner output series without
/// comparing against `expect.json`. The returned report includes a
/// `runner_expect` field (an `expect.json`-shaped template with zero tolerance
/// and no fixture metadata) suitable for copy-pasting into a new smoke fixture.
pub fn run_actual(slug: &str) -> Result<IndicatorActualReport> {
    let fixture = fixture::load_fixture(slug)?;
    runner::run_fixture_actual(&fixture)
}

/// Run all indicator fixtures matching the optional `grep` and `baseline_filter`
/// through the piners runner in one batch. Returns `Err` when no fixtures
/// match the filter. `IndicatorBatchReport.ok` is `false` when any fixture
/// fails; individual fixture results are in `reports`.
pub fn run_strict_filtered(
    grep: Option<&str>,
    baseline_filter: Option<&str>,
) -> Result<IndicatorBatchReport> {
    let fixtures = list_fixtures_filtered(grep, baseline_filter)?;
    if fixtures.is_empty() {
        bail!(
            "no indicator fixtures matched ({})",
            fixture::filter_description(grep, baseline_filter)
        );
    }
    let mut reports = Vec::with_capacity(fixtures.len());
    for listing in &fixtures {
        match run_strict(&listing.slug) {
            Ok(report) => reports.push(report),
            Err(err) => reports.push(IndicatorReport {
                slug: listing.slug.clone(),
                baseline: listing.baseline,
                ok: false,
                bar_count: 0,
                compared_bar_count: 0,
                output_count: 0,
                expected_output_keys: vec![],
                actual_output_keys: vec![],
                mismatch_count: 0,
                tolerance: 0.0,
                test_range: None,
                pine_version: None,
                tv_snapshot: None,
                runtime_error: Some(format!("fixture error: {err:#}")),
                stub_dependencies: vec![],
                mismatches: vec![],
            }),
        }
    }
    let fixture_count = reports.len();
    let passed_count = reports.iter().filter(|report| report.ok).count();
    let failed_count = fixture_count - passed_count;
    Ok(IndicatorBatchReport {
        ok: failed_count == 0,
        fixture_count,
        passed_count,
        failed_count,
        reports,
    })
}
