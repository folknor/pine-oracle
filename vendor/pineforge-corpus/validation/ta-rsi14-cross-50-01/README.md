# ta-isolate-01-rsi14-cross-50

## Purpose
Isolate `ta.rsi(close, 14)` (Wilder seed + smoothing) end-to-end. Drift on this probe narrows root cause to:
- RSI math: engine `RMA::compute` / `RSI::compute` (now Pine bit-exact post fc22c34) - should be 0 ULP drift
- OHLCV input drift: TV chart's bar OHLC ≠ shipped corpus OHLCV - see `../README.md` step 1

Indirectly diagnoses other RSI(14)-consuming probes in this corpus (e.g. the regex / string-filter and typed-matrix bool-regime-mask surfaces) by isolating the RSI math from the surrounding strategy logic.

## Setup + export
See `../README.md` for the full workflow (chart-feed verification, scraper-based OHLC check, validator command, deeper per-bar diff via `tv_ta_basic_helper.pine` which already publishes RSI(14)).

Quick reference:
- Symbol/TF: ETH-USDT-USDT 15m, full warmup6m window
- Apply `strategy.pine` → Strategy Tester → List of Trades → CSV → `tv_trades.csv` next to this README

## Diagnostic interpretation
| Result | Conclusion |
|---|---|
| `excellent` (0 trade-count drift, pnl_p90 < 0.001) | Engine RSI(14) bit-exact vs TV. 37-regex / typed-matrix-01 gaps are not caused by RSI math - they're OHLCV-input-side OR a non-RSI part of those strategies. |
| `strong/moderate` 1-3 trade drift | Threshold-equality tie-break at `rsi == 50.0` ULP boundaries. Investigate engine `crossover` precision around exact threshold. |
| `moderate/weak` >5% trade drift | RSI math drifts. Deep-dive via `fetch_tv_indicator.py` + `tv_ta_basic_helper.pine` (RSI(14) already published) → diff per-bar against engine `// @pf-trace`. |
