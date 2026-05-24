# The pine oracle

A systemwide CLI tool that answers every Pine v6 question we currently answer from memory. Lives in its own Rust repo `pine-oracle/` -- not in piners, not in pine-tools. Installed once per machine. Queried by any agent in any session in any Pine-adjacent project.

## Problem

Across review sessions, claims about Pine v6 semantics drift. A reviewer in session N says "TV's `array.mode` returns the smallest value on ties"; a reviewer three sessions later says "TV returns the first-encountered value." Neither is wrong from memory; both are wrong from evidence. There is no single command an agent can run to ask the canonical source.

Two canonical sources exist:

- **pine-tools** (`../pine-tools/`) -- scraped TV docs: function signatures, types, polymorphism, behavior flags, type-coercion rules, the parser/validator itself.
- **The corpus** (`vendor/pineforge-corpus/validation/`) -- 235 Pine strategies cross-validated trade-for-trade against TradingView's broker emulator, baked into the binary. The executable parity oracle. See `docs/corpus.md`.

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

Working name: `pine`. The only meaningful conflict is the legacy Pine email client (mostly extinct; survives as `alpine` in some distros). Fallbacks if conflict matters: `pinec`, `pine-oracle`, `pq` (pine query). Pick at install time; the doc uses `pine`.

## Architecture

Single binary, four layers.

### 1. Vendored pine-data (compiled into the binary)

Two sources, merged at build time:

- **Primary: Pinecone's `crates/pine-reference/spec/v6.md`** -- 918 KB / 24,847 lines / **941 entries**, one per `### name`, uniform sub-sections (`Syntax`, `Arguments`, `Example`, `Type`, `Remarks`, `See also`). MPL-2.0 vendored copy of TradingView's published v6 reference. The single highest-ROI artifact in the research corpus. Embed via `include_str!`, parse once with `comrak` at startup (cached), produce the lookup table and BM25 documents.
- **Secondary: pine-tools' `pine-data/v6/*.ts`** -- consumed as JSON. The normalization belongs upstream in pine-tools (a new `pnpm run export:json` step emitting `pine-data/v6/*.json` alongside the `.ts`), not in the oracle's build -- vendoring `.ts` would force a node toolchain into the oracle's pipeline and defeat the "pure Rust binary" goal. Adds polymorphism markers (`function-behavior.json`), TextMate grammar, structured signature metadata that complements v6.md's prose.

About 1-2 MB combined; trivial to compile in.

### 2. BM25 index (compiled into the binary)

Built at `pine` build time over:

- pine-data function descriptions (per `### name` block in v6.md)
- PineForge's `docs/pine_v6_audit_master.md` -- 38 critical + ~62 minor known TV-vs-PineForge divergences. Exactly what `pine behavior <name>` should return for "documented divergence" queries.
- PineForge's `docs/pages/*.md` -- 16 narrative docs (magnifier, mtf, timeframes, report-schema, lifecycle, abi-stability). High-ROI for "explain X" queries.
- PineForge's `pineforge.h` doxygen blocks -- broker/strategy/magnifier semantic enumerations.
- Pinecone's `tests/testdata/` -- 120 atomic `.pine` files with embedded `// Expected output:` comments. Per-feature behavior substrate.
- per-probe `strategy.pine` source from the corpus
- per-probe summaries from `docs/probe-summaries.md` (currently 0/235 aligned to published slugs; re-derivation pending -- see "Per-probe descriptions" below)
- PineTS-derived Pine quirk patterns, paraphrased clean-room from `research/PineTS/src/namespaces/README.md` (the seven enumerated patterns: auto-gen indices, OO collections, `param()` shim, `__value` rewrite, epsilon equality, dual-getter properties, per-call-site state IDs).

Stored as a `tantivy` index serialized into the binary or sidecar files. Queryable in <10 ms.

### 3. Vendored corpus (baked)

