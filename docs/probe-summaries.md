# Corpus probe summaries

Per-probe explanations for the PineForge validation corpus (`corpus/validation/<NN-slug>/`). Each probe is a TV-cross-validated Pine v6 strategy exercising a specific broker / fill / strategy semantic. This file is the BM25 substrate for the oracle's `pine probes --grep <text>` and `pine probe <slug>` subcommands (see `docs/pine-oracle.md`).

## Status

> **Two summary tracks, only the first is wired in.**
>
> 1. **Live-extracted from each `strategy.pine` header.** `corpus::summary_for(slug)` walks the leading comment block of every baked strategy.pine, skips license / SPDX / copyright / version-directive boilerplate, and returns the strategy author's own one-paragraph description. >=80% of the 235 baked probes get a real summary this way without any LLM curation. This is what `pine probes` / `pine probes --grep` use today.
> 2. **The engine-internals prose below** is keyed to engine-internal probe identifiers (`magnifier-dist-probe-01..08b`, `ies-probe-08`, `parity-probe-03..06`, `oca-three-way-probe-02`, `typed-matrix-probe-01-bool-regime-mask`, `anomaly-equity-mirror`, plus engine-history numbers 52..97) that **do not appear in the published corpus** under `vendor/pineforge-corpus/validation/`. The published corpus uses different topical slugs (e.g. `oca-multi-bracket-isolation-01`, `magnifier-tick-dist-endpoints-01`, `anomaly-equity-mirror-strategy-equity-01`). The prose is preserved as forensic reference for PineForge engine internals but is not loaded by the binary.

**21 forensic summaries** were harvested from PineForge engine source comments (Apache-2.0, attributed), concentrating on bug-bearing edge cases. **0 / 235** of those align to published corpus slugs; the live-extraction path covers the gap for everyday use.

## Attribution

Probe semantic descriptions are derived from comments in PineForge engine source (Apache-2.0, copyright PineForge contributors). Verbatim original comments are preserved under "Quote". Prose summaries are paraphrased for neutrality.

## Numbered single-script probes

### Probe 52 -- deferred-flip carry, replace-with-same-id chain reset

- **Sources:** `src/engine_strategy_commands.cpp:156-167`, `src/engine_orders.cpp:427-435`, `src/engine_fills.cpp:213`, `include/pineforge/engine.hpp:161, 166`
- **Semantic:** `strategy.entry` with the same id REPLACES the pending order entirely, including a fresh `tv_carry_qty = position_qty_` snapshot. When SE/LE is re-placed every bar a cross condition holds: once `strategy.close` flushes the position to 0, subsequent re-placements capture carry=0, so when the priced entry finally fires the chain resets to qty=1.
- **Quote:** "TradingView empirical rule (probe 52 trade 113): the deferred-flip carry is the position size at THIS placement, not the original."

### Probe 54 -- pyramiding=1 same-direction same-bar market entries

- **Sources:** `src/engine_orders.cpp:507-509`
- **Semantic:** Two same-bar same-direction market entries with pyramiding=1: TV keeps only the first. Market entries respect the pyramiding limit; only flat-armed or pre-armed-opposite priced entries can bypass it.
- **Quote:** "in the SAME direction -- still respect the limit (probe 54's two same-bar same-direction market entries with pyramiding=1 must keep only the first one)."

### Probe 62 -- same-id stop replace, default-qty leak, longModify

- **Sources:** `src/engine_strategy_commands.cpp:178-186`, `tests/test_same_id_stop_replace.cpp`
- **Semantic:** `position_qty_` is undefined whenever `position_side_` is FLAT -- a default `position_qty_ = 1.0` would leak into `tv_carry_qty` for the first priced entry of any session, fabricating a phantom carry. Probe 62: longModify stop placed at 03:30 (warmup gate skipped longFirst at 03:15) captured carry=1 from the default qty, then fired with qty=2 instead of 1, breaking parity from trade #1.
- **Quote:** "a phantom carry. Probe 62 manifests this: the longModify stop placed at 03:30 ... captured carry=1 from the default qty, then fired later with qty=2 instead of 1, breaking parity from trade #1 onward."

### Probe 63 -- deferred-flip carry chain (group with 52/72/92)

- **Sources:** `src/engine_orders.cpp:427`, `src/engine_fills.cpp:213`, `include/pineforge/engine.hpp:161`, `tests/test_integration.cpp:3091`
- **Semantic:** Validates that priced entries placed during an OPPOSITE-direction position carry that position's qty forward; the new post-flip position grows by `qty + tv_carry_qty`.
- **Quote:** "TradingView's deferred-flip growth rule (probes 52, 63, 72, 92): a priced (stop/limit) entry that was placed while the strategy held an OPPOSITE-direction position carries that position's qty forward."

