# AGENTS.md

## Project

pine-oracle is a Rust crate producing the `po` binary: a single-binary CLI that answers Pine v6 semantic questions across every Pine-adjacent project. Vendors TradingView's published v6 reference (via Pinecone's snapshot), the PineForge audit + narrative docs, and the pine-data behavior surface; exposes them as one-shot subcommands (`po lookup`, `po search`, `po validate`). Design doc: `docs/pine-oracle.md`.

The oracle is not a Pine runtime substitute. It answers questions about Pine; it never executes user strategies.

## Workspace

Single crate at the repo root.

- `pine-oracle` (binary name `po`). Modules grow as subcommands land.

### Layout

The library crate (`src/lib.rs`, surface = `pine_oracle::*`) owns the domain modules listed below: pure logic with no CLI concerns. The binary crate (`src/main.rs` + `src/output.rs` + `src/commands/*.rs`) owns the CLI surface:

- `src/main.rs` - clap `Cli` + `Command` definitions, shared Pine source input resolution (`CODE_OR_FILE`, `--code`, `--file`, `-`/stdin) for source-driven commands, `OutputFormat::Auto/Text/Json` resolution, `main()` dispatch, `cmd_version` (the only subcommand that stays inline because it self-describes the binary it lives in).
- `src/output.rs` - shared output primitives every subcommand uses: `ResolvedFormat`, `Style` (ANSI colour wrapper with TTY / `NO_COLOR` / `--no-color` resolution), `SCHEMA_VERSION`, `versioned_json`, `print_json`. All `pub(crate)` (the binary has no external API).
- `src/commands/<name>.rs` - one file per `po <subcommand>` (every command except `version`): `lookup`, `search`, `validate`. Each exposes `pub(crate) fn run(...)` taking parsed args + `ResolvedFormat` (+ `Style` when the command emits styled text). Subcommand-only helpers (per-command text formatters) live in the same file as their consumer. `lookup` owns the merged identifier view: it joins the structured `behavior` surface with the v6 `reference` prose, and absorbs the behavior catalog browsing (`--list` / `--kind` / `--grep`).

### Domain modules (`src/`)

- `reference`: in-process lookup + substring search over the vendored TradingView v6 reference (`vendor/pine-reference/spec/v6.md`, 941 entries). Cached behind `OnceLock`. MPL-2.0, lifted from pinecone. `enrichment(name) -> Option<Enrichment>` parses the entry body's bare-label sections (`Remarks`, `See also`, per-argument prose under `Arguments`) into the prose fields the structured `behavior` surface lacks; `po lookup` folds these onto the structured signature.
- `search`: BM25 via tantivy over four sources: the v6 reference (941 entries), PineForge's Pine v6 audit doc (`pine_v6_audit_master.md`, H2/H3 sections - 38 critical + ~62 minor documented TV-vs-engine divergences), PineForge's 18 narrative explainer pages (`pages/*.md`, H2/H3 sections - magnifier, mtf, timeframes, lifecycle, report-schema, etc.), and pine-data behavior entries (functions / variables / constants / keywords / types / annotations with signatures, params, examples, and polymorphism notes). RAM-backed Index built on first invocation (~10-15 ms), OnceLock-cached. Schema fields: `name`, `category`, `kind` ("reference" / "audit" / "docs" / "behavior"), `content` (STORED for retrieval) + `content_search` (TEXT for ranking). `name` gets a 5x boost over `content_search`. Hits carry full body in `SearchHit.content`.
- `validate`: two tiers, with inverted authority vs. an earlier draft of the design doc.
  - **Local (`validate::check`)**: lex + parse + type + semantic analysis via piners-syntax, backed by piners-runtime builtins plus pine-data gap-fill. Returns every diagnostic piners-syntax can recover as `Diagnostic { severity, stage, code, message, line, column }`.
  - **Strict (`validate::strict`)**: POSTs the source as `multipart/form-data` to `pine-facade.tradingview.com/pine-facade/translate_light` via `ureq`, maps every error + warning the API returns into Diagnostics with `Stage::Strict`. **Yes / no oracle only - the diagnostic prose is non-actionable**. TV's pine-lint stops at the first error, breaks on trailing whitespace, and reports wrong line / column numbers; the `success` bit is the only trustworthy output. Use after the local tier reports clean, not for iterative debugging. No auth (endpoint is open), no on-disk cache, 10s timeout. Response decoding pinned by inline fixture tests; never hits the network in CI.
