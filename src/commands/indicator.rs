use anyhow::{Result, bail};
use pine_cli::indicator;

use crate::output::{ResolvedFormat, print_json};

pub(crate) fn run(
    slug: Option<&str>,
    strict: bool,
    list: bool,
    format: ResolvedFormat,
) -> Result<()> {
    if list {
        return print_fixture_list(format);
    }
    if !strict {
        bail!("`pine indicator` currently supports `--strict <slug>` or `--list`");
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

fn print_fixture_list(format: ResolvedFormat) -> Result<()> {
    let fixtures = indicator::list_fixtures()?;
    match format {
        ResolvedFormat::Json => print_json(&fixtures)?,
        ResolvedFormat::Text => {
            if fixtures.is_empty() {
                println!("no indicator fixtures baked");
            } else {
                for fixture in &fixtures {
                    println!("{}", fixture.slug);
                }
            }
        }
    }
    Ok(())
}

fn print_report_text(report: &indicator::IndicatorReport) {
    println!("indicator:   {}", report.slug);
    println!("bars:        {}", report.bar_count);
    println!("outputs:     {}", report.output_count);
    println!("tolerance:   {}", report.tolerance);
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
