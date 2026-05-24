# Magnifier dist probe 01-endpoints - ENDPOINTS

**Gap targeted**: X3 (Pine #22 + C++ #11) - magnifier 6 distributions (11/12 codepaths cold).

**Expected behavior**: Excellent (parity baseline). TV uses ENDPOINTS so this dir is the TV-export ground truth for the magnifier-dist family.

**TV capture notes**:
- 15m ETH-USDT-USDT, full corpus window.
- Magnifier ON in TV settings (TV default = ENDPOINTS).
- Save TV List of trades as trades.csv. Other 5 sibling dirs reuse this baseline; their runs are harness-only.