- `behavior`: structured signature + polymorphism lookup over pine-tools' JSON exports (`vendor/pine-data/v6/{functions,variables,constants,keywords,types,annotations}.json`). This is a data-layer module - there is no `po behavior` command; `po lookup` consumes it. Public API: `lookup(name) -> Option<Behavior>`, `list(kind, grep)`, `kind_catalog()`, `search_entries()`, and `snapshot()`. `Behavior` is one of `Function` / `Variable` / `Constant` / `Keyword` / `Type` / `Annotation`. Polymorphism is sourced entirely from each function's `flags` object (`polymorphic` = "input" | "element" | "numeric", plus `returnTypeParam`) since upstream retired the separate `function-behavior.json`; `FunctionFlags::is_polymorphic()` and `Behavior::is_polymorphic()` read it. Function entries also carry per-parameter `default` / `allowedValues` / `min` / `max`, a per-overload `overloads[]` array, and an optional `deprecated` note - all surfaced in `po behavior` and folded into search content. The `types` catalog (chart.point, line, array, ...) carries `classification` + object `fields`; the `annotations` catalog (@version=, @param, ...) carries `syntax` + examples. Direct `lookup` of a primitive type name (int, float, ...) resolves the cast function of the same name; the type itself is reachable via `--list --kind type` and search. Snapshot metadata (`version`, `generated_at`) is baked (`PINE_DATA_VERSION` / `PINE_DATA_SNAPSHOT`) since the JSON files are bare arrays with no envelope. Lenient deserialization (serde defaults on optional fields) so pine-tools schema tweaks don't break the binary.
**`--kind` namespace note.** Two subcommands expose a `--kind` filter but they cover different namespaces. `po search --kind` accepts the search source kinds: "reference", "audit", "docs", "behavior". `po lookup --kind` (used with `--list`) accepts the behavior entry kinds: "function", "variable", "constant", "keyword", "type", "annotation" (the type and annotation entries are also indexed by search under the "behavior" search kind, carrying category "Type" / "Annotation"). These are disjoint; passing a search kind to `po lookup --kind` (or vice versa) is an error. Both subcommands accept `--kind ?` to list their own catalog.

Canonical homes (so cross-module duplicates collapse to one):

- `Entry` (category + name + content) lives in `reference`.

## Vendoring

`research/` is gitignored. It is third-party source we consult, not ship. Anything we ship goes under `vendor/<source>/` with:

- A `LICENSE` copy of the upstream license.
- A `NOTICE` naming the upstream, the path lifted, and the snapshot date / git ref.
- Per-file `SPDX-License-Identifier` header on every lifted source file that has comment syntax. Files with no comment syntax (e.g. JSON) satisfy attribution via the adjacent LICENSE and NOTICE instead.

Current vendors:

- `vendor/pine-reference/`: pinecone's `crates/pine-reference/spec/v6.md` (MPL-2.0). Local mod: U+00A0 NO-BREAK SPACE rewritten to U+0020 SPACE for the gremlin scan. See `vendor/pine-reference/NOTICE`.
- `vendor/pine-data/v6/`: structured JSON snapshots from `../pine-tools/pine-data/v6/` (MIT, folknor owns pine-tools). Six files: `functions.json`, `variables.json`, `constants.json`, `keywords.json`, `types.json`, `annotations.json`. Polymorphism lives in each function's `flags` (the separate `function-behavior.json` was retired upstream). The files are bare arrays with no `generatedAt` envelope, so the snapshot ref/date is baked into `behavior::PINE_DATA_SNAPSHOT`; bump it on refresh. Refresh by re-running pine-tools' `pnpm run scrape`, copying the JSON files in, and updating the counts / snapshot ref in `vendor/pine-data/v6/NOTICE`.
- `vendor/pineforge-docs/`: PineForge's `docs/pine_v6_audit_master.md` (38 critical + ~62 minor known divergences) + `docs/pages/*` (18 narrative explainers covering magnifier, mtf, timeframes, lifecycle, report-schema, abi-stability, etc.). All Apache-2.0. Local mod: em/en-dashes / NBSPs rewritten to ASCII for the gremlin scan; see `vendor/pineforge-docs/NOTICE`.

## Rules

### General rules

