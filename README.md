# pine-oracle

A single-binary CLI that answers Pine Script v6 semantic questions.
Reference lookups, structured signatures, BM25 search, parsing, validation,
corpus probes, trade-list parity scoring - all baked into one Rust binary
with no on-disk state, no network calls (except `validate --strict`), no
runtime configuration.

## What it does

| Command | Output |
|---|---|
| `po lookup <name>` | Function / constant / variable details from TradingView's v6 reference |
| `po search <query> [--kind reference\|probe\|audit\|docs\|behavior]` | BM25 across v6 reference, baked corpus, PineForge audit/doc pages, and structured pine-data behavior; `--kind` is case-insensitive, and `--kind ?` lists source kinds |
| `po behavior <name>` / `po behavior [TEXT] --list [--kind function\|variable\|constant\|keyword] [--grep TEXT]` | Structured signature + polymorphism + argument ordering from pine-tools' JSON exports; list-mode `TEXT` acts as grep, `--kind` is case-insensitive, and `--kind ?` lists behavior kinds |
| `po parse <code-or-file>` / `--code CODE` / `--file PATH` / `-` | piners-syntax AST tree |
| `po tokens <code-or-file>` / `--code CODE` / `--file PATH` / `-` | piners-syntax lexer token stream |
| `po validate <code-or-file>` / `--code CODE` / `--file PATH` / `-` | Local lex + parse + type + semantic diagnostics from piners-syntax, backed by piners-runtime builtins plus pine-data gap-fill; text diagnostics include source-line caret frames |
| `po validate --strict <code-or-file>` | POST to TradingView's pine-lint endpoint. Yes/no oracle; diagnostic prose is non-actionable |
| `po probe <slug>` | Pull a baked PineForge corpus probe (strategy.pine + tv_trades.csv + author-extracted summary) |
| `po probes [--grep TEXT] [--feature NAME]` | List baked probes; `--grep` matches slug or summary substring, `--feature` restricts by detected Pine-feature usage (`oca`, `trail`, `pyramiding`, `mtf`, `varip`, `magnifier`, `matrix`, `map`, `udt`, `method`, `process_orders_on_close`, `barstate_isfirst`; pass `?` to list the catalog) |
| `po diff <probe> <trades.csv> [--show-diffs N]` | Tier-classify a user trade list against the probe's TV ground truth (port of PineForge's verify_corpus.py); `--show-diffs N` emits the worst-N matched pairs plus every TV / user orphan |
| `po indicator --list [--grep TEXT] [--baseline smoke\|tv]` | List baked indicator strict fixtures with smoke vs TV baseline kind, symbol/timeframe, bar count, output count, and range window; `--baseline ?` lists baseline kinds |
| `po indicator <slug> [--metadata-only]` | Inspect one baked indicator fixture: source, bar window, expected output keys, actual runner output keys, first/last expected values, tolerance, and baseline metadata; `--metadata-only` skips the runner key check |
| `po indicator <slug> --actual` | Run one fixture through piners-runner and print actual output keys + series without comparing against `expect.json`; JSON includes a runner-only `runner_expect` object shaped like fixture `expect.json` for smoke authoring |
| `po indicator --strict <slug>` / `po indicator --strict --all [--grep TEXT] [--baseline smoke\|tv]` | Run baked indicator fixtures through piners-runner and diff per-bar outputs against `expect.json`, including optional `test_range` windows; expected outputs may use plot titles, with duplicate titles disambiguated as `#1`, `#2`, etc.; `smoke-*` fixtures cover plots, warmup, ranges, bool `plotshape`, and same-symbol `request.security`; TV-captured baselines are still pending |
| `po version` | Binary version + bake counts and pinned snapshot metadata (reference, corpus, PineForge docs, pine-data, indicator fixtures split smoke vs TV) |

All subcommands accept `--format json|text|auto`. JSON outputs carry
`schema_version: 1`. Object payloads attach the version inline; arrays wrap
as `{schema_version, items}`. Text output accepts `--quiet` to suppress
non-data headers, snippets, status lines, and validation notes where a command
emits them. Bump rules live in `docs/pine-oracle.md`.

## Install

```
cargo install --path .
```

Binary lands as `po`. ~75-80 MB stripped (the PineForge corpus is the
bulk; OHLCV joins later for `po indicator --strict`).

Rust 1.92+, edition 2024.

## License

Mozilla Public License 2.0 - see `LICENSE`. Vendored sources retain their
own licenses (Apache-2.0 for PineForge, MIT for pine-tools data, MPL-2.0 for
Pinecone). Per-file `SPDX-License-Identifier` headers point back to each
upstream. Top-level `NOTICE` consolidates the per-component attributions.

## Acknowledgements

- **Pinecone** (MPL-2.0) - the Pine v6 reference markdown snapshot
  (`vendor/pine-reference/spec/v6.md`).
- **PineForge contributors** (Apache-2.0) - the 239-probe validation
  corpus (`vendor/pineforge-corpus/`), the Pine v6 audit doc
  (`vendor/pineforge-docs/pine_v6_audit_master.md`, 38 critical + ~62
  minor documented TV-vs-engine divergences), the 18 narrative explainer
  pages (`vendor/pineforge-docs/pages/`), and the `scripts/verify_corpus.py`
  algorithm ported as `src/diff.rs`.
- **folknor / pine-tools** (MIT) - the structured pine-data JSON exports
  (functions, variables, constants, keywords, types, annotations) that
  back `po behavior` and fill gaps in piners-runtime's validation
  builtins. Same upstream that builds the VS Code Pine extension + LSP +
  MCP server.
- **piners** (MIT OR Apache-2.0) - piners-syntax powers `po parse`,
  `po tokens`, and local `po validate`; piners-runtime provides the
  primary builtins table used by local validation; piners-runner backs
  indicator fixture replay for `po indicator --strict`.
- **TradingView** - source of the Pine v6 reference content vendored
  through Pinecone's snapshot. TradingView and PineScript are trademarks
  of their respective owners. This project is not affiliated with or
  endorsed by TradingView.
