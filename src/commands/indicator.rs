use anyhow::{Result, bail};
use pine_cli::indicator;

use crate::output::{ResolvedFormat, print_json};

pub(crate) fn run(
    slug: Option<&str>,
    strict: bool,
    list: bool,
    all: bool,
    grep: Option<&str>,
    baseline: Option<&str>,
    format: ResolvedFormat,
) -> Result<()> {
    if baseline.is_some_and(indicator::is_baseline_catalog_request) {
        return print_baseline_catalog(format);
    }
    if list {
        if strict {
            bail!("`pine indicator --list` cannot combine with `--strict`");
        }
        let grep = resolve_list_grep(slug, grep)?;
        if all {
            bail!("`pine indicator --list` cannot combine with `--all`");
        }
        return print_fixture_list(grep, baseline, format);
    }
    if all {
        if slug.is_some() {
            bail!("`pine indicator --strict --all` cannot combine with a fixture slug");
        }
        if !strict {
            bail!("`pine indicator --all` requires `--strict`");
        }
        return print_batch_report(grep, baseline, format);
    }
    if grep.is_some() || baseline.is_some() {
        bail!("`pine indicator --grep/--baseline` requires `--list` or `--all`");
    }
    if !strict {
        bail!(
            "`pine indicator` currently supports `--strict <slug>`, `--strict --all`, or `--list`"
        );
    }
    let Some(slug) = slug else {
        bail!("`pine indicator --strict` requires a fixture slug");
    };
    let report = indicator::run_strict(slug)?;
    match format {
        ResolvedFormat::Json => print_json(&report)?,
        ResolvedFormat::Text => print_report_text(&report),
    }
    if !report.ok {
        std::process::exit(1);
    }
    Ok(())
}

fn resolve_list_grep<'a>(slug: Option<&'a str>, grep: Option<&'a str>) -> Result<Option<&'a str>> {
    match (slug, grep) {
        (Some(_), Some(_)) => bail!("`pine indicator <slug> --list` cannot combine with `--grep`"),
        (Some(slug), None) => Ok(Some(slug)),
        (None, grep) => Ok(grep),
    }
}

fn print_baseline_catalog(format: ResolvedFormat) -> Result<()> {
    let baselines = indicator::baseline_catalog()?;
    match format {
        ResolvedFormat::Json => {
            print_json(&serde_json::json!({
                "baselines": baselines,
            }))?;
        }
        ResolvedFormat::Text => {
            for baseline in &baselines {
                println!(
                    "{:<6} {:>5}  {}",
                    baseline.baseline, baseline.count, baseline.description
                );
            }
        }
    }
    Ok(())
}

fn print_fixture_list(
    grep: Option<&str>,
    baseline: Option<&str>,
    format: ResolvedFormat,
) -> Result<()> {
    let fixtures = indicator::list_fixtures_filtered(grep, baseline)?;
    match format {
        ResolvedFormat::Json => print_json(&fixtures)?,
        ResolvedFormat::Text => {
            if fixtures.is_empty() {
                if grep.is_some() || baseline.is_some() {
                    println!("no indicator fixtures matched");
                } else {
                    println!("no indicator fixtures baked");
                }
            } else {
                for fixture in &fixtures {
                    println!("{}", fixture_list_line(fixture));
                }
            }
        }
    }
    Ok(())
}

fn print_batch_report(
    grep: Option<&str>,
    baseline: Option<&str>,
    format: ResolvedFormat,
) -> Result<()> {
    let report = indicator::run_strict_filtered(grep, baseline)?;
    match format {
        ResolvedFormat::Json => print_json(&report)?,
        ResolvedFormat::Text => print_batch_report_text(&report),
    }
    if !report.ok {
        std::process::exit(1);
    }
    Ok(())
}

fn fixture_list_line(fixture: &indicator::IndicatorListing) -> String {
    let mut parts = vec![fixture.baseline.to_string()];
    if let Some(symbol) = &fixture.symbol {
        parts.push(format!("symbol={symbol}"));
    }
    if let Some(timeframe) = &fixture.timeframe {
        parts.push(format!("tf={timeframe}"));
    }
    if let Some(bar_count) = fixture.bar_count {
        parts.push(format!("bars={bar_count}"));
    }
    if let Some(output_count) = fixture.output_count {
        parts.push(format!("outputs={output_count}"));
    }
    if let Some(range) = &fixture.test_range {
        parts.push(format!("range={}..{}", range.start, range.end));
    }
    if let Some(pine) = &fixture.pine_version {
        parts.push(format!("pine={pine}"));
    }
    if let Some(snapshot) = &fixture.tv_snapshot {
        parts.push(format!("tv={snapshot}"));
    }
    format!("{}  [{}]", fixture.slug, parts.join(" "))
}

