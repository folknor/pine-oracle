# ta-isolate-04-hma-55-close-cross

## Purpose
Isolate `ta.hma(close, 55)`. HMA = `wma(2*wma(src, n/2) - wma(src, n), sqrt(n))`. Validates:
- WMA partial-window seed (n=27 first inner WMA, n=55 second inner WMA)
- Nested WMA composition with intermediate negative weights via `2*wma_half - wma_full`
- `sqrt(n)` rounding: `sqrt(55) = 7.416...` → Pine floors to **7**; verify engine matches

Targets the community-style market-structure-shift HMA logic (this corpus does not redistribute that script) and any other HMA(55)-consuming surface in this tree.

## Setup + export
See `../README.md`. Quick reference:
- Symbol/TF: ETH-USDT-USDT 15m, full warmup6m window
- Apply `strategy.pine` → Strategy Tester → List of Trades → CSV → `tv_trades.csv` next to this README

## Diagnostic interpretation
| Result | Conclusion |
|---|---|
| `excellent` | Engine HMA(55) bit-exact. MarketShift HMA path not at fault - check probe 05 SMA(152) and the strategy's composition logic. |
| `strong/moderate` 1-3 trades | Threshold-equality at `close == hma` ULP boundaries. Inspect engine `crossover` boundary semantics. |
| `moderate/weak` significant drift | Engine HMA math drifts. Likely candidates: WMA seed bar index, `sqrt(n)` rounding (must match Pine `math.floor(math.sqrt(n))`), nested-WMA weight chain. **HMA(55) is NOT in tv_ta_basic_helper.pine** - extend that helper with `float t_hma55 = ta.hma(close, 55)` and re-publish, or fork as `tv_ta_isolate_helper.pine`, then per-bar diff. |
