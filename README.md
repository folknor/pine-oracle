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
| `pine search <query> [--kind reference\|probe\|audit\|docs]` | BM25 across v6 reference, baked corpus, PineForge audit doc, and PineForge narrative pages |
| `pine behavior <name>` | Structured signature + polymorphism + argument ordering from pine-tools' JSON exports |
| `pine parse <code>` | Pretty-printed AST tree |
| `pine tokens <code>` | Lexer token stream |
| `pine validate <code>` | Local lex + parse diagnostics. **First error only today**; deepens when piners-syntax 0.1 ships |
| `pine validate --strict <code>` | POST to TradingView's pine-lint endpoint. Yes/no oracle; diagnostic prose is non-actionable |
| `pine probe <slug>` | Pull a baked PineForge corpus probe (strategy.pine + tv_trades.csv + author-extracted summary) |
| `pine probes [--grep TEXT]` | List baked probes; substring-match on slug or summary |
| `pine diff <probe> <trades.csv>` | Tier-classify a user trade list against the probe's TV ground truth (port of PineForge's verify_corpus.py) |
| `pine version` | Binary version + bake counts (reference entries, corpus probes, audit + narrative sections) |

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
  (`vendor/pine-reference/spec/v6.md`) and the lexer / AST / parser
  lifted into `src/syntax/`. The single highest-ROI vendoring source.
- **PineForge contributors** (Apache-2.0) - the 235-probe validation
  corpus (`vendor/pineforge-corpus/`), the Pine v6 audit doc
  (`vendor/pineforge-docs/pine_v6_audit_master.md`, 38 critical + ~62
  minor documented TV-vs-engine divergences), the 18 narrative explainer
  pages (`vendor/pineforge-docs/pages/`), and the `scripts/verify_corpus.py`
  algorithm ported as `src/diff.rs`.
- **folknor / pine-tools** (MIT) - the structured pine-data JSON exports
  (functions, variables, constants, keywords, function-behavior) that
  back `pine behavior`. Same upstream that builds the VS Code Pine
  extension + LSP + MCP server.
- **TradingView** - source of the Pine v6 reference content vendored
  through Pinecone's snapshot. TradingView and PineScript are trademarks
  of their respective owners. This project is not affiliated with or
  endorsed by TradingView.
