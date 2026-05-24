# AGENTS.md

## Project

pine-oracle is a Rust crate producing the `pine` binary: a single-binary CLI that answers Pine v6 semantic questions across every Pine-adjacent project. Vendors TradingView's published v6 reference (via Pinecone's snapshot) plus the PineForge cross-validation corpus, exposes them as one-shot subcommands (`pine lookup`, `pine search`, `pine probe`, `pine diff`, etc.). Design doc: `docs/pine-oracle.md`.

The oracle is not a Pine runtime substitute. It answers questions about Pine; it does not run Pine.

## Workspace

Single crate at the repo root.

- `pine-cli` (binary name `pine`). Modules grow as subcommands land.

Modules currently in `src/`:

- `reference`: in-process lookup + substring search over the vendored TradingView v6 reference (`vendor/pine-reference/spec/v6.md`, 941 entries). Cached behind `OnceLock`. MPL-2.0, lifted from pinecone.

Modules currently in `src/`:

- `corpus`: in-binary PineForge validation corpus, embedded via `include_dir`. Exposes `load_probe(slug)` and `list_probes(grep)` over 235 probes (flat + nested under `symbol-specified/<SYMBOL>/`). Slug-aligned per-probe summaries are pending re-curation; `summary_for` returns `None` until that lands. Apache-2.0 + MPL-2.0 dual-licensed, attributing PineForge.
- `search`: BM25 via tantivy over the v6 reference. RAM-backed Index built on first invocation (single-digit ms), OnceLock-cached. `name` field gets a 5x boost over `content`. Eventually grows to index `corpus` probe summaries + PineForge audit docs once those land.
- `syntax`: Pine v6 lexer + AST + parser lifted from pinecone (MPL-2.0). Three sub-modules (`ast`, `lexer`, `parser`) re-exported through `syntax::*`. Drives `pine parse` and `pine tokens`. Parser regression tests run across 72 vendored `.pine` fixtures with `_ast.json` goldens.
- `validate`: two tiers. Local: lex + parse via `syntax`, returns first failure as a structured `Diagnostic { severity, stage, message, line, column }` (single-error today; the lifted parser bails on the first failure). Strict: POSTs the source to `pine-facade.tradingview.com/pine-facade/translate_light` via `ureq`, maps every error + warning the API returns into Diagnostics with `Stage::Strict`. No auth (endpoint is open), no on-disk cache, 10-second timeout. Response decoding pinned by inline fixture tests; never hits the network in CI.
- `behavior`: structured signature + polymorphism lookup over pine-tools' JSON exports (`vendor/pine-data/v6/{functions,variables,constants,keywords,function-behavior}.json`). Public API: `lookup(name) -> Option<Behavior>`, where `Behavior` is one of `Function` / `Variable` / `Constant` / `Keyword`. Function entries optionally carry a `RawBehaviorEntry` with polymorphism markers + argument-ordering. Lenient deserialization (serde defaults on optional fields) so pine-tools schema tweaks don't break the binary.

Planned modules per design:

- `parse` / `tokens`: ports of piners-syntax for `pine parse` and `pine tokens`.
- `validate`: local-tier diagnostics; `--strict` shells to TradingView's pine-lint API (no auth required, no on-disk response cache).
- `behavior`: polymorphism / na-propagation / series-vs-simple lookup over pine-tools' `pine-data/v6/*.json` (once the upstream export step lands).
- `diff`: Rust port of PineForge's `verify_corpus.py` alignment + tier logic.

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
| `pine search <query>` | done (tantivy BM25, 5x name boost) |
| `pine probe <slug>` | done (baked corpus, flat + nested slugs) |
| `pine probes [--grep TEXT]` | done (slug substring match; summary-text grep returns when summaries re-curate) |
| `pine parse` | done (lifted pinecone parser; JSON / pretty-JSON output) |
| `pine tokens` | done (lifted pinecone lexer; JSON / one-per-line text) |
| `pine validate` | done v1 (first lex/parse error as structured Diagnostic; exit 1 on error) |
| `pine version` | done |
| `pine behavior <name>` | done (functions / variables / constants / keywords from baked pine-tools JSON) |
| `pine validate --strict` | done (POSTs to TV's pine-lint, maps errors + warnings to Diagnostics; no auth, no cache, 10s timeout) |
| `pine diff` | TODO (needs verify_corpus.py port; will require OHLCV bake) |
| `pine indicator --strict` | TODO (per-bar parity; will require OHLCV bake) |
