# ta-isolate-02-rsi14-bands-30-70

## Purpose
Same isolated `ta.rsi(close, 14)` as probe 01 but with mean-reversion thresholds at 30 and 70. Cross-validates engine RSI at deeper extrema where Wilder smoothing can show different ULP-boundary behavior than mid-50 thresholds.

## Setup + export
See `../README.md`. Quick reference:
- Symbol/TF: ETH-USDT-USDT 15m, full warmup6m window
- Apply `strategy.pine` → Strategy Tester → List of Trades → CSV → `tv_trades.csv` next to this README

## Diagnostic interpretation
| Result | Conclusion |
|---|---|
| `excellent` AND probe 01 `excellent` | Engine RSI bit-exact across mid + extrema. RSI math fully validated. |
| `excellent` but probe 01 `strong/moderate` | Drift at the mid-50 boundary specifically. Investigate `crossover(x, 50.0)` precision. |
| `strong/moderate` while probe 01 `excellent` | Drift only at extrema - possible Wilder-saturation bug at RSI close to 0 or 100. Check engine RSI bounds + Pine clamp semantics. |
| Both `strong/moderate` | Global RSI drift OR OHLCV mismatch. Compare per-bar trace from Task B baseline. |
