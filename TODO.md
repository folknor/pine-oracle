# TODO

Living list of pending work. External blockers separated from internal
polish. Each item should be deletable in one commit when done.

## Piners-syntax migration cleanups

The migration is complete: `src/syntax/` is gone, `piners-syntax` is on
a path dependency, and `validate`, `parse`, and `tokens` all flow through
`piners_syntax::*`. Remaining cleanup items:

- Replace `{script:#?}` Debug output in `pine parse` text mode
  (`src/commands/parse.rs:18`) with a stable structured printer once
  piners-syntax exposes a Display / pretty-printer for AST nodes.
  `ast.rs` currently has no Display impl. JSON is already the stable AST
  surface.

## OHLCV wiring

`piners-data` (in the piners workspace) ships full OHLCV machinery:
`load_ohlcv` / `load_ohlcv_multi`, parquet cache, provider sources
(Binance / Coinbase / Dukascopy / yfinance), timeframe aggregation. The
remaining work is in-repo wiring:

- Add `piners-data` as a path dependency.
- Bake TradingView-captured `expect.json` baselines under `indicators/`
  using real OHLCV bars. Existing `smoke-*` fixtures are deterministic
  substrate checks, not oracle-grade captures. Real TV baselines unlock
  treating `pine indicator --strict` as an oracle.
- Wire `trim_bars` / `warmup_bars` in `pine diff` (needs
  `ohlcv_first_ms` + `ohlcv_last_ms` + `bar_ms`). Currently skipped; full
  common-window trim used instead.
- Add cross-symbol / cross-timeframe `request.security(...)` TV fixtures
  once baseline OHLCV is in place.

## Distribution

- Homebrew tap setup. Tap name TBD.
- Pre-built binaries on GitHub Releases (linux-x86_64, macos-arm64,
  macos-x86_64, windows-x86_64). Cross-compile via cargo + matrix CI.
- `cargo install pine-cli` from crates.io publication. Requires final
  license confirmation (Open Q9 + Q10 already resolved; this is the
  publish step itself).

## CI

- GitHub Actions workflow: `brokkr check` on push + PR. Matrix: linux
  + macos at least. Cache the cargo registry + `vendor/pineforge-corpus/`
  to keep build under a minute.
- Release automation that builds the four-target binary matrix on tag
  push.

## Documentation

- Per-subcommand worked example in the README or a separate
  `docs/examples.md`. Show `pine lookup math.max --format json`,
  `pine behavior input`, `pine search magnifier --kind docs`, etc.
