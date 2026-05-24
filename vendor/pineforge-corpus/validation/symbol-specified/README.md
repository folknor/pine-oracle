# symbol-specified/ - probes that require non-default symbol data

Probes here require:

1. **OHLCV file** for the specific symbol (not in `corpus/data/` today -
   default bundled data is ETH-USDT-USDT only).
2. **SymInfo overrides** matching the symbol's exchange (`syminfo.session`,
   `syminfo.timezone`). The validator currently has no API to inject these
   per-probe - pending **pineforge-data** integration which will own
   per-symbol metadata.

## Layout

```
symbol-specified/
├── <SYMBOL>/                 # e.g. AAPL, QQQ, SPY
│   └── <probe-name>/
│       ├── strategy.pine
│       ├── inputs.json       # ohlcv_csv + engine_chart_timezone overrides
│       └── tv_trades.csv     # TV ground truth
```

## Status

These probes are **BLOCKED on pineforge-data**. They cannot reach excellent
parity in the validator until:

- The bundled OHLCV registry expands to cover the listed symbols, OR
- pineforge-data ships a metadata API to supply per-symbol OHLCV +
  syminfo to the harness on demand.

In the meantime they live here so:
- The Pine script is preserved and version-controlled.
- The engine surface they exercise (`session.is*`, `time_tradingday`) is
  fully tested via ctest (`tests/test_session_predicates.cpp`,
  `tests/test_time_tradingday.cpp`) using mocked session strings + tz
  inputs - so engine correctness is not gated on these corpus probes.

## Currently in this folder

| Symbol | Probe | Engine surface tested |
|---|---|---|
| `AAPL` | `session-ismarket-nyse-rth-01` | `session.ismarket` (NYSE 0930-1600 ET) |
| `AAPL` | `time-tradingday-daily-reset-counter-01` | `time_tradingday` (NYSE trading day) |
| `QQQ` | `session-ispremarket-nasdaq-01` | `session.ispremarket` (NASDAQ ETH) |
| `SPY` | `session-firstbar-vwap-anchor-01` | `session.isfirstbar` (NYSE open bar) |
| `SPY` | `session-lastbar-flatten-01` | `session.islastbar` (NYSE close bar) |

## Excluded from validator default sweep

The corpus CMake glob and validator's auto-discover scan
`validation/*` (depth 1 dirs). Nested probes under `validation/symbol-specified/`
are **not** picked up by the default sweep - by design - so they don't
contribute spurious "weak / missing OHLCV" failures to the headline parity
number. Validate them individually by passing their full path explicitly,
once their `ohlcv_csv` is provided.
