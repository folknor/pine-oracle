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
// Interior trim (verify_corpus.py::interior_time_bounds): when
// inputs.json carries `trim_bars` and/or `warmup_bars` plus an OHLCV
// span (`ohlcv_first_ms`, `ohlcv_last_ms`, and either `bar_ms` or a
// timeframe we can derive `bar_ms` from), the diff drops edge / warmup
// trades from the headline stats. When any of those fields is absent
// the diff falls back to the full common-window trim, matching the
// pre-OHLCV-bake behaviour.
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
use crate::util::pine_text;

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

// Direction is intentionally exhaustive: Long / Short is the complete set of
// directions TradingView's trade-list CSV can express. New variants would
// require a TV format change, so exhaustive matching is correct here.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Direction {
    Long,
    Short,
}

// Profile may grow new variants (e.g. a "relaxed" or "custom" profile),
// so callers should not rely on exhaustive matching.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Profile {
    Strict,
    Production,
}

// Tier may grow new classification levels as the algorithm matures, so
// callers should not rely on exhaustive matching.
#[non_exhaustive]
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

impl std::fmt::Display for Profile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Profile::Strict => f.write_str("strict"),
            Profile::Production => f.write_str("production"),
        }
    }
}

impl std::fmt::Display for Tier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Each arm mirrors the serde snake_case rename so text and JSON output
        // use the same spelling (e.g. "engine_only", not "engineonly").
        match self {
            Tier::Excellent => f.write_str("excellent"),
            Tier::Strong => f.write_str("strong"),
            Tier::Moderate => f.write_str("moderate"),
            Tier::Weak => f.write_str("weak"),
            Tier::Minimal => f.write_str("minimal"),
            Tier::Anomaly => f.write_str("anomaly"),
            Tier::EngineOnly => f.write_str("engine_only"),
        }
    }
}

#[must_use]
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
    /// Interior window `[lo_ms, hi_ms]` applied to drop edge/warmup
    /// trades, populated when `inputs.json` declares trim_bars/warmup_bars
    /// plus an OHLCV span. `None` when interior trim wasn't applied
    /// (no metadata, trivial padding, or empty interior).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub interior_window: Option<InteriorWindow>,
}

#[must_use]
#[derive(Debug, Clone, Copy, Serialize)]
pub struct InteriorWindow {
    pub lo_ms: i64,
    pub hi_ms: i64,
    pub trim_bars: i32,
    pub warmup_bars: i32,
}

#[must_use]
#[derive(Debug, Clone, Copy, Serialize)]
pub struct Thresholds {
    pub count: f64,
    pub entry: f64,
    pub exit: f64,
    pub pnl: f64,
}

/// Options for `diff`. Construct via `DiffOptions::default()` then chain
/// builder methods (`.with_show_diffs(n)`) to set fields. The builder pattern
/// keeps call sites forward-compatible as new fields land.
#[derive(Debug, Clone, Copy, Default)]
pub struct DiffOptions {
    /// If non-zero, the worst N matched pairs (ranked descending by
    /// per-pair `max(entry_delta, exit_delta, pnl_delta)`) are emitted
    /// in `DiffReport::pair_diffs`, and all unmatched trades from the
    /// trimmed window are listed in `tv_orphans` / `user_orphans`.
    /// `usize::MAX` keeps every matched pair.
    pub show_diffs: usize,
}

impl DiffOptions {
    #[must_use]
    pub fn with_show_diffs(mut self, n: usize) -> Self {
        self.show_diffs = n;
        self
    }
}

