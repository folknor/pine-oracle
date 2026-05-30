# pine-oracle

A single-binary CLI that answers Pine Script v6 semantic questions.
Reference lookups joined with structured signatures, BM25 search, and
validation - all baked into one Rust binary with no on-disk state, no
network calls (except `validate --strict`), no runtime configuration. It
answers questions *about* Pine; it never executes user strategies.

## What it does

| Command | Output |
|---|---|
| `po lookup <name>` | Describe an identifier: the structured pine-data signature (typed params with default / allowedValues / min / max, per-overload signatures, polymorphism + deprecation flags) joined with the v6 reference prose the structured surface lacks - per-argument descriptions, `Remarks`, and `See also`. Reference-only entries (e.g. Operators) fall back to verbatim prose; a partial name surfaces prefix matches |
| `po lookup [TEXT] --list [--kind function\|variable\|constant\|keyword\|type\|annotation] [--grep TEXT]` | Browse the pine-data behavior catalog; list-mode `TEXT` acts as grep, `--kind` is case-insensitive, and `--kind ?` lists behavior kinds |
| `po search <query> [--kind reference\|behavior]` | BM25 across the v6 reference and structured pine-data behavior - the two "name" sources. Search is the index into `po lookup`: reach for it when you don't yet know the identifier to look up. `--kind` is case-insensitive, and `--kind ?` lists source kinds |
| `po validate <code-or-file>` / `--code CODE` / `--file PATH` / `-` | Local lex + parse + type + semantic diagnostics from piners-syntax, backed by piners-runtime builtins plus pine-data gap-fill; text diagnostics include source-line caret frames. Codes documented in `docs/diagnostics.md` |
| `po validate --strict <code-or-file>` | POST to TradingView's pine-lint endpoint. Yes/no oracle; diagnostic prose is non-actionable |
| `po version` | Binary version + bake counts and pinned snapshot metadata (reference, pine-data) |

All subcommands accept `--format json|text|auto`. JSON outputs carry
`schema_version: 1`. Object payloads attach the version inline; arrays wrap
as `{schema_version, items}`. Text output accepts `--quiet` to suppress
non-data headers, snippets, status lines, and validation notes where a command
emits them.

### Schema versioning

`schema_version` is bumped on any breaking shape change to a subcommand's JSON
output. The constant lives in `src/output.rs` (`SCHEMA_VERSION`) and is
hard-pinned by a test; bumping it requires updating both. Once assigned, a
version's payload shapes do not change in place.

## Install

```
cargo install --path .
```

Binary lands as `po`. Rust 1.92+, edition 2024.

## License

Mozilla Public License 2.0 - see `LICENSE`. Vendored sources retain their
own licenses (MIT for pine-tools data, MPL-2.0 for Pinecone). Per-file
`SPDX-License-Identifier` headers point back to each upstream. Top-level
`NOTICE` consolidates the per-component attributions.

## Acknowledgements

- **Pinecone** (MPL-2.0) - the Pine v6 reference markdown snapshot
  (`vendor/pine-reference/spec/v6.md`).
- **folknor / pine-tools** (MIT) - the structured pine-data JSON exports
  (functions, variables, constants, keywords, types, annotations) that
  back `po lookup`'s structured signatures and fill gaps in
  piners-runtime's validation builtins. Same upstream that builds the
  VS Code Pine extension + LSP + MCP server.
- **piners** (MIT OR Apache-2.0) - piners-syntax powers local
  `po validate`; piners-runtime provides the primary builtins table used
  by local validation.
- **TradingView** - source of the Pine v6 reference content vendored
  through Pinecone's snapshot. TradingView and PineScript are trademarks
  of their respective owners. This project is not affiliated with or
  endorsed by TradingView.
