# AGENTS.md

## Project

pine-oracle is a Rust crate producing the `pine` binary: a single-binary CLI that answers Pine v6 semantic questions across every Pine-adjacent project. Vendors TradingView's published v6 reference (via Pinecone's snapshot) plus the PineForge cross-validation corpus, exposes them as one-shot subcommands (`pine lookup`, `pine search`, `pine probe`, `pine diff`, etc.). Design doc: `docs/pine-oracle.md`.

The oracle is not a Pine runtime substitute. It answers questions about Pine; the narrow exception is baked indicator fixture replay for `pine indicator --strict`, which runs piners only to compare against a frozen baseline.

## Workspace

Single crate at the repo root.

- `pine-cli` (binary name `pine`). Modules grow as subcommands land.

### Layout

The library crate (`src/lib.rs`, surface = `pine_cli::*`) owns the domain modules listed below: pure logic with no CLI concerns. The binary crate (`src/main.rs` + `src/output.rs` + `src/commands/*.rs`) owns the CLI surface:

- `src/main.rs` - clap `Cli` + `Command` definitions, `OutputFormat::Auto/Text/Json` resolution, `main()` dispatch, `cmd_version` (the only subcommand that stays inline because it self-describes the binary it lives in).
- `src/output.rs` - shared output primitives every subcommand uses: `ResolvedFormat`, `Style` (ANSI colour wrapper with TTY / `NO_COLOR` / `--no-color` resolution), `SCHEMA_VERSION`, `versioned_json`, `print_json`. All `pub(crate)` (the binary has no external API).
- `src/commands/<name>.rs` - one file per `pine <subcommand>` (every command except `version`): `lookup`, `search`, `validate`, `parse`, `tokens`, `behavior`, `probe`, `probes`, `diff`, `indicator`. Each exposes `pub(crate) fn run(...)` taking parsed args + `ResolvedFormat` (+ `Style` when the command emits styled text). Subcommand-only helpers (AST pretty-printer, per-command text formatters) live in the same file as their consumer.

### Domain modules (`src/`)

- `reference`: in-process lookup + substring search over the vendored TradingView v6 reference (`vendor/pine-reference/spec/v6.md`, 941 entries). Cached behind `OnceLock`. MPL-2.0, lifted from pinecone.
- `corpus`: in-binary PineForge validation corpus, embedded via `include_dir`. Exposes `load_probe(slug)` and `list_probes(grep)` over 239 probes (flat + nested under `symbol-specified/<SYMBOL>/`). `summary_for(slug)` collects every prose comment line from each `strategy.pine` header up to the first real code line (skipping license / SPDX / copyright / version-directive noise, collapsing blank `//` paragraph separators), cached behind a OnceLock; covers 100% of baked probes with multi-paragraph summaries (median ~650 chars) that join the slug-title line with the author's `Purpose:` / `Trade shape:` / `TV setup:` paragraphs. `list_probes(grep)` matches against slug OR summary text. The wrapping Rust code is MPL-2.0 (project umbrella); the vendored corpus data under `vendor/pineforge-corpus/` is Apache-2.0, attributing PineForge contributors.
- `search`: BM25 via tantivy over five sources: the v6 reference (941 entries), the baked PineForge corpus (239 probes), PineForge's Pine v6 audit doc (`pine_v6_audit_master.md`, H2/H3 sections - 38 critical + ~62 minor documented TV-vs-engine divergences), PineForge's 18 narrative explainer pages (`pages/*.md`, H2/H3 sections - magnifier, mtf, timeframes, lifecycle, report-schema, etc.), and pine-data behavior entries (functions / variables / constants / keywords with signatures, params, examples, and polymorphism notes). RAM-backed Index built on first invocation (~10-15 ms), OnceLock-cached. Schema fields: `name`, `category`, `kind` ("reference" / "probe" / "audit" / "docs" / "behavior"), `content` (STORED for retrieval) + `content_search` (TEXT for ranking). `name` gets a 5x boost over `content_search`. Hits carry full body in `SearchHit.content`.
- `validate`: two tiers, with inverted authority vs. an earlier draft of the design doc.
  - **Local (`validate::check`)**: lex + parse + type + semantic analysis via piners-syntax, backed by piners-runtime builtins plus pine-data gap-fill. Returns every diagnostic piners-syntax can recover as `Diagnostic { severity, stage, code, message, line, column }`.
  - **Strict (`validate::strict`)**: POSTs the source as `multipart/form-data` to `pine-facade.tradingview.com/pine-facade/translate_light` via `ureq`, maps every error + warning the API returns into Diagnostics with `Stage::Strict`. **Yes / no oracle only - the diagnostic prose is non-actionable**. TV's pine-lint stops at the first error, breaks on trailing whitespace, and reports wrong line / column numbers; the `success` bit is the only trustworthy output. Use after the local tier reports clean, not for iterative debugging. No auth (endpoint is open), no on-disk cache, 10s timeout. Response decoding pinned by inline fixture tests; never hits the network in CI.
