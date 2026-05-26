use anyhow::Result;
use pine_cli::corpus;

use crate::output::{ResolvedFormat, Style, print_json};

pub(crate) fn run(slug: &str, format: ResolvedFormat, _style: Style, quiet: bool) -> Result<()> {
    let probe = corpus::load_probe(slug)?;
    match format {
        ResolvedFormat::Json => {
            print_json(&probe)?;
        }
        ResolvedFormat::Text => {
            if quiet {
                println!("{}", probe.strategy_pine);
                return Ok(());
            }
            println!("slug: {}", probe.slug);
            match probe.summary {
                Some(summary) => println!("summary: {summary}"),
                None => println!("summary: (none)"),
            }
            let trade_lines = probe.tv_trades_csv.lines().count();
            let trade_bytes = probe.tv_trades_csv.len();
            println!(
                "tv_trades.csv: {trade_lines} lines, {trade_bytes} bytes (use --format json for structured fields + full CSV)"
            );
            println!(
                "inputs.json: {}",
                if probe.inputs_json.is_some() {
                    "present"
                } else {
                    "(none)"
                }
            );
            println!("\nstrategy.pine:\n{}", probe.strategy_pine);
        }
    }
    Ok(())
}