- Don't use gremlins! Em-dash, en-dash, strange quotes, whatever - they're all verboten.
- Don't remind the user of the rules. They wrote them, so they know them.
- The user can exempt you from any rule at any time.

**Exit-code convention.** Most failure paths use `bail!` (anyhow renders the error and exits 1 with a message). Commands that print their own structured failure report (`validate`) call `std::process::exit(1)` directly after printing, so stderr stays clean and no redundant anyhow error string appears. The asymmetry is intentional: `bail!` is for unexpected failures; `process::exit(1)` is for expected "the check failed" outcomes that the command has already reported in full.

### Bash rules

This is the single source of truth for Bash rules in this project. The project CLAUDE.md imports these via `@AGENTS.md`.

- Never chain commands with `&&`.
- Never chain commands with `;`.
- Never chain/pipe commands with `|`. Exception: piping into `review` is allowed.
- Never capture stdout into env vars (`UUID=$(...)`).
- Never read or write from `/tmp`. All data lives in the project.
- Never run raw `cargo`, `curl`, `pkill`. Use `brokkr`.
- Never use `sed`, `find`, `awk`, `head`, `tail`, or complex bash commands.
- Never run `find /` (scans the full filesystem).
- Never run `git` with `-C <path>`.
- One Bash() invocation === one command.

### git commit rules

- Always run `brokkr fmt` before a commit.
- Never commit markdown changes alone. Bundle them with upcoming code commits.
- When committing other changes: always tag along markdown files if dirty.
- Write substantive engineering-focused commit messages.
- Has `Cargo.lock` changed? Commit it.
- Never `git push` unless the user explicitly asks. Stop after the commit.

### Vendoring rules

- New vendored sources go under `vendor/<name>/` with a LICENSE copy and a NOTICE file describing source path + snapshot date.
- Every lifted source file carries an `SPDX-License-Identifier` header pointing back to the upstream. Exception: binary or structured-data files with no comment syntax (e.g. JSON, compiled assets) cannot carry an inline header; the adjacent `LICENSE` file and the vendor `NOTICE` satisfy attribution for those files.
- The `research/` tree is read-only consultation material. Never edit it, never depend on its paths at runtime.

### Testing rules

- Tests are small and technical. Markdown parsing pinning, lookup-table sanity, JSON output shape, piners-syntax parse/token/validate behavior.
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

Current Pine lint source of truth:

- `node /home/folk/Programs/pine-tools/dist/packages/cli/src/cli.js --code '<pine source>'` - local pine-tools validator. This is the real `pine-lint2` target; `pine-lint2` itself may only exist as an interactive shell alias. Long-term goal: replace this with pine-oracle / piners validation once parity is strong enough.

## Subcommand status

| Subcommand | Status |
|---|---|
| `po lookup <name>` | done (merged identifier view: structured pine-data signature - params with default / allowedValues / min / max, per-overload signatures, polymorphism + deprecation flags - joined with the v6 reference prose the structured surface lacks (per-argument descriptions, `Remarks`, `See also`); reference-only entries such as Operators fall back to verbatim prose; prefix fallback when no exact hit. `--list` + case-insensitive `--kind` + `--grep` browse the behavior catalog (functions / variables / constants / keywords / types / annotations); `po lookup TEXT --list` treats `TEXT` as an implicit grep; pass `--kind ?` for the catalog) |
| `po search <query>` | done (tantivy BM25, 5x name boost; indexes v6 reference + PineForge audit doc + 18 narrative pages + pine-data behavior entries (functions / variables / constants / keywords / types / annotations); hits carry `kind` = "reference" / "audit" / "docs" / "behavior" and a content snippet; case-insensitive `--kind <kind>` narrows the result set, pass `--kind ?` to list the catalog) |
| `po validate` | done via piners-syntax lex / parse / type / semantic diagnostics, backed by piners-runtime builtins plus pine-data gap-fill; source input can be inline positional, existing file path, `--code CODE`, `--file PATH`, or `-`/stdin; text diagnostics include source-line caret frames |
| `po validate --strict` | done as a TV-broker yes/no oracle. POSTs as multipart/form-data; `success` is trustworthy, the diagnostic prose is non-actionable (first error only, breaks on trailing whitespace, wrong line/column). Use after the local tier reports clean - not for iterative debugging. |
| `po version` | done (binary version + reference / pineforge-docs / pine-data bake counts) |
