# process-orders-probe-01-on-close-vs-next-bar

**Gap addressed**: Pine#9 - `process_orders_on_close=false` (TV default) has zero corpus coverage today.

**Expected behavior**: excellent on its own TV export. Cross-compare with `process-orders-probe-02-on-close-true` (identical logic) reveals expected next-bar-open shift in fill prices and bar indices.

**Risk**: TV-default semantics drift; signal-bar vs next-bar fill timing.

**TV capture**: export TradingView "List of trades" to `tv_trades.csv` once captured. Same window as `data/ohlcv_ETH-USDT-USDT_15m.csv`. Diff against probe-02 export to validate next-bar shift.
