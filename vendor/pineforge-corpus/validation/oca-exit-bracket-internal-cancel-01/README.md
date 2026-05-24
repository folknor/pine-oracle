# oca-three-way-probe-01-same-bar-touch

**Gap addressed**: X4 / Pine#11 / C++#8 - OCA `NONE` vs `CANCEL` vs `REDUCE` three-way routing on a single OHLC waypoint.

**Expected behavior**: excellent. All three OCA groups fire on the same bracket geometry; runtime must route fills per `oca_type`.

**Risk**: REDUCE qty math, CANCEL ordering vs OCA siblings under wide-range / gap bars.

**TV capture**: export TradingView "List of trades" to `tv_trades.csv` once captured. Same window as `data/ohlcv_ETH-USDT-USDT_15m.csv`.
