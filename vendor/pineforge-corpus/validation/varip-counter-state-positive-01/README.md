# varip-counter-state-positive-01

## Purpose

Validates `varip` keyword passthrough in PineForge. As of Pine v6 sprint
(2026-05), `varip` is treated as `var` in batch mode: a warning is emitted
but compilation succeeds and the variable behaves identically to `var`.

This probe uses `varip int counter` incrementing every bar, triggering entries
every 50 bars. In PineForge batch mode, the counter accumulates bar-to-bar
(no intrabar tick resets), which is identical to TradingView's `var` semantics
on confirmed bars.

## Expected behavior
- `transpile(...)` succeeds. Codegen emits a `static int counter = 0;` member
  (same as `var int counter`).
- A single diagnostic with `level=WARNING` and message containing "varip" is
  emitted (no ERROR).
- Strategy produces long trades at every 50th bar, closed 5 bars later.
- PineForge trade list matches TradingView export when `varip` counter semantics
  are bar-aligned (as they are in all confirmed-bar strategies).

## TV capture notes (TV_RUN_NEEDED)
- Symbol: ETHUSDT.P, TF: 15m, Range: 2024-01-01 to 2024-06-01
- Export the List of Trades panel → save as `tv_trades.csv` in this directory.
- The strategy is deterministic (counter-based, no price crossover), so TV/PF
  should produce exactly the same entry/exit bars.

## Deviations from TradingView
In TradingView's realtime mode, `varip` would persist intrabar (tick-by-tick).
In PineForge batch mode, only bar-close values are processed, so `varip`
and `var` produce identical results for confirmed-bar increment strategies.
No accuracy degradation expected for this pattern.
