# ta-isolate-06-rsi14-gt-50-continuous-state

## Purpose
Contrast against probe 01 (`ta.crossover(rsi, 50)` event trigger). This probe uses **value-comparison** (`rsi > 50`) every bar with the **continuous-state** pattern from `validation/37-regex-string-filter`:

```
if cond
    strategy.entry("L", strategy.long)
else
    strategy.close("L")
```

If probe 01 excellent but this drifts → engine bug is in continuous-state strategy.entry/close sequencing, not RSI math.

## Setup + export
See `../README.md`. ETH-USDT-USDT.P 15m, full warmup6m window. Apply strategy.pine, export trade list as `tv_trades.csv`.

## Diagnostic interpretation
| 06 result | Conclusion |
|---|---|
| `excellent` | Engine handles continuous-state pattern correctly. 37-regex bug is in str.match OR MACD-side OR AND-combination. Check probes 07 + 08. |
| `strong/moderate` | Engine continuous-state strategy.entry/close diverges from TV. Inspect: same-bar entry-after-close ordering, `strategy.close` while flat, or `strategy.entry` no-op semantics under pyramiding=1. |