### Probe 72 -- pre-armed opposite-cycle siblings; S2 fires despite pyramiding=1

- **Sources:** `src/engine_orders.cpp:501-509`, `src/engine_fills.cpp:716`
- **Semantic:** Pyramiding applies at fill time EXCEPT for priced entries placed while position was FLAT or holding OPPOSITE direction. Probe 72's S2, placed while LONG via L2, emits the second-sibling short trade despite pyramiding=1 once the source position closes.
- **Quote:** "previous opposite-direction cycle that has since closed (probe 72's S2 placed while LONG'ing via L2 and TV emits the second-sibling short trade despite pyramiding=1)."

### Probes 80 / 80-87 -- flat-armed pyramid within bar; cross-bar bracket persistence

- **Sources:** `src/engine_fills.cpp:507-514, 713-715`, `tests/test_integration.cpp:748, 846`
- **Semantic:** Flat-armed priced entries firing the same bar in the same direction must both fill -- TV does not throttle pre-armed bracket legs the way it throttles fresh in-position priced entries. Brackets persist across bars: probes 80-87 confirm TV closes the position on a later bar when only one leg fired earlier and the opposite leg's stop is touched subsequently.
- **Quote:** "across bars: probes 80-87 confirm TV closes the position on a later bar when only one leg fired earlier and the opposite leg's stop is touched subsequently."

### Probe 83 -- dual-stop open-tie arbitration, long wins

- **Sources:** `src/engine_internal.hpp:75-83`, `src/engine_path_resolve.cpp:104-111`, `tests/test_integration.cpp:793-842`
- **Semantic:** When `bar.open` equals the stop level for both a long and a short flat-armed entry, both legs are armed at the open and TV picks the long leg (short becomes the bracket exit). For the open-equals-stop case both legs return path-position 0 simultaneously; the dual-stop arbitration breaks the tie in favor of the long leg. Source order does not matter.
- **Quote:** "arbitration breaks the tie in favour of the long leg -- this matches TV's broker emulator on probe 83."

### Probe 92 -- deferred-flip with daily `strategy.close_all`, cross-bar carry

- **Sources:** `src/engine_orders.cpp:435-439, 564-566`, `include/pineforge/engine.hpp:154-156`
- **Semantic:** Carry persists across bars: probe 92's daily cleanup closes the long at chart 12:15, the SE stop fires hours later at 21:30 and still applies the carry. `tv_carry_qty` MUST persist across bars rather than being per-bar transient. 328 qty=1 in-position flips and 20 qty=2 flips that all happen AFTER a same-day cleanup closed the long.
- **Quote:** "Verified empirically with probe 92's 20 deferred flips that fire after the daily strategy.close_all cleanup, hours after the closing bar -- so this MUST persist across bars rather than being a per-bar transient state."

### Probe 93 -- pyramiding=2 opposite-direction stops, cycle-scoped carry consumption (cycles A and B)

- **Sources:** `src/engine_orders.cpp:392-407, 455`, `src/engine_strategy_commands.cpp:165-172`, `src/engine_fills.cpp:247, 716`, `include/pineforge/engine.hpp:279-283`, `tests/test_strategy_pyramiding.cpp:12-22, 67-180`
- **Semantic:** Two cycles. Cycle B: a `strategy.close` call earlier in the same on_bar must be subtracted off because TV evaluates calls in source order. `pending_close_qty_in_bar_` accumulates qty of `strategy.close*` calls during the current on_bar; resets at the top of each bar. Close-before-entry: entry captures post-close size; entry-before-close: carry equals open position. Cycle A: opposite-direction stops armed during a long cycle -- first sibling grows by `|old|+qty=2`, second sibling fires fresh at qty=1.
- **Quote:** "Probe 93 cycle B refinement: a strategy.close call earlier in the SAME on_bar must be subtracted off because TV evaluates calls in source order."

### Probes 95 / 96 -- multi-cycle open-guaranteed deferred-flip chains

