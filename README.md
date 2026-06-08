# pine-oracle

A single-binary CLI that answers Pine Script v6 semantic questions.
Structured-signature lookups (with BM25 "did you mean ...?" recovery), Pine
User Manual prose search, and validation - all baked into one Rust binary with
no on-disk state, no network calls (except `validate --strict`), no runtime
configuration. It answers questions *about* Pine; it never executes user
strategies.

## What it does

| Command | Output |
|---|---|
| `po lookup <name>` | Describe an identifier from pine-data: the structured signature (typed params with per-argument prose + default / allowedValues / min / max, per-overload signatures, polymorphism + deprecation flags) plus the prose sub-sections `Remarks`, `See also`, and the `Returns` sentence. Operators (`+`, `?:`, `[]`, ...) are a first-class catalog. Names that live in several catalogs at once (`na` is a function, a variable, and a keyword; `time` is a function and a variable) dump **every** meaning. On a miss, BM25 "did you mean ...?" suggestions of the closest identifier names |
| `po lookup [TEXT] --list [--kind function\|variable\|constant\|keyword\|type\|annotation\|operator] [--grep TEXT]` | Browse the pine-data behavior catalog; list-mode `TEXT` acts as grep, `--kind` is case-insensitive, and `--kind ?` lists behavior kinds |
| `po search <query>` | **Find** sections of the **Pine User Manual** (how does X work) - the complement to `lookup`'s "what is X". Prints a menu of matching sections, one per row as `<8-hex id>  page / H2 / H3` (no prose); feed an id to `po show`. `--limit` defaults to 8. `-1` / `--top` renders the top hit directly |
| `po show <id> [<id>...]` | Print Manual section(s) by the id `po search` prints. Renders the section **plus all its subsections** to the terminal with its canonical TradingView URL; multiple ids render in order. Hash-only addressing |
| `po validate <code-or-file>` / `--code CODE` / `--file PATH` / `-` | Local lex + parse + type + semantic diagnostics from piners-syntax, backed by piners-runtime builtins plus pine-data gap-fill; text diagnostics include source-line caret frames. Codes documented in `docs/diagnostics.md` |
| `po validate --strict <code-or-file>` | POST to TradingView's pine-lint endpoint. Yes/no oracle; diagnostic prose is non-actionable |
| `po version` | Binary version + pine-data bake counts (functions, variables, constants, keywords, types, annotations, operators) and the pinned snapshot date |

Output is text-only except `po validate`, which alone takes `--format
json|text|auto` for a machine-readable yes/no + diagnostics path. Text output
accepts `--quiet` to suppress non-data headers, status lines, and validation
notes where a command emits them.

## Where the data comes from

Every answer is served from data **compiled into the binary at build time** -
`po search` queries an embedded snapshot of the Pine User Manual, not the live
website, and `po lookup` reads embedded JSON, not a remote API. There are no
network calls (except `validate --strict`, which POSTs to TradingView's
pine-lint endpoint), no on-disk index, and no cache to warm.

Two vendored sources under `vendor/` are baked in:

- **pine-data** - the seven structured JSON catalogs (functions, variables,
  constants, keywords, types, annotations, operators) from
  [pine-tools](https://github.com/folknor/pine-tools), embedded with
  `include_str!`. Backs `po lookup` and fills gaps in local validation.
- **Pine User Manual** - the manual as a per-page markdown tree, embedded with
  `include_dir!` and BM25-indexed in RAM on first query. Backs `po search` /
  `po show`.

Because the data is a point-in-time snapshot, it tracks Pine v6 as of the
vendored scrape - `po version` prints the pinned snapshot date. Refreshing the
data means re-vendoring and rebuilding; there is nothing to update at runtime.

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
- **markdown-peek** (MIT, (c) tkcd / takeshiD) -
  https://github.com/takeshiD/markdown-peek - its `emitter/term.rs`
  markdown-to-terminal renderer was adapted into `src/render.rs` (trimmed
  of syntect/emoji) to render `po search -1` / `po show` manual sections.
- **TradingView** - the Pine v6 reference documentation that pine-tools
  crawls to produce the pine-data exports, and the Pine User Manual
  vendored under `vendor/pine-manual/` for `po search` / `po show`. TradingView and
  PineScript are trademarks of their respective owners. This project is
  not affiliated with or endorsed by TradingView.
