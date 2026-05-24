# na-chain-probe-01-deep-history-int-na

## Gap
Cross-confirmed P1 (X9, Pine#5, C++#14): `na` propagation past `max_bars_back`, int-NA undefined behavior (`INT_MIN` sentinel collision), `nz` chain truncation, `na->bool` cast.

## Expected behavior
- `time(session)` returns `na<int64>` outside NY equity hours.
- `gatedSrc` is `na` outside session; `nz` only at the leaf (deep history reach `[499]`).
- RSI fed a partially-na source with deep history must not crash or emit phantom orders.
- Entry only when in-session and RSI crosses 30; >=10 closed trades expected.

## TV capture notes
- 15m chart, ETH-USDT-USDT, window must extend long enough to exercise the 500-bar history depth.
- Manual TradingView export → `tv_trades.csv`.
- Any divergence in the first ~500 bars points at `max_bars_back` priming differences.
