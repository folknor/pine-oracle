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
    BaselineKind, EXPECT_SCHEMA_VERSION, IndicatorActualReport, IndicatorBaselineInfo,
    IndicatorBatchReport, IndicatorExpectedOutput, IndicatorFixtureCounts, IndicatorFixtureDetail,
    IndicatorGeneratedExpect, IndicatorListing, IndicatorMismatch, IndicatorReport, MismatchReason,
    OutputValue, TestRange,
};

pub fn run_strict(slug: &str) -> Result<IndicatorReport> {
    let fixture = fixture::load_fixture(slug)?;
    runner::run_fixture(&fixture)
}

pub fn load_fixture_detail(slug: &str) -> Result<IndicatorFixtureDetail> {
    let fixture = fixture::load_fixture(slug)?;
    Ok(detail::fixture_detail(&fixture, None))
}

pub fn load_fixture_detail_with_actual(slug: &str) -> Result<IndicatorFixtureDetail> {
    let fixture = fixture::load_fixture(slug)?;
    let actual = runner::run_fixture_actual(&fixture)?;
    Ok(detail::fixture_detail(&fixture, Some(&actual)))
}

pub fn run_actual(slug: &str) -> Result<IndicatorActualReport> {
    let fixture = fixture::load_fixture(slug)?;
    runner::run_fixture_actual(&fixture)
}

pub fn run_strict_filtered(
    grep: Option<&str>,
    baseline_filter: Option<&str>,
) -> Result<IndicatorBatchReport> {
    let fixtures = list_fixtures_filtered(grep, baseline_filter)?;
    let mut reports = Vec::with_capacity(fixtures.len());
    for fixture in fixtures {
        reports.push(run_strict(&fixture.slug)?);
    }
    let fixture_count = reports.len();
    if fixture_count == 0 {
        bail!(
            "no indicator fixtures matched ({})",
            fixture::filter_description(grep, baseline_filter)
        );
    }
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
