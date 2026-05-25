# TODO

Living list of pending work. External blockers separated from internal
polish. Each item should be deletable in one commit when done.

## Piners-syntax migration cleanups

The migration is complete: `src/syntax/` is gone, `piners-syntax` is on
a path dependency, and `validate`, `parse`, and `tokens` all flow through
`piners_syntax::*`. Remaining cleanup items:

- Drop `thiserror` from `Cargo.toml` once `piners-syntax` exports its own
  error types publicly and no local code needs `thiserror` directly.
- Update `docs/pine-oracle.md` Architecture section + `AGENTS.md` to drop
  any remaining "lifted from pinecone, temporary" framing.

## Waiting on OHLCV bake

- `pine indicator --strict <slug>`: runner substrate is shipped; real
  TradingView-captured `expect.json` baselines are pending. Existing
  `smoke-*` fixtures are deterministic substrate checks, not oracle-grade
  captures. Bake TV baselines per indicator before treating `--strict` as
  an oracle.
- `pine diff` interior trim: `trim_bars` / `warmup_bars` honouring needs
  `ohlcv_first_ms` + `ohlcv_last_ms` + `bar_ms`. Currently skipped; full
  common-window trim used instead. Will land alongside the OHLCV bake.
  Also blocked: cross-symbol / cross-timeframe `request.security(...)` TV
  fixtures (OHLCV feed not yet baked).

## Waiting on pine-tools schema evolutions

- Whenever `pine-tools/pine-data/v6/*.json` adds fields, re-vendor by
  copy and verify `src/behavior.rs` consumes them. `#[serde(default)]`
  on every optional field absorbs additions for free; removals or
  renames need code changes.
- If `examples: string[]` ever changes shape again (currently per-function
  array of strings with `\n` preserved), update `RawFunction::examples`
  and the `print_behavior_text` enumeration.

## Internal polish (small)

- Replace `Box::leak`-per-summary in `corpus::build_summary_index` with
  a single arena allocator (e.g. `&'static [u8]` slab) if leak count
  ever becomes a memory concern. Bounded today at 239 entries with
  median ~650 chars each (~150 KB total); not urgent.
- `pine parse` text mode uses `{:#?}` Debug output because piners-syntax
  has no stable pretty-printer yet. When piners-syntax exposes a
  structured printer, replace Debug with it.

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
- `docs/diagnostics.md` mapping `pine validate` diagnostic codes to
  human-readable explanations - lands when piners-syntax brings the
  stable `code` field on `Diagnostic`.
