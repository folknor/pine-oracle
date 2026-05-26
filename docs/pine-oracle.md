# The pine oracle

A systemwide CLI tool that answers every Pine v6 question reviewers would otherwise answer from memory. Lives in its own Rust repo `pine-oracle/` -- not in piners, not in pine-tools. Installed once per machine. Queried by any agent in any session in any Pine-adjacent project.

This is the target design for pine-oracle. It describes the oracle the project is building toward; temporary implementation notes are included only when they explain sequencing or migration risk.

## Problem

Across review sessions, claims about Pine v6 semantics drift. A reviewer in session N says "TV's `array.mode` returns the smallest value on ties"; a reviewer three sessions later says "TV returns the first-encountered value." Neither is wrong from memory; both are wrong from evidence. There is no single command an agent can run to ask the canonical source.

Two canonical sources exist:

- **pine-tools** (`../pine-tools/`) -- scraped TV docs: function signatures, types, polymorphism, behavior flags, type-coercion rules, the parser/validator itself.
- **The corpus** (`vendor/pineforge-corpus/validation/`) -- 239 Pine strategies cross-validated trade-for-trade against TradingView's broker emulator, baked into the binary. The executable parity oracle. Sourced from <https://github.com/fullpass-4pass/pineforge-corpus> (Apache-2.0); refreshed via `scripts/prune-vendored-corpus.sh`. Background on the underlying ETH/USDT-USDT 15m Binance feed + the verifier's tier definitions (excellent / strong / moderate / weak / minimal + the anomaly / engine_only overrides) lives in `src/diff.rs`'s preamble and `vendor/pineforge-corpus/VENDORING_NOTES.md`.

Together they cover almost every Pine claim a reviewer can make. But pine-tools is a pnpm/node project (slow startup, brittle dependency graph), the corpus is a directory of .pine + .csv files with no query layer, and neither is reachable from a piners shell without remembering the right incantation.

The fix: a single pre-compiled binary that bundles both, queryable in one bash call, zero setup per session.

## Why its own repo (not piners, not pine-tools)

- **Reused across projects.** piners is one consumer. pine-tools dogfooding, third-party Pine work, future pine-* tools all want the same lookups. Putting it inside piners would signal "piners helper" even though it isn't; putting it inside pine-tools (TypeScript LSP) would force a Rust crate into a TS monorepo's toolchain.
- **No piners build dependency.** Agents on a fresh piners checkout should not pay a 3-minute compile to ask "is `math.max` documented as na-propagating".
- **No language-mix friction.** pine-tools is TS; the oracle is Rust. Each repo stays language-coherent. Cross-repo coupling is one `pnpm run export:json` step in pine-tools producing JSON the oracle vendors at build time.
- **Versioned independently.** When TV publishes a v6.2, the oracle bumps; every consumer picks it up via one binary upgrade. piners' CLAUDE.md does not change.
- **Same answer in every context.** A piners agent and a pine-tools agent asking the same question get the same answer, byte for byte. Reproducibility is the whole point.
- **Extraction-cost avoided.** Starting in piners and extracting later would mean git history rewrite, import-path churn, and dual maintenance during cutover. Repo setup is one afternoon; extraction is a week.

## Naming

Binary name: `po`. Short for pine-oracle; `pine` itself is too generic (says nothing about what the tool does) and clashes with the legacy Pine email client (mostly extinct; survives as `alpine` in some distros).

## Architecture

Single binary, four layers.

### 1. Vendored pine-data (compiled into the binary)

Two sources, merged at build time:

- **Primary: Pinecone's `crates/pine-reference/spec/v6.md`** -- 918 KB / ~25k lines / **941 entries**, one per `### name`, uniform sub-sections (`Syntax`, `Arguments`, `Example`, `Type`, `Remarks`, `See also`). MPL-2.0 vendored copy of TradingView's published v6 reference. Embedded via `include_str!`, parsed once with `comrak` and cached behind a `OnceLock`. Drives `po lookup` + the BM25 reference docs.
- **Secondary: pine-tools' `pine-data/v6/*.json`** -- published in pine-tools' git tree (`functions.json`, `variables.json`, `constants.json`, `keywords.json`, `function-behavior.json`). Copied into `vendor/pine-data/v6/` and `include_str!`'d. Adds polymorphism markers + structured signatures + argument-ordering metadata that complement v6.md's prose. Drives `po behavior`.

Combined size: ~1.5 MB. Trivial.

### 2. BM25 index (built lazily at startup)

