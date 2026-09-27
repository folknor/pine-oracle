# pine-oracle

A single-binary CLI that answers Pine Script v6 semantic questions.
Structured-signature lookups (with BM25 "did you mean ...?" recovery), Pine
User Manual prose search, and an authored Pine cookbook - all baked into one
Rust binary with no on-disk state, no network calls, no runtime configuration.
The one exception is `po verdict`, a record of measured TradingView behavior
that lives in a directory the caller names explicitly on every command.
It answers questions *about* Pine; it never executes user strategies.

## What it does

| Command | Output |
|---|---|
| `po lookup <name>` | Describe an identifier from pine-data: the structured signature (typed params with per-argument prose + default / allowedValues / min / max, per-overload signatures, polymorphism + deprecation flags) plus the prose sub-sections `Remarks`, `See also`, and the `Returns` sentence. Operators (`+`, `?:`, `[]`, ...) are a first-class catalog. Names that live in several catalogs at once (`na` is a function, a variable, and a keyword; `time` is a function and a variable) dump **every** meaning. On a miss, BM25 "did you mean ...?" suggestions of the closest identifier names |
| `po lookup [TEXT] --list [--kind function\|variable\|constant\|keyword\|type\|annotation\|operator] [--grep TEXT]` | Browse the pine-data behavior catalog; list-mode `TEXT` acts as grep, `--kind` is case-insensitive, and `--kind ?` lists behavior kinds |
| `po search <query>` | **Find** across **both** the Pine User Manual and the authored TA recipes at once (one BM25 index). Prints a menu, no prose: a manual row is `<8-hex id>  page / H2 / H3` (feed the id to `po show`), a recipe row is `recipe <name>  recipe / category / Title` (run it directly). `--limit` defaults to 8. `-1` / `--top` renders the top hit directly (manual section or recipe body) |
| `po show <id> [<id>...]` | Print Manual section(s) by the id `po search` prints. Renders the section **plus all its subsections** to the terminal with its canonical TradingView URL; multiple ids render in order. Hash-only addressing |
| `po recipe <name>` | A Pine cookbook: how to build something that has **no** TradingView builtin and no Manual page - prose + a self-contained Pine v6 recipe. Covers indicators (custom moving averages, oscillators, candlestick patterns, volatility/volume/trend tools) and general helpers (easing/animation curves, risk metrics). Match by name or alias; `--list` / `--category` / `--grep` browse (`--category ?` lists the catalog); a miss offers fuzzy "did you mean ...?" suggestions |
| `po verdict add\|observe\|search\|show\|list --records <DIR>` | Record and query what TradingView measurably did, per oracle source (editor, `translate_light` endpoint, chart run), and whether each question is settled. Records are TOML plus content-addressed fixtures in a caller-named directory; `--records` is required on every verb and never defaulted. See [docs/verdict.md](docs/verdict.md) |
| `po version` | Binary version + pine-data bake counts (functions, variables, constants, keywords, types, annotations, operators), manual page/section counts, recipe entry/category counts, and the pinned snapshot date |

Output is text-only across the board. Text output accepts `--quiet` to suppress
non-data headers and status lines where a command emits them.

## Where the data comes from

Every answer is served from data **compiled into the binary at build time** -
`po search` queries an embedded snapshot of the Pine User Manual, not the live
website, and `po lookup` reads embedded JSON, not a remote API. There are no
network calls, no on-disk index, and no cache to warm.

Two vendored sources under `vendor/` are baked in:

- **pine-data** - the seven structured JSON catalogs (functions, variables,
  constants, keywords, types, annotations, operators) from
  [pine-tools](https://github.com/folknor/pine-tools), embedded with
  `include_str!`. Backs `po lookup`.
- **Pine User Manual** - the manual as a per-page markdown tree, embedded with
  `include_dir!` and BM25-indexed in RAM on first query. Backs `po search` /
  `po show`.

The **recipes** behind `po recipe` are different: not vendored but
**authored in-repo** under `assets/recipes/<category>/<name>.md` (also embedded
with `include_dir!`), a Pine cookbook covering anything TradingView ships no
builtin for - indicators plus general helpers (easing curves, risk metrics).
They are independent reimplementations - the formulas are derived from reference
sources (the pandas-ta-classic library, TradingView's open-source `ta` library,
the easings.net catalog, and standard quant-finance definitions) and
reimplemented in Pine v6, not copied, with each recipe's Pine checked clean by
`pine-lint`. The same recipes are folded into `po search`'s unified index.

Because the data is a point-in-time snapshot, it tracks Pine v6 as of the
vendored scrape - `po version` prints the pinned snapshot date. Refreshing the
data means re-vendoring and rebuilding; there is nothing to update at runtime.

**Verdicts** behind `po verdict` are the exception: they are neither baked in
nor found automatically. Each command reads (or writes) only the records
directories passed with `--records`, which in practice live in the git repo of
the project that took the measurements.

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
  operators) that back `po lookup`. Same upstream that builds the VS Code
  Pine extension + LSP + MCP server.
- **markdown-peek** (MIT, (c) tkcd / takeshiD) -
  https://github.com/takeshiD/markdown-peek - its `emitter/term.rs`
  markdown-to-terminal renderer was adapted into `src/render.rs` (trimmed
  of syntect/emoji) to render `po search -1` / `po show` manual sections.
- **pandas-ta-classic** (MIT) -
  https://github.com/xgboosted/pandas-ta-classic - the indicator library whose
  implementations are the primary reference for the `po recipe` formulas. The
  recipes are independently reimplemented in Pine v6, not copied.
- **easings.net** (Andrey Sitnik & Ivan Solovev, GPL-3.0 for the site code; the
  easing *formulas* themselves are standard and uncopyrightable) - the reference
  for the `easing` recipe family. The Pine ports were also cross-checked against
  RicardoSantos' `MathEasingFunctions` library (MPL-2.0) on TradingView.
  Independently reimplemented, not copied.
- **TradingView** - the Pine v6 reference documentation that pine-tools
  crawls to produce the pine-data exports, and the Pine User Manual
  vendored under `vendor/pine-manual/` for `po search` / `po show`. Its
  open-source `ta` library (MPL-2.0) was used as a correctness reference when
  authoring the `po recipe` formulas (independently reimplemented, not copied).
  TradingView and PineScript are trademarks of their respective owners. This
  project is not affiliated with or endorsed by TradingView.
