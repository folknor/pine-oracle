# session-lastbar-flatten-01

**Status:** TV_RUN_NEEDED

## Purpose

Parity probe for `session.islastbar`. Strategy enters on SMA crossovers
within the regular session and force-closes all positions on the last session
bar to avoid overnight exposure.

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

`session.islastbar_regular` is aliased to `session.islastbar` in the engine.
The islastbar detection uses one-bar lookahead on the simple loop path; in
the aggregation path it fires on `barstate.islast` as an approximation.
