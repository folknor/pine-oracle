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
| `pine parse <code>` | piners-syntax AST tree |
| `pine tokens <code>` | piners-syntax lexer token stream |
| `pine validate <code>` | Local lex + parse + type + semantic diagnostics from piners-syntax, backed by piners-runtime builtins plus pine-data gap-fill |
| `pine validate --strict <code>` | POST to TradingView's pine-lint endpoint. Yes/no oracle; diagnostic prose is non-actionable |
| `pine probe <slug>` | Pull a baked PineForge corpus probe (strategy.pine + tv_trades.csv + author-extracted summary) |
| `pine probes [--grep TEXT] [--feature NAME]` | List baked probes; `--grep` matches slug or summary substring, `--feature` restricts by detected Pine-feature usage (`oca`, `trail`, `pyramiding`, `mtf`, `varip`, `magnifier`, `matrix`, `map`, `udt`, `method`, `process_orders_on_close`, `barstate_isfirst`; pass `?` to list the catalog) |
| `pine diff <probe> <trades.csv> [--show-diffs N]` | Tier-classify a user trade list against the probe's TV ground truth (port of PineForge's verify_corpus.py); `--show-diffs N` emits the worst-N matched pairs plus every TV / user orphan |
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
  primary builtins table used by local validation.
- **TradingView** - source of the Pine v6 reference content vendored
  through Pinecone's snapshot. TradingView and PineScript are trademarks
  of their respective owners. This project is not affiliated with or
  endorsed by TradingView.
