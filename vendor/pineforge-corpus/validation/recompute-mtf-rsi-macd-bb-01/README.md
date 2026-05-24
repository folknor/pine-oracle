# TA recompute probe 01 - MTF RSI/MACD/BB

**Gap targeted**: X6 (Pine #4 + C++ #5) - per-class TA save/restore under request.security.

**Expected behavior**: Excellent. Three concurrent TA classes (RSI, MACD-hist, BB middle) inside 1H request.security from 15m chart force the runtime to push/pop saved series state for three distinct classes within the same MTF context. Trade list should match TV exactly; any drift implicates per-class state corruption.

**TV capture notes**:
- 15m ETH-USDT-USDT, full corpus window.
- Magnifier OFF in TV settings.
- Save TV List of trades as trades.csv in this folder.
