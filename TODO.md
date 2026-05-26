# TODO

Living list of pending work. External blockers separated from internal
polish. Each item should be deletable in one commit when done.

## Piners-syntax migration cleanups

The migration is complete: `src/syntax/` is gone, `piners-syntax` is on
a path dependency, and `validate`, `parse`, and `tokens` all flow through
`piners_syntax::*`. Remaining cleanup items:

- Replace `{script:#?}` Debug output in `po parse` text mode
  (`src/commands/parse.rs:18`) with a stable structured printer once
  piners-syntax exposes a Display / pretty-printer for AST nodes.
  `ast.rs` currently has no Display impl. JSON is already the stable AST
  surface.

## OHLCV wiring

Sub-tasks A and B done. C remains.

Background: upstream pineforge-corpus ships a `data/` directory
(~75 MB of Binance ETH/USDT OHLCV across 4 feeds) that pine-oracle
distills into `vendor/pineforge-corpus/data/ohlcv_spans.json` at
vendor time and then drops the CSVs themselves. The diff side
consults the spans via `corpus::ohlcv_span_for_probe` when
`inputs.json` doesn't carry the span explicitly.

`piners-data` (in the piners workspace) was considered as a fetch
backend, but it's a heavy dep tree (tokio full, reqwest, parquet,
arrow, chrono-tz). Keeping the bake script purely Python keeps
piners-data out of pine-oracle entirely.

### Sub-task C: real TradingView indicator baselines

Capturing TV `expect.json` baselines is a manual TV-side process
(open the indicator on a chart, export values). Existing `smoke-*`
fixtures stay as substrate checks; real TV baselines slot in
alongside them under `indicators/<slug>/`.

Cross-symbol / cross-timeframe `request.security(...)` fixtures
need two OHLCV streams (source + requested). The 1m and 15m feeds
are already available via the corpus-data clone path used by the
span bake; replay them through piners-runner once a TV-captured
`expect.json` exists.

## Documentation

- Per-subcommand worked example in the README or a separate
  `docs/examples.md`. Show `po lookup math.max --format json`,
  `po behavior input`, `po search magnifier --kind docs`, etc.