- **Sources:** `include/pineforge/engine.hpp:161`, `tests/test_strategy_pyramiding.cpp:20-22, 57-65, 131`
- **Semantic:** Multi-cycle chains where open-guaranteed stops (e.g. high*10, low*0.1) eliminate sub-bar precision so any qty mismatch must come from the carry rule. Probe 95 is the oracle: any per-leg PnL drift across the whole chain (~qty x mintick) indicates incorrect cycle-scoping.
- **Quote:** "~qty x mintick of per-leg PnL drift across the whole chain (probe 95 is the oracle: open-guaranteed stops eliminate sub-bar precision so any mismatch must come from the carry rule itself)."

### Probes 97 / 97a / 97b -- intraday-cap LATCH + cap-close pricing

- **Sources:** `src/engine_strategy_commands.cpp:85-96`, `src/engine_fills.cpp:298-475, 617`, `include/pineforge/engine.hpp:325-365`, `tests/test_intraday_cap_auto_close.cpp:15-18`, `tests/test_intraday_rollover_chart_tz.cpp:11-15`, `tests/test_oca_raw_pyramid_add.cpp:5-15`
- **Semantic:** Three sub-probes:
  - **97**: when the cap-triggering fill is a STOP entry that fired intra-bar (stop > bar.open for long, stop < bar.open for short), TV's synthetic "Close Position (Max number of filled orders in one day)" exit emits at the bar's FAVORABLE extreme (bar.high for long, bar.low for short), not the entry's stop trigger price. 152 cap-close trades in probe 97 verify this.
  - **97a**: short -> long MA-cross flip leaves the pre-existing buy-stop bracket alive; its `created_position_side` is SHORT but the live position is LONG. The `pre_armed_opposite_priced` semantic in `add_to_pyramid_market` admits the add even when pyramiding would reject it. Pre-fix: probe 97a lost 90 trades (96.0% -> 100.0% post-fix).
  - **97b**: when the entry filled AT bar.open (gap-fill or market, no intra-bar travel), TV's cap-close emits at `fill_price = bar.open`. 382/382 cap-closes at `fill_price = entry_price = bar.open` across 13 months. Pre-fix: 3459 engine trades vs 1957 TV trades (43% over-count, engine recharged the counter after each cap-cycle).
- **Quote:** "Verified empirically against validation probe 97b's tv_trades.csv: - 382 cap-close exits across 13 months of data (~one per chart-day where the cap fires). NOT multiple per day."

## Topical / family probes

### magnifier-dist-probe-01..08b -- magnifier wrong-side gap-fill at entry-bar open

- **Sources:** `src/engine_path_resolve.cpp:766-771`, `tests/test_magnifier_real_bars.cpp:183-184`
- **Semantic:** With magnifier ON, TV treats each lower-TF sub-bar's open as a fresh gap event and DOES fill wrong-side exits at the entry bar's open. 340 of 871 trades on probe-01 are wrong-side gap fills. Allow gap shortcut on the entry bar in magnifier mode only; without this, the legacy non-magnifier rule makes entry==exit for these trades.
- **Quote:** "entry bar's open (verified across magnifier-dist-probe-01..08b -- 340 of 871 trades on probe-01 are wrong-side gap fills)."

### ies-probe-08 -- margin gate at signal time, dynamic-qty over-leverage

- **Sources:** `src/engine_orders.cpp:461-465, 490-491`, `include/pineforge/engine.hpp:258-260`
- **Semantic:** TV margin check: `required_margin = qty * fill_price * margin_pct / 100`. If `required_margin > available equity`, TV silently rejects the fill. Default margin = 100 (1x leverage) so "position value <= equity". Without the gate, dynamic-qty strategies (community/IES, community/VCP) over-leverage on low-ATR bars and produce ~5x more trades than TV. The check happens at SIGNAL time (with signal-bar close), NOT at fill time. Empirically, matched-trade qty ratio in probe 08 was equal to `engine_equity / TV_equity`.
- **Quote:** "Reproduces the IES/VCP/ies-probe-08 entry-skip behaviour where dynamic-qty strategies over-leverage on low-ATR bars and produces ~5x more trades than TV."

### parity-probe-03..06 -- margin gate signal-time validation

- **Sources:** `src/engine_strategy_commands.cpp:99-113`, `src/engine_orders.cpp:483-491`
- **Semantic:** Validates that margin fires at signal time (with `current_bar_.close`) NOT at fill time (`next_bar.open`). Close-vs-open slippage routinely inflates overshoot from ~zero to ~$20; pre-fix engine rejected those at fill while TV accepted them at signal. 57/57 matched post-fix.
- **Quote:** "Verified empirically by parity-probe-04..06 (all 57/57 matched)."

### validation_oca/oca-three-way-probe-02 -- OCA cancel after full fill, oca_name plumbing

