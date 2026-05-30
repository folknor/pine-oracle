# AGENTS.md

## Project

pine-oracle is a Rust crate producing the `po` binary: a single-binary CLI that answers Pine v6 semantic questions across every Pine-adjacent project. Vendors the pine-tools pine-data behavior surface (structured signatures, params, polymorphism, and prose sub-sections for the full Pine v6 public surface); exposes it as one-shot subcommands (`po lookup`, `po search`, `po validate`).

The oracle is not a Pine runtime substitute. It answers questions about Pine; it never executes user strategies.

## Workspace

Single crate at the repo root.

- `pine-oracle` (binary name `po`). Modules grow as subcommands land.

### Layout

The library crate (`src/lib.rs`, surface = `pine_oracle::*`) owns the domain modules listed below: pure logic with no CLI concerns. The binary crate (`src/main.rs` + `src/output.rs` + `src/commands/*.rs`) owns the CLI surface:

- `src/main.rs` - clap `Cli` + `Command` definitions, shared Pine source input resolution (`CODE_OR_FILE`, `--code`, `--file`, `-`/stdin) for source-driven commands, `OutputFormat::Auto/Text/Json` resolution, `main()` dispatch, `cmd_version` (the only subcommand that stays inline because it self-describes the binary it lives in).
- `src/output.rs` - shared output primitives every subcommand uses: `ResolvedFormat`, `Style` (ANSI colour wrapper with TTY / `NO_COLOR` / `--no-color` resolution), `SCHEMA_VERSION`, `versioned_json`, `print_json`. All `pub(crate)` (the binary has no external API).
- `src/commands/<name>.rs` - one file per `po <subcommand>` (every command except `version`): `lookup`, `search`, `validate`. Each exposes `pub(crate) fn run(...)` taking parsed args + `ResolvedFormat` (+ `Style` when the command emits styled text). Subcommand-only helpers (per-command text formatters) live in the same file as their consumer. `lookup` owns the identifier view: it renders the structured `behavior` surface (signature, typed params with per-argument prose, flags, plus the `remarks` / `seeAlso` / `returnsDescription` sub-sections and the operator catalog) and absorbs the behavior catalog browsing (`--list` / `--kind` / `--grep`).

### Domain modules (`src/`)

- `search`: BM25 via tantivy over the pine-data behavior surface (functions / variables / constants / keywords / types / annotations / operators), the single "name" source. Search is the index into `po lookup` - you reach for it when you don't yet know the identifier to pass lookup. `query(q, limit) -> Vec<NameHit>` returns a ranked list of names: each pine-data symbol is one document so names are already distinct (the group-by-name + sum step is a no-op safety net, load-bearing only while a second overlapping source existed). The name is the whole payload (feed it back to lookup); there is no per-hit content/category/kind and no `--kind` filter. RAM-backed Index built on first invocation, OnceLock-cached. Schema is just `name` (TEXT|STORED, 5x boost) + `content_search` (TEXT, ranking only).
- `validate`: two tiers, with inverted authority vs. an earlier draft of the design doc.
  - **Local (`validate::check`)**: lex + parse + type + semantic analysis via piners-syntax, backed by piners-runtime builtins plus pine-data gap-fill. Returns every diagnostic piners-syntax can recover as `Diagnostic { severity, stage, code, message, line, column }`.
  - **Strict (`validate::strict`)**: POSTs the source as `multipart/form-data` to `pine-facade.tradingview.com/pine-facade/translate_light` via `ureq`, maps every error + warning the API returns into Diagnostics with `Stage::Strict`. **Yes / no oracle only - the diagnostic prose is non-actionable**. TV's pine-lint stops at the first error, breaks on trailing whitespace, and reports wrong line / column numbers; the `success` bit is the only trustworthy output. Use after the local tier reports clean, not for iterative debugging. No auth (endpoint is open), no on-disk cache, 10s timeout. Response decoding pinned by inline fixture tests; never hits the network in CI.
