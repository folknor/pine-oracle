// SPDX-License-Identifier: Apache-2.0 OR MPL-2.0
//
// Trade-list diff: pine-oracle's port of PineForge's verify_corpus.py.
// `verify_corpus.py` is Apache-2.0, copyright PineForge contributors. This
// port retains the upstream license + adds the project's MPL-2.0 umbrella.
//
// `diff(probe_slug, user_csv)` aligns a user-supplied trade list against the
// probe's baked tv_trades.csv (the TradingView ground truth) and classifies
// the divergence into one of:
//   excellent / strong / moderate / weak / minimal
// or one of two overrides honoured from the probe's inputs.json:
//   anomaly / engine_only
//
// Algorithm (v1, matching verify_corpus.py's core):
//   1. Parse both CSVs into TradePair (entry + exit joined by Trade #).
//   2. Greedy time-window align: each TV trade pairs with the closest
//      same-direction user trade entering within 1 hour AND within a $3
//      entry-price gate.
//   3. Trim both lists to the matched window plus or minus 1h.
//   4. Re-align.
//   5. Compute 4 deltas: count (absolute relative), entry / exit / pnl p90
//      (relative-to-tv with near-zero pnl scratch trades excluded).
//   6. Pick profile (strict or production) based on whether the probe's
//      strategy.pine uses any trail_* parameter, or honour
//      inputs.json::parity_profile if set.
//   7. Classify into excellent / strong / moderate / weak / minimal using
//      the threshold table for the chosen profile.
//   8. Override to anomaly / engine_only when inputs.json says so (but only
//      when the computed tier is below excellent, so a real fix isn't masked).
//
// V1 limitations (vs upstream):
//   - No interior trim. OHLCV isn't baked into the binary today, so the
//     trim_bars / warmup_bars trimming that needs ohlcv_first_ms / last_ms
//     is skipped. The headline stats use the full trim_to_common_window.
//
// Threshold values mirror verify_corpus.py exactly; bumping them here
// without bumping them upstream is a regression flag.
//
// Per-pair detail (`DiffOptions::show_diffs`): when set, the report's
// `pair_diffs` carries the worst N matched pairs ranked by per-pair
// max(entry_delta, exit_delta, pnl_delta) - the same metrics that drive
// tier classification - and `tv_orphans` / `user_orphans` list the
// unmatched trades from the trimmed window. The ranking metric is a
// local design choice (verify_corpus.py's exact format isn't reproduced
// here); it surfaces the trade pairs that the tier classifier itself
// would weight most heavily.

use std::collections::HashSet;

use anyhow::{Context, Result, anyhow};
use chrono::{DateTime, FixedOffset, NaiveDateTime, TimeZone, Utc};
use serde::Serialize;

use crate::corpus;

// ---------- thresholds (lifted verbatim from verify_corpus.py) ----------

const STRICT_COUNT_DELTA: f64 = 0.01;
const STRICT_ENTRY_DELTA: f64 = 0.0001;
const STRICT_EXIT_DELTA: f64 = 0.0001;
const STRICT_PNL_DELTA: f64 = 0.01;

const PRODUCTION_COUNT_DELTA: f64 = 0.01;
const PRODUCTION_ENTRY_DELTA: f64 = 0.0001;
const PRODUCTION_EXIT_DELTA: f64 = 0.0005;
const PRODUCTION_PNL_DELTA: f64 = 1.0;

const STRONG_COUNT_DELTA: f64 = 0.05;
const STRONG_ENTRY_DELTA: f64 = 0.001;
const STRONG_EXIT_DELTA: f64 = 0.005;
const STRONG_PNL_DELTA: f64 = 1.0;

const MATCH_WINDOW_SECONDS: i64 = 3600;
const ENTRY_PRICE_GATE: f64 = 3.00;
const PNL_NEAR_ZERO_USD: f64 = 0.01;

const TV_CSV_TZ_OFFSET_HOURS_DEFAULT: i32 = 8; // Asia/Taipei, per upstream

