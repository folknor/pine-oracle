# session-ismarket-nyse-rth-01

**Status:** TV_RUN_NEEDED

## Purpose

Parity probe for `session.ismarket`. Strategy gates all entries on
`session.ismarket` to ensure the engine correctly identifies bars inside the
NYSE regular trading hours (09:30-16:00 ET).

## Setup

- Symbol: AAPL  
- Timeframe: 5m  
- Date range: 2024-01-02 - 2024-03-29  
- Session override: `"0930-1600"` (NYSE RTH), timezone `America/New_York`

## TV capture instructions

1. Open AAPL, 5m chart in TradingView.
2. Add strategy from `strategy.pine`.
3. Set date range 2024-01-02 - 2024-03-29.
4. Export: Strategy Tester → List of Trades → save as `tv_trades.csv`.

## Known limitation

`session.isfirstbar_regular` and `session.islastbar_regular` are aliased to
`session.isfirstbar` / `session.islastbar` because the engine has a single
`syminfo.session` string and cannot distinguish RTH from ETH. Future work:
add `SymInfo.regular_session` field.