Five sources, all baked into the binary as source markdown / extracted-at-runtime content:

- **v6 reference**: 941 entries from `vendor/pine-reference/spec/v6.md`, indexed as `kind: "reference"`.
- **Corpus probes**: 239 baked probes, indexed by their published slug as `kind: "probe"` with their full author-extracted summary as content.
- **PineForge audit doc**: `vendor/pineforge-docs/pine_v6_audit_master.md`, 38 critical + ~62 minor documented TV-vs-engine divergences, sliced on H2 / H3 boundaries, indexed as `kind: "audit"`.
- **PineForge narrative pages**: `vendor/pineforge-docs/pages/*.md`, 18 explainer docs (magnifier, mtf, timeframes, lifecycle, report-schema, abi-stability, examples, tutorials), section-sliced on H2 / H3, indexed as `kind: "docs"`.
- **pine-data behavior entries**: function / variable / constant / keyword exports from `vendor/pine-data/v6/*.json`, indexed as `kind: "behavior"` with signatures, parameter prose, examples, and polymorphism notes.

Total: a few thousand compact documents. Index is a `tantivy` RAM directory rebuilt on first query (~10-15 ms one-shot cost, then sub-millisecond per query), cached behind a `OnceLock`. Schema: `name` (TEXT|STORED, 5x boost), `category` (STRING|STORED), `kind` (STRING|STORED), `content` (STORED for retrieval) + `content_search` (TEXT, drives ranking). Hits carry the full content body in `SearchHit.content` so consumers don't need a follow-up lookup. `po search --kind <kind>` narrows by source case-insensitively; `--kind ?` lists the source catalog and document counts.

Per-probe summaries are extracted live at runtime from each `strategy.pine`'s header by `corpus::summary_for`: every prose comment line up to the first real code line, with license / SPDX / copyright / version-directive noise filtered and blank `//` paragraph separators collapsed. Covers 100% of the 239 baked probes with multi-paragraph summaries (median ~650 chars) - no LLM-curation pass required.

### 3. Vendored corpus (baked)

