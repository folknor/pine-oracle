# pyramid-carry-probe-01-deferred-flip

**Gap addressed**: X5 / Pine#10 / C++#7 - pyramiding>1 with deferred-flip entries surviving `strategy.close_all`.

**Expected behavior**: excellent. Pending stop-flip entries queued before `close_all` must either be cancelled or correctly resequenced (close-then-flip) per TV semantics.

**Risk**: carry capture order in single `on_bar`, stale stop-entry leftover after flat reset, pyramiding stack reset boundary.

**TV capture**: export TradingView "List of trades" to `tv_trades.csv` once captured. Same window as `data/ohlcv_ETH-USDT-USDT_15m.csv`.