// ---------- public types ----------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Direction {
    Long,
    Short,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Profile {
    Strict,
    Production,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Tier {
    Excellent,
    Strong,
    Moderate,
    Weak,
    Minimal,
    Anomaly,
    EngineOnly,
}

#[derive(Debug, Clone, Serialize)]
pub struct DiffReport {
    pub probe_slug: String,
    pub profile: Profile,
    pub tier: Tier,
    pub tv_trade_count: usize,
    pub user_trade_count: usize,
    pub matched_count: usize,
    pub count_delta: f64,
    pub entry_p90_delta: f64,
    pub exit_p90_delta: f64,
    pub pnl_p90_delta: f64,
    pub thresholds: Thresholds,
    /// Worst-N matched pairs, ranked descending by
    /// `max(entry_delta, exit_delta, pnl_delta)`. Empty unless
    /// `DiffOptions::show_diffs > 0`.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub pair_diffs: Vec<PairDiff>,
    /// TV trades in the trimmed window that didn't pair with any user
    /// trade. Empty unless `DiffOptions::show_diffs > 0`.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub tv_orphans: Vec<TradeRow>,
    /// User trades in the trimmed window that didn't pair with any TV
    /// trade. Empty unless `DiffOptions::show_diffs > 0`.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub user_orphans: Vec<TradeRow>,
}

#[derive(Debug, Clone, Copy, Serialize)]
pub struct Thresholds {
    pub count: f64,
    pub entry: f64,
    pub exit: f64,
    pub pnl: f64,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct DiffOptions {
    /// If non-zero, the worst N matched pairs (ranked descending by
    /// per-pair `max(entry_delta, exit_delta, pnl_delta)`) are emitted
    /// in `DiffReport::pair_diffs`, and all unmatched trades from the
    /// trimmed window are listed in `tv_orphans` / `user_orphans`.
    /// `usize::MAX` keeps every matched pair.
    pub show_diffs: usize,
}

/// One matched (TV, user) trade pair, normalized for display.
#[derive(Debug, Clone, Serialize)]
pub struct PairDiff {
    pub direction: Direction,
    pub tv_entry_time: String,
    pub user_entry_time: String,
    /// `user_entry_time - tv_entry_time` in seconds (signed).
    pub time_skew_seconds: i64,
    pub tv_entry_price: f64,
    pub user_entry_price: f64,
    pub entry_delta: f64,
    pub tv_exit_price: f64,
    pub user_exit_price: f64,
    pub exit_delta: f64,
    pub tv_pnl: f64,
    pub user_pnl: f64,
    /// Empty when `tv_pnl` is near-zero (matches the headline pnl-p90
    /// gate that drops scratch trades from the percentile).
    pub pnl_delta: Option<f64>,
    /// Ranking key: `max(entry_delta, exit_delta, pnl_delta.unwrap_or(0))`.
    pub worst_delta: f64,
}

/// One trade row, normalized for display in the orphan lists.
#[derive(Debug, Clone, Serialize)]
pub struct TradeRow {
    pub direction: Direction,
    pub entry_time: String,
    pub entry_price: f64,
    pub exit_price: f64,
    pub pnl: f64,
}

#[derive(Debug, Clone)]
struct TradePair {
    direction: Direction,
    entry_time: i64, // unix seconds
    entry_price: f64,
    exit_price: f64,
    pnl: f64,
}

// ---------- public entrypoint ----------

pub fn diff(probe_slug: &str, user_csv: &str, opts: DiffOptions) -> Result<DiffReport> {
    let probe = corpus::load_probe(probe_slug)?;

    let meta = parse_inputs_json(probe.inputs_json);
    let tv_tz = tv_csv_tz_offset(&meta);

    let tv = parse_trades(probe.tv_trades_csv, tv_tz).context("parsing baked tv_trades.csv")?;
    let user = parse_trades(user_csv, 0).context("parsing user-supplied trade list")?;

    let initial_indices = align_by_time(&tv, &user);
    let (tv_trim, user_trim) = trim_to_common_window(&tv, &user, &initial_indices);
    let final_indices = align_by_time(&tv_trim, &user_trim);

    let profile = resolve_profile(probe.strategy_pine, &meta);
    let thresh = thresholds_for(profile);

    let count_delta = relative_max(tv_trim.len() as f64, user_trim.len() as f64);
    let mut entry_deltas: Vec<f64> = Vec::with_capacity(final_indices.len());
    let mut exit_deltas: Vec<f64> = Vec::with_capacity(final_indices.len());
    let mut pnl_deltas: Vec<f64> = Vec::with_capacity(final_indices.len());
    for &(ti, ui) in &final_indices {
        let tv_t = &tv_trim[ti];
        let eng_t = &user_trim[ui];
        entry_deltas.push(relative_max(tv_t.entry_price, eng_t.entry_price));
        exit_deltas.push(relative_max(tv_t.exit_price, eng_t.exit_price));
        if tv_t.pnl.abs() >= PNL_NEAR_ZERO_USD {
            pnl_deltas.push((tv_t.pnl - eng_t.pnl).abs() / tv_t.pnl.abs());
        }
    }

    let entry_p90 = percentile(&entry_deltas, 0.90);
    let exit_p90 = percentile(&exit_deltas, 0.90);
    let pnl_p90 = percentile(&pnl_deltas, 0.90);

    let tier = classify_tier(
        final_indices.len(),
        tv_trim.len(),
        count_delta,
        entry_p90,
        exit_p90,
        pnl_p90,
        thresh,
    );
    let tier = apply_overrides(tier, &meta);

    let (pair_diffs, tv_orphans, user_orphans) = if opts.show_diffs > 0 {
        build_details(&tv_trim, &user_trim, &final_indices, opts.show_diffs)
    } else {
        (Vec::new(), Vec::new(), Vec::new())
    };

    Ok(DiffReport {
        probe_slug: probe.slug,
        profile,
        tier,
        tv_trade_count: tv_trim.len(),
        user_trade_count: user_trim.len(),
        matched_count: final_indices.len(),
        count_delta,
        entry_p90_delta: entry_p90,
        exit_p90_delta: exit_p90,
        pnl_p90_delta: pnl_p90,
        thresholds: thresh,
        pair_diffs,
        tv_orphans,
        user_orphans,
    })
}

// ---------- worst-N + orphan extraction ----------

fn build_details(
    tv: &[TradePair],
    user: &[TradePair],
    matched: &[(usize, usize)],
    limit: usize,
) -> (Vec<PairDiff>, Vec<TradeRow>, Vec<TradeRow>) {
    let mut pairs: Vec<PairDiff> = matched
        .iter()
        .map(|&(ti, ui)| pair_diff(&tv[ti], &user[ui]))
        .collect();
    pairs.sort_by(|a, b| {
        b.worst_delta
            .partial_cmp(&a.worst_delta)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    pairs.truncate(limit);

    let tv_used: HashSet<usize> = matched.iter().map(|&(i, _)| i).collect();
    let user_used: HashSet<usize> = matched.iter().map(|&(_, j)| j).collect();
    let tv_orphans = tv
        .iter()
        .enumerate()
        .filter(|(i, _)| !tv_used.contains(i))
        .map(|(_, t)| trade_row(t))
        .collect();
    let user_orphans = user
        .iter()
        .enumerate()
        .filter(|(j, _)| !user_used.contains(j))
        .map(|(_, t)| trade_row(t))
        .collect();
    (pairs, tv_orphans, user_orphans)
}

fn pair_diff(tv: &TradePair, user: &TradePair) -> PairDiff {
    let entry_delta = relative_max(tv.entry_price, user.entry_price);
    let exit_delta = relative_max(tv.exit_price, user.exit_price);
    let pnl_delta = if tv.pnl.abs() >= PNL_NEAR_ZERO_USD {
        Some((tv.pnl - user.pnl).abs() / tv.pnl.abs())
    } else {
        None
    };
    let worst_delta = entry_delta.max(exit_delta).max(pnl_delta.unwrap_or(0.0));
    PairDiff {
        direction: tv.direction,
        tv_entry_time: format_ts(tv.entry_time),
        user_entry_time: format_ts(user.entry_time),
        time_skew_seconds: user.entry_time - tv.entry_time,
        tv_entry_price: tv.entry_price,
        user_entry_price: user.entry_price,
        entry_delta,
        tv_exit_price: tv.exit_price,
        user_exit_price: user.exit_price,
        exit_delta,
        tv_pnl: tv.pnl,
        user_pnl: user.pnl,
        pnl_delta,
        worst_delta,
    }
}

fn trade_row(t: &TradePair) -> TradeRow {
    TradeRow {
        direction: t.direction,
        entry_time: format_ts(t.entry_time),
        entry_price: t.entry_price,
        exit_price: t.exit_price,
        pnl: t.pnl,
    }
}

fn format_ts(unix_seconds: i64) -> String {
    match DateTime::<Utc>::from_timestamp(unix_seconds, 0) {
        Some(dt) => dt.format("%Y-%m-%d %H:%M UTC").to_string(),
        None => format!("@{unix_seconds}"),
    }
}

// ---------- CSV parsing ----------

fn parse_trades(csv_data: &str, tz_offset_hours: i32) -> Result<Vec<TradePair>> {
    let tz = FixedOffset::east_opt(tz_offset_hours * 3600)
        .ok_or_else(|| anyhow!("invalid tz offset hours: {tz_offset_hours}"))?;

    // The CSV may carry a UTF-8 BOM; strip it before handing to the parser.
    let trimmed = csv_data.strip_prefix('\u{feff}').unwrap_or(csv_data);
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(true)
        .flexible(true)
        .from_reader(trimmed.as_bytes());

    let headers = reader.headers()?.clone();
    let col = |needles: &[&str]| -> Option<usize> {
        for (i, h) in headers.iter().enumerate() {
            let h = h.trim();
            if needles.iter().any(|n| n.eq_ignore_ascii_case(h)) {
                return Some(i);
            }
        }
        None
    };
    let col_trade_num = col(&["Trade #"]).ok_or_else(|| anyhow!("missing `Trade #` column"))?;
    let col_type = col(&["Type"]).ok_or_else(|| anyhow!("missing `Type` column"))?;
    let col_time = col(&["Date and time", "Date/time", "Time"])
        .ok_or_else(|| anyhow!("missing time column"))?;
    let col_price = col(&["Price USDT", "Price"]).ok_or_else(|| anyhow!("missing price column"))?;
    let col_pnl = col(&["Net P&L USD", "Net PnL", "P&L USD"]);

    #[derive(Default)]
    struct Partial {
        direction: Option<Direction>,
        entry_time: Option<i64>,
        entry_price: Option<f64>,
        exit_price: Option<f64>,
        pnl: Option<f64>,
    }
    let mut by_num: std::collections::BTreeMap<u32, Partial> = Default::default();

    for rec in reader.records() {
        let rec = rec?;
        let Some(num_s) = rec.get(col_trade_num) else {
            continue;
        };
        let num: u32 = num_s
            .trim()
            .parse()
            .with_context(|| format!("parsing Trade # `{num_s}`"))?;
        let kind = rec.get(col_type).unwrap_or("").trim().to_ascii_lowercase();
        let time_s = rec.get(col_time).unwrap_or("").trim();
        let price_s = rec.get(col_price).unwrap_or("").trim();
        let pnl_s = col_pnl.and_then(|i| rec.get(i)).unwrap_or("").trim();

        let direction = if kind.contains("long") {
            Direction::Long
        } else if kind.contains("short") {
            Direction::Short
        } else {
            continue;
        };
        let price: f64 = price_s
            .parse()
            .with_context(|| format!("parsing price `{price_s}`"))?;
        let pnl: f64 = if pnl_s.is_empty() {
            0.0
        } else {
            pnl_s
                .parse()
                .with_context(|| format!("parsing pnl `{pnl_s}`"))?
        };
        let time = parse_dt(time_s, tz).with_context(|| format!("parsing datetime `{time_s}`"))?;

        let entry = by_num.entry(num).or_default();
        entry.direction = Some(direction);
        entry.pnl = Some(pnl);
        if kind.starts_with("entry") {
            entry.entry_time = Some(time);
            entry.entry_price = Some(price);
        } else {
            entry.exit_price = Some(price);
            // Exit row often carries the canonical pnl; prefer it when present.
            entry.pnl = Some(pnl);
        }
    }

    let mut pairs: Vec<TradePair> = Vec::new();
    for (_num, p) in by_num {
        let (Some(direction), Some(entry_time), Some(entry_price), Some(exit_price)) =
            (p.direction, p.entry_time, p.entry_price, p.exit_price)
        else {
            continue;
        };
        pairs.push(TradePair {
            direction,
            entry_time,
            entry_price,
            exit_price,
            pnl: p.pnl.unwrap_or(0.0),
        });
    }
    pairs.sort_by_key(|t| t.entry_time);
    Ok(pairs)
}

fn parse_dt(s: &str, tz: FixedOffset) -> Result<i64> {
    // Common TV format: "YYYY-MM-DD HH:MM" (no seconds, no tz marker).
    // Fall back to "YYYY-MM-DD HH:MM:SS" for engines that emit seconds.
    let naive = NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M")
        .or_else(|_| NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S"))
        .map_err(|e| anyhow!("unsupported datetime format `{s}`: {e}"))?;
    let dt = tz
        .from_local_datetime(&naive)
        .single()
        .ok_or_else(|| anyhow!("ambiguous datetime `{s}`"))?;
    Ok(dt.timestamp())
}

// ---------- alignment ----------

/// Greedy time-window align. Returns matched `(tv_index, user_index)` pairs.
/// Each TV trade pairs with at most one user trade and vice versa; the
/// match is the unused, same-direction, within-window, within-price-gate
/// candidate closest in entry time.
fn align_by_time(tv: &[TradePair], eng: &[TradePair]) -> Vec<(usize, usize)> {
    let mut matched: Vec<(usize, usize)> = Vec::new();
    let mut used: HashSet<usize> = HashSet::new();
    let mut j_start = 0usize;
    for (ti, tv_t) in tv.iter().enumerate() {
        while j_start < eng.len()
            && eng[j_start].entry_time < tv_t.entry_time - MATCH_WINDOW_SECONDS
        {
            j_start += 1;
        }
        let mut best_j: Option<usize> = None;
        let mut best_dt = MATCH_WINDOW_SECONDS + 1;
        for (offset, e) in eng[j_start..].iter().enumerate() {
            let j = j_start + offset;
            if used.contains(&j) {
                continue;
            }
            if e.entry_time > tv_t.entry_time + MATCH_WINDOW_SECONDS {
                break;
            }
            if e.direction != tv_t.direction {
                continue;
            }
            if (e.entry_price - tv_t.entry_price).abs() > ENTRY_PRICE_GATE {
                continue;
            }
            let dt = (e.entry_time - tv_t.entry_time).abs();
            if dt < best_dt {
                best_dt = dt;
                best_j = Some(j);
            }
        }
        if let Some(j) = best_j {
            matched.push((ti, j));
            used.insert(j);
        }
    }
    matched
}

fn trim_to_common_window(
    tv: &[TradePair],
    eng: &[TradePair],
    matched: &[(usize, usize)],
) -> (Vec<TradePair>, Vec<TradePair>) {
    if matched.is_empty() {
        return (tv.to_vec(), eng.to_vec());
    }
    let lo = matched
        .iter()
        .map(|&(ti, ui)| tv[ti].entry_time.min(eng[ui].entry_time))
        .min()
        .unwrap()
        - MATCH_WINDOW_SECONDS;
    let hi = matched
        .iter()
        .map(|&(ti, ui)| tv[ti].entry_time.max(eng[ui].entry_time))
        .max()
        .unwrap()
        + MATCH_WINDOW_SECONDS;
    let tv_trim = tv
        .iter()
        .filter(|t| lo <= t.entry_time && t.entry_time <= hi)
        .cloned()
        .collect();
    let eng_trim = eng
        .iter()
        .filter(|t| lo <= t.entry_time && t.entry_time <= hi)
        .cloned()
        .collect();
    (tv_trim, eng_trim)
}

// ---------- metrics ----------

fn relative_max(a: f64, b: f64) -> f64 {
    let denom = a.abs().max(b.abs()).max(1e-9);
    (a - b).abs() / denom
}

fn percentile(xs: &[f64], p: f64) -> f64 {
    if xs.is_empty() {
        return 0.0;
    }
    let mut s: Vec<f64> = xs.to_vec();
    s.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let k = (s.len() - 1) as f64 * p;
    let f = k.floor() as usize;
    let c = (f + 1).min(s.len() - 1);
    if f == c {
        s[f]
    } else {
        let kf = k - f as f64;
        s[f] * (c as f64 - k) + s[c] * kf
    }
}

// ---------- profile + tier classification ----------

fn thresholds_for(profile: Profile) -> Thresholds {
    match profile {
        Profile::Strict => Thresholds {
            count: STRICT_COUNT_DELTA,
            entry: STRICT_ENTRY_DELTA,
            exit: STRICT_EXIT_DELTA,
            pnl: STRICT_PNL_DELTA,
        },
        Profile::Production => Thresholds {
            count: PRODUCTION_COUNT_DELTA,
            entry: PRODUCTION_ENTRY_DELTA,
            exit: PRODUCTION_EXIT_DELTA,
            pnl: PRODUCTION_PNL_DELTA,
        },
    }
}

fn resolve_profile(pine_source: &str, meta: &InputsMeta) -> Profile {
    if let Some(forced) = &meta.parity_profile {
        match forced.to_ascii_lowercase().as_str() {
            "production" => return Profile::Production,
            "strict" => return Profile::Strict,
            _ => {}
        }
    }
    if detect_profile_from_source(pine_source) {
        Profile::Production
    } else {
        Profile::Strict
    }
}

fn detect_profile_from_source(pine_source: &str) -> bool {
    // Strip block + line comments, then look for any trail_(points|offset|price)= token.
    let mut stripped = String::with_capacity(pine_source.len());
    let bytes = pine_source.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if i + 1 < bytes.len() && bytes[i] == b'/' && bytes[i + 1] == b'*' {
            // skip block comment
            i += 2;
            while i + 1 < bytes.len() && !(bytes[i] == b'*' && bytes[i + 1] == b'/') {
                i += 1;
            }
            i = (i + 2).min(bytes.len());
        } else if i + 1 < bytes.len() && bytes[i] == b'/' && bytes[i + 1] == b'/' {
            // skip line comment
            while i < bytes.len() && bytes[i] != b'\n' {
                i += 1;
            }
        } else {
            stripped.push(bytes[i] as char);
            i += 1;
        }
    }
    let lower = stripped.to_ascii_lowercase();
    for needle in ["trail_points", "trail_offset", "trail_price"] {
        if let Some(pos) = lower.find(needle) {
            let rest = &lower[pos + needle.len()..];
            // require `=` after optional whitespace
            if rest.trim_start().starts_with('=') {
                return true;
            }
        }
    }
    false
}

fn classify_tier(
    matched_count: usize,
    tv_pool_count: usize,
    count_delta: f64,
    entry_p90: f64,
    exit_p90: f64,
    pnl_p90: f64,
    thresh: Thresholds,
) -> Tier {
    let all_ok = count_delta < thresh.count
        && entry_p90 < thresh.entry
        && exit_p90 < thresh.exit
        && pnl_p90 < thresh.pnl;
    if all_ok {
        return Tier::Excellent;
    }
    let match_rate = matched_count as f64 / tv_pool_count.max(1) as f64;
    if match_rate >= 0.99
        && count_delta < STRONG_COUNT_DELTA
        && entry_p90 < STRONG_ENTRY_DELTA
        && exit_p90 < STRONG_EXIT_DELTA
        && pnl_p90 < STRONG_PNL_DELTA
    {
        return Tier::Strong;
    }
    if match_rate >= 0.90 {
        return Tier::Moderate;
    }
    if matched_count > 0 {
        return Tier::Weak;
    }
    Tier::Minimal
}

// ---------- inputs.json metadata ----------

#[derive(Debug, Default)]
struct InputsMeta {
    parity_profile: Option<String>,
    tv_trades_csv_tz: Option<String>,
    expected_tier: Option<String>,
    expect_tv_match: Option<bool>,
}

fn parse_inputs_json(raw: Option<&'static str>) -> InputsMeta {
    let Some(raw) = raw else {
        return InputsMeta::default();
    };
    let v: serde_json::Value = match serde_json::from_str(raw) {
        Ok(v) => v,
        Err(_) => return InputsMeta::default(),
    };
    let parity_profile = v
        .get("parity_profile")
        .and_then(|s| s.as_str())
        .map(String::from);
    let tv_trades_csv_tz = v
        .get("tv_trades_csv_tz")
        .and_then(|s| s.as_str())
        .map(String::from);
    let expected_tier = v
        .get("expected_tier")
        .and_then(|s| s.as_str())
        .map(String::from);
    let expect_tv_match = v
        .get("validation_overrides")
        .and_then(|o| o.get("expect_tv_match"))
        .and_then(serde_json::Value::as_bool);
    InputsMeta {
        parity_profile,
        tv_trades_csv_tz,
        expected_tier,
        expect_tv_match,
    }
}

fn tv_csv_tz_offset(meta: &InputsMeta) -> i32 {
    match meta
        .tv_trades_csv_tz
        .as_deref()
        .map(str::to_ascii_lowercase)
        .as_deref()
    {
        Some("utc_plus_8" | "asia_taipei") => 8,
        Some("utc") => 0,
        _ => TV_CSV_TZ_OFFSET_HOURS_DEFAULT,
    }
}

fn apply_overrides(computed: Tier, meta: &InputsMeta) -> Tier {
    if computed == Tier::Excellent {
        return computed;
    }
    if let Some(expected) = meta.expected_tier.as_deref() {
        match expected.to_ascii_lowercase().as_str() {
            "anomaly" => return Tier::Anomaly,
            "engine_only" => return Tier::EngineOnly,
            _ => {}
        }
    }
    if matches!(meta.expect_tv_match, Some(false)) {
        return Tier::EngineOnly;
    }
    computed
}

#[cfg(test)]
mod tests {
    use super::*;

    const PROBE: &str = "anomaly-equity-mirror-strategy-equity-01";

    fn baked_tv_csv() -> &'static str {
        corpus::load_probe(PROBE).unwrap().tv_trades_csv
    }

    fn pair(direction: Direction, entry_time: i64, entry_price: f64) -> TradePair {
        TradePair {
            direction,
            entry_time,
            entry_price,
            exit_price: entry_price + 1.0,
            pnl: 0.0,
        }
    }

    #[test]
    fn relative_max_handles_zero_safely() {
        assert!(relative_max(0.0, 0.0).abs() < 1e-12);
    }

    #[test]
    fn percentile_basics() {
        assert_eq!(percentile(&[1.0, 2.0, 3.0, 4.0], 0.5), 2.5);
        assert_eq!(percentile(&[1.0], 0.9), 1.0);
        assert_eq!(percentile(&[], 0.9), 0.0);
    }

    #[test]
    fn detect_profile_finds_trail_points() {
        let src = "strategy.exit(\"x\", \"e\", trail_points=10)\n";
        assert!(detect_profile_from_source(src));
    }

    #[test]
    fn detect_profile_ignores_commented_trail() {
        let src = "// trail_points=10\n/* trail_offset = 5 */\n";
        assert!(!detect_profile_from_source(src));
    }

    #[test]
    fn parses_baked_tv_csv() {
        let trades = parse_trades(baked_tv_csv(), TV_CSV_TZ_OFFSET_HOURS_DEFAULT)
            .expect("baked tv csv must parse");
        assert!(!trades.is_empty(), "baked tv csv should yield trades");
    }

    #[test]
    fn align_matches_identical_lists() {
        let tv = vec![
            pair(Direction::Long, 100, 50.0),
            pair(Direction::Short, 200, 51.0),
            pair(Direction::Long, 300, 52.0),
        ];
        let user = tv.clone();
        let m = align_by_time(&tv, &user);
        assert_eq!(m, vec![(0, 0), (1, 1), (2, 2)]);
    }

    #[test]
    fn align_rejects_direction_mismatch() {
        let tv = vec![pair(Direction::Long, 100, 50.0)];
        let user = vec![pair(Direction::Short, 100, 50.0)];
        assert!(align_by_time(&tv, &user).is_empty());
    }

    #[test]
    fn align_rejects_outside_window() {
        let tv = vec![pair(Direction::Long, 100, 50.0)];
        let user = vec![pair(Direction::Long, 100 + MATCH_WINDOW_SECONDS + 1, 50.0)];
        assert!(align_by_time(&tv, &user).is_empty());
    }

    #[test]
    fn align_rejects_price_gate_violation() {
        let tv = vec![pair(Direction::Long, 100, 50.0)];
        let user = vec![pair(Direction::Long, 100, 50.0 + ENTRY_PRICE_GATE + 0.01)];
        assert!(align_by_time(&tv, &user).is_empty());
    }

    #[test]
    fn classify_excellent_when_all_under_threshold() {
        let tier = classify_tier(1, 1, 0.0, 0.0, 0.0, 0.0, thresholds_for(Profile::Strict));
        assert_eq!(tier, Tier::Excellent);
    }

    #[test]
    fn classify_minimal_when_no_matches() {
        let tier = classify_tier(0, 0, 0.5, 0.5, 0.5, 0.5, thresholds_for(Profile::Strict));
        assert_eq!(tier, Tier::Minimal);
    }

    #[test]
    fn anomaly_probe_with_empty_user_csv_returns_anomaly() {
        let empty = "Trade #,Type,Date and time,Price USDT,Net P&L USD\n";
        let report =
            diff(PROBE, empty, DiffOptions::default()).expect("diff must run on empty user csv");
        assert_eq!(report.matched_count, 0);
        assert!(report.tv_trade_count > 0);
        // Anomaly override fires when computed tier is below excellent.
        assert_eq!(report.tier, Tier::Anomaly);
        // show_diffs=0 keeps the detail vectors empty.
        assert!(report.pair_diffs.is_empty());
        assert!(report.tv_orphans.is_empty());
        assert!(report.user_orphans.is_empty());
    }

    #[test]
    fn production_profile_loosens_exit_and_pnl() {
        let strict = thresholds_for(Profile::Strict);
        let prod = thresholds_for(Profile::Production);
        // Count + entry stay tight in both profiles.
        assert_eq!(strict.count, prod.count);
        assert_eq!(strict.entry, prod.entry);
        // Production relaxes exit (sub-bar broker drift) and pnl (catastrophic only).
        assert!(prod.exit > strict.exit, "production exit must be looser");
        assert!(prod.pnl > strict.pnl, "production pnl must be looser");
    }

    #[test]
    fn inputs_meta_parity_profile_override_forces_production() {
        let meta = InputsMeta {
            parity_profile: Some("production".into()),
            ..InputsMeta::default()
        };
        // Pine source with NO trail_* - auto-detect would say Strict - but
        // the inputs.json override wins.
        let source = "strategy.exit(\"x\", \"e\", profit=10)\n";
        assert_eq!(resolve_profile(source, &meta), Profile::Production);
    }

    #[test]
    fn inputs_meta_parity_profile_override_forces_strict() {
        let meta = InputsMeta {
            parity_profile: Some("strict".into()),
            ..InputsMeta::default()
        };
        // Pine source WITH trail_* - auto-detect would say Production -
        // but the inputs.json override wins.
        let source = "strategy.exit(\"x\", \"e\", trail_points=10)\n";
        assert_eq!(resolve_profile(source, &meta), Profile::Strict);
    }

    #[test]
    fn expect_tv_match_false_yields_engine_only() {
        let meta = InputsMeta {
            expect_tv_match: Some(false),
            ..InputsMeta::default()
        };
        // Override applies only when the computed tier is below excellent.
        assert_eq!(apply_overrides(Tier::Weak, &meta), Tier::EngineOnly);
        assert_eq!(apply_overrides(Tier::Moderate, &meta), Tier::EngineOnly);
        // Excellent is preserved so a genuine engine improvement isn't masked.
        assert_eq!(apply_overrides(Tier::Excellent, &meta), Tier::Excellent);
    }

    #[test]
    fn expected_tier_engine_only_wins_over_anomaly_when_both_set() {
        // Override precedence: expected_tier checked first; expect_tv_match
        // only fires when no expected_tier override matched.
        let meta = InputsMeta {
            expected_tier: Some("engine_only".into()),
            expect_tv_match: Some(false),
            ..InputsMeta::default()
        };
        assert_eq!(apply_overrides(Tier::Weak, &meta), Tier::EngineOnly);
    }

    #[test]
    fn classify_strong_when_match_rate_high_and_within_relaxed_thresholds() {
        // 100 TV trades, 100 matched (100% match rate), entry/exit p90
        // just above strict but below strong thresholds.
        // entry_p90=0.0005 (>strict 0.0001, <strong 0.001),
        // exit_p90=0.001 (>strict 0.0001, <strong 0.005), pnl_p90=0.
        let tier = classify_tier(
            100,
            100,
            0.0,
            0.0005,
            0.001,
            0.0,
            thresholds_for(Profile::Strict),
        );
        assert_eq!(tier, Tier::Strong);
    }

    #[test]
    fn classify_moderate_when_match_rate_drops_below_strong() {
        // 95 matched out of 100 (=> below 99% strong gate but above 90%
        // moderate gate).
        let tier = classify_tier(
            95,
            100,
            0.05,
            0.001,
            0.005,
            1.0,
            thresholds_for(Profile::Strict),
        );
        assert_eq!(tier, Tier::Moderate);
    }

    #[test]
    fn classify_weak_when_match_rate_drops_below_moderate() {
        // 50 matched out of 100 (50%) => below 90% moderate gate but
        // matched > 0 so not Minimal.
        let tier = classify_tier(50, 100, 0.5, 0.5, 0.5, 5.0, thresholds_for(Profile::Strict));
        assert_eq!(tier, Tier::Weak);
    }

    // ---------- pair-diff details ----------

    fn full_pair(
        direction: Direction,
        entry_time: i64,
        entry: f64,
        exit: f64,
        pnl: f64,
    ) -> TradePair {
        TradePair {
            direction,
            entry_time,
            entry_price: entry,
            exit_price: exit,
            pnl,
        }
    }

    #[test]
    fn pair_diff_ranks_by_max_of_entry_exit_pnl() {
        // Pair A: entry exact, exit exact, pnl 100% off -> worst = 1.0
        let a_tv = full_pair(Direction::Long, 100, 50.0, 51.0, 10.0);
        let a_us = full_pair(Direction::Long, 100, 50.0, 51.0, 20.0);
        // Pair B: entry 10% off, exit/pnl exact -> worst = ~0.0909
        let b_tv = full_pair(Direction::Long, 200, 100.0, 101.0, 5.0);
        let b_us = full_pair(Direction::Long, 200, 110.0, 101.0, 5.0);
        let a = pair_diff(&a_tv, &a_us);
        let b = pair_diff(&b_tv, &b_us);
        assert!(a.worst_delta > b.worst_delta);
        assert_eq!(a.pnl_delta, Some(1.0));
        // Entry delta: |100-110|/110 = ~0.0909
        assert!((b.entry_delta - 10.0 / 110.0).abs() < 1e-9);
    }

    #[test]
    fn pair_diff_drops_pnl_for_near_zero_scratch() {
        // tv pnl below scratch threshold -> pnl_delta is None and
        // doesn't contribute to worst_delta.
        let tv = full_pair(Direction::Long, 100, 50.0, 50.001, 0.005);
        let us = full_pair(Direction::Long, 100, 50.0, 50.001, 1000.0);
        let d = pair_diff(&tv, &us);
        assert!(d.pnl_delta.is_none());
        assert!(d.worst_delta < 1e-3);
    }

    #[test]
    fn show_diffs_zero_keeps_detail_empty() {
        let report = diff(PROBE, baked_tv_csv(), DiffOptions::default()).expect("diff must run");
        assert!(report.pair_diffs.is_empty());
        assert!(report.tv_orphans.is_empty());
        assert!(report.user_orphans.is_empty());
    }

    #[test]
    fn show_diffs_truncates_to_n_and_sorts_descending() {
        // Self-diff: the baked csv is parsed with the probe's chart
        // timezone (UTC+8 by default) while the user-supplied csv is
        // parsed as UTC, so the two copies land 8h apart and nothing
        // matches. The truncation + sort logic still has to behave.
        let report =
            diff(PROBE, baked_tv_csv(), DiffOptions { show_diffs: 3 }).expect("diff must run");
        assert!(report.pair_diffs.len() <= 3);
        for w in report.pair_diffs.windows(2) {
            assert!(w[0].worst_delta >= w[1].worst_delta);
        }
        // Conservation: every trimmed TV trade is either matched or in
        // tv_orphans (show_diffs > 0 emits all orphans, not a top-N).
        assert_eq!(
            report.matched_count + report.tv_orphans.len(),
            report.tv_trade_count
        );
        assert_eq!(
            report.matched_count + report.user_orphans.len(),
            report.user_trade_count
        );
    }

    #[test]
    fn build_details_reports_orphans_on_both_sides() {
        // TV has 3 trades, user has 2 (one matches TV[0], one is orphan
        // outside the match window). TV[1] and TV[2] are orphans.
        let tv = vec![
            pair(Direction::Long, 1000, 50.0),
            pair(Direction::Long, 5000, 50.0),
            pair(Direction::Long, 9000, 50.0),
        ];
        let user = vec![
            pair(Direction::Long, 1000, 50.0),
            // Outside the 1h match window from any TV entry -> orphan.
            pair(Direction::Long, 50000, 50.0),
        ];
        let matched = align_by_time(&tv, &user);
        assert_eq!(matched, vec![(0, 0)]);
        let (pairs, tv_orph, user_orph) = build_details(&tv, &user, &matched, 10);
        assert_eq!(pairs.len(), 1);
        assert_eq!(tv_orph.len(), 2);
        assert_eq!(user_orph.len(), 1);
        assert!((user_orph[0].entry_price - 50.0).abs() < 1e-9);
    }

    #[test]
    fn pair_diff_time_skew_is_signed() {
        let tv = full_pair(Direction::Long, 1000, 50.0, 51.0, 10.0);
        let us = full_pair(Direction::Long, 1300, 50.0, 51.0, 10.0);
        let d = pair_diff(&tv, &us);
        assert_eq!(d.time_skew_seconds, 300);
        let d2 = pair_diff(&us, &tv);
        assert_eq!(d2.time_skew_seconds, -300);
    }

    #[test]
    fn format_ts_is_iso_utc() {
        // 2024-01-15 10:30:00 UTC = unix 1705314600
        assert_eq!(format_ts(1705314600), "2024-01-15 10:30 UTC");
    }
}
