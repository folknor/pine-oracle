# pine-oracle

A single-binary CLI that answers Pine Script v6 semantic questions.
Structured-signature lookups, BM25 search, and validation - all baked into
one Rust binary with no on-disk state, no network calls (except
`validate --strict`), no runtime configuration. It answers questions *about*
Pine; it never executes user strategies.

## What it does

| Command | Output |
|---|---|
| `po lookup <name>` | Describe an identifier from pine-data: the structured signature (typed params with per-argument prose + default / allowedValues / min / max, per-overload signatures, polymorphism + deprecation flags) plus the prose sub-sections `Remarks`, `See also`, and the `Returns` sentence. Operators (`+`, `?:`, `[]`, ...) are a first-class catalog. Names that live in several catalogs at once (`na` is a function, a variable, and a keyword; `time` is a function and a variable) dump **every** meaning. A partial name surfaces prefix matches |
| `po lookup [TEXT] --list [--kind function\|variable\|constant\|keyword\|type\|annotation\|operator] [--grep TEXT]` | Browse the pine-data behavior catalog; list-mode `TEXT` acts as grep, `--kind` is case-insensitive, and `--kind ?` lists behavior kinds |
| `po search <query> [--limit N]` | The index into `po lookup`: a ranked `score name` list (one line per identifier) over the pine-data behavior surface via BM25. Reach for it when you don't yet know the identifier to look up; text-only output |
| `po validate <code-or-file>` / `--code CODE` / `--file PATH` / `-` | Local lex + parse + type + semantic diagnostics from piners-syntax, backed by piners-runtime builtins plus pine-data gap-fill; text diagnostics include source-line caret frames. Codes documented in `docs/diagnostics.md` |
| `po validate --strict <code-or-file>` | POST to TradingView's pine-lint endpoint. Yes/no oracle; diagnostic prose is non-actionable |
| `po version` | Binary version + pine-data bake counts (functions, variables, constants, keywords, types, annotations, operators) and the pinned snapshot date |

All subcommands accept `--format json|text|auto`. JSON outputs carry
`schema_version: 2`. Object payloads attach the version inline; arrays wrap
as `{schema_version, items}`. `po lookup` JSON is `{query, exact, matches: [...]}`
where `matches` holds every catalog entry for the name. Text output accepts
`--quiet` to suppress non-data headers, snippets, status lines, and validation
notes where a command emits them.

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
own licenses (MIT for pine-tools data). Per-file `SPDX-License-Identifier`
headers point back to each upstream. Top-level `NOTICE` consolidates the
per-component attributions.

## Acknowledgements

- **folknor / pine-tools** (MIT) - the structured pine-data JSON exports
  (functions, variables, constants, keywords, types, annotations,
  operators) that back `po lookup` and fill gaps in piners-runtime's
  validation builtins. Same upstream that builds the VS Code Pine
  extension + LSP + MCP server.
- **piners** (MIT OR Apache-2.0) - piners-syntax powers local
  `po validate`; piners-runtime provides the primary builtins table used
  by local validation.
- **TradingView** - the Pine v6 documentation that pine-tools crawls to
  produce the vendored data. TradingView and PineScript are trademarks
  of their respective owners. This project is not affiliated with or
  endorsed by TradingView.
