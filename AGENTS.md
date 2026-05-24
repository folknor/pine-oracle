# AGENTS.md

## Project

pine-oracle is a Rust crate producing the `pine` binary: a single-binary CLI that answers Pine v6 semantic questions across every Pine-adjacent project. Vendors TradingView's published v6 reference (via Pinecone's snapshot) plus the PineForge cross-validation corpus, exposes them as one-shot subcommands (`pine lookup`, `pine search`, `pine probe`, `pine diff`, etc.). Design doc: `docs/pine-oracle.md`.

The oracle is not a Pine runtime substitute. It answers questions about Pine; it does not run Pine.

## Workspace

Single crate at the repo root.

- `pine-cli` (binary name `pine`). Modules grow as subcommands land.

Modules currently in `src/`:

- `reference`: in-process lookup + substring search over the vendored TradingView v6 reference (`vendor/pine-reference/spec/v6.md`, 941 entries). Cached behind `OnceLock`. MPL-2.0, lifted from pinecone.

Planned modules per design:

- `parse` / `tokens`: ports of piners-syntax for `pine parse` and `pine tokens`.
- `validate`: local-tier diagnostics; `--strict` shells to TradingView's pine-lint API.
- `behavior`: polymorphism / na-propagation / series-vs-simple lookup over pine-tools' `pine-data/v6/*.json` (once the upstream export step lands).
- `probe` / `probes`: corpus loader (filesystem under `$XDG_DATA_HOME/pine/corpus/`).
- `diff`: Rust port of PineForge's `verify_corpus.py` alignment + tier logic.
- `search`: BM25 over reference + corpus summaries + PineForge audit docs; tantivy is v2 (the current `search` is a substring ranker placeholder).

Canonical homes (so cross-module duplicates collapse to one):

- `Entry` (category + name + content) lives in `reference`.
- Future `Probe`, `TradeList`, `TierReport` will live in `probe` / `diff`.

## Vendoring

`research/` is gitignored. It is third-party source we consult, not ship. Anything we ship goes under `vendor/<source>/` with:

- A `LICENSE` copy of the upstream license.
- A `NOTICE` naming the upstream, the path lifted, and the snapshot date / git ref.
- Per-file `SPDX-License-Identifier` header on every lifted source file.

Current vendors:

- `vendor/pine-reference/`: pinecone's `crates/pine-reference/spec/v6.md` (MPL-2.0).

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
| `pine search <query>` | placeholder substring ranker; BM25 in v2 |
| `pine version` | done |
| `pine validate` | TODO |
| `pine parse` | TODO |
| `pine tokens` | TODO |
| `pine behavior` | TODO (needs pine-tools `export:json`) |
| `pine probe` | TODO (needs corpus install) |
| `pine probes` | TODO |
| `pine diff` | TODO (needs verify_corpus.py port) |
| `pine corpus install/update` | TODO |