/// One matched (TV, user) trade pair, normalized for display.
#[must_use]
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
#[must_use]
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

    let meta = parse_inputs_json(probe.inputs_json).context("parsing inputs.json")?;
    let tv_tz = tv_csv_tz_offset(&meta);

    let tv = parse_trades(probe.tv_trades_csv, tv_tz).context("parsing baked tv_trades.csv")?;
    let user = parse_trades(user_csv, 0).context("parsing user-supplied trade list")?;

    let initial_indices = align_by_time(&tv, &user);
    let (tv_trim, user_trim) = trim_to_common_window(&tv, &user, &initial_indices);
    let final_indices = align_by_time(&tv_trim, &user_trim);

    let profile = resolve_profile(probe.strategy_pine, &meta);
    let thresh = thresholds_for(profile);

    // The OHLCV span comes from `inputs.json` when explicitly set,
    // otherwise from the baked per-feed catalog (slug -> upstream feed).
    // Individual fields can be overridden piecemeal.
    let fallback_span = corpus::ohlcv_span_for_probe(&probe.slug);
    let bounds = interior_time_bounds(
        meta.trim_bars,
        meta.warmup_bars,
        meta.ohlcv_first_ms
            .or_else(|| fallback_span.map(|span| span.first_ms)),
        meta.ohlcv_last_ms
            .or_else(|| fallback_span.map(|span| span.last_ms)),
        meta.bar_ms
            .or_else(|| fallback_span.map(|span| span.bar_ms)),
    );

    // Per `verify_corpus.py`, the headline counts use interior-only
    // totals when bounds are set, while the per-pair p90 metrics use
    // gating_matched (interior-only when non-empty, all matched
    // otherwise). The empty fallback preserves percentile data on
    // pathological corner cases without smuggling edge bars back into
    // the count delta.
    let interior_indices: Vec<(usize, usize)> = bounds
        .map(|b| {
            final_indices
                .iter()
                .copied()
                .filter(|&(ti, _)| is_interior(tv_trim[ti].entry_time, b))
                .collect()
        })
        .unwrap_or_default();
    let (tv_gate_count, user_gate_count) = match bounds {
        Some(b) => (
            tv_trim
                .iter()
                .filter(|t| is_interior(t.entry_time, b))
                .count(),
            user_trim
                .iter()
                .filter(|t| is_interior(t.entry_time, b))
                .count(),
        ),
        None => (tv_trim.len(), user_trim.len()),
    };
    let gating_indices: &[(usize, usize)] = if bounds.is_some() && !interior_indices.is_empty() {
        &interior_indices
    } else {
        &final_indices
    };

    let count_delta = relative_max(tv_gate_count as f64, user_gate_count as f64);
    let mut entry_deltas: Vec<f64> = Vec::with_capacity(gating_indices.len());
    let mut exit_deltas: Vec<f64> = Vec::with_capacity(gating_indices.len());
    let mut pnl_deltas: Vec<f64> = Vec::with_capacity(gating_indices.len());
    for &(ti, ui) in gating_indices {
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
        gating_indices.len(),
        tv_gate_count,
        count_delta,
        entry_p90,
        exit_p90,
        pnl_p90,
        thresh,
    );
    let tier = apply_overrides(tier, &meta);

    let (pair_diffs, tv_orphans, user_orphans) = if opts.show_diffs > 0 {
        build_details(&tv_trim, &user_trim, gating_indices, opts.show_diffs)
    } else {
        (Vec::new(), Vec::new(), Vec::new())
    };

    let interior_window = bounds.map(|(lo, hi)| InteriorWindow {
        lo_ms: lo,
        hi_ms: hi,
        trim_bars: meta.trim_bars,
        warmup_bars: meta.warmup_bars,
    });

    Ok(DiffReport {
        probe_slug: probe.slug,
        profile,
        tier,
        tv_trade_count: tv_gate_count,
        user_trade_count: user_gate_count,
        matched_count: gating_indices.len(),
        count_delta,
        entry_p90_delta: entry_p90,
        exit_p90_delta: exit_p90,
        pnl_p90_delta: pnl_p90,
        thresholds: thresh,
        pair_diffs,
        tv_orphans,
        user_orphans,
        interior_window,
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
    let col_price = col(&["Price USDT", "Price USD", "Price"])
        .ok_or_else(|| anyhow!("missing price column"))?;
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
        // Parse pnl as Option<f64>: None when the column is absent or blank,
        // Some(v) only when the CSV actually contains a value. This preserves
        // the emptiness signal through to the merge step below, so a blank
        // exit-row pnl column cannot silently overwrite a non-zero entry-row
        // pnl (the old code converted empty to 0.0 and then unconditionally
        // assigned, losing the entry value).
        let pnl_opt: Option<f64> = if pnl_s.is_empty() {
            None
        } else {
            Some(
                pnl_s
                    .parse()
                    .with_context(|| format!("parsing pnl `{pnl_s}`"))?,
            )
        };
        let time = parse_dt(time_s, tz).with_context(|| format!("parsing datetime `{time_s}`"))?;

        let entry = by_num.entry(num).or_default();
        entry.direction = Some(direction);
        // Merge rule: last non-None value wins. Entry row sets the initial
        // pnl when present; exit row overrides it when the exit column is
        // also present (exit row carries the canonical settled pnl in TV's
        // export). An absent column on either row leaves the other row's
        // value intact.
        if let Some(p) = pnl_opt {
            entry.pnl = Some(p);
        }
        if kind.starts_with("entry") {
            entry.entry_time = Some(time);
            entry.entry_price = Some(price);
        } else {
            entry.exit_price = Some(price);
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
        .expect("matched checked non-empty above")
        - MATCH_WINDOW_SECONDS;
    let hi = matched
        .iter()
        .map(|&(ti, ui)| tv[ti].entry_time.max(eng[ui].entry_time))
        .max()
        .expect("matched checked non-empty above")
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
    let stripped = pine_text::strip_pine_comments(pine_source);
    pine_text::uses_trail_exits(&stripped)
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

// Recognised inputs.json fields used by pine-oracle:
//   - parity_profile: "strict" | "production" - force a profile (default: auto-detect)
//   - tv_trades_csv_tz: timezone string for interpreting TV CSV timestamps
//   - expected_tier: "anomaly" | "engine_only" - override tier when below excellent
//   - validation_overrides.expect_tv_match: false -> always EngineOnly when below excellent
//   - trim_bars: i32 (default 0) symmetric edge trim, in bars, applied to both ends
//   - warmup_bars: i32 (default 0) extra asymmetric lead pad, in bars
//   - ohlcv_first_ms, ohlcv_last_ms: OHLCV window bounds in milliseconds (epoch)
//   - bar_ms: bar interval in milliseconds; required when ohlcv_*_ms are present
//     and we have no other way to derive it
//
// Additional fields present in some corpus probes that are silently ignored here
// (used by the PineForge engine, not by pine-oracle):
//   - _comment: free-form annotation string
//   - runtime_overrides: engine runtime parameter overrides
//   - ohlcv_start_ms: PineForge engine override (single-sided); the oracle uses
//     the explicit ohlcv_first_ms / ohlcv_last_ms pair instead
#[derive(Debug, Default)]
struct InputsMeta {
    parity_profile: Option<String>,
    tv_trades_csv_tz: Option<String>,
    expected_tier: Option<String>,
    expect_tv_match: Option<bool>,
    trim_bars: i32,
    warmup_bars: i32,
    ohlcv_first_ms: Option<i64>,
    ohlcv_last_ms: Option<i64>,
    bar_ms: Option<i64>,
}

fn parse_inputs_json(raw: Option<&'static str>) -> Result<InputsMeta> {
    let Some(raw) = raw else {
        return Ok(InputsMeta::default());
    };
    let v: serde_json::Value =
        serde_json::from_str(raw).context("inputs.json: JSON parse error")?;
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
    let trim_bars = i32::try_from(
        v.get("trim_bars")
            .and_then(serde_json::Value::as_i64)
            .unwrap_or(0),
    )
    .unwrap_or(0)
    .max(0);
    let warmup_bars = i32::try_from(
        v.get("warmup_bars")
            .and_then(serde_json::Value::as_i64)
            .unwrap_or(0),
    )
    .unwrap_or(0)
    .max(0);
    let ohlcv_first_ms = v.get("ohlcv_first_ms").and_then(serde_json::Value::as_i64);
    let ohlcv_last_ms = v.get("ohlcv_last_ms").and_then(serde_json::Value::as_i64);
    let bar_ms = v.get("bar_ms").and_then(serde_json::Value::as_i64);
    Ok(InputsMeta {
        parity_profile,
        tv_trades_csv_tz,
        expected_tier,
        expect_tv_match,
        trim_bars,
        warmup_bars,
        ohlcv_first_ms,
        ohlcv_last_ms,
        bar_ms,
    })
}

/// Backport of `verify_corpus.py::interior_time_bounds`. Returns
/// `Some((lo_ms, hi_ms))` when `trim_bars`/`warmup_bars` are set AND
/// we have a usable OHLCV span; `None` otherwise. `trim_bars`
/// symmetrically excludes edge bars from both ends; `warmup_bars`
/// is an extra asymmetric lead pad. Returns `None` if the resulting
/// window is empty or inverted.
fn interior_time_bounds(
    trim_bars: i32,
    warmup_bars: i32,
    ohlcv_first_ms: Option<i64>,
    ohlcv_last_ms: Option<i64>,
    bar_ms: Option<i64>,
) -> Option<(i64, i64)> {
    if trim_bars <= 0 && warmup_bars <= 0 {
        return None;
    }
    let first = ohlcv_first_ms?;
    let last = ohlcv_last_ms?;
    let bar = bar_ms?;
    if bar <= 0 {
        return None;
    }
    let lead_pad = (i64::from(trim_bars) + i64::from(warmup_bars.max(0))) * bar;
    let tail_pad = i64::from(trim_bars) * bar;
    let lo = first + lead_pad;
    let hi = last - tail_pad;
    if lo >= hi { None } else { Some((lo, hi)) }
}

/// Returns true when `entry_time_seconds` falls inside the interior
/// window in milliseconds. Mirrors `verify_corpus.py::is_interior`
/// which expects an entry-time-in-ms input; trades carry seconds, so
/// the conversion happens here.
fn is_interior(entry_time_seconds: i64, bounds: (i64, i64)) -> bool {
    let entry_ms = entry_time_seconds.saturating_mul(1000);
    let (lo, hi) = bounds;
    lo <= entry_ms && entry_ms <= hi
}

/// Return the UTC offset in whole hours for the timezone string stored in
/// `tv_trades_csv_tz`.
///
/// Recognition order:
///   1. Known snake-case aliases from the corpus (`utc_plus_8`, `utc`, ...).
///   2. IANA names for the timezones actually present in the corpus (hardcoded;
///      static-offset only - DST is NOT honoured; see TODO below).
///   3. Explicit numeric offsets: `"+8"`, `"-5"`, `"+09:00"`, `"-05:30"`, etc.
///   4. Unrecognised strings fall back to `TV_CSV_TZ_OFFSET_HOURS_DEFAULT`
///      (UTC+8, Asia/Taipei) with no warning, matching upstream behaviour.
///
/// TODO: for DST-aware alignment, replace the hardcoded IANA table with a
/// dependency on `chrono-tz` and resolve the offset per-trade-timestamp.
fn tv_csv_tz_offset(meta: &InputsMeta) -> i32 {
    let s = match meta.tv_trades_csv_tz.as_deref() {
        Some(s) => s.to_ascii_lowercase(),
        None => return TV_CSV_TZ_OFFSET_HOURS_DEFAULT,
    };
    let s = s.trim();

    // 1. Known snake-case aliases.
    match s {
        "utc_plus_8" | "asia_taipei" | "asia/taipei" => return 8,
        "utc" | "europe/london" | "gmt" => return 0,
        "asia/tokyo" => return 9,
        "america/new_york" | "us/eastern" => return -5, // EST; DST not honoured
        _ => {}
    }

    // 2. Explicit numeric offset strings: "+8", "-5", "+09:00", "-05:30", etc.
    //    Accept an optional leading "utc" or "gmt" prefix, then a sign + digits.
    let stripped = s
        .strip_prefix("utc")
        .or_else(|| s.strip_prefix("gmt"))
        .unwrap_or(s)
        .trim();
    if let Some(hours) = parse_offset_string(stripped) {
        return hours;
    }

    TV_CSV_TZ_OFFSET_HOURS_DEFAULT
}

/// Parse a bare numeric offset like `"+8"`, `"-5"`, `"+09:00"`, `"-05:30"`.
/// Returns the whole-hour component (truncates any sub-hour part).
/// Returns `None` for anything that doesn't look like an offset.
fn parse_offset_string(s: &str) -> Option<i32> {
    // Must start with '+' or '-'.
    #[allow(clippy::question_mark)]
    let (sign, rest) = if let Some(r) = s.strip_prefix('+') {
        (1i32, r)
    } else if let Some(r) = s.strip_prefix('-') {
        (-1i32, r)
    } else {
        return None;
    };
    // Hours, optional ":" + minutes.
    let (hour_s, _min_s) = match rest.split_once(':') {
        Some((h, m)) => (h, m),
        None => (rest, ""),
    };
    let hours: i32 = hour_s.parse().ok()?;
    Some(sign * hours)
}

fn apply_overrides(computed: Tier, meta: &InputsMeta) -> Tier {
    if computed == Tier::Excellent {
        return computed;
    }
    // expect_tv_match=false takes precedence over expected_tier: if the author
    // explicitly disabled TV-match validation, the result is always EngineOnly
    // regardless of any expected_tier annotation. This matches upstream
    // verify_corpus.py where the expect_tv_match check happens first.
    if matches!(meta.expect_tv_match, Some(false)) {
        return Tier::EngineOnly;
    }
    if let Some(expected) = meta.expected_tier.as_deref() {
        match expected.to_ascii_lowercase().as_str() {
            "anomaly" => return Tier::Anomaly,
            "engine_only" => return Tier::EngineOnly,
            _ => {}
        }
    }
    computed
}

#[cfg(test)]
mod tests {
    include!("tests.rs");
}
