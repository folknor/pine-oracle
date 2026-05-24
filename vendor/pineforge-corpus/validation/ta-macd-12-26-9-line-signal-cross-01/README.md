# ta-isolate-03-macd-12-26-9-cross

## Purpose
Isolate `ta.macd(close, 12, 26, 9)` as `EMA(12) - EMA(26)` with `EMA(macd, 9)` signal line. Validates engine EMA chain depth + tuple destructuring. Distinct from `validation/01-macd-histogram` (which uses histogram-zero crossings).

Targets the MACD path used by the regex / string-filter probe in this tree.

## Setup + export
See `../README.md`. Quick reference:
- Symbol/TF: ETH-USDT-USDT 15m, full warmup6m window
- Apply `strategy.pine` → Strategy Tester → List of Trades → CSV → `tv_trades.csv` next to this README

## Diagnostic interpretation
| Result | Conclusion |
|---|---|
| `excellent` AND probe 01 `excellent` | EMA chain bit-exact. 37-regex MACD path is not the bug - investigate its RSI side or its codegen. |
| `excellent` but probe 01 `strong/moderate` | RSI-specific bug, MACD-EMA path is fine. |
| `strong/moderate` | Engine EMA(close, N) or chained EMA(EMA, 9) drifts. Check `ta_moving_averages.cpp::EMA::compute` seed (TV uses `ta.sma(src, length)` for first valid bar). Per-bar diff via tv_ta_basic_helper.pine (MACD already published). |