- `behavior`: structured signature + polymorphism lookup over pine-tools' JSON exports (`vendor/pine-data/v6/{functions,variables,constants,keywords,types,annotations,operators}.json`). This is a data-layer module - there is no `po behavior` command; `po lookup` consumes it. Public API: `lookup(name) -> Option<Behavior>`, `prefix_search(name)`, `list(kind, grep)`, `kind_catalog()`, `search_entries()`, and `snapshot()`. `Behavior` is one of `Function` / `Variable` / `Constant` / `Keyword` / `Type` / `Annotation` / `Operator`. Polymorphism is sourced entirely from each function's `flags` object (`polymorphic` = "input" | "element" | "numeric", plus `returnTypeParam`) since upstream retired the separate `function-behavior.json`; `FunctionFlags::is_polymorphic()` and `Behavior::is_polymorphic()` read it. Function entries also carry per-parameter `default` / `allowedValues` / `min` / `max`, a per-overload `overloads[]` array, and an optional `deprecated` note. Every catalog that documents them also carries the prose sub-sections `remarks` / `seeAlso` / `returnsDescription` (the "Returns" sentence, distinct from the typed return) - these are the fields that let `po lookup` render the full reference card with no markdown source. The `types` catalog carries `classification` + object `fields`; the `annotations` catalog carries `syntax` + examples; the `operators` catalog (+, -, ?:, [], +=, ...) carries `syntax` + `description` + the prose sub-sections (operators have no typed return). Direct `lookup` of a primitive type name (int, float, ...) resolves the cast function of the same name; the type itself is reachable via `--list --kind type` and search. Snapshot metadata (`version`, `generated_at`) is baked (`PINE_DATA_VERSION` / `PINE_DATA_SNAPSHOT`) since the JSON files are bare arrays with no envelope. Lenient deserialization (serde defaults on optional fields) so pine-tools schema tweaks don't break the binary.
**`--kind` note.** Only `po lookup --kind` (used with `--list`) takes a `--kind`; it accepts the behavior entry kinds: "function", "variable", "constant", "keyword", "type", "annotation", "operator". Pass `--kind ?` to list the catalog. `po search` has no `--kind` - it has a single source, so there is nothing to filter.

## Vendoring

`research/` is gitignored. It is third-party source we consult, not ship. Anything we ship goes under `vendor/<source>/` with:

- A `LICENSE` copy of the upstream license.
- A `NOTICE` naming the upstream, the path lifted, and the snapshot date / git ref.
- Per-file `SPDX-License-Identifier` header on every lifted source file that has comment syntax. Files with no comment syntax (e.g. JSON) satisfy attribution via the adjacent LICENSE and NOTICE instead.

Current vendors:

- `vendor/pine-data/v6/`: structured JSON snapshots from `../pine-tools/pine-data/v6/` (MIT, folknor owns pine-tools). Six files: `functions.json`, `variables.json`, `constants.json`, `keywords.json`, `types.json`, `annotations.json`. Polymorphism lives in each function's `flags` (the separate `function-behavior.json` was retired upstream). The files are bare arrays with no `generatedAt` envelope, so the snapshot ref/date is baked into `behavior::PINE_DATA_SNAPSHOT`; bump it on refresh. Refresh by re-running pine-tools' `pnpm run scrape`, copying the JSON files in, and updating the counts / snapshot ref in `vendor/pine-data/v6/NOTICE`.

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
- Vendored data is the test fixture: pin behavior against the real `vendor/pine-data/v6/*.json` entries, not against hand-crafted toy data.
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
| `po lookup <name>` | done (identifier view rendered straight from pine-data: signature with params (default / allowedValues / min / max + per-argument prose), per-overload signatures, polymorphism + deprecation flags, and the prose sub-sections `remarks` / `seeAlso` / `returnsDescription`; operators are a first-class kind; prefix fallback over all catalog names when no exact hit. `--list` + case-insensitive `--kind` + `--grep` browse the behavior catalog (functions / variables / constants / keywords / types / annotations / operators); `po lookup TEXT --list` treats `TEXT` as an implicit grep; pass `--kind ?` for the catalog) |
| `po search <query>` | done (tantivy BM25, 5x name boost over the pine-data behavior surface; the index into `po lookup`. Output is a ranked `score name` list, one line per identifier. Text-only (no JSON), no `--kind`; `--limit` defaults to 25) |
| `po validate` | done via piners-syntax lex / parse / type / semantic diagnostics, backed by piners-runtime builtins plus pine-data gap-fill; source input can be inline positional, existing file path, `--code CODE`, `--file PATH`, or `-`/stdin; text diagnostics include source-line caret frames |
| `po validate --strict` | done as a TV-broker yes/no oracle. POSTs as multipart/form-data; `success` is trustworthy, the diagnostic prose is non-actionable (first error only, breaks on trailing whitespace, wrong line/column). Use after the local tier reports clean - not for iterative debugging. |
| `po version` | done (binary version + pine-data bake counts incl. operators) |
