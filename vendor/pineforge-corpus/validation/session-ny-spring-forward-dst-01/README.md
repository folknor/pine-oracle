# session-dst-probe-01-ny-spring-forward

## Gap
P1 (X8, Pine#7, C++#12): session-time + DST behavior, `pine_tz::ScopedTimezone` correctness across spring-forward, `time(session)` returning `na<int64>` on Sunday.

## Expected behavior
- `time("0930-1600", "America/New_York")` is `na` on Sundays and outside session.
- Session-open / session-close transitions fire exactly once per trading day.
- Entry counts pre-DST vs post-DST should match wall-clock session boundaries (the 02:00 -> 03:00 spring-forward Sunday is non-trading; first post-DST entry should be Monday 09:30 ET).
- >=10 closed trades over a corpus window spanning the second Sunday of March.

## TV capture notes
- 15m chart, ETH-USDT-USDT.
- The corpus window MUST include a US spring-forward weekend (second Sunday of March).
- Manual TradingView export → `tv_trades.csv`.
- Any drift in entry timestamps by exactly 1 hour around the DST boundary indicates tz-db or `ScopedTimezone` divergence.
