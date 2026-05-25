# TODO

Living list of pending work. External blockers separated from internal
polish. Each item should be deletable in one commit when done.

## Waiting on piners-syntax 0.1

When piners-syntax ships its public API (spec sketched in the design doc
and earlier chat), swap the pinecone lift out:

- Add `piners-syntax = "0.1"` to `Cargo.toml`; drop `thiserror` (piners-
  syntax owns its own error types).
- Delete `src/syntax/{mod.rs, ast.rs, lexer.rs, parser.rs}` and the
  `vendor/pine-syntax/` tree (LICENSE, NOTICE, 72 testdata fixtures).
- Rewrite `src/validate.rs::check` to call
  `piners_syntax::validate(source, &builtins())` where `builtins()`
  builds a `BuiltinsTable` from `behavior::index()`. Map every
  `piners_syntax::Diagnostic` to local `Diagnostic`.
- Rewrite `cmd_parse` to call `piners_syntax::parse`, `cmd_tokens` to
  call `piners_syntax::lex`. Update the AST pretty-printer in `main.rs`
  to render piners-syntax's `Program` / `Stmt` / `Expr` shapes.
- Update `docs/pine-oracle.md` Architecture section + `AGENTS.md` to
  drop the "lifted from pinecone, temporary" framing. Mark Open Q1 fully
  resolved (currently flagged as "near-term").

Estimated effort: half a day mechanical work once piners-syntax is on
crates.io or pinned by git ref.

## Waiting on piners' engine + OHLCV bake

- `pine indicator --strict <slug>`: per-bar parity oracle. Requires
  piners' runtime + an `.expect.json` fixture per indicator + the OHLCV
  feed baked into the binary (~75 MB for the four ETH/USDT-USDT CSVs).
  Schema for `expect.json` already filed in `docs/pine-oracle.md` Open Q11
  (`schema_version` + `pine_version` + `tv_snapshot` + outputs map).
- `pine diff` interior trim: `trim_bars` / `warmup_bars` honouring needs
  `ohlcv_first_ms` + `ohlcv_last_ms` + `bar_ms`. Currently skipped; full
  common-window trim used instead. Will land alongside the OHLCV bake.

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
- The AST pretty-printer renders type annotations as raw strings
  (`: int`). When piners-syntax lands and exposes structured types,
  print qualifier + type kind separately.

## Internal polish (large / deferred)

- Multi-error parser recovery in `src/syntax/parser.rs`. **Throwaway
  work** - piners-syntax replaces this; doing it twice is wasted
  effort. Listed for completeness only.

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
