// Trade-list diff: pine-oracle's port of PineForge's verify_corpus.py.
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
//   - No --show-diffs ranked output. The DiffReport carries the headline
//     numbers only; verbose worst-N is future work.
//
// Threshold values mirror verify_corpus.py exactly; bumping them here
// without bumping them upstream is a regression flag.

use std::collections::HashSet;

use anyhow::{anyhow, Context, Result};
use chrono::{FixedOffset, NaiveDateTime, TimeZone};
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
}

#[derive(Debug, Clone, Copy, Serialize)]
pub struct Thresholds {
    pub count: f64,
    pub entry: f64,
    pub exit: f64,
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

pub fn diff(probe_slug: &str, user_csv: &str) -> Result<DiffReport> {
    let probe = corpus::load_probe(probe_slug)?;

    let meta = parse_inputs_json(probe.inputs_json);
    let tv_tz = tv_csv_tz_offset(&meta);

    let tv = parse_trades(probe.tv_trades_csv, tv_tz).context("parsing baked tv_trades.csv")?;
    let user = parse_trades(user_csv, 0).context("parsing user-supplied trade list")?;

    let matched_initial = align_by_time(&tv, &user);
    let (tv_trim, user_trim) = trim_to_common_window(&tv, &user, &matched_initial);
    let matched = align_by_time(&tv_trim, &user_trim);

    let profile = resolve_profile(probe.strategy_pine, &meta);
    let thresh = thresholds_for(profile);

    let count_delta = relative_max(tv_trim.len() as f64, user_trim.len() as f64);
    let mut entry_deltas: Vec<f64> = Vec::with_capacity(matched.len());
    let mut exit_deltas: Vec<f64> = Vec::with_capacity(matched.len());
    let mut pnl_deltas: Vec<f64> = Vec::with_capacity(matched.len());
    for (tv_t, eng_t) in &matched {
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
        &matched,
        &tv_trim,
        count_delta,
        entry_p90,
        exit_p90,
        pnl_p90,
        thresh,
    );
    let tier = apply_overrides(tier, &meta);

    Ok(DiffReport {
        probe_slug: probe.slug,
        profile,
        tier,
        tv_trade_count: tv_trim.len(),
        user_trade_count: user_trim.len(),
        matched_count: matched.len(),
        count_delta,
        entry_p90_delta: entry_p90,
        exit_p90_delta: exit_p90,
        pnl_p90_delta: pnl_p90,
        thresholds: thresh,
    })
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

fn align_by_time(tv: &[TradePair], eng: &[TradePair]) -> Vec<(TradePair, TradePair)> {
    let mut matched: Vec<(TradePair, TradePair)> = Vec::new();
    let mut used: HashSet<usize> = HashSet::new();
    let mut j_start = 0usize;
    for tv_t in tv {
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
            matched.push((tv_t.clone(), eng[j].clone()));
            used.insert(j);
        }
    }
    matched
}

fn trim_to_common_window(
    tv: &[TradePair],
    eng: &[TradePair],
    matched: &[(TradePair, TradePair)],
) -> (Vec<TradePair>, Vec<TradePair>) {
    if matched.is_empty() {
        return (tv.to_vec(), eng.to_vec());
    }
    let lo = matched
        .iter()
        .map(|(t, e)| t.entry_time.min(e.entry_time))
        .min()
        .unwrap()
        - MATCH_WINDOW_SECONDS;
    let hi = matched
        .iter()
        .map(|(t, e)| t.entry_time.max(e.entry_time))
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
    matched: &[(TradePair, TradePair)],
    tv_pool: &[TradePair],
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
    let match_rate = matched.len() as f64 / tv_pool.len().max(1) as f64;
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
    if !matched.is_empty() {
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
        .and_then(|b| b.as_bool());
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
        Some("utc_plus_8") | Some("asia_taipei") => 8,
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
        assert_eq!(m.len(), 3);
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
        let tv = vec![pair(Direction::Long, 100, 50.0)];
        let matched = vec![(tv[0].clone(), tv[0].clone())];
        let tier = classify_tier(
            &matched,
            &tv,
            0.0,
            0.0,
            0.0,
            0.0,
            thresholds_for(Profile::Strict),
        );
        assert_eq!(tier, Tier::Excellent);
    }

    #[test]
    fn classify_minimal_when_no_matches() {
        let tv: Vec<TradePair> = Vec::new();
        let matched: Vec<(TradePair, TradePair)> = Vec::new();
        let tier = classify_tier(
            &matched,
            &tv,
            0.5,
            0.5,
            0.5,
            0.5,
            thresholds_for(Profile::Strict),
        );
        assert_eq!(tier, Tier::Minimal);
    }

    #[test]
    fn anomaly_probe_with_empty_user_csv_returns_anomaly() {
        let empty = "Trade #,Type,Date and time,Price USDT,Net P&L USD\n";
        let report = diff(PROBE, empty).expect("diff must run on empty user csv");
        assert_eq!(report.matched_count, 0);
        assert!(report.tv_trade_count > 0);
        // Anomaly override fires when computed tier is below excellent.
        assert_eq!(report.tier, Tier::Anomaly);
    }
}
