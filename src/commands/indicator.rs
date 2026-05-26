use anyhow::{Result, bail};
use pine_cli::indicator;

use crate::output::{ResolvedFormat, Style, is_catalog_request, print_catalog, print_json};

pub(crate) struct Args<'a> {
    pub(crate) slug: Option<&'a str>,
    pub(crate) strict: bool,
    pub(crate) list: bool,
    pub(crate) all: bool,
    pub(crate) actual: bool,
    pub(crate) metadata_only: bool,
    pub(crate) quiet: bool,
    pub(crate) grep: Option<&'a str>,
    pub(crate) baseline: Option<&'a str>,
}

pub(crate) fn run(args: &Args<'_>, format: ResolvedFormat, _style: Style) -> Result<()> {
    if is_catalog_request(args.baseline) {
        return print_baseline_catalog(format, args.quiet);
    }
    if args.list {
        if args.strict {
            bail!("`pine indicator --list` cannot combine with `--strict`");
        }
        if args.actual {
            bail!("`pine indicator --list` cannot combine with `--actual`");
        }
        if args.metadata_only {
            bail!("`pine indicator --list` cannot combine with `--metadata-only`");
        }
        let grep = resolve_list_grep(args.slug, args.grep)?;
        if args.all {
            bail!("`pine indicator --list` cannot combine with `--all`");
        }
        return print_fixture_list(grep, args.baseline, format, args.quiet);
    }
    if args.all {
        if args.slug.is_some() {
            bail!("`pine indicator --strict --all` cannot combine with a fixture slug");
        }
        if args.actual {
            bail!("`pine indicator --strict --all` cannot combine with `--actual`");
        }
        if args.metadata_only {
            bail!("`pine indicator --strict --all` cannot combine with `--metadata-only`");
        }
        if !args.strict {
            bail!("`pine indicator --all` requires `--strict`");
        }
        return print_batch_report(args.grep, args.baseline, format);
    }
    if args.grep.is_some() || args.baseline.is_some() {
        bail!("`pine indicator --grep/--baseline` requires `--list` or `--all`");
    }
    let Some(slug) = args.slug else {
        bail!("`pine indicator` requires a fixture slug, `--strict --all`, or `--list`");
    };
    if args.actual {
        if args.strict {
            bail!("`pine indicator --actual` cannot combine with `--strict`");
        }
        if args.metadata_only {
            bail!("`pine indicator --actual` cannot combine with `--metadata-only`");
        }
        return print_actual_report(slug, format);
    }
    if !args.strict {
        return print_fixture_detail(slug, args.metadata_only, format);
    }
    if args.metadata_only {
        bail!("`pine indicator --metadata-only` cannot combine with `--strict`");
    }
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

fn print_actual_report(slug: &str, format: ResolvedFormat) -> Result<()> {
    let report = indicator::run_actual(slug)?;
    match format {
        ResolvedFormat::Json => print_json(&report)?,
        ResolvedFormat::Text => print_actual_report_text(&report),
    }
    Ok(())
}

fn print_fixture_detail(slug: &str, metadata_only: bool, format: ResolvedFormat) -> Result<()> {
    let detail = if metadata_only {
        indicator::load_fixture_detail(slug)?
    } else {
        indicator::load_fixture_detail_with_actual(slug)?
    };
    match format {
        ResolvedFormat::Json => print_json(&detail)?,
        ResolvedFormat::Text => print_fixture_detail_text(&detail),
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

fn print_baseline_catalog(format: ResolvedFormat, quiet: bool) -> Result<()> {
    let baselines = indicator::baseline_catalog()?;
    print_catalog(
        "baselines",
        &baselines,
        |b| format!("{:<6} {:>5}  {}", b.baseline, b.count, b.description),
        |b| b.baseline.to_string(),
        format,
        quiet,
    )
}

fn print_fixture_list(
    grep: Option<&str>,
    baseline: Option<&str>,
    format: ResolvedFormat,
    quiet: bool,
) -> Result<()> {
    let fixtures = indicator::list_fixtures_filtered(grep, baseline)?;
    match format {
        ResolvedFormat::Json => print_json(&fixtures)?,
        ResolvedFormat::Text => {
            if fixtures.is_empty() {
                if !quiet {
                    if grep.is_some() || baseline.is_some() {
                        println!("no indicator fixtures matched");
                    } else {
                        println!("no indicator fixtures baked");
                    }
                }
            } else {
                for fixture in &fixtures {
                    if quiet {
                        println!("{}", fixture.slug);
                    } else {
                        println!("{}", fixture_list_line(fixture));
                    }
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

fn print_fixture_detail_text(detail: &indicator::IndicatorFixtureDetail) {
    println!("indicator:   {}", detail.slug);
    println!("baseline:    {}", detail.baseline);
    if let Some(notes) = &detail.notes {
        println!("notes:       {notes}");
    }
    println!("bars:        {}", detail.bar_count);
    if let Some(symbol) = &detail.symbol {
        println!("symbol:      {symbol}");
    }
    if let Some(timeframe) = &detail.timeframe {
        println!("timeframe:   {timeframe}");
    }
    if let Some(source) = &detail.data_source {
        println!("data source: {source}");
    }
    if let (Some(first), Some(last)) = (detail.first_bar_timestamp, detail.last_bar_timestamp) {
        println!("bar window:  {first} .. {last}");
    }
    println!("outputs:     {}", detail.output_count);
    for output in &detail.expected_outputs {
        println!("  {}", expected_output_summary(output));
    }
    if detail.actual_outputs_checked {
        println!("actual keys: {}", output_keys(&detail.actual_output_keys));
        if !detail.missing_expected_output_keys.is_empty() {
            println!(
                "missing:     {}",
                output_keys(&detail.missing_expected_output_keys)
            );
        }
        if !detail.unexpected_actual_output_keys.is_empty() {
            println!(
                "unexpected:  {}",
                output_keys(&detail.unexpected_actual_output_keys)
            );
        }
    } else {
        println!("actual keys: skipped");
    }
    println!("tolerance:   {}", detail.tolerance);
    if let Some(range) = &detail.test_range {
        println!("range:       {} .. {}", range.start, range.end);
    }
    if let Some(pine_version) = &detail.pine_version {
        println!("pine:        {pine_version}");
    }
    if let Some(tv_snapshot) = &detail.tv_snapshot {
        println!("tv snapshot: {tv_snapshot}");
    }
    if let Some(runtime_error) = &detail.runtime_error {
        println!("runtime:     {runtime_error}");
    }
    if !detail.stub_dependencies.is_empty() {
        println!("stubs:       {}", detail.stub_dependencies.len());
        for stub in &detail.stub_dependencies {
            println!("  {} ({} call(s))", stub.name, stub.call_count);
        }
    }
    println!("\nsource.pine:\n{}", detail.source_pine);
}

fn print_actual_report_text(report: &indicator::IndicatorActualReport) {
    println!("indicator:   {}", report.slug);
    println!("baseline:    {}", report.baseline);
    println!("bars:        {}", report.bar_count);
    println!("outputs:     {}", report.output_count);
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
    for (key, values) in &report.outputs {
        println!("  {}", output_series_summary(key, values));
    }
    println!("runner expect: available in --format json as runner_expect");
}

fn expected_output_summary(output: &indicator::IndicatorExpectedOutput) -> String {
    output_summary(
        &output.key,
        output.value_count,
        output.first_value,
        output.last_value,
    )
}

fn output_series_summary(key: &str, values: &[indicator::OutputValue]) -> String {
    output_summary(
        key,
        values.len(),
        values.first().copied(),
        values.last().copied(),
    )
}

fn output_summary(
    key: &str,
    value_count: usize,
    first_value: Option<indicator::OutputValue>,
    last_value: Option<indicator::OutputValue>,
) -> String {
    match (first_value, last_value) {
        (Some(first), Some(last)) => {
            format!("{key} ({value_count} value(s), first={first}, last={last})")
        }
        _ => format!("{key} ({value_count} value(s))"),
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
    fn expected_output_summary_includes_value_preview() {
        let detail = indicator::load_fixture_detail("smoke-titled-outputs").expect("detail");
        assert_eq!(
            expected_output_summary(&detail.expected_outputs[0]),
            "Close Line (4 value(s), first=9, last=13)"
        );
        assert_eq!(
            expected_output_summary(&detail.expected_outputs[1]),
            "Up Shape (4 value(s), first=false, last=true)"
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
        let err = run(
            &Args {
                slug: None,
                strict: true,
                list: true,
                all: false,
                actual: false,
                metadata_only: false,
                quiet: false,
                grep: None,
                baseline: None,
            },
            ResolvedFormat::Text,
            Style::forced(false),
        )
        .expect_err("must reject");
        assert!(err.to_string().contains("cannot combine"));
    }

    #[test]
    fn actual_output_summary_includes_value_preview() {
        let report = indicator::run_actual("smoke-titled-outputs").expect("actual report");
        assert_eq!(
            output_series_summary("Close Line", &report.outputs["Close Line"]),
            "Close Line (4 value(s), first=9, last=13)"
        );
        assert_eq!(
            output_series_summary("Up Shape", &report.outputs["Up Shape"]),
            "Up Shape (4 value(s), first=false, last=true)"
        );
    }

    #[test]
    fn actual_rejects_strict_flag() {
        let err = run(
            &Args {
                slug: Some("smoke-close"),
                strict: true,
                list: false,
                all: false,
                actual: true,
                metadata_only: false,
                quiet: false,
                grep: None,
                baseline: None,
            },
            ResolvedFormat::Text,
            Style::forced(false),
        )
        .expect_err("must reject");
        assert!(err.to_string().contains("cannot combine"));
    }

    #[test]
    fn actual_rejects_metadata_only_flag() {
        let err = run(
            &Args {
                slug: Some("smoke-close"),
                strict: false,
                list: false,
                all: false,
                actual: true,
                metadata_only: true,
                quiet: false,
                grep: None,
                baseline: None,
            },
            ResolvedFormat::Text,
            Style::forced(false),
        )
        .expect_err("must reject");
        assert!(err.to_string().contains("cannot combine"));
    }

    #[test]
    fn strict_rejects_metadata_only_flag() {
        let err = run(
            &Args {
                slug: Some("smoke-close"),
                strict: true,
                list: false,
                all: false,
                actual: false,
                metadata_only: true,
                quiet: false,
                grep: None,
                baseline: None,
            },
            ResolvedFormat::Text,
            Style::forced(false),
        )
        .expect_err("must reject");
        assert!(err.to_string().contains("cannot combine"));
    }

    #[test]
    fn list_rejects_actual_flag() {
        let err = run(
            &Args {
                slug: None,
                strict: false,
                list: true,
                all: false,
                actual: true,
                metadata_only: false,
                quiet: false,
                grep: None,
                baseline: None,
            },
            ResolvedFormat::Text,
            Style::forced(false),
        )
        .expect_err("must reject");
        assert!(err.to_string().contains("cannot combine"));
    }

    #[test]
    fn list_rejects_metadata_only_flag() {
        let err = run(
            &Args {
                slug: None,
                strict: false,
                list: true,
                all: false,
                actual: false,
                metadata_only: true,
                quiet: false,
                grep: None,
                baseline: None,
            },
            ResolvedFormat::Text,
            Style::forced(false),
        )
        .expect_err("must reject");
        assert!(err.to_string().contains("cannot combine"));
    }

    #[test]
    fn list_rejects_all_flag() {
        let err = run(
            &Args {
                slug: None,
                strict: false,
                list: true,
                all: true,
                actual: false,
                metadata_only: false,
                quiet: false,
                grep: None,
                baseline: None,
            },
            ResolvedFormat::Text,
            Style::forced(false),
        )
        .expect_err("must reject");
        assert!(err.to_string().contains("cannot combine"));
    }

    #[test]
    fn all_without_strict_rejected() {
        let err = run(
            &Args {
                slug: None,
                strict: false,
                list: false,
                all: true,
                actual: false,
                metadata_only: false,
                quiet: false,
                grep: None,
                baseline: Some("smoke"),
            },
            ResolvedFormat::Text,
            Style::forced(false),
        )
        .expect_err("must reject");
        assert!(err.to_string().contains("requires `--strict`"));
    }

    #[test]
    fn all_with_slug_rejected() {
        let err = run(
            &Args {
                slug: Some("smoke-close"),
                strict: true,
                list: false,
                all: true,
                actual: false,
                metadata_only: false,
                quiet: false,
                grep: None,
                baseline: None,
            },
            ResolvedFormat::Text,
            Style::forced(false),
        )
        .expect_err("must reject");
        assert!(err.to_string().contains("cannot combine"));
    }

    #[test]
    fn all_with_actual_rejected() {
        let err = run(
            &Args {
                slug: None,
                strict: true,
                list: false,
                all: true,
                actual: true,
                metadata_only: false,
                quiet: false,
                grep: None,
                baseline: Some("smoke"),
            },
            ResolvedFormat::Text,
            Style::forced(false),
        )
        .expect_err("must reject");
        assert!(err.to_string().contains("cannot combine"));
    }

    #[test]
    fn all_with_metadata_only_rejected() {
        let err = run(
            &Args {
                slug: None,
                strict: true,
                list: false,
                all: true,
                actual: false,
                metadata_only: true,
                quiet: false,
                grep: None,
                baseline: Some("smoke"),
            },
            ResolvedFormat::Text,
            Style::forced(false),
        )
        .expect_err("must reject");
        assert!(err.to_string().contains("cannot combine"));
    }

    #[test]
    fn grep_without_list_or_all_rejected() {
        let err = run(
            &Args {
                slug: None,
                strict: false,
                list: false,
                all: false,
                actual: false,
                metadata_only: false,
                quiet: false,
                grep: Some("smoke"),
                baseline: None,
            },
            ResolvedFormat::Text,
            Style::forced(false),
        )
        .expect_err("must reject");
        assert!(err.to_string().contains("requires `--list` or `--all`"));
    }

    #[test]
    fn baseline_catalog_request_does_not_error() {
        // Exercises the `--baseline ?` dispatch path (print_baseline_catalog).
        let result = run(
            &Args {
                slug: None,
                strict: false,
                list: false,
                all: false,
                actual: false,
                metadata_only: false,
                quiet: true,
                grep: None,
                baseline: Some("?"),
            },
            ResolvedFormat::Text,
            Style::forced(false),
        );
        assert!(
            result.is_ok(),
            "baseline=? catalog request should succeed: {:?}",
            result.err()
        );
    }
}
