use anyhow::Result;
use pine_cli::diff;

use crate::output::{ResolvedFormat, print_json};

pub(crate) fn run(
    probe_slug: &str,
    trades_csv_path: &str,
    show_diffs: usize,
    format: ResolvedFormat,
) -> Result<()> {
    let user_csv = std::fs::read_to_string(trades_csv_path)
        .map_err(|e| anyhow::anyhow!("reading {trades_csv_path}: {e}"))?;
    let report = diff::diff(probe_slug, &user_csv, diff::DiffOptions { show_diffs })?;
    match format {
        ResolvedFormat::Json => {
            print_json(&report)?;
        }
        ResolvedFormat::Text => print_diff_text(&report),
    }
    Ok(())
}

fn print_diff_text(r: &diff::DiffReport) {
    println!("probe:       {}", r.probe_slug);
    println!("profile:     {:?}", r.profile);
    println!(
        "TV trades:   {}  user trades: {}  matched: {}",
        r.tv_trade_count, r.user_trade_count, r.matched_count
    );
    println!(
        "count delta:           {:>10.4}%  (threshold {:>7.4}%)",
        r.count_delta * 100.0,
        r.thresholds.count * 100.0
    );
    println!(
        "entry-price p90 delta: {:>10.4}%  (threshold {:>7.4}%)",
        r.entry_p90_delta * 100.0,
        r.thresholds.entry * 100.0
    );
    println!(
        "exit-price  p90 delta: {:>10.4}%  (threshold {:>7.4}%)",
        r.exit_p90_delta * 100.0,
        r.thresholds.exit * 100.0
    );
    println!(
        "pnl         p90 delta: {:>10.4}%  (threshold {:>7.4}%)",
        r.pnl_p90_delta * 100.0,
        r.thresholds.pnl * 100.0
    );
    println!("tier:        {:?}", r.tier);
    print_diff_details(r);
}

fn print_diff_details(r: &diff::DiffReport) {
    if !r.pair_diffs.is_empty() {
        println!();
        println!(
            "worst {} matched pair(s) (ranked by max of entry / exit / pnl deltas):",
            r.pair_diffs.len()
        );
        for (i, p) in r.pair_diffs.iter().enumerate() {
            let dir = match p.direction {
                diff::Direction::Long => "long ",
                diff::Direction::Short => "short",
            };
            let pnl_cell = match p.pnl_delta {
                Some(d) => format!("{:>6.2}%", d * 100.0),
                None => "  (na)".to_string(),
            };
            println!(
                "  {:>2}. {dir}  worst {:>6.2}%  skew {:>+5}s",
                i + 1,
                p.worst_delta * 100.0,
                p.time_skew_seconds
            );
            println!(
                "      tv:   entry {} @ {:>10.4}    exit @ {:>10.4}    pnl {:>+10.4}",
                p.tv_entry_time, p.tv_entry_price, p.tv_exit_price, p.tv_pnl
            );
            println!(
                "      user: entry {} @ {:>10.4}    exit @ {:>10.4}    pnl {:>+10.4}",
                p.user_entry_time, p.user_entry_price, p.user_exit_price, p.user_pnl
            );
            println!(
                "      delta:                entry {:>6.2}%      exit {:>6.2}%      pnl {pnl_cell}",
                p.entry_delta * 100.0,
                p.exit_delta * 100.0
            );
        }
    }
    print_orphan_block("TV-only", &r.tv_orphans);
    print_orphan_block("user-only", &r.user_orphans);
}

fn print_orphan_block(label: &str, rows: &[diff::TradeRow]) {
    if rows.is_empty() {
        return;
    }
    println!();
    println!("{label} trades ({} unmatched):", rows.len());
    for t in rows {
        let dir = match t.direction {
            diff::Direction::Long => "long ",
            diff::Direction::Short => "short",
        };
        println!(
            "  {dir}  entry {} @ {:>10.4}    exit @ {:>10.4}    pnl {:>+10.4}",
            t.entry_time, t.entry_price, t.exit_price, t.pnl
        );
    }
}