fn print_batch_report_text(report: &indicator::IndicatorBatchReport) {
    if report.fixture_count == 0 {
        println!("no indicator fixtures matched");
        return;
    }
    for fixture in &report.reports {
        let status = if fixture.ok { "ok" } else { "fail" };
        println!(
            "{:<32} {:<4} bars={} outputs={} mismatches={}",
            fixture.slug, status, fixture.bar_count, fixture.output_count, fixture.mismatch_count
        );
    }
    println!(
        "summary: {}/{} ok, {} failed",
        report.passed_count, report.fixture_count, report.failed_count
    );
    if report.ok {
        return;
    }
    for fixture in report.reports.iter().filter(|fixture| !fixture.ok) {
        println!();
        print_report_text(fixture);
    }
}

fn print_report_text(report: &indicator::IndicatorReport) {
    println!("indicator:   {}", report.slug);
    println!("baseline:    {}", report.baseline);
    println!("bars:        {}", report.bar_count);
    if report.compared_bar_count != report.bar_count {
        println!("compared:    {}", report.compared_bar_count);
    }
    println!("outputs:     {}", report.output_count);
    println!("tolerance:   {}", report.tolerance);
    if let Some(range) = &report.test_range {
        println!("range:       {} .. {}", range.start, range.end);
    }
    if let Some(pine_version) = &report.pine_version {
        println!("pine:        {pine_version}");
    }
    if let Some(tv_snapshot) = &report.tv_snapshot {
        println!("tv snapshot: {tv_snapshot}");
    }
    if let Some(runtime_error) = &report.runtime_error {
        println!("runtime:     {runtime_error}");
    }
    if report.stub_dependencies.is_empty() {
        println!("stubs:       0");
    } else {
        println!("stubs:       {}", report.stub_dependencies.len());
        for stub in &report.stub_dependencies {
            println!("  {} ({} call(s))", stub.name, stub.call_count);
        }
    }
    if report.ok {
        println!("ok");
        return;
    }
    println!(
        "expected keys: {}",
        output_keys(&report.expected_output_keys)
    );
    println!("actual keys:   {}", output_keys(&report.actual_output_keys));
    println!("mismatches:  {}", report.mismatch_count);
    for mismatch in &report.mismatches {
        let index = mismatch
            .bar_index
            .map_or_else(|| "-".to_string(), |index| index.to_string());
        match (mismatch.expected, mismatch.actual) {
            (Some(expected), Some(actual)) => println!(
                "  {}[{index}] {:?}: expected {expected}, actual {actual}",
                mismatch.output, mismatch.reason
            ),
            _ => println!(
                "  {}[{index}] {:?}: expected_len {}, actual_len {}",
                mismatch.output, mismatch.reason, mismatch.expected_len, mismatch.actual_len
            ),
        }
    }
}

fn output_keys(keys: &[String]) -> String {
    if keys.is_empty() {
        "<none>".to_string()
    } else {
        keys.join(", ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fixture_list_line_includes_shape_metadata() {
        let fixture = indicator::IndicatorListing {
            slug: "smoke-close".to_string(),
            baseline: indicator::BaselineKind::Smoke,
            symbol: Some("SMOKE:FIXTURE".to_string()),
            timeframe: Some("1D".to_string()),
            bar_count: Some(4),
            output_count: Some(1),
            test_range: None,
            pine_version: Some("6.0.0".to_string()),
            tv_snapshot: None,
        };

        assert_eq!(
            fixture_list_line(&fixture),
            "smoke-close  [smoke symbol=SMOKE:FIXTURE tf=1D bars=4 outputs=1 pine=6.0.0]"
        );
    }

    #[test]
    fn fixture_list_line_includes_range_window() {
        let fixture = indicator::IndicatorListing {
            slug: "smoke-test-range".to_string(),
            baseline: indicator::BaselineKind::Smoke,
            symbol: Some("SMOKE:FIXTURE".to_string()),
            timeframe: Some("1D".to_string()),
            bar_count: Some(4),
            output_count: Some(1),
            test_range: Some(indicator::TestRange {
                start: "2025-01-02T00:00:00Z".to_string(),
                end: "2025-01-03T00:00:00Z".to_string(),
            }),
            pine_version: None,
            tv_snapshot: None,
        };

        assert_eq!(
            fixture_list_line(&fixture),
            "smoke-test-range  [smoke symbol=SMOKE:FIXTURE tf=1D bars=4 outputs=1 range=2025-01-02T00:00:00Z..2025-01-03T00:00:00Z]"
        );
    }

    #[test]
    fn output_keys_reports_empty_and_joined_keys() {
        assert_eq!(output_keys(&[]), "<none>");
        assert_eq!(
            output_keys(&["Close".to_string(), "Signal#1".to_string()]),
            "Close, Signal#1"
        );
    }

    #[test]
    fn list_slug_becomes_implicit_grep() {
        assert_eq!(
            resolve_list_grep(Some("request"), None).expect("valid"),
            Some("request")
        );
    }

    #[test]
    fn list_rejects_slug_and_explicit_grep() {
        let err = resolve_list_grep(Some("request"), Some("security")).expect_err("must reject");
        assert!(err.to_string().contains("cannot combine"));
    }

    #[test]
    fn list_rejects_strict_flag() {
        let err = run(None, true, true, false, None, None, ResolvedFormat::Text)
            .expect_err("must reject");
        assert!(err.to_string().contains("cannot combine"));
    }
}
