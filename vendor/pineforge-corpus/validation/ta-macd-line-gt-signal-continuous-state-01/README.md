# ta-isolate-07-macd-line-gt-signal-continuous-state

## Purpose
Contrast against probe 03 (MACD crossover - event trigger). Uses value-comparison `macd > sig` every bar with continuous-state pattern.

If probe 03 excellent and probe 06 (RSI value-cmp) excellent but this drifts → bug in MACD value comparison `macd > sig` despite crossover being bit-exact (would imply boundary-equality drift at exact macd == sig).

## Setup + export
See `../README.md`. Apply strategy.pine, export `tv_trades.csv`.

## Diagnostic interpretation
| 07 result | Conclusion |
|---|---|
| `excellent` | MACD continuous-state value-cmp matches. 37-regex bug is in str.match OR AND-combination. |
| `strong/moderate` | Boundary-equality drift between TV and engine `macd > sig` at exact equality bars (despite crossover being bit-exact). Inspect engine signed-zero / NaN propagation in `>` operator. |
