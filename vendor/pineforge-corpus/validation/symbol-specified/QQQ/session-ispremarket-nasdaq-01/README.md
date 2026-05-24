# session-ispremarket-nasdaq-01

**Status:** TV_RUN_NEEDED

## Purpose

Parity probe for `session.ispremarket`. Strategy enters long positions during
the pre-market window (04:00-09:30 ET) on bullish bars and closes them at
regular market open.

## Setup

- Symbol: QQQ  
- Timeframe: 5m  
- Date range: 2024-01-02 - 2024-03-29  
- Session override: `"0930-1600"`, timezone `America/New_York`

Pre-market window assumed: 04:00 - RTH_open (standard US ETH).

## TV capture instructions

1. Open QQQ, 5m chart in TradingView with extended hours enabled.
2. Add strategy from `strategy.pine`.
3. Set date range 2024-01-02 - 2024-03-29.
4. Export: Strategy Tester → List of Trades → save as `tv_trades.csv`.