- `behavior`: structured signature + polymorphism lookup over pine-tools' JSON exports (`vendor/pine-data/v6/{functions,variables,constants,keywords,function-behavior}.json`). Public API: `lookup(name) -> Option<Behavior>`, `list(kind, grep)`, `kind_catalog()`, `search_entries()`, and `snapshot()`. `Behavior` is one of `Function` / `Variable` / `Constant` / `Keyword`. Function entries optionally carry a `RawBehaviorEntry` with polymorphism markers + argument-ordering. Lenient deserialization (serde defaults on optional fields) so pine-tools schema tweaks don't break the binary.
- `diff`: trade-list parity scorer, port of PineForge's `scripts/verify_corpus.py`. Public API: `diff(probe_slug, user_csv, opts) -> DiffReport`. Parses both CSVs into entry / exit pairs (Trade # joined, TV's "Date and time" interpreted in the chart timezone with default Asia/Taipei +8), aligns by direction + 1h window + $3 entry-price gate, trims to common window, computes 4-dim p90 deltas, classifies as excellent / strong / moderate / weak / minimal. Honours `inputs.json::expected_tier` ("anomaly", "engine_only") and `validation_overrides.expect_tv_match`. Strict vs production profile is auto-detected from `trail_*` parameters in `strategy.pine` (or forced via `inputs.json::parity_profile`). Threshold values mirror `verify_corpus.py` exactly. `DiffOptions::show_diffs > 0` populates `pair_diffs` (worst-N matched pairs, ranked descending by per-pair `max(entry_delta, exit_delta, pnl_delta)`) + `tv_orphans` / `user_orphans` (all unmatched trades from the trimmed window); default 0 keeps the report headline-only. V1 does not implement interior trim (`trim_bars` / `warmup_bars`) since the OHLCV feed isn't baked.
- `indicator`: per-bar indicator fixture replay. Fixtures live under `indicators/<slug>/` (`source.pine`, `bars.json`, `expect.json`, optional `metadata.json`) and are embedded with `include_dir`. Public API: `list_fixtures()` and `run_strict(slug) -> IndicatorReport`. Runs source through piners-runner, compares plot outputs against `expect.json` using the documented `__NaN__` / `__Infinity__` / `__-Infinity__` / `__undefined__` tokens, honors optional `test_range` windows, and reports output + bar-index mismatches. The `smoke-*` fixtures are deterministic substrate checks; real TV baselines are still pending.

Planned changes:

- Bake real TradingView indicator baselines under `indicators/` so `pine indicator --strict` has useful fixtures beyond the implemented runner/differ substrate.
- Add OHLCV-backed interior trim for `pine diff` once the OHLCV feed is baked.

Upstream `../piners` gaps blocking real TV indicator baselines:

- `plotshape(close > open, ...)` runner support in piners' `plot*` builtin family - pine-oracle's local validator supplements the syntax table from pine-data and accepts the bool condition, but `pine indicator --strict` still compiles through piners-runner.
- `request.security(...)` runner bridge compilation - the call shape compiles in piners-syntax but the runner has no bridge, so MTF indicators cannot replay.

Canonical homes (so cross-module duplicates collapse to one):

- `Entry` (category + name + content) lives in `reference`.
- Future `Probe`, `TradeList`, `TierReport` will live in `probe` / `diff`.

## Vendoring

`research/` is gitignored. It is third-party source we consult, not ship. Anything we ship goes under `vendor/<source>/` with:

- A `LICENSE` copy of the upstream license.
- A `NOTICE` naming the upstream, the path lifted, and the snapshot date / git ref.
- Per-file `SPDX-License-Identifier` header on every lifted source file.

Current vendors:

- `vendor/pine-reference/`: pinecone's `crates/pine-reference/spec/v6.md` (MPL-2.0). Local mod: U+00A0 NO-BREAK SPACE rewritten to U+0020 SPACE for the gremlin scan. See `vendor/pine-reference/NOTICE`.
- `vendor/pineforge-corpus/`: <https://github.com/fullpass-4pass/pineforge-corpus> (Apache-2.0), pruned to the subset baked into the binary. See `vendor/pineforge-corpus/VENDORING_NOTES.md` for kept / dropped manifest and refresh procedure. Refresh via `scripts/prune-vendored-corpus.sh`.
- `vendor/pine-data/v6/`: structured JSON snapshots from `../pine-tools/pine-data/v6/` (MIT, folknor owns pine-tools). Five files: `functions.json`, `variables.json`, `constants.json`, `keywords.json`, `function-behavior.json`. Refresh by re-running pine-tools' `pnpm run scrape` + `pnpm run discover:behavior`, then copying the JSON files in.
- `vendor/pineforge-docs/`: PineForge's `docs/pine_v6_audit_master.md` (38 critical + ~62 minor known divergences) + `docs/pages/*` (18 narrative explainers covering magnifier, mtf, timeframes, lifecycle, report-schema, abi-stability, etc.). All Apache-2.0. Local mod: em/en-dashes / NBSPs rewritten to ASCII for the gremlin scan; see `vendor/pineforge-docs/NOTICE`.