The corpus ships **inside the binary**. Reality after vendoring: the published PineForge corpus is ~245 MB cloned (38 MB git history, 75 MB OHLCV across four feeds, 235 probes' worth of strategy.pine + tv_trades.csv + engine_trades.csv + generated.cpp + reports). The vendored subset pine-oracle bakes is the strict minimum the public subcommands need: per-probe `strategy.pine`, `tv_trades.csv`, and optional `inputs.json`. Everything else (OHLCV feeds, PineForge's own `engine_trades.csv`, the transpiler's `generated.cpp`, validation reports, upstream tooling) is pruned out of `vendor/pineforge-corpus/` and not in the binary.

Baked subset size: ~72 MB. Final binary size lands around 75-80 MB. Embedded via `include_dir!()` at compile time, queried as `&'static str` slices at runtime. Zero on-disk scratch, zero env vars, zero settings files.

Pruning is reproducible: `scripts/prune-vendored-corpus.sh` operates on a fresh clone of <https://github.com/fullpass-4pass/pineforge-corpus>; see `vendor/pineforge-corpus/VENDORING_NOTES.md` for the kept / dropped manifest and the refresh procedure.

When `pine indicator --strict` and `pine diff` v1 land, OHLCV gets added to the bake (the prune script grows a `--keep-data` flag or similar). Binary grows ~75 MB at that point.

### 4. Validator backend

Two tiers, in inverted authority order vs. an earlier draft of this doc:

- **Local tier (`pine validate`)**. The workhorse. IDE-quality diagnostics: every lex / parse error, every type error, every semantic warning, with correct line + column positions, multi-error. This is what serious work uses. Three feasible implementations:
  - **Use piners-syntax** once it stabilises. piners has to lex + parse + type-check Pine to execute it for backtesting; that analyzer is the natural Rust home and pine-oracle should depend on it.
  - **Transpile pine-tools' TS analyzer to WASM** at build time. Brings the mature IDE validator along intact; preserves single-binary purity. Bridge option if piners-syntax isn't ready in time.
  - **Today: the pinecone-lifted lexer + parser in `src/syntax/`**. v0 stand-in: catches the first lex / parse error and stops. No type checking. Replaced by one of the two options above when ready.
- **Strict tier (`pine validate --strict`)**. **Yes / no oracle only. Do not try to fix your script from its diagnostics.** TradingView's `pine-facade/translate_light` endpoint is profoundly bad as a validator: it stops at the first error, breaks on trailing whitespace (e.g. an extra space at end of line is "invalid"), and reports the wrong line / column for essentially every diagnostic. The diagnostic prose is non-actionable: it tells you *something* is wrong but not where or what in any reliable way. The only trustworthy output is the `success` bit (true / false). Use this exactly once, after you believe `pine validate` (local tier) reports clean: a final yes / no from TV's broker before you publish. Do not iterate against it; iterate against the local tier. No auth required, no on-disk cache.
- **Indicator strict tier (`pine indicator --strict <probe>`)**. Different oracle: runs an indicator against fixture bars and diffs per-bar values against a vendored baseline using the PineTS-derived `.expect.json` schema (see "Strict-mode indicator test format" below). Corpus is trade-list parity; this is per-bar indicator parity.

## Vendoring inventory

What to pull from where, in priority order. The oracle's license is chosen to be compatible with whatever we vendor (MPL-2.0 or Apache-2.0 are the natural umbrella choices; AGPL is unnecessary because we paraphrase the AGPL sources rather than copy them).

| # | Artifact | Source | License | Use | Effort |
|---|---|---|---|---|---|
| 1 | `spec/v6.md` (941 reference entries) | `research/pinecone/crates/pine-reference/` | MPL-2.0 | Primary BM25 substrate + lookup table | S |
| 2 | Markdown query layer (~250 LOC) | `research/pinecone/crates/pine-reference/src/lib.rs:60-203` + `bin/main.rs` | MPL-2.0 | MVP `pine lookup` backend before BM25 lands | S |
| 3 | 21 probe summaries harvested from engine comments | `research/pineforge-engine/src/engine_*.cpp` + tests | Apache-2.0 | Per-probe BM25 substrate; see `docs/probe-summaries.md` | Done (raw); polish + upstream pending |
| 4 | `docs/pine_v6_audit_master.md` (38 critical + ~62 minor divergences) | `research/pineforge-engine/` | Apache-2.0 | `pine behavior <name>` "known divergence" payload | S |
| 5 | `docs/pages/*.md` (16 narrative docs) | `research/pineforge-engine/` | Apache-2.0 | BM25 substrate for "explain X" queries | S |
| 6 | `pineforge.h` (393 lines, doxygen-rich) | `research/pineforge-engine/include/pineforge/` | Apache-2.0 | Broker / strategy / magnifier semantic enumerations | S |
| 7 | 120 runtime golden fixtures | `research/pinecone/tests/testdata/` | MPL-2.0 | `pine behavior <feature>` per-feature substrate | S |
| 8 | 46 parser golden fixtures | `research/pinecone/crates/pine-parser/testdata/` | MPL-2.0 | `pine parse` validator corpus | S |
| 9 | Vendored TV docs scraper (50 LOC) | `research/pinecone/crates/pine-reference/src/lib.rs:9-58` | MPL-2.0 | Refresh v6.md snapshot when TV publishes updates | S |
| 10 | `scripts/verify_corpus.py` (26.1 KB Python) | `research/pineforge-engine/` | Apache-2.0 | Port to Rust for `pine diff <probe> <trades.csv>` tier classification | M |
| 11 | Seven Pine quirk patterns (paraphrased) | `research/PineTS/src/namespaces/README.md` | AGPL paraphrase (clean-room) | BM25 substrate for "how does Pine handle X" | S |
| 12 | Namespace enumeration (KNOWN_NAMESPACES, FACTORY_METHODS, etc.) | `research/PineTS/src/transpiler/settings.ts` | Not copyrightable (facts) | Structured data for `pine namespaces` / `pine factories` | S |
| 13 | `.pine.ts` + `.expect.json` compat-test format | `research/PineTS/tests/compatibility/` | Format only (data not copyrightable) | Schema for `pine indicator --strict` | M |

PineForge's Python verifier (`scripts/verify_corpus.py`) is the canonical implementation of trade-list alignment + tiering. Re-implement in Rust for native integration; the algorithm is documented in `docs/corpus.md` and the Python source.

### What we don't pull from pine-tools

Explicit exclusions so a future reader doesn't assume these are in scope:

- **The TypeScript parser / lexer / type-checker.** piners-syntax is our own Rust parser; the oracle uses it for `pine parse`, `pine tokens`, and local-tier `pine validate`. Lifting pine-tools' parser would force a node runtime into the oracle and diverge our validator behavior from piners' own runtime behavior. Dogfood instead.
- **The LSP server (`packages/lsp/bin/pine-lsp.js`).** The oracle is a CLI for one-shot queries, not a long-lived editor backend.
- **The MCP server (`packages/mcp/bin/pine-mcp.js`).** The oracle CLI is itself the integration surface; we don't want a server-of-servers.
- **The VS Code extension (`packages/vscode/`).** Out of scope entirely.
- **`pnpm run discover:behavior` runtime invocation.** The output (`function-behavior.json`) is vendored; we don't re-derive it from the oracle.

The data pipeline (`crawl`, `scrape`, `generate`, `discover:behavior`) stays in pine-tools as the upstream source of truth for refreshing pine-data. The oracle consumes the generated artifacts but does not run the pipeline.

## MVP path: ship `pine lookup` before BM25

Pinecone has a working ~250-LOC markdown query layer at `crates/pine-reference/src/lib.rs:60-203` plus `bin/main.rs`. It parses `spec/v6.md` with `comrak`, splits on level-2 / level-3 boundaries, supports exact-match and prefix search, and runs as a CLI today.

MVP for `pine`: lift this verbatim (MPL-2.0, file-level copyleft, add SPDX header), wrap in our subcommand surface, add JSON output. That gives us `pine lookup <name>` and `pine search <prefix>` for the entire 941-entry v6 reference on day one. BM25 with `tantivy` plus the supplementary indexed material (PineForge divergences, PineTS patterns, probe summaries) is the v2.

Skipping BM25 for v1 also defers the question of how to ship the index (in-binary blob vs sidecar file vs build-on-first-use).

## Subcommands

```
pine lookup <name>              function/constant/var details
pine validate <code>            local validator: every lex / parse / type / semantic diagnostic
pine validate --strict <code>   TV-broker yes/no oracle; error messages are not actionable
pine parse <code>               AST as JSON
pine tokens <code>              lexer tokens with line/indent
pine search <query>             BM25 across all sources, ranked
pine behavior <name>            polymorphism, side-effects, series-vs-simple, na-propagation
pine probe <slug>               probe contents: strategy.pine + tv_trades.csv + summary
pine probes                     list all baked probes
pine probes --grep <text>       list probes whose slug matches text (summary text once re-curated)
pine diff <probe> <trades.csv>  tier-classify a piners trade list against the probe's tv_trades
pine version                    pine-data snapshot date + corpus revision + binary version
```

Global flags:

- `--format json|text` (default: `text` for tty, `json` for pipes)
- `--no-color`
- `--quiet` (suppress headers, just return the data)

## Output format

Every subcommand emits stable JSON under `--format json`. Agents parse in one read. Example for `lookup`:

```json
{
  "name": "math.max",
  "kind": "function",
  "overloads": [
    {
      "parameters": [
        {"name": "number0", "type": "series<int|float>"},
        {"name": "number1", "type": "series<int|float>"}
      ],
      "return_type": "series<int|float>"
    }
  ],
  "behavior": {
    "na_propagation": "yes",
    "polymorphic": false,
    "series_or_simple": "both",
    "variadic": true
  },
  "source": "tradingview-docs",
  "snapshot_date": "2026-04-12"
}
```

Schema versioning: every JSON payload carries `"schema_version": N`. Bumps when output shape changes. Agents pin a minimum version.

## Per-probe descriptions: the BM25 unlock

The corpus has no human-readable per-probe descriptions today. `strategy.pine` is the only text per probe, and raw Pine is poor BM25 substrate (keywords overlap, function names dominate, intent is opaque).

Adding a 1-3 sentence "what Pine semantic does this probe exercise" per slug is the single biggest oracle win. Shifts the corpus from "235 .pine files BM25 can barely use" to "235 searchable forensic cases".

### Current status: 0/235 aligned to published slugs

`docs/probe-summaries.md` contains 21 harvested summaries, but those summaries reference engine-internal probe identifiers (`magnifier-dist-probe-01..08b`, `ies-probe-08`, `parity-probe-03..06`, `oca-three-way-probe-02`, `typed-matrix-probe-01-bool-regime-mask`, `anomaly-equity-mirror`, and engine-history numbers 52..97) that **do not appear in the published corpus**. The prose is solid; only the slug keys are wrong. Re-mapping is open work. Until that lands, `corpus::summary_for(slug)` returns `None` for every slug and `pine probes --grep` filters on slug substring only.

### Re-derivation paths

Two paths, not mutually exclusive:

- **LLM pass against real slugs.** Feed each of the 235 baked `strategy.pine` files to a model with prompt "in 2 sentences, what Pine v6 semantic does this probe exercise". Commit output to `docs/probe-summaries.md` keyed by the published slug. Human-review suspect ones.
- **Pattern-match against `docs/pine_v6_audit_master.md`** -- some probes exercise the divergences PineForge already documented. Link probe -> divergence-class in the summary.

Land descriptions upstream in PineForge if possible (every consumer benefits). Fork pine-oracle-side if not.

### Engine-history vs published slugs

PineForge engine source comments reference probes by old numbers and engine-internal slugs (52, 62, 80, 83, 92, 93, 95-97; `magnifier-dist-probe-08b`; etc.). The published corpus uses entirely different topical slugs (`oca-multi-bracket-isolation-01`, `magnifier-tick-dist-endpoints-01`, etc.). The renaming was not bijective and no mapping table ships with the corpus. Recovering that mapping is a separate forensic exercise; for now, treat engine-history slugs as prose annotations and the published slugs as the canonical lookup key.

## Strict-mode indicator test format

Adopted from PineTS's `.pine.ts` + `.expect.json` compatibility-test architecture (data format only; AGPL implementation is paraphrased clean-room).

Per-indicator fixture layout:

```
indicators/<slug>/
  source.pine        # Pine v6 indicator source
  bars.json          # OHLCV fixture; typically BTCUSDC daily 2025-01-01..2025-11-20 cited window
  expect.json        # per-bar expected outputs, with custom NaN/Infinity tokens
  metadata.json      # which TV chart version was used to generate the baseline, snapshot date
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

Custom value tokens:

| Token | Pine value |
|---|---|
| `"__NaN__"` | `na` (NaN float) |
| `"__Infinity__"` | `+inf` |
| `"__-Infinity__"` | `-inf` |
| `"__undefined__"` | unset / before warmup |

`pine indicator --strict <slug>` runs `source.pine` through piners' engine against `bars.json`, serializes outputs with the same token convention, diffs against `expect.json`. Discrepancy report cites bar index + output name + expected vs actual.

Baselines are regenerated by running the indicator on TV (manual paste + log capture, similar to the PRNG fixture workflow in `docs/prng-parity.md`). `metadata.json` pins the TV version + date so regenerated baselines are reproducible.

## Installation

Three paths, all systemwide:

- `cargo install pine-cli` (binary name `pine`)
- `brew install <tap>/pine/pine` (Homebrew tap, tap name TBD)
- Manual `git clone && cargo install --path .`

No first-run setup, no `corpus install` flow, no on-disk state. The corpus is baked in; the binary is self-contained. Final binary is ~75-80 MB today and grows to ~150 MB when OHLCV joins the bake for `pine diff` and `pine indicator --strict` v1.

## Agent integration

Once `pine` is installed, piners' AGENTS.md gets one rule:

> Before making a Pine-semantics or trade-list-parity claim, query `pine`. Cite the query in the finding. If `pine` disagrees with your initial read, use `pine`'s answer.

The same rule lands in pine-tools' AGENTS.md, in any future Pine-related project, and in `~/.claude/CLAUDE.md` for global default behavior. Reviewers get oracle access by default rather than via per-prompt reminders.

`.claude/settings.json` in piners pre-approves `pine` invocations so no permission prompts fire:

```json
{
  "permissions": {
    "allow": ["Bash(pine *)"]
  }
}
```

Concrete reviewer flow, today vs after:

- **Today.** Reviewer claims "`array.mode` returns smallest on ties". Orchestrator reads claim, has no way to check, files it as MAJOR. Three sessions later, a different reviewer claims the opposite. Both findings exist; nobody knows which is right.
- **After.** Reviewer claims "`array.mode` returns smallest on ties". Orchestrator (or the reviewer itself) runs `pine lookup array.mode`. JSON answer cites TV docs. Claim is corrected or confirmed before being filed. Contradictions across sessions are impossible because every claim cites the same oracle.

## Build pipeline

In pine-tools (or sister repo):

1. `pnpm run generate` produces `pine-data/v6/*.ts` (already exists today).
2. `pnpm run export:json` (new) emits `pine-data/v6/*.json` -- vendor-friendly snapshot, committed.
3. `cargo build --release` in `pine-cli/` reads JSON via `include_str!` at compile time, builds the BM25 index with `tantivy`, links into the binary.
4. CI publishes a release per pine-data update (semver: patch for data refresh, minor for new subcommands, major for output-schema breakage).

Release cadence: pin to pine-data scrape cadence. When TV's docs change, regenerate, rebuild, release.

## Open questions

1. **Validator backend.** **Partially resolved.** The local tier (`pine validate`) wants IDE-quality output: every lex / parse / type / semantic diagnostic with correct positions, not the shallow first-error stop the pinecone-lifted code does today. Three paths, in preference order:
   - **Wait for piners-syntax.** piners has to lex + parse + type-check Pine to execute it for backtesting; that analyzer is the natural Rust home and the only path that keeps the binary pure-Rust without duplicating work. pine-oracle should depend on it once it stabilises.
   - **WASM-bundle pine-tools' TS analyzer.** Bridge option: preserves single-binary purity, brings the mature IDE validator along intact. Use this if piners-syntax slips.
   - **Today: pinecone lift in `src/syntax/`.** v0 stand-in. First-error-only, no type checks. Replaced by one of the two above when ready.
   The "bundled Node runtime + pine-tools JS" option from an earlier draft is dropped: forces a 50 MB+ node payload into the binary, violates the zero-on-disk-scratch contract via npm cache assumptions.

2. **Repo layout.** **Resolved:** new sibling Rust repo `pine-oracle/`. Not inside piners (would signal "piners helper", slow piners' build), not inside pine-tools (would force a Rust crate into a TS monorepo). pine-tools stays the upstream data source via `pnpm run export:json`.

3. **Corpus distribution.** **Resolved:** baked into the binary. Vendored under `vendor/pineforge-corpus/`, embedded via `include_dir!()`, ~72 MB baked subset, no on-disk state at runtime.

4. **Output schema stability.** Agents will parse this. Schema breakage breaks every downstream prompt and every cited finding. Lock in `schema_version` early; document the deprecation policy.

5. **`validate --strict` auth.** **Resolved:** no auth required. The TradingView pine-lint endpoint is open; pine-oracle stores no credentials anywhere because it stores nothing anywhere.

6. **Per-probe summary ownership.** Re-derivation pending (see "Per-probe descriptions" above): the 21 harvested summaries in `docs/probe-summaries.md` reference engine-internal slugs that don't match the published corpus, so the alignment work is reset.

7. **Search corpus coverage.** Does BM25 also index the pine-tools issue tracker, the TV release notes, the Pine v6 migration guide? Each broadens recall but dilutes precision. Start narrow (pine-data + per-probe summaries), grow with demand.

8. **Caching `validate --strict` responses.** **Resolved:** no cache. pine-oracle has zero on-disk state; a CLI invocation hits the API once and exits, so cross-invocation caching has nowhere to live.

9. **TV v6.md redistribution.** Pinecone's vendored copy (`spec/v6.md`, 918 KB) is TradingView copyright. Pinecone's redistribution under MPL-2.0 is precedent but not blanket legal cover. Two paths:
   - Vendor with prominent "snapshot of TV docs as of <date>, all content (c) TradingView" disclaimer; rely on Pinecone's precedent.
   - Don't redistribute; ship only the *index* (function names, BM25 tokens, no prose) and link out to live TV docs for full content.
   Resolution likely depends on whether the oracle is published publicly or internal-only.

10. **License umbrella for the oracle binary.** Vendoring decisions span MPL-2.0 (Pinecone), Apache-2.0 (PineForge), and clean-room paraphrases of AGPL (PineTS, Pinescription). Cleanest umbrella: MPL-2.0 for files derived from Pinecone (file-level copyleft only), Apache-2.0 for everything else. Need to confirm before shipping.

11. **PineTS compat-test schema versioning.** Adopting the `.expect.json` format means committing to a token convention (`__NaN__`, `__Infinity__`, etc.). If TV changes a value's behavior across Pine versions, do baselines age out or migrate? Cleaner if `expect.json` carries the Pine version that generated it.

## Out of scope

- **A piners runtime substitute.** `pine` does not run Pine; it answers questions about Pine. Running Pine to produce trades is piners' job.
- **Pine code generation.** `pine` does not write Pine; it explains and validates Pine.
- **A general TV API client.** No charts, no symbols, no quotes. Strictly Pine semantics + the cross-validated corpus.
- **PRNG fingerprinting automation.** `pine` does not run scripts on TV's broker. The TV-pasteable fixture in `docs/prng-parity.md` is a manual workflow; the oracle just consumes the resulting baseline once we have it.

## What this unlocks

Once `pine` is in place and AGENTS.md cites it, the failure mode that prompted this doc (reviewers contradicting each other across sessions on what TV "actually does") becomes structurally hard. Every parity claim has a CLI receipt. Every disagreement points at the oracle, not at a human. Reviews stop relitigating semantics and focus on whether piners matches the cited semantics.

Secondary win: the same tool serves pine-tools' own dogfooding, future Pine projects, and anyone outside our orbit who wants a fast Pine reference CLI. The investment compounds across every Pine workflow we touch.
