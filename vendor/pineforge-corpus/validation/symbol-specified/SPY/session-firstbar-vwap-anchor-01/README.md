# session-firstbar-vwap-anchor-01

**Status:** TV_RUN_NEEDED

## Purpose

Parity probe for `session.isfirstbar`. Anchors a running VWAP accumulator to
the first bar of each trading session and gates entries on
`session.ismarket`.

## Setup

- Symbol: SPY  
- Timeframe: 5m  
- Date range: 2024-01-02 - 2024-03-29  
- Session override: `"0930-1600"`, timezone `America/New_York`

## TV capture instructions

1. Open SPY, 5m chart in TradingView.
2. Add strategy from `strategy.pine`.
3. Set date range 2024-01-02 - 2024-03-29.
4. Export: Strategy Tester → List of Trades → save as `tv_trades.csv`.

## Known limitation

`session.isfirstbar_regular` is aliased to `session.isfirstbar` because the
engine uses a single `syminfo.session` string. See session_time.hpp header.
