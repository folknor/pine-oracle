# pyramid-carry-probe-02-cash-fractional

**Gap addressed**: X5 / Pine#10 / C++#7 - `default_qty_type=cash` + `commission.cash_per_contract` over fractional contract counts with pyramiding.

**Expected behavior**: parity-anomaly. Rounding drift between TV's cash→contract conversion and runtime is expected; document the divergence per pyramid add. `margin_liquidation_price` currently returns `na` in runtime.

**Risk**: per-add fractional rounding direction, commission double-counting on fractional contracts, equity drift accumulating across pyramid stack.

**TV capture**: export TradingView "List of trades" to `tv_trades.csv` once captured. Same window as `data/ohlcv_ETH-USDT-USDT_15m.csv`.
