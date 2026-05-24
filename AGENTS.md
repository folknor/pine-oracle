# AGENTS.md

## Project

pine-oracle is a Rust crate producing the `pine` binary: a single-binary CLI that answers Pine v6 semantic questions across every Pine-adjacent project. Vendors TradingView's published v6 reference (via Pinecone's snapshot) plus the PineForge cross-validation corpus, exposes them as one-shot subcommands (`pine lookup`, `pine search`, `pine probe`, `pine diff`, etc.). Design doc: `docs/pine-oracle.md`.

The oracle is not a Pine runtime substitute. It answers questions about Pine; it does not run Pine.

## Workspace

Single crate at the repo root.

- `pine-cli` (binary name `pine`). Modules grow as subcommands land.

Modules currently in `src/`:

- `reference`: in-process lookup + substring search over the vendored TradingView v6 reference (`vendor/pine-reference/spec/v6.md`, 941 entries). Cached behind `OnceLock`. MPL-2.0, lifted from pinecone.
- `corpus`: in-binary PineForge validation corpus, embedded via `include_dir`. Exposes `load_probe(slug)` and `list_probes(grep)` over 235 probes (flat + nested under `symbol-specified/<SYMBOL>/`). `summary_for(slug)` extracts the author-written prose comment block from each `strategy.pine` header (skipping license / SPDX / copyright / version-directive lines), cached behind a OnceLock; >=80% of probes get a real summary out of the box. `list_probes(grep)` matches against slug OR summary text. The richer engine-internals prose in `docs/probe-summaries.md` is keyed to engine-internal slugs that do not match the published corpus and is not loaded here. Apache-2.0 + MPL-2.0 dual-licensed, attributing PineForge.
- `search`: BM25 via tantivy over three sources: the v6 reference (941 entries), the baked PineForge corpus (235 probes), and PineForge's Pine v6 audit doc (H2/H3 sections of `pine_v6_audit_master.md`, ~38 critical + ~62 minor known TV-vs-engine divergences). RAM-backed Index built on first invocation (~10-15 ms), OnceLock-cached. Schema fields: `name`, `category`, `kind` ("reference" / "probe" / "audit"), `content`. `name` gets a 5x boost over `content`. Probes are indexed with slug as `name`, author-extracted summary as `content` (slug-as-fallback when no summary).
- `syntax`: Pine v6 lexer + AST + parser lifted from pinecone (MPL-2.0). Three sub-modules (`ast`, `lexer`, `parser`) re-exported through `syntax::*`. Drives `pine parse` and `pine tokens`. Parser regression tests run across 72 vendored `.pine` fixtures with `_ast.json` goldens. **Temporary**: this lift is the v0 backing for `pine validate`. Long-term it gets replaced by piners-syntax (the analyzer piners builds for its runtime) or, as a bridge, a WASM transpile of pine-tools' TS analyzer. The lifted pinecone parser stops at the first lex / parse error and has no type checker.
- `validate`: two tiers, with inverted authority vs. an earlier draft of the design doc.
  - **Local (`validate::check`)**: lex + parse via `syntax`, returns first failure as `Diagnostic { severity, stage, message, line, column }`. Today catches one error only; will become an IDE-quality multi-error multi-stage validator once piners-syntax (or a WASM pine-tools transpile) lands.
  - **Strict (`validate::strict`)**: POSTs the source as `multipart/form-data` to `pine-facade.tradingview.com/pine-facade/translate_light` via `ureq`, maps every error + warning the API returns into Diagnostics with `Stage::Strict`. **Yes / no oracle only - the diagnostic prose is non-actionable**. TV's pine-lint stops at the first error, breaks on trailing whitespace, and reports wrong line / column numbers; the `success` bit is the only trustworthy output. Use after the local tier reports clean, not for iterative debugging. No auth (endpoint is open), no on-disk cache, 10s timeout. Response decoding pinned by inline fixture tests; never hits the network in CI.
- `behavior`: structured signature + polymorphism lookup over pine-tools' JSON exports (`vendor/pine-data/v6/{functions,variables,constants,keywords,function-behavior}.json`). Public API: `lookup(name) -> Option<Behavior>`, where `Behavior` is one of `Function` / `Variable` / `Constant` / `Keyword`. Function entries optionally carry a `RawBehaviorEntry` with polymorphism markers + argument-ordering. Lenient deserialization (serde defaults on optional fields) so pine-tools schema tweaks don't break the binary.
- `diff`: trade-list parity scorer, port of PineForge's `scripts/verify_corpus.py`. Public API: `diff(probe_slug, user_csv) -> DiffReport`. Parses both CSVs into entry / exit pairs (Trade # joined, TV's "Date and time" interpreted in the chart timezone with default Asia/Taipei +8), aligns by direction + 1h window + $3 entry-price gate, trims to common window, computes 4-dim p90 deltas, classifies as excellent / strong / moderate / weak / minimal. Honours `inputs.json::expected_tier` ("anomaly", "engine_only") and `validation_overrides.expect_tv_match`. Strict vs production profile is auto-detected from `trail_*` parameters in `strategy.pine` (or forced via `inputs.json::parity_profile`). Threshold values mirror `verify_corpus.py` exactly. V1 does not implement interior trim (`trim_bars` / `warmup_bars`) since the OHLCV feed isn't baked.

Planned changes:

- Swap the pinecone-lifted lexer / parser for piners-syntax once it lands (or a WASM-bundled pine-tools analyzer as a bridge). Same public subcommand surface for `pine parse` / `pine tokens` / `pine validate`, deeper diagnostics behind it. The current pinecone lift catches only the first lex / parse error and has no type checks.
- `pine indicator --strict`: per-bar parity oracle. Requires piners' engine + OHLCV bake; not yet started.

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

- Tests are small and technical. Markdown parsing pinning, lookup-table sanity, JSON output shape, lexer/parser fixtures (when those modules land).
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
| `pine search <query>` | done (tantivy BM25, 5x name boost; indexes v6 reference + corpus probes + PineForge audit doc; hits carry `kind` = "reference" / "probe" / "audit"; `--kind <kind>` narrows the result set) |
| `pine probe <slug>` | done (baked corpus, flat + nested slugs) |
| `pine probes [--grep TEXT]` | done (matches against slug or extracted-from-source summary text) |
| `pine parse` | done via the pinecone lift; will deepen when piners-syntax replaces it |
| `pine tokens` | done via the pinecone lift; will deepen when piners-syntax replaces it |
| `pine validate` | v0 only: first lex/parse error from the pinecone lift, no type checks. v1 = IDE-quality multi-error output backed by piners-syntax (or a WASM pine-tools transpile as a bridge). |
| `pine validate --strict` | done as a TV-broker yes/no oracle. POSTs as multipart/form-data; `success` is trustworthy, the diagnostic prose is non-actionable (first error only, breaks on trailing whitespace, wrong line/column). Use after the local tier reports clean - not for iterative debugging. |
| `pine behavior <name>` | done (functions / variables / constants / keywords from baked pine-tools JSON) |
| `pine diff <probe> <trades.csv>` | done v1 (verify_corpus port: align + p90 + tier; no interior trim until OHLCV bake) |
| `pine version` | done (binary version + reference / corpus / pineforge-docs bake counts) |
| `pine indicator --strict` | TODO (per-bar parity; needs piners' engine + OHLCV bake) |
