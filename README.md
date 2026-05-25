# pine-oracle

A single-binary CLI that answers Pine Script v6 semantic questions.
Reference lookups, structured signatures, BM25 search, parsing, validation,
corpus probes, trade-list parity scoring - all baked into one Rust binary
with no on-disk state, no network calls (except `validate --strict`), no
runtime configuration.

## What it does

| Command | Output |
|---|---|
| `pine lookup <name>` | Function / constant / variable details from TradingView's v6 reference |
| `pine search <query> [--kind reference\|probe\|audit\|docs\|behavior]` | BM25 across v6 reference, baked corpus, PineForge audit/doc pages, and structured pine-data behavior; `--kind` is case-insensitive, and `--kind ?` lists source kinds |
| `pine behavior <name>` / `pine behavior [TEXT] --list [--kind function\|variable\|constant\|keyword] [--grep TEXT]` | Structured signature + polymorphism + argument ordering from pine-tools' JSON exports; list-mode `TEXT` acts as grep, `--kind` is case-insensitive, and `--kind ?` lists behavior kinds |
| `pine parse <code>` | piners-syntax AST tree |
| `pine tokens <code>` | piners-syntax lexer token stream |
| `pine validate <code>` | Local lex + parse + type + semantic diagnostics from piners-syntax, backed by piners-runtime builtins plus pine-data gap-fill |
| `pine validate --strict <code>` | POST to TradingView's pine-lint endpoint. Yes/no oracle; diagnostic prose is non-actionable |
| `pine probe <slug>` | Pull a baked PineForge corpus probe (strategy.pine + tv_trades.csv + author-extracted summary) |
| `pine probes [--grep TEXT] [--feature NAME]` | List baked probes; `--grep` matches slug or summary substring, `--feature` restricts by detected Pine-feature usage (`oca`, `trail`, `pyramiding`, `mtf`, `varip`, `magnifier`, `matrix`, `map`, `udt`, `method`, `process_orders_on_close`, `barstate_isfirst`; pass `?` to list the catalog) |
| `pine diff <probe> <trades.csv> [--show-diffs N]` | Tier-classify a user trade list against the probe's TV ground truth (port of PineForge's verify_corpus.py); `--show-diffs N` emits the worst-N matched pairs plus every TV / user orphan |
| `pine indicator --list [--grep TEXT] [--baseline smoke\|tv]` | List baked indicator strict fixtures with smoke vs TV baseline kind, symbol/timeframe, bar count, output count, and range window; `--baseline ?` lists baseline kinds |
| `pine indicator --strict <slug>` / `pine indicator --strict --all [--grep TEXT] [--baseline smoke\|tv]` | Run baked indicator fixtures through piners-runner and diff per-bar outputs against `expect.json`, including optional `test_range` windows; expected outputs may use plot titles, with duplicate titles disambiguated as `#1`, `#2`, etc.; `smoke-*` fixtures cover plots, warmup, ranges, bool `plotshape`, and same-symbol `request.security`; TV-captured baselines are still pending |
| `pine version` | Binary version + bake counts and pinned snapshot metadata (reference, corpus, PineForge docs, pine-data, indicator fixtures split smoke vs TV) |

All subcommands accept `--format json|text|auto`. JSON outputs carry
`schema_version: 1`. Object payloads attach the version inline; arrays wrap
as `{schema_version, items}`. Bump rules in `docs/pine-oracle.md`.

## Install

```
cargo install --path .
```

Binary lands as `pine`. ~75-80 MB stripped (the PineForge corpus is the
bulk; OHLCV joins later for `pine indicator --strict`).

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
  (functions, variables, constants, keywords, function-behavior) that
  back `pine behavior` and fill gaps in piners-runtime's validation
  builtins. Same upstream that builds the VS Code Pine extension + LSP +
  MCP server.
- **piners** (MIT OR Apache-2.0) - piners-syntax powers `pine parse`,
  `pine tokens`, and local `pine validate`; piners-runtime provides the
  primary builtins table used by local validation; piners-runner backs
  indicator fixture replay for `pine indicator --strict`.
- **TradingView** - source of the Pine v6 reference content vendored
  through Pinecone's snapshot. TradingView and PineScript are trademarks
  of their respective owners. This project is not affiliated with or
  endorsed by TradingView.
