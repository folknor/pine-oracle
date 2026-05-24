# oca-three-way-probe-02-multi-group-partial

**Gap addressed**: X4 / Pine#11 / C++#8 - multi-group OCA isolation with partial-fill cancellation across two distinct groups.

**Expected behavior**: excellent. Group A (CANCEL) and Group B (REDUCE) must operate independently; partial fill in B reduces sibling qty without leaking into A.

**Risk**: cross-group accounting drift, REDUCE remaining-qty math under partial fills, same-bar order processing sequence.

**TV capture**: export TradingView "List of trades" to `tv_trades.csv` once captured. Same window as `data/ohlcv_ETH-USDT-USDT_15m.csv`.
