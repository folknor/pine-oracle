# ta-isolate-10-typed-matrix-explicit-utc

## Purpose
typed-matrix-probe-01-bool-regime-mask uses bare `hour(time)` / `dayofweek(time)`. Pine resolves these to `syminfo.timezone` (exchange TZ). Engine defaults `syminfo_.timezone = "UTC"`. If TV's BINANCE:ETHUSDT.P syminfo.timezone ≠ "UTC", engine's hour/dayofweek differ from TV's by that offset → different mask buckets → different hotCount/sample gating → different trade timing.

This probe pins both sides to `"UTC"` explicitly via `hour(time, "UTC")` / `dayofweek(time, "UTC")`. Pine v6 honours explicit tz arg over syminfo.

## Diagnostic
| Result | Conclusion |
|---|---|
| `excellent` | typed-matrix-01 bug is engine `syminfo_.timezone="UTC"` ≠ TV's BINANCE:ETHUSDT.P syminfo.timezone. Engine fix: validator passes BINANCE's syminfo.timezone to engine via `syminfo_` (in addition to `chart_timezone_`). |
| `strong/moderate` | TZ NOT the cause. Bug in matrix bool dispatch (set/get/transpose) OR hotCount accumulator OR sample retrieval. Add probe 11 with mask-only access (skip transpose) to localize further. |

## Setup + export
See `../README.md`. Apply `strategy.pine`, export `tv_trades.csv`.
