# process-orders-probe-02-on-close-true

**Gap addressed**: Pine#9 - companion to probe-01. `process_orders_on_close=true` matches current corpus default; same-bar-close fills.

**Expected behavior**: excellent on its own TV export. Acts as the baseline reference for cross-compare with probe-01.

**Risk**: same-bar fill ordering vs intra-bar signal evaluation.

**TV capture**: export TradingView "List of trades" to `tv_trades.csv` once captured. Same window as `data/ohlcv_ETH-USDT-USDT_15m.csv`. Diff against probe-01 export to validate next-bar shift.