## Rules

### General rules

- Don't use gremlins! Em-dash, en-dash, strange quotes, whatever - they're all verboten.
- Don't remind the user of the rules. They wrote them, so they know them.
- The user can exempt you from any rule at any time.

### Bash rules

- Never chain commands with `&&`.
- Never chain commands with `;`.
- Never chain/pipe commands with `|`. Exception: piping into `review` is allowed.
- Never capture stdout into env vars (`UUID=$(...)`).
- Never read or write from `/tmp`. All data lives in the project.
- Never run raw `cargo`, `curl`, `pkill`. Use `brokkr`.

### git commit rules

- Always run `brokkr fmt` before a commit.
- Never commit markdown changes alone. Bundle them with upcoming code commits.
- When committing other changes: always tag along markdown files if dirty.
- Write substantive engineering-focused commit messages.
- Has `Cargo.lock` changed? Commit it.
- Never `git push` unless the user explicitly asks. Stop after the commit.

### Vendoring rules

- New vendored sources go under `vendor/<name>/` with a LICENSE copy and a NOTICE file describing source path + snapshot date.
- Every lifted source file carries an `SPDX-License-Identifier` header pointing back to the upstream.
- The `research/` tree is read-only consultation material. Never edit it, never depend on its paths at runtime.

### Testing rules

- Tests are small and technical. Markdown parsing pinning, lookup-table sanity, JSON output shape, piners-syntax parse/token/validate behavior, indicator fixture diffing.
- Do not add tests that hit live TradingView or any network. The `--strict` validator tier is exercised manually, not in CI.
- Vendored data is the test fixture: pin behavior against `vendor/pine-reference/spec/v6.md`, not against a hand-crafted toy markdown.
- When in doubt, write the smallest deterministic unit test that pins the behavior.

## Commands

Use `brokkr` (not `cargo`) for check/test. Output is filtered by default.

- `brokkr check` - gremlins + clippy + all tests
- `brokkr check --all` - show every diagnostic, no cap
- `brokkr test <NAME>` - release-mode focused single-test runner. `<NAME>` is a case-sensitive substring filter. Streams the test's own stdout/stderr live.
  - `-N, --repeat <N>` - run the test N times (flaky-test hunting).
  - `--raw` - bypass output filtering.
  - `--debug` - build/run in dev profile (faster compile, when release-LTO time dominates).

Single-crate workspace, so `-p` is unnecessary.

## Subcommand status

| Subcommand | Status |
|---|---|
| `pine lookup <name>` | done (cross-category exact, prefix fallback) |
| `pine search <query>` | done (tantivy BM25, 5x name boost; indexes v6 reference + corpus probes + PineForge audit doc + 18 narrative pages + pine-data behavior entries; hits carry `kind` = "reference" / "probe" / "audit" / "docs" / "behavior" and a content snippet; case-insensitive `--kind <kind>` narrows the result set, pass `--kind ?` to list the catalog) |
| `pine probe <slug>` | done (baked corpus, flat + nested slugs) |
| `pine probes [--grep TEXT] [--feature NAME]` | done (`--grep` matches against slug or extracted-from-source summary text; `--feature` restricts by Pine-feature usage detected from each `strategy.pine` source - `oca`, `trail`, `pyramiding`, `varip`, `mtf`, `magnifier`, `matrix`, `map`, `udt`, `method`, `process_orders_on_close`, `barstate_isfirst`; pass `?` to list the catalog) |
| `pine parse` | done via piners-syntax |
| `pine tokens` | done via piners-syntax |
| `pine validate` | done via piners-syntax lex / parse / type / semantic diagnostics, backed by piners-runtime builtins plus pine-data gap-fill |
| `pine validate --strict` | done as a TV-broker yes/no oracle. POSTs as multipart/form-data; `success` is trustworthy, the diagnostic prose is non-actionable (first error only, breaks on trailing whitespace, wrong line/column). Use after the local tier reports clean - not for iterative debugging. |
| `pine behavior <name>` | done (exact lookup plus `--list`, case-insensitive `--kind`, and `--grep` over functions / variables / constants / keywords from baked pine-tools JSON; `pine behavior TEXT --list` treats `TEXT` as an implicit grep) |
| `pine diff <probe> <trades.csv>` | done v1 (verify_corpus port: align + p90 + tier; `--show-diffs N` emits worst-N matched pairs + every TV / user orphan; no interior trim until OHLCV bake) |
| `pine version` | done (binary version + reference / corpus / pineforge-docs / pine-data / indicator fixture bake counts) |
| `pine indicator --list` | done (lists baked strict fixtures, including deterministic `smoke-*` checks) |
| `pine indicator --strict <slug>` | substrate done (fixture loader + piners-runner replay + per-bar diff); deterministic smoke fixtures baked, real TV baselines pending |
