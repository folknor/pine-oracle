# typed-matrix-probe-01-bool-regime-mask

## Gap
Cross-confirmed P0 (X7): typed matrix dispatch - `matrix.new<bool>` bool-proxy storage, `set/get/transpose` round-trip, bool-matrix reductions flowing into entry conditions. Zero corpus coverage today.

## Expected behavior
- Bool matrix should round-trip through `matrix.transpose` losslessly.
- `hotCount` accumulates monotonically across the corpus window, gating entries once it crosses 6.
- >=10 closed long trades on the 15m ETH-USDT-USDT window.

## TV capture notes
- 15m chart, ETH-USDT-USDT, same window as `data/ohlcv_ETH-USDT-USDT_15m.csv`.
- Manual TradingView export → `tv_trades.csv` in this folder.
- Watch for any first-bar mask init drift between Pine and PineForge.

## ohlcv_start_ms requirement

`inputs.json` pins `ohlcv_start_ms: 1743379200000` (2025-03-31 00:00 UTC). This trims the
warmup CSV so the mask's bucket-fill depth matches the history depth visible on the TV chart
used for the `tv_trades.csv` export. Without this pin, engine and TV diverge because TV's
`hour(time)` / `dayofweek(time)` accumulation starts from a different bar than the engine's
warmup window, producing a different `hotCount` trajectory and different entry timing.