The corpus ships **inside the binary**. The published PineForge corpus is ~245 MB cloned (38 MB git history, 75 MB OHLCV across four feeds, 239 probes' worth of strategy.pine + tv_trades.csv + engine_trades.csv + generated.cpp + reports). The vendored subset pine-oracle bakes is the strict minimum the public subcommands need: per-probe `strategy.pine`, `tv_trades.csv`, optional `inputs.json`, plus a single `data/ohlcv_spans.json` (~500 bytes) carrying `{first_ms, last_ms, bar_ms}` per upstream feed. Everything else (the OHLCV CSVs themselves, PineForge's own `engine_trades.csv`, the transpiler's `generated.cpp`, validation reports, upstream tooling) is pruned out of `vendor/pineforge-corpus/` and not in the binary.

Baked subset size: ~72 MB. Final binary size lands around 75-80 MB. Embedded via `include_dir!()` at compile time, queried as `&'static str` slices at runtime. Zero on-disk scratch, zero env vars, zero settings files.

Pruning is reproducible: `scripts/bake-ohlcv-spans.py` distills the OHLCV CSVs into `data/ohlcv_spans.json`, then `scripts/prune-vendored-corpus.sh` drops the bulk CSVs. Both run on a fresh clone of <https://github.com/fullpass-4pass/pineforge-corpus>; see `vendor/pineforge-corpus/VENDORING_NOTES.md` for the kept / dropped manifest and the four-step refresh procedure.

The span metadata is what `po diff`'s interior-trim machinery consults via `corpus::ohlcv_span_for_probe`. Baking the raw CSVs themselves (~75 MB) is **not** planned: `po indicator --strict` reads bars from its own `bars.json` fixture rather than the corpus OHLCV feed, so binary size stays in the 75-80 MB band.

### 4. Validator backend

Two tiers, in inverted authority order vs. an earlier draft of this doc:

- **Local tier (`po validate`)**. The workhorse. Uses **piners-syntax** for lexing, parsing, type checking, and semantic analysis, returning every diagnostic the pipeline can recover with correct line + column positions. The builtins table starts from **piners-runtime** so validation matches piners where the runtime has an implementation or stub, then pine-oracle fills any missing public symbols from the vendored pine-tools JSON.
- **Strict tier (`po validate --strict`)**. **Yes / no oracle only. Do not try to fix your script from its diagnostics.** TradingView's `pine-facade/translate_light` endpoint is profoundly bad as a validator: it stops at the first error, breaks on trailing whitespace (e.g. an extra space at end of line is "invalid"), and reports the wrong line / column for essentially every diagnostic. The diagnostic prose is non-actionable: it tells you *something* is wrong but not where or what in any reliable way. The only trustworthy output is the `success` bit (true / false). Use this exactly once, after you believe `po validate` (local tier) reports clean: a final yes / no from TV's broker before you publish. Do not iterate against it; iterate against the local tier. No auth required, no on-disk cache.
- **Indicator strict tier (`po indicator --strict <slug>`)**. Different oracle: runs an indicator against fixture bars through **piners-runner** and diffs per-bar values against a baked baseline using the PineTS-derived `.expect.json` schema (see "Strict-mode indicator test format" below). Corpus is trade-list parity; this is per-bar indicator parity. The runner/differ/CLI substrate is implemented and covered by deterministic smoke fixtures, including plot-title matching, duplicate-title disambiguation, bool `plotshape`, and same-symbol `request.security`; real TV baselines are the remaining data gap.

## Vendoring inventory

What to pull from where, in priority order. The oracle's license is the natural umbrella over whatever we vendor (MPL-2.0 for files derived from Pinecone, Apache-2.0 for everything else); AGPL sources are paraphrased clean-room and not vendored as-is.

| # | Artifact | Source | License | Use | Disposition |
|---|---|---|---|---|---|
| 1 | `spec/v6.md` (941 reference entries) | `research/pinecone/crates/pine-reference/` | MPL-2.0 | Primary BM25 substrate + lookup table | Vendor at `vendor/pine-reference/spec/v6.md` |
| 2 | Markdown query layer (~250 LOC) | `research/pinecone/crates/pine-reference/src/lib.rs:60-203` | MPL-2.0 | Backs `po lookup` | Lift into `src/reference.rs` |
| 3 | Pine v6 syntax pipeline | `../piners/crates/piners-syntax/` | MIT OR Apache-2.0 | Backs `po parse` / `po tokens` / local `po validate` | Path dependency; canonical syntax backend |
| 4 | Pinecone lexer + parser lift + 72 parser goldens | `research/pinecone/crates/pine-{lexer,ast,parser}/` | MPL-2.0 | Former migration baseline | Removed from shipped sources after piners-syntax integration; not vendored or compiled |
| 5 | `pine-data/v6/*.json` (functions / variables / constants / keywords / function-behavior) | `pine-tools/pine-data/v6/*.json` | MIT (folknor) | Backs `po behavior` | Vendor at `vendor/pine-data/v6/` |
| 6 | PineForge validation corpus (239 probes) | `https://github.com/fullpass-4pass/pineforge-corpus` | Apache-2.0 | Backs `po probe` / `po probes` / `po diff`; corpus-kind BM25 docs | Vendor at `vendor/pineforge-corpus/` |
| 7 | `docs/pine_v6_audit_master.md` (38 critical + ~62 minor divergences) | `research/pineforge-engine/docs/` | Apache-2.0 | Audit-kind BM25 docs | Vendor at `vendor/pineforge-docs/pine_v6_audit_master.md` |
| 8 | `docs/pages/*.md` (18 narrative docs) | `research/pineforge-engine/docs/pages/` | Apache-2.0 | Docs-kind BM25 substrate | Vendor at `vendor/pineforge-docs/pages/` |
| 9 | 21 engine-internals probe summaries | `research/pineforge-engine/src/engine_*.cpp` + tests | Apache-2.0 | Per-probe BM25 substrate | **Superseded.** The harvested summaries were keyed to engine-internal probe identifiers that do not match the published-corpus slugs. `corpus::summary_for` collects every prose comment line from each baked `strategy.pine`'s header instead, covering 100% of the 239 baked probes with multi-paragraph summaries (median ~650 chars) without any LLM curation. |
| 10 | `scripts/verify_corpus.py` (622 lines Python) | `research/pineforge-engine/` | Apache-2.0 | Trade-list alignment + tier classification | Ported to `src/diff.rs` including interior trim; OHLCV spans baked into `vendor/pineforge-corpus/data/ohlcv_spans.json` |
| 10a | piners runtime builtins | `../piners/crates/piners-runtime/` | MIT OR Apache-2.0 | Primary builtins table for local validation | Path dependency; augmented with pine-data gap-fill |
| 10b | piners runner | `../piners/crates/piners-runner/` | MIT OR Apache-2.0 | Runs baked indicator fixtures for `po indicator --strict` | Path dependency; command substrate implemented with deterministic smoke fixtures, pending real TV baseline data under `indicators/` |
| 11 | `pineforge.h` doxygen blocks (~390 LOC) | `research/pineforge-engine/include/pineforge/` | Apache-2.0 | C ABI documentation | **Deferred.** Niche substrate (describes the C ABI consumers integrate against, not Pine semantics). Re-evaluate if `po indicator --strict` needs it. |
| 12 | 120 runtime golden fixtures | `research/pinecone/tests/testdata/` | MPL-2.0 | `po behavior <feature>` per-feature substrate | **Deferred.** Lower yield once pine-tools JSON ships the structured signatures (item 5). |
| 13 | Vendored TV docs scraper (50 LOC) | `research/pinecone/crates/pine-reference/src/lib.rs:9-58` | MPL-2.0 | Refresh v6.md snapshot when TV publishes updates | **Deferred.** pine-tools' scraper is the upstream of record now; pine-oracle re-vendors from pine-tools, not from TV directly. |
| 14 | Seven Pine quirk patterns | `research/PineTS/src/namespaces/README.md` | AGPL paraphrase (clean-room) | BM25 substrate for "how does Pine handle X" | **Deferred.** PineForge narrative pages (item 8) cover most of the same ground without the AGPL paraphrase cost. |
| 15 | Namespace enumeration (`KNOWN_NAMESPACES`, `FACTORY_METHODS`) | `research/PineTS/src/transpiler/settings.ts` | Not copyrightable (facts) | Structured data for `po namespaces` / `po factories` | **Deferred.** Subsumed by pine-tools' constants.json + keywords.json. |
| 16 | `.pine.ts` + `.expect.json` compat-test format | `research/PineTS/tests/compatibility/` | Format only | Schema for `po indicator --strict` | Adopted as the `indicators/<slug>/expect.json` value-token shape; implementation is clean-room Rust |

### What we don't pull from pine-tools

Explicit exclusions so a future reader doesn't assume these are in scope:

- **The TypeScript parser / lexer / type-checker.** piners-syntax is our own Rust parser; the oracle uses it for `po parse`, `po tokens`, and local-tier `po validate`. Lifting pine-tools' parser would force a node runtime into the oracle and diverge our validator behavior from piners' own runtime behavior. Dogfood instead.
- **The LSP server (`packages/lsp/bin/pine-lsp.js`).** The oracle is a CLI for one-shot queries, not a long-lived editor backend.
- **The MCP server (`packages/mcp/bin/pine-mcp.js`).** The oracle CLI is itself the integration surface; we don't want a server-of-servers.
- **The VS Code extension (`packages/vscode/`).** Out of scope entirely.
- **`pnpm run discover:behavior` runtime invocation.** The output (`function-behavior.json`) is vendored; we don't re-derive it from the oracle.

The data pipeline (`crawl`, `scrape`, `generate`, `discover:behavior`) stays in pine-tools as the upstream source of truth for refreshing pine-data. The oracle consumes the generated artifacts but does not run the pipeline.

## Delivery phases

Pinecone has a working ~250-LOC markdown query layer at `crates/pine-reference/src/lib.rs:60-203` plus `bin/main.rs`. It parses `spec/v6.md` with `comrak`, splits on level-2 / level-3 boundaries, supports exact-match and prefix search, and runs as a CLI in Pinecone.

Phase 1 lifts this verbatim (MPL-2.0, file-level copyleft, add SPDX header), wraps it in the subcommand surface, and adds JSON output. That gives `po lookup <name>` and reference-only lookup/search over the entire 941-entry v6 reference.

Phase 2 adds the `tantivy` BM25 index over all baked knowledge sources: the v6 reference, corpus probe summaries, the PineForge audit doc, PineForge narrative docs, and structured pine-data behavior entries. The index is rebuilt in memory on first search rather than shipped as a sidecar.

Phase 3 bakes the PineForge corpus and adds the probe, probe-listing, behavior, parser, token, validation, and trade-list diff commands.

Phase 4 integrates piners' runtime pieces: piners-syntax and piners-runtime back deep local validation, and piners-runner backs `po indicator --strict`. The command substrate is in place; OHLCV and real TV baseline fixtures unlock useful strict coverage.

## Subcommands

```
pine lookup <name>              function/constant/var details
pine validate <code-or-file>    local validator; also --code CODE, --file PATH, or -
pine validate --strict <input>  TV-broker yes/no oracle; diagnostics are not actionable
pine parse <code-or-file>       AST as JSON; also --code CODE, --file PATH, or -
pine tokens <code-or-file>      lexer tokens with line/indent; also --code CODE, --file PATH, or -
pine search <query>             BM25 across all sources, ranked
pine search <query> --kind ?    list searchable source kinds + document counts
pine behavior <name>            polymorphism, side-effects, series-vs-simple, na-propagation
pine behavior [text] --list     list baked behavior entries; add --kind / --grep to narrow
pine probe <slug>               probe contents: strategy.pine + tv_trades.csv + summary
pine probes                     list all baked probes
pine probes --grep <text>       list probes whose slug or extracted summary matches text
pine probes --feature <name>    list probes whose strategy.pine uses the named Pine feature (`?` lists the catalog)
pine diff <probe> <trades.csv>  tier-classify a piners trade list against the probe's tv_trades
pine diff ... --show-diffs N    + worst-N matched pairs (ranked) + every TV/user orphan trade
pine indicator --list           list baked indicator strict fixtures; add --grep / --baseline to narrow
pine indicator --baseline ?     list indicator baseline kinds with counts
pine indicator <slug>           inspect fixture source, bars, expected keys/previews, actual keys, tolerance, and metadata
pine indicator <slug> --metadata-only
                               inspect fixture metadata without running the source
pine indicator <slug> --actual  run fixture and print actual runner output series without diffing
pine indicator --strict <slug>  per-bar indicator parity against a vendored TV baseline
pine indicator --strict --all   run every matching indicator fixture; add --grep / --baseline to narrow
pine version                    pine-data snapshot metadata + bake counts + binary version
```

Global flags:

- `--format json|text` (default: `text` for tty, `json` for pipes)
- `--no-color` (suppress ANSI styling in text mode; also auto-suppressed when `NO_COLOR=1` is set, when output is JSON, or when stdout isn't a tty; applies to styled text emitters such as `po search` and `po validate`)
- `--quiet` (suppress non-data status/note text where a text-mode command emits it; JSON output is unchanged; effectively a no-op for `po parse`, `po tokens`, and `po diff` because those commands emit only data -- there is no status/note text to suppress)

## Output format

Every subcommand emits stable JSON under `--format json`. Agents parse in one read. Human-oriented `po validate` text includes source-line caret frames; JSON diagnostics remain compact and location-bearing.

### Schema versioning

Every JSON payload carries a top-level `"schema_version": <integer>`. The initial schema is **`1`**. Object payloads (e.g. `lookup`, `probe`, `behavior`, `diff`, `validate`) receive the field inline at the top level alongside the payload's own fields. Array payloads (e.g. `tokens`, `probes`) are wrapped as `{ "schema_version": 1, "items": [...] }`. Scalar payloads (none in the initial surface, but reserved) wrap as `{ "schema_version": 1, "value": ... }`.

Agents should:

- Read `schema_version` first; if it does not match the version they were written against, refuse the payload or fall back to a less specific interpretation.
- Treat any unknown top-level fields as forward-compatible additions and ignore them.

The version bumps when:

- A field is removed or renamed (breaking).
- A field's type changes (breaking).
- An enum variant is removed or renamed (breaking).
- The array-vs-object wrapping of a payload changes (breaking).

The version does **not** bump when:

- A new optional field is added.
- A new enum variant is added (callers should already handle "unknown variant" defensively).
- Field values gain new categories of content (e.g. `tier` adds a new label).

### Example: `lookup`

```json
{
  "schema_version": 1,
  "category": "Functions",
  "name": "math.max",
  "content": "..."
}
```

### Example: `tokens`

```json
{
  "schema_version": 1,
  "items": [
    { "typ": { "Ident": "x" }, "lexeme": "x", "line": 1, "column": 1 },
    { "typ": "Eof", "lexeme": "", "line": 1, "column": 2 }
  ]
}
```

## Per-probe descriptions

Each baked `strategy.pine` carries an author-written header comment block (title line + `Purpose:` paragraph + often `Trade shape:` / `TV setup:` paragraphs) below the Apache-2.0 boilerplate. `corpus::summary_for` collects every prose comment line from the header up to the first real code line at first invocation, caches the result behind a `OnceLock`, and exposes it through `Probe::summary` + `ProbeListing::summary`. License / SPDX / copyright / `//@version=` directive lines are filtered as noise; blank `//` separators between paragraphs are collapsed so multi-paragraph headers join into one space-separated string. 100% of the 239 baked probes yield a substantive summary (median ~650 chars). `po probes --grep <text>` matches against slug OR summary.

PineForge engine source comments reference probes by engine-internal slugs (`magnifier-dist-probe-08b`, engine-history numbers 52..97, etc.) that don't appear in the published corpus. The renaming to topical slugs (`oca-multi-bracket-isolation-01`, `magnifier-tick-dist-endpoints-01`, etc.) was not bijective and no mapping table ships with the corpus. Treat engine-history names as prose annotations only; the published slugs are the canonical lookup key.

## Strict-mode indicator test format

Adopted from PineTS's `.pine.ts` + `.expect.json` compatibility-test architecture (data format only; AGPL implementation is paraphrased clean-room).

Per-indicator fixture layout:

```
indicators/<slug>/
  source.pine        # Pine v6 indicator source
  bars.json          # OHLCV fixture; typically BTCUSDC daily 2025-01-01..2025-11-20 cited window
  expect.json        # per-bar expected outputs, with custom NaN/Infinity tokens
  metadata.json      # baseline kind plus optional TV capture metadata
```

`bars.json` accepts either a bare array of OHLCV bars or the richer object form below. The object form is preferred because it pins chart context for time/session-sensitive scripts:

```json
{
  "symbol": "BTCUSDC",
  "timeframe": "1D",
  "source": "tradingview",
  "bars": [
    { "timestamp": 1735689600, "open": 1.0, "high": 1.0, "low": 1.0, "close": 1.0, "volume": 1.0 }
  ]
}
```

`expect.json` schema:

```json
{
  "schema_version": 1,
  "indicator_slug": "ema-cross",
  "outputs": {
    "ema_fast": [12.3, 12.4, "__NaN__", 12.6],
    "ema_slow": ["__undefined__", 11.9, 12.0, 12.1],
    "signal":   [false, false, true, false]
  },
  "test_range": {
    "start": "2025-10-01T00:00:00Z",
    "end":   "2025-11-20T00:00:00Z"
  }
}
```

`metadata.json` schema:

```json
{
  "baseline": "tv",
  "pine_version": "6.0.0",
  "tv_snapshot": "2026-05-20",
  "notes": "Manual TV capture notes"
}
```

`baseline` must be `"smoke"` (deterministic substrate fixtures) or `"tv"` (TradingView-captured baselines). There is no implicit third tier; any other value is rejected at load time, and a missing `metadata.json` is treated as `"smoke"`. TV baselines additionally require `pine_version` + `tv_snapshot`; smoke fixtures must not define `tv_snapshot`. `po indicator --list`, `po indicator --baseline ?`, and `po version` report smoke vs TV counts separately.

`test_range` is optional. When present, the comparer only checks bars whose `bars.json` Unix-second timestamps fall between `start` and `end` inclusive. Expected output arrays may be either full-series length or already sliced to the range length; mismatch reports still use the original zero-based bar index. Output keys may use the runner's generated keys (`plot`, `plot#1`, `plotshape`, etc.) or a plot title such as `"Close Line"` when the Pine call supplies one. Duplicate titles are disambiguated with `#1`, `#2`, etc. Empty output keys and empty expected series are rejected by strict fixture validation. Tolerance must be finite, non-negative, and no greater than `0.001`. For recognized fixed timeframes, adjacent bar timestamps must not be shorter than the timeframe. Fixed `M` month timeframes use a conservative 27-day minimum spacing check because calendar months vary.

Custom value tokens:

| Token | Pine value |
|---|---|
| `"__NaN__"` | `na` (NaN float) |
| `"__Infinity__"` | `+inf` |
| `"__-Infinity__"` | `-inf` |
| `"__undefined__"` | unset / before warmup |

`po indicator <slug>` inspects one fixture without running value parity: source, bar count, bar window, expected output keys, actual runner output keys, expected lengths, first/last expected values, tolerance, notes, and baseline metadata. It compiles and runs the fixture once only to populate actual output keys and key-drift fields; add `--metadata-only` to skip that runner check for a cheap source/bars/expect read. `po indicator <slug> --actual` runs the fixture once and reports piners-runner's actual output keys + series without comparing against `expect.json`; JSON output also includes `runner_expect`, an `expect.json`-shaped object for deterministic smoke fixture authoring. `runner_expect` intentionally omits fixture metadata such as `pine_version` and `test_range`, and uses zero tolerance. This is the authoring/debug path for smoke fixtures and output-key drift. `po indicator --strict <slug>` runs `source.pine` through piners-runner against `bars.json`, serializes outputs with the same token convention, diffs against `expect.json`, and exits non-zero on mismatch. Discrepancy report cites bar index + output name + expected vs actual. `po indicator --strict --all` runs every matching fixture and returns an aggregate report. `po indicator --list` lists baked fixtures with symbol/timeframe, bar count, output count, range window, and baseline metadata; list mode and batch strict mode both accept `--grep TEXT` plus `--baseline smoke|tv`. The `smoke-*` fixtures are deterministic substrate checks covering basic plot replay, plot-title matching, duplicate-title disambiguation, warmup `na`, ranged comparison, bool `plotshape`, and same-symbol `request.security`; real TV baselines still need to be added under `indicators/`.

Strict reports include `expected_output_keys` and `actual_output_keys` so fixture authors can see the exact keys produced by piners-runner when a capture uses titles, duplicate titles, or generated fallback names.

Baselines are regenerated by running the indicator on TV (manual paste + log capture, similar to the PRNG fixture workflow in `docs/prng-parity.md`). `metadata.json` pins the TV version + date so regenerated baselines are reproducible.

## Installation

Three paths, all systemwide:

- `cargo install pine-oracle` (binary name `po`)
- `brew install <tap>/pine-oracle/pine-oracle` (Homebrew tap, tap name TBD)
- Manual `git clone && cargo install --path .`

No first-run setup, no `corpus install` flow, no on-disk state. The corpus is baked in; the binary is self-contained. Binary size lands around 75-80 MB and stays there: the OHLCV span metadata is baked (~500 bytes) but the raw OHLCV CSVs are not, and `po indicator --strict` reads bars from per-fixture `bars.json` rather than the corpus feed.

## Agent integration

Once `po` is installed, piners' AGENTS.md gets one rule:

> Before making a Pine-semantics or trade-list-parity claim, query `po`. Cite the query in the finding. If `po` disagrees with your initial read, use `po`'s answer.

The same rule lands in pine-tools' AGENTS.md, in any future Pine-related project, and in `~/.claude/CLAUDE.md` for global default behavior. Reviewers get oracle access by default rather than via per-prompt reminders.

`.claude/settings.json` in piners pre-approves `po` invocations so no permission prompts fire:

```json
{
  "permissions": {
    "allow": ["Bash(po *)"]
  }
}
```

Concrete reviewer flow, before vs after:

- **Before.** Reviewer claims "`array.mode` returns smallest on ties". Orchestrator reads claim, has no way to check, files it as MAJOR. Three sessions later, a different reviewer claims the opposite. Both findings exist; nobody knows which is right.
- **After.** Reviewer claims "`array.mode` returns smallest on ties". Orchestrator (or the reviewer itself) runs `po lookup array.mode`. JSON answer cites TV docs. Claim is corrected or confirmed before being filed. Contradictions across sessions are impossible because every claim cites the same oracle.

## Build pipeline

In pine-tools:

1. `pnpm run crawl` + `pnpm run scrape` + `pnpm run generate` produce `pine-data/v6/*.ts`.
2. `pnpm run discover:behavior` produces `pine-data/v6/function-behavior.json`.
3. JSON snapshots (functions / variables / constants / keywords) ship alongside the `.ts` in pine-tools' git tree, so pine-oracle vendors them by copy.

In pine-oracle:

4. Refresh: copy `pine-tools/pine-data/v6/*.json` into `vendor/pine-data/v6/`, commit.
5. `brokkr check` to rebuild + revalidate; `include_str!` picks up the new JSON at compile time.
6. CI publishes a release per pine-data refresh (semver: patch for data refresh, minor for new subcommands, major for output-schema breakage).

Release cadence: pin to pine-tools' scrape cadence. When TV's docs change, refresh, rebuild, release.

## Resolved decisions

1. **Validator backend.** The local tier (`po validate`) is backed by **piners-syntax** and **piners-runtime**: piners-syntax owns lex / parse / type / semantic diagnostics, while piners-runtime supplies the primary builtins table. pine-oracle augments that table from vendored pine-tools JSON for public symbols piners does not expose yet. The WASM-bundle-pine-tools bridge option from an earlier draft is dropped: it would add a second syntax authority. The "bundled Node runtime + pine-tools JS" option is dropped too: it forces a 50 MB+ node payload into the binary and violates the zero-on-disk-scratch contract via npm cache assumptions.

2. **Repo layout.** New sibling Rust repo `pine-oracle/`. Not inside piners (would signal "piners helper", slow piners' build), not inside pine-tools (would force a Rust crate into a TS monorepo). pine-tools stays the upstream data source via `pnpm run export:json`.

3. **Corpus distribution.** Baked into the binary. Vendored under `vendor/pineforge-corpus/`, embedded via `include_dir!()`, ~72 MB baked subset, no on-disk state at runtime.

4. **Output schema stability.** Every JSON payload carries `schema_version: 1` (objects inline, arrays wrapped as `{schema_version, items}`). Bump rules + agent-side contract documented in the "Output format" section.

5. **`validate --strict` auth.** No auth required. The TradingView pine-lint endpoint is open; pine-oracle stores no credentials anywhere because it stores nothing anywhere.

6. **Per-probe summary ownership.** `corpus::summary_for(slug)` collects every prose comment line from each baked `strategy.pine`'s header (title + `Purpose:` + `Trade shape:` + `TV setup:` etc.), joined on whitespace, no LLM curation pass required, no external dependency. 100% of the 239 probes get a substantive multi-paragraph summary out of the box (median ~650 chars).

7. **Search corpus coverage.** BM25 indexes the v6 reference (941 entries), the baked corpus probes (239, with author-extracted summaries), the PineForge audit doc, and PineForge narrative pages. Hits carry a `kind` discriminator ("reference" / "probe" / "audit" / "docs") so consumers can route. Broader sources (pine-tools issue tracker, TV release notes, Pine v6 migration guide) are off the table for now: each broadens recall but dilutes precision; grow with demand.

8. **Caching `validate --strict` responses.** No cache. pine-oracle has zero on-disk state; a CLI invocation hits the API once and exits, so cross-invocation caching has nowhere to live.

9. **TV v6.md redistribution.** **Resolved:** vendor with disclaimer, consistent with the standing "bake everything into the binary" decision. `vendor/pine-reference/NOTICE` already calls out the file as a TV-docs snapshot with TradingView copyright. The top-level NOTICE adds the trademark disclaimer. Relies on Pinecone's MPL-2.0 redistribution precedent; if TV objects on a public release, the lift target is to switch to a stripped-reference shape (names + signatures only, no prose) without re-architecting the binary.

10. **License umbrella for the oracle binary.** Vendoring decisions span MPL-2.0 (Pinecone), Apache-2.0 (PineForge), MIT (pine-tools, folknor-owned), and clean-room paraphrases of AGPL (PineTS, deferred). The crate itself uses MPL-2.0; vendored artifacts retain their upstream licenses through their own LICENSE / NOTICE files.

11. **Indicator expect schema versioning.** Every `expect.json` carries `schema_version`. `pine_version` is optional fixture metadata; `tv_snapshot` is only legal for `baseline: "tv"` fixtures and may live in `expect.json` or `metadata.json`. Smoke fixtures must not define `tv_snapshot`, and `po indicator <slug> --actual` omits optional metadata from its generated `runner_expect`. Concrete TV-baseline shape:

    ```json
    {
      "schema_version": 1,
      "indicator_slug": "ema-cross",
      "pine_version": "6.0.0",
      "tv_snapshot": "2026-05-20",
      "outputs": { "ema_fast": [12.3, "__NaN__"], ... },
      "test_range": { "start": "...", "end": "..." }
    }
    ```

## Out of scope

- **A piners runtime substitute.** `po` does not run arbitrary Pine or produce trades. The narrow exception is `po indicator --strict`, which replays baked indicator fixtures through piners-runner only to compare against a frozen baseline.
- **Pine code generation.** `po` does not write Pine; it explains and validates Pine.
- **A general TV API client.** No charts, no symbols, no quotes. Strictly Pine semantics + the cross-validated corpus.
- **PRNG fingerprinting automation.** `po` does not run scripts on TV's broker. The TV-pasteable fixture in `docs/prng-parity.md` is a manual workflow; the oracle just consumes the resulting baseline once we have it.

## What this unlocks

With `po` in place and AGENTS.md citing it, the failure mode that prompted this doc (reviewers contradicting each other across sessions on what TV "actually does") becomes structurally hard. Every parity claim has a CLI receipt. Every disagreement points at the oracle, not at a human. Reviews stop relitigating semantics and focus on whether piners matches the cited semantics.

Secondary win: the same tool serves pine-tools' own dogfooding, future Pine projects, and anyone outside our orbit who wants a fast Pine reference CLI. The investment compounds across every Pine workflow we touch.
