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

Three orthogonal sub-tasks, not one. Background: upstream
pineforge-corpus ships a `data/` directory (~75 MB of Binance ETH/USDT
OHLCV) that pine-oracle intentionally dropped during vendoring
(`vendor/pineforge-corpus/VENDORING_NOTES.md`). That CSV data is what
`verify_corpus.py:_ohlcv_span_ms` reads to compute
`first_ms` / `last_ms` / `bar_ms` for interior trim.

`piners-data` (in the piners workspace) ships full OHLCV machinery:
`load_ohlcv` / `load_ohlcv_multi`, parquet cache, providers (Binance /
Coinbase / Dukascopy / yfinance), timeframe aggregation. It is a heavy
dep tree (tokio full, reqwest, parquet, arrow, chrono-tz), so the
right shape is to use it as a vendoring-time / dev-time tool, not as a
pine-oracle runtime dependency.

### Sub-task A: `pine diff` interior trim (no new runtime deps)

Pure Rust port of `verify_corpus.py::interior_time_bounds` plus the
plumbing in `diff::align_pairs`. Drives off
`InputsMeta { ohlcv_first_ms, ohlcv_last_ms, bar_ms }`. When metadata
is absent the behavior matches today (skip trim); when present,
interior trimming kicks in.

Doable now without baking any OHLCV; the diff-side machinery is the
gating change. Population of the metadata is sub-task B.

### Sub-task B: bake per-probe OHLCV span metadata

Write a vendoring-time script (probably `scripts/bake-ohlcv-spans.{sh,
py,rs}`) that uses `piners-data` or reads the upstream `data/` CSVs to
emit a tiny `ohlcv_span.json` sidecar per probe under
`vendor/pineforge-corpus/validation/<slug>/` with
`{first_ms, last_ms, bar_ms}`. Bake via `include_dir` like the rest of
the corpus. KB per probe, not MB. piners-data stays out of the
shipped binary.

Refresh procedure becomes: re-clone corpus, run prune script, run
bake-ohlcv-spans script.

### Sub-task C: real TradingView indicator baselines

Independent of A/B. Capturing TV `expect.json` baselines is a manual
TV-side process (open the indicator on a chart, export values),
unblocked neither by piners-data nor by the OHLCV bake. Existing
`smoke-*` fixtures stay as substrate checks; real TV baselines slot in
alongside them under `indicators/<slug>/`.

Cross-symbol / cross-timeframe `request.security(...)` fixtures need
two OHLCV streams (source + requested); after sub-task B those bars
are available as parquet/CSV, ready for piners-runner to consume.

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
