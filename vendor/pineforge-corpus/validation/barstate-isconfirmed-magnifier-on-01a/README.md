# Barstate-magnifier probe 01a - isconfirmed under magnifier ON

**Gap targeted**: X6 partial / Pine #4 - barstate.isconfirmed semantics under magnifier.

**Expected behavior**: Excellent. Fill bar must be byte-identical to 01b (magnifier OFF). Any drift implicates is_first_tick_/is_last_tick_ flip on the last sub-bar.

**TV capture notes**:
- 15m ETH-USDT-USDT, magnifier ON.\n- Save TV trades as trades.csv. Compare bar list to 01b/trades.csv.