- **Sources:** `src/engine_fills.cpp:388-407`, `src/engine_strategy_commands.cpp:305-307, 354`
- **Semantic:** TV cancels CANCEL-group siblings only after the originating order is FULLY filled, not after the first contract. qty=4 long + qty=2 sibling A_TP: A_TP fills qty=2, position=2 remaining, A_SL stays alive until the second sibling fires. Plus: `strategy.exit`'s `oca_name` plumbing -- without it, all `strategy.exit`-issued orders shared an empty name and the first bracket's TP would silently leave the other bracket's legs intact. ~42% trade loss without the fix.
- **Quote:** "the first bracket's TP would silently leave the other bracket's legs intact (probe oca-three-way-02 lost ~42% of its trades)."

### validation_typed_matrix/typed-matrix-probe-01-bool-regime-mask -- chart vs exchange timezone divergence

- **Sources:** `include/pineforge/engine.hpp:1140-1167`, `tests/test_chart_timezone.cpp:20, 137`
- **Semantic:** Pre-fix the engine wrote chart TZ into `syminfo_.timezone`, which codegen reads as the default tz of the 1-arg `hour(time)`/`minute(time)`/`dayofweek(time)` form -- conflating two distinct TV concepts and silently shifting results by the chart-vs-exchange offset (Asia/Taipei vs UTC = +8h for crypto). The shift cascaded into `hour`-bucketed accumulators: the 24x7 `matrix<bool>` regime mask filled in 8 hours earlier than TV. Pre-fix trade counts: TV 773, engine 714; post-fix ~778.
- **Quote:** "validation_typed_matrix/typed-matrix-probe-01-bool-regime-mask, whose 24x7 matrix<bool> regime mask filled in 8 hours earlier than TV and produced ~9% trade-count divergence."

### parity-anomalies/equity-mirror -- TV non-determinism at 1x margin boundary

- **Sources:** `src/engine_strategy_commands.cpp:110-113`, slug renamed to `validation/anomaly-equity-mirror`
- **Semantic:** Full-equity sizing right at the 1x margin boundary, where TV's behavior is itself non-deterministic. Documented in `corpus/parity-anomalies/README.md`. The close-vs-open margin distinction is load-bearing here for `qty = strategy.equity / close` sizing patterns. piners should inherit the anomaly tier (not count as a parity failure).
- **Quote:** "parity-anomalies/equity-mirror (full-equity sizing right at the 1x boundary, where TV's behaviour is itself non-deterministic)."

## Renamed `basic/*` probes (97-105 series)

Per `publish-validation-corpus-design.md:84-96`, basic/* probes are renumbered to topical slugs in the public corpus. Renamed strategies are public-license clean-room rewrites; PineForge engine comments still reference the old names. The oracle needs to disambiguate engine-history-cited probe numbers (52, 62, 80, 83, 92, 93, 95-97) from published-corpus probe slugs (`validation/97-tp-sl-gap-reversal-oca` etc.).

| Slug | Source slug | Purpose hint |
|---|---|---|
| validation/97-tp-sl-gap-reversal-oca | basic/greedy | TP/SL gap-reversal with OCA (intersects 97/a/b cap-close work) |
| validation/98-inside-bar-engulf | basic/inside-bar | Inside-bar engulfing pattern |
| validation/99-keltner-channel-break | basic/keltner | Keltner channel breakout |
| validation/100-ma-dual-cross | basic/ma-cros | Dual moving-average cross |
| validation/101-parabolic-sar-flip | basic/parabolic-asr | Parabolic SAR flip (driver for directional-mintick rounding fix; 2,513 non-gap stop fills with one-sided +/- 0.01 bias, engine.hpp:456-460) |
| validation/102-pivot-extension-break | basic/pivot-ext | Pivot extension breakout |
| validation/103-stochastic-slow-cross | basic/stochastic-slow | Slow stochastic cross |
| validation/104-supertrend-flip | basic/supertrend | Supertrend flip |
| validation/105-volty-expansion-close | basic/volty-expan | Volatility-expansion close |

## Open items for the remaining ~207 probes

- Run an LLM pass over each `corpus/validation/<NN-slug>/strategy.pine` to produce a 1-3 sentence "what does this probe exercise" summary.
- Human-review the suspect cases (anything where the prose says "I think" or "probably").
- Consider upstreaming the polished summaries to PineForge so every consumer benefits.
- Pattern-match against PineForge's `docs/pine_v6_audit_master.md` known-divergences -- some probes likely exercise documented divergences and that linkage should be in the summary.
