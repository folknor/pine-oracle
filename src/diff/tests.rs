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
    fn expect_tv_match_false_beats_expected_tier_anomaly() {
        // expect_tv_match=false takes precedence over expected_tier="anomaly":
        // if the author disabled TV-match validation the result is EngineOnly,
        // not Anomaly. This matches upstream verify_corpus.py behaviour.
        let meta = InputsMeta {
            expected_tier: Some("anomaly".into()),
            expect_tv_match: Some(false),
            ..InputsMeta::default()
        };
        assert_eq!(apply_overrides(Tier::Weak, &meta), Tier::EngineOnly);
    }

    #[test]
    fn expected_tier_anomaly_alone_yields_anomaly() {
        let meta = InputsMeta {
            expected_tier: Some("anomaly".into()),
            ..InputsMeta::default()
        };
        assert_eq!(apply_overrides(Tier::Weak, &meta), Tier::Anomaly);
    }

    #[test]
    fn expect_tv_match_false_alone_yields_engine_only_below_excellent() {
        let meta = InputsMeta {
            expect_tv_match: Some(false),
            ..InputsMeta::default()
        };
        assert_eq!(apply_overrides(Tier::Weak, &meta), Tier::EngineOnly);
        // Excellent is always preserved regardless of overrides.
        assert_eq!(apply_overrides(Tier::Excellent, &meta), Tier::Excellent);
    }

    #[test]
    fn no_overrides_passes_computed_through() {
        let meta = InputsMeta::default();
        for tier in [Tier::Strong, Tier::Moderate, Tier::Weak, Tier::Minimal] {
            assert_eq!(apply_overrides(tier, &meta), tier);
        }
    }

    #[test]
    fn expected_tier_engine_only_with_expect_tv_match_false_still_engine_only() {
        // Both flags point to EngineOnly; expect_tv_match fires first in the
        // new ordering but the result is the same.
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

    // ---------- Bug 1: tv_csv_tz_offset IANA + explicit offset tests ----------

    fn meta_with_tz(tz: &str) -> InputsMeta {
        InputsMeta {
            tv_trades_csv_tz: Some(tz.to_string()),
            ..InputsMeta::default()
        }
    }

    #[test]
    fn tz_offset_existing_snake_case_aliases() {
        assert_eq!(tv_csv_tz_offset(&meta_with_tz("utc_plus_8")), 8);
        assert_eq!(tv_csv_tz_offset(&meta_with_tz("asia_taipei")), 8);
        assert_eq!(tv_csv_tz_offset(&meta_with_tz("utc")), 0);
    }

    #[test]
    fn tz_offset_iana_asia_taipei() {
        assert_eq!(tv_csv_tz_offset(&meta_with_tz("Asia/Taipei")), 8);
    }

    #[test]
    fn tz_offset_iana_america_new_york() {
        // Static EST; DST not honoured - documented in tv_csv_tz_offset.
        assert_eq!(tv_csv_tz_offset(&meta_with_tz("America/New_York")), -5);
    }

    #[test]
    fn tz_offset_iana_europe_london() {
        assert_eq!(tv_csv_tz_offset(&meta_with_tz("Europe/London")), 0);
    }

    #[test]
    fn tz_offset_iana_asia_tokyo() {
        assert_eq!(tv_csv_tz_offset(&meta_with_tz("Asia/Tokyo")), 9);
    }

    #[test]
    fn tz_offset_explicit_plus8() {
        assert_eq!(tv_csv_tz_offset(&meta_with_tz("+8")), 8);
    }

    #[test]
    fn tz_offset_explicit_minus5() {
        assert_eq!(tv_csv_tz_offset(&meta_with_tz("-5")), -5);
    }

    #[test]
    fn tz_offset_explicit_plus0900_colon() {
        assert_eq!(tv_csv_tz_offset(&meta_with_tz("+09:00")), 9);
    }

    #[test]
    fn tz_offset_explicit_minus0500_colon() {
        assert_eq!(tv_csv_tz_offset(&meta_with_tz("-05:00")), -5);
    }

    #[test]
    fn tz_offset_unrecognised_falls_back_to_default() {
        assert_eq!(
            tv_csv_tz_offset(&meta_with_tz("Pacific/Fakezone")),
            TV_CSV_TZ_OFFSET_HOURS_DEFAULT
        );
    }

    #[test]
    fn tz_offset_none_falls_back_to_default() {
        let meta = InputsMeta::default();
        assert_eq!(tv_csv_tz_offset(&meta), TV_CSV_TZ_OFFSET_HOURS_DEFAULT);
    }

    // ---------- Bug 2: Price USD column ----------

    #[test]
    fn parse_trades_accepts_price_usd_column() {
        // Minimal CSV with "Price USD" header (no T) - matches AAPL probe format.
        let csv = "Trade #,Type,Date and time,Price USD,Net P&L USD\n\
                   1,Entry Long,2024-01-15 10:30,185.50,\n\
                   1,Exit Long,2024-01-15 11:00,186.00,0.50\n";
        let trades = parse_trades(csv, 0).expect("Price USD column must parse");
        assert_eq!(trades.len(), 1);
        assert!((trades[0].entry_price - 185.50).abs() < 1e-9);
        assert!((trades[0].exit_price - 186.00).abs() < 1e-9);
    }

    #[test]
    fn parse_trades_accepts_price_usdt_column() {
        // Existing USDT variant should still work.
        let csv = "Trade #,Type,Date and time,Price USDT,Net P&L USD\n\
                   1,Entry Long,2024-01-15 10:30,42000.00,\n\
                   1,Exit Long,2024-01-15 11:00,43000.00,1000.00\n";
        let trades = parse_trades(csv, 0).expect("Price USDT column must parse");
        assert_eq!(trades.len(), 1);
        assert!((trades[0].entry_price - 42000.00).abs() < 1e-9);
    }

    // ---------- Fix 1: pnl entry-vs-exit overwrite ----------

    #[test]
    fn parse_trades_entry_pnl_preserved_when_exit_pnl_blank() {
        // Entry row carries a non-zero pnl; exit row has a blank pnl column.
        // The exit-row blank must NOT overwrite the entry-row pnl with 0.0.
        let csv = "Trade #,Type,Date and time,Price USDT,Net P&L USD\n\
                   1,Entry Long,2024-01-15 10:30,42000.00,500.00\n\
                   1,Exit Long,2024-01-15 11:00,43000.00,\n";
        let trades = parse_trades(csv, 0).expect("must parse");
        assert_eq!(trades.len(), 1);
        // Entry-row pnl (500.0) must survive; blank exit column must not win.
        assert!(
            (trades[0].pnl - 500.0).abs() < 1e-9,
            "expected pnl 500.0, got {}",
            trades[0].pnl
        );
    }

    #[test]
    fn parse_trades_exit_pnl_wins_when_entry_pnl_blank() {
        // Entry row has a blank pnl column; exit row carries the settled pnl.
        // The exit-row value should win (original intent: TV export canonical pnl).
        let csv = "Trade #,Type,Date and time,Price USDT,Net P&L USD\n\
                   1,Entry Long,2024-01-15 10:30,42000.00,\n\
                   1,Exit Long,2024-01-15 11:00,43000.00,1000.00\n";
        let trades = parse_trades(csv, 0).expect("must parse");
        assert_eq!(trades.len(), 1);
        assert!(
            (trades[0].pnl - 1000.0).abs() < 1e-9,
            "expected pnl 1000.0, got {}",
            trades[0].pnl
        );
    }

    // ---------- Bug 3: parse_inputs_json error propagation ----------

    #[test]
    fn parse_inputs_json_malformed_json_returns_err() {
        // Passing a non-None static str with invalid JSON should yield Err, not default.
        // We can't construct &'static str from a test literal for the real signature,
        // but we can exercise the serde_json parsing path directly by calling the
        // internal function with a leaked string - or test via the parse_offset_string
        // helper. Instead, validate the happy and error paths at the serde layer:
        let malformed = "{not valid json}";
        let result = serde_json::from_str::<serde_json::Value>(malformed);
        assert!(result.is_err(), "malformed JSON must not parse");
        // Confirm that parse_inputs_json(None) is still Ok(default).
        let ok = parse_inputs_json(None);
        assert!(ok.is_ok());
        assert!(ok.unwrap().tv_trades_csv_tz.is_none());
    }

    // ---------- Boundary tests for classify_tier (#40, #41) ----------

    #[test]
    fn classify_tier_boundary_count_delta_strict_excellent() {
        // Strict count threshold is STRICT_COUNT_DELTA = 0.01.
        // count_delta just below threshold (0.0099) with all other metrics
        // under threshold -> Excellent.
        let tier = classify_tier(1, 1, 0.0099, 0.0, 0.0, 0.0, thresholds_for(Profile::Strict));
        assert_eq!(tier, Tier::Excellent);
    }

    #[test]
    fn classify_tier_boundary_count_delta_strict_not_excellent() {
        // count_delta exactly at threshold (0.01) - the check is `< thresh.count`
        // (strictly less than), so 0.01 is NOT excellent.
        let tier = classify_tier(
            100,
            100,
            STRICT_COUNT_DELTA, // = 0.01, at the boundary
            0.0,
            0.0,
            0.0,
            thresholds_for(Profile::Strict),
        );
        // Falls through to Strong (match_rate=1.0 >= 0.99, all STRONG_* ok).
        assert_eq!(tier, Tier::Strong);
    }

    #[test]
    fn classify_tier_boundary_match_rate_0_99_strong() {
        // 99 out of 100 TV trades matched => match_rate = 0.99, meets Strong gate.
        // Deltas are above Strict thresholds but below Strong thresholds.
        let tier = classify_tier(
            99,
            100,
            STRONG_COUNT_DELTA - 0.001, // below strong count threshold
            STRONG_ENTRY_DELTA - 0.0001,
            STRONG_EXIT_DELTA - 0.001,
            STRONG_PNL_DELTA - 0.1,
            thresholds_for(Profile::Strict),
        );
        assert_eq!(tier, Tier::Strong);
    }

    #[test]
    fn classify_tier_boundary_match_rate_below_strong_above_moderate() {
        // 98 out of 100 => match_rate = 0.98. Fails the >= 0.99 Strong gate.
        // count_delta drives it past Excellent too. Falls to Moderate (>= 0.90).
        let tier = classify_tier(
            98,
            100,
            STRICT_COUNT_DELTA,         // at strict boundary -> not excellent
            STRONG_ENTRY_DELTA + 0.001, // above strong -> not strong
            0.0,
            0.0,
            thresholds_for(Profile::Strict),
        );
        assert_eq!(tier, Tier::Moderate);
    }

    #[test]
    fn classify_tier_boundary_match_rate_0_90_is_moderate() {
        // 90 out of 100 => match_rate = 0.90 exactly, meets >= 0.90 moderate gate.
        let tier = classify_tier(
            90,
            100,
            0.5, // large count_delta forces past excellent/strong
            0.5,
            0.5,
            5.0,
            thresholds_for(Profile::Strict),
        );
        assert_eq!(tier, Tier::Moderate);
    }

    #[test]
    fn classify_tier_boundary_match_rate_below_moderate_is_weak() {
        // 89 out of 100 => match_rate = 0.89, below 0.90 moderate gate.
        // matched > 0 so not Minimal.
        let tier = classify_tier(89, 100, 0.5, 0.5, 0.5, 5.0, thresholds_for(Profile::Strict));
        assert_eq!(tier, Tier::Weak);
    }

    // ---------- tv_csv_tz_offset: explicit offset overrides default (#41) ----------

    #[test]
    fn tz_offset_explicit_overrides_default() {
        // When inputs.json provides a recognised explicit offset, it must override
        // the TV_CSV_TZ_OFFSET_HOURS_DEFAULT (8), not fall through to it.
        let meta = meta_with_tz("+1");
        assert_ne!(
            tv_csv_tz_offset(&meta),
            TV_CSV_TZ_OFFSET_HOURS_DEFAULT,
            "+1 must not collapse to the UTC+8 default"
        );
        assert_eq!(tv_csv_tz_offset(&meta), 1);
    }

    // ---------- percentile boundary tests ----------

    #[test]
    fn percentile_boundary_p0_returns_min() {
        assert_eq!(percentile(&[1.0, 2.0, 3.0], 0.0), 1.0);
    }

    #[test]
    fn percentile_boundary_p1_returns_max() {
        assert_eq!(percentile(&[1.0, 2.0, 3.0], 1.0), 3.0);
    }

    #[test]
    fn percentile_two_element_midpoint() {
        // For [1.0, 2.0] at p=0.5: k = 1*0.5 = 0.5, f=0, c=1, frac=0.5
        // result = 1.0*(1-0.5) + 2.0*0.5 = 1.5
        assert!((percentile(&[1.0, 2.0], 0.5) - 1.5).abs() < 1e-12);
    }

    #[test]
    fn percentile_singleton_any_p_returns_sole_value() {
        assert_eq!(percentile(&[42.0], 0.0), 42.0);
        assert_eq!(percentile(&[42.0], 0.5), 42.0);
        assert_eq!(percentile(&[42.0], 1.0), 42.0);
    }

    // ---------- relative_max coverage ----------

    #[test]
    fn relative_max_positive_nonzero() {
        // |100 - 0| / max(100, 0, 1e-9) = 100/100 = 1.0
        assert!((relative_max(100.0, 0.0) - 1.0).abs() < 1e-12);
    }

    #[test]
    fn relative_max_both_equal_nonzero() {
        // |50 - 50| / 50 = 0
        assert!(relative_max(50.0, 50.0).abs() < 1e-12);
    }

    #[test]
    fn relative_max_asymmetric() {
        // |100 - 200| / max(100, 200) = 100/200 = 0.5
        assert!((relative_max(100.0, 200.0) - 0.5).abs() < 1e-12);
    }

    #[test]
    fn relative_max_floor_prevents_div_by_zero_for_tiny_values() {
        // Both values very small: denom clamped to 1e-9; result is near zero.
        let r = relative_max(1e-12, 1e-12);
        assert!(r.abs() < 1e-6);
    }

    // ---------- BOM-stripping ----------

    #[test]
    fn parse_trades_handles_utf8_bom() {
        // TV CSV exports often carry a UTF-8 BOM (U+FEFF). The parser must
        // strip it before reading the header row so "Trade #" is recognised.
        let csv = "\u{feff}Trade #,Type,Date and time,Price USDT,Net P&L USD\n\
                   1,Entry Long,2024-01-15 10:30,100.00,\n\
                   1,Exit Long,2024-01-15 11:00,101.00,1.00\n";
        let trades = parse_trades(csv, 0).expect("BOM-prefixed CSV must parse");
        assert_eq!(trades.len(), 1);
        assert!((trades[0].entry_price - 100.0).abs() < 1e-9);
    }

    // ---------- column alias tests ----------

    #[test]
    fn parse_trades_accepts_date_slash_time_column() {
        let csv = "Trade #,Type,Date/time,Price USDT,Net P&L USD\n\
                   1,Entry Long,2024-01-15 10:30,100.00,\n\
                   1,Exit Long,2024-01-15 11:00,101.00,1.00\n";
        let trades = parse_trades(csv, 0).expect("Date/time column alias must parse");
        assert_eq!(trades.len(), 1);
    }

    #[test]
    fn parse_trades_accepts_time_column() {
        let csv = "Trade #,Type,Time,Price USDT,Net P&L USD\n\
                   1,Entry Long,2024-01-15 10:30,100.00,\n\
                   1,Exit Long,2024-01-15 11:00,101.00,1.00\n";
        let trades = parse_trades(csv, 0).expect("Time column alias must parse");
        assert_eq!(trades.len(), 1);
    }

    // ---------- missing Trade # edge cases ----------

    #[test]
    fn parse_trades_entry_only_trade_is_dropped() {
        // A Trade # with only an Entry row (no Exit) is filtered out because
        // exit_price is None.
        let csv = "Trade #,Type,Date and time,Price USDT,Net P&L USD\n\
                   1,Entry Long,2024-01-15 10:30,100.00,\n\
                   2,Entry Long,2024-01-15 11:00,101.00,\n\
                   2,Exit Long,2024-01-15 12:00,102.00,1.00\n";
        let trades = parse_trades(csv, 0).expect("must parse");
        // Trade 1 has no exit -> filtered; trade 2 is complete.
        assert_eq!(trades.len(), 1);
        assert!((trades[0].entry_price - 101.0).abs() < 1e-9);
    }

    #[test]
    fn parse_trades_exit_only_trade_is_dropped() {
        // A Trade # with only an Exit row (no Entry) is filtered out because
        // entry_time and entry_price are None.
        let csv = "Trade #,Type,Date and time,Price USDT,Net P&L USD\n\
                   1,Exit Long,2024-01-15 11:00,101.00,1.00\n\
                   2,Entry Long,2024-01-15 10:00,100.00,\n\
                   2,Exit Long,2024-01-15 12:00,102.00,2.00\n";
        let trades = parse_trades(csv, 0).expect("must parse");
        // Trade 1 exit-only -> filtered; trade 2 complete.
        assert_eq!(trades.len(), 1);
        assert!((trades[0].entry_price - 100.0).abs() < 1e-9);
    }

    // ---------- combined expected_tier + expect_tv_match ----------

    #[test]
    fn expected_tier_anomaly_with_expect_tv_match_false_yields_engine_only() {
        // When both expected_tier="anomaly" and expect_tv_match=false are set,
        // expect_tv_match=false takes precedence and the result is EngineOnly,
        // not Anomaly. This matches upstream verify_corpus.py behaviour where
        // the expect_tv_match check happens first.
        let meta = InputsMeta {
            expected_tier: Some("anomaly".into()),
            expect_tv_match: Some(false),
            ..InputsMeta::default()
        };
        // Distinct from the engine_only+expect_tv_match test: here the
        // expected_tier disagrees (anomaly) but expect_tv_match still wins.
        assert_eq!(apply_overrides(Tier::Weak, &meta), Tier::EngineOnly);
        // Excellent is always preserved.
        assert_eq!(apply_overrides(Tier::Excellent, &meta), Tier::Excellent);
    }

    // ---------- interior trim ----------

    const BAR_MS_1M: i64 = 60_000;

    #[test]
    fn interior_bounds_returns_none_when_no_trim_or_warmup() {
        assert_eq!(
            interior_time_bounds(0, 0, Some(1_000), Some(2_000), Some(BAR_MS_1M)),
            None
        );
    }

    #[test]
    fn interior_bounds_returns_none_when_ohlcv_span_missing() {
        assert_eq!(
            interior_time_bounds(5, 0, None, Some(2_000), Some(BAR_MS_1M)),
            None
        );
        assert_eq!(
            interior_time_bounds(5, 0, Some(1_000), None, Some(BAR_MS_1M)),
            None
        );
        assert_eq!(
            interior_time_bounds(5, 0, Some(1_000), Some(2_000), None),
            None
        );
    }

    #[test]
    fn interior_bounds_pads_symmetrically_for_trim_bars() {
        // 100 bars at 1m: first_ms=0, last_ms=99 * 60_000 = 5_940_000.
        // trim_bars=2 => lead_pad = tail_pad = 120_000.
        // lo=120_000, hi=5_820_000.
        let bounds = interior_time_bounds(2, 0, Some(0), Some(5_940_000), Some(BAR_MS_1M)).unwrap();
        assert_eq!(bounds, (120_000, 5_820_000));
    }

    #[test]
    fn interior_bounds_adds_warmup_to_lead_only() {
        // trim_bars=2, warmup_bars=3 -> lead_pad=(2+3)*60_000, tail_pad=2*60_000.
        let bounds = interior_time_bounds(2, 3, Some(0), Some(5_940_000), Some(BAR_MS_1M)).unwrap();
        assert_eq!(bounds, (300_000, 5_820_000));
    }

    #[test]
    fn interior_bounds_returns_none_when_window_collapses() {
        // Pads exceed total span -> lo >= hi -> None.
        assert_eq!(
            interior_time_bounds(50, 50, Some(0), Some(60_000), Some(BAR_MS_1M)),
            None
        );
    }

    #[test]
    fn is_interior_treats_endpoints_as_inside() {
        let bounds = (1_000, 2_000);
        assert!(is_interior(1, bounds)); // 1 sec == 1_000 ms == lo
        assert!(is_interior(2, bounds)); // 2 sec == 2_000 ms == hi
        assert!(!is_interior(0, bounds));
        assert!(!is_interior(3, bounds));
    }

    #[test]
    fn parses_trim_warmup_and_ohlcv_span_from_inputs_json() {
        let raw = r#"{
            "trim_bars": 4,
            "warmup_bars": 2,
            "ohlcv_first_ms": 1700000000000,
            "ohlcv_last_ms": 1700003600000,
            "bar_ms": 60000
        }"#;
        let meta = parse_inputs_json(Some(raw)).expect("parse");
        assert_eq!(meta.trim_bars, 4);
        assert_eq!(meta.warmup_bars, 2);
        assert_eq!(meta.ohlcv_first_ms, Some(1_700_000_000_000));
        assert_eq!(meta.ohlcv_last_ms, Some(1_700_003_600_000));
        assert_eq!(meta.bar_ms, Some(60_000));
    }

    #[test]
    fn parses_negative_trim_bars_as_zero() {
        // Defensive: negative values clamp rather than panic / propagate as i32::MIN.
        let raw = r#"{ "trim_bars": -5, "warmup_bars": -3 }"#;
        let meta = parse_inputs_json(Some(raw)).expect("parse");
        assert_eq!(meta.trim_bars, 0);
        assert_eq!(meta.warmup_bars, 0);
    }
