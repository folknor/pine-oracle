# varip-var-udt-in-security-positive-01

## Purpose

Confirms that `var` UDT state mutated inside a `request.security` callable compiles
and produces trades. This probe uses `var` (not `varip`) - it is a positive smoke
test for security-callable state persistence.

As of Pine v6 sprint (2026-05): `varip` is now a **warning** (not an error) in
PineForge. `varip` is treated as `var` in batch mode - no intrabar tick semantics
are provided. This probe remains a purely `var`-based test; the new
`varip-counter-state-positive-01` probe demonstrates `varip` passthrough directly.

## Expected behavior
- `transpile(...)` succeeds with no errors (no `validation_overrides.expect_reject`).
- `var Counter c` inside the security callable persists across HTF bars.
- Each HTF EMA crossover increments `c.hits`; chart-side `htfFired` flags the bar
  where the counter advances.
- Strategy produces long trades on the 15m ETH-USDT-USDT window.

## TV capture notes
- 15m chart, ETH-USDT-USDT, same window as `data/ohlcv_ETH-USDT-USDT_15m.csv`.
- Manual TradingView export → `tv_trades.csv`.
- Any divergence in HTF crossover counts indicates security-callable state drift.
