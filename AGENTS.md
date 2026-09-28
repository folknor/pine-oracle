# AGENTS.md

## Project

pine-oracle is a Rust crate producing the `po` binary: a single-binary CLI that answers Pine v6 semantic questions across every Pine-adjacent project. Vendors the pine-tools pine-data behavior surface (structured signatures, params, polymorphism, prose) for "what is X" and the Pine User Manual prose for "how does X work"; exposes them as one-shot subcommands (`po lookup`, `po search`, `po show`, `po recipe`). `po lookup` answers about named identifiers (with a BM25 "did you mean ...?" miss path). `po search` is a BM25 *finder* over **both** prose corpora at once - the User Manual and the authored TA recipes - via the unified `find` index: it prints a menu of matching results (never prose), one per row as a runnable handle + breadcrumb. A manual hit shows a stable 8-hex id + `page / H2 / H3` (feed the id to `po show <id> [<id>...]`, which prints the section plus its subsections); a recipe hit shows `recipe <name>` + `recipe / category / Title` (run it directly). `po search -1` shortcuts straight to the top hit (manual section subtree or recipe body). `po recipe <name>` is a Pine cookbook: how to build something that has *no* TradingView builtin and no manual page, as a self-contained Pine v6 script, from an authored-in-repo markdown corpus. Scope is the broad cookbook (set by the user): indicators (custom moving averages, oscillators, candlestick patterns, volatility/volume/trend tools) *and* general helpers (easing/animation curves, risk metrics). The recipe analogue of `po lookup`. `po verdict` is the measured-behavior record: what TradingView's editor, `translate_light` endpoint and chart runs actually did for a Pine question, and whether it is settled, kept in a caller-named records directory (`--records`, always explicit) rather than baked in.

Output is text-only across the board.

The oracle is not a Pine runtime substitute. It answers questions about Pine; it never executes user strategies.

## Workspace

Single crate at the repo root.

- `pine-oracle` (binary name `po`). Modules grow as subcommands land.

### Layout

The library crate (`src/lib.rs`, surface = `pine_oracle::*`) owns the domain modules listed below: pure logic with no CLI concerns. The binary crate (`src/main.rs` + `src/output.rs` + `src/commands/*.rs`) owns the CLI surface:

- `src/main.rs` - clap `Cli` + `Command` definitions, the global `--no-color` / `--quiet` flags, `main()` dispatch, `cmd_version` (the only subcommand that stays inline because it self-describes the binary it lives in).
- `src/output.rs` - shared output primitives. `Style` (ANSI colour wrapper with TTY / `NO_COLOR` / `--no-color` resolution) and the `print_catalog` text rows reach every command. All `pub(crate)` (the binary has no external API). Output is text-only everywhere.
- `src/commands/<name>.rs` - one file per `po <subcommand>` (every command except `version`): `lookup`, `search`, `show`, `recipe`, `verdict`. `verdict` is a command group; its clap `VerdictCommand` subcommand enum lives in `commands/verdict.rs` rather than `main.rs`. Each exposes `pub(crate) fn run(...)` taking parsed args + `Style`. Subcommand-only helpers (per-command text formatters) live in the same file as their consumer. `lookup` owns the identifier view: it renders the structured `behavior` surface (signature, typed params with per-argument prose, flags, plus the `remarks` / `seeAlso` / `returnsDescription` sub-sections and the operator catalog), absorbs the behavior catalog browsing (`--list` / `--kind` / `--grep`), and on a miss offers BM25 "did you mean ...?" suggestions via the `suggest` module. `search` is the manual finder (menu of `<id>  breadcrumb` rows) and `show` is its fetcher (renders a section subtree by id) - they split the old find-and-render `search` into two verbs. `recipe` mirrors `lookup`'s shape against the `recipe` corpus: an exact name/alias hit renders the recipe body via `render`, `--list` / `--category` / `--grep` browse, and a miss offers fuzzy "did you mean ...?" suggestions.

### Domain modules (`src/`)

- `suggest`: BM25 via tantivy over the pine-data name surface (functions / variables / constants / keywords / types / annotations / operators). NOT a user-facing command - it is the engine behind `po lookup`'s miss path: `suggest(q, limit) -> Vec<Suggestion>` returns the closest identifier names ("did you mean ...?"). One doc per symbol; `name` field 5x boosted over `content_search`. RAM-backed Index built on first call, OnceLock-cached.
- `manual`: the Pine User Manual prose surface, backing `po search` + `po show`. Embeds the vendored per-page markdown tree (`vendor/pine-manual/v6`) via `include_dir`, splits each page into sections at its anchored **H1-H3** headings (`## Heading {#anchor}` and finer; H4+ fold into their parent), and BM25-indexes one doc per section keyed `page#anchor` (the same string is the index key, the result provenance, and - with the page `source` URL - a real fragment URL). Each `Section` carries: a stable 8-hex `id` (first 8 hex of FNV-1a(`page#anchor`); the handle `po search` prints and `po show` resolves), its heading `level` (1-3), and a breadcrumb `trail` (the H2..leaf titles; H1 is replaced by the page path). `Section::breadcrumb()` renders `page / H2 / H3`. Public API: `search(q, limit) -> Vec<SectionHit>` (title 4x boosted), `by_id(id) -> Vec<&Section>` (collision-aware id lookup), `subtree_markdown(section)` (the section + all deeper-heading descendants until the next same-or-shallower heading - what `po show` prints), `get_section(page, anchor)`, `get_page(page)` (whole page markdown), `sections()`, `page_count()`, `section_count()`. Pages without YAML frontmatter (the nav `README.md`) are skipped. The 8-hex id space is collision-free over the current corpus; `all_section_ids_are_unique` is the safety net that fails a future re-vendor instead of shipping an ambiguous id.
- `find`: the unified finder backing `po search`. Builds ONE BM25 index (tantivy) over both prose corpora - every `manual` section and every `recipe` - so a query ranks across both with comparable scores and shared corpus statistics (separate per-corpus indexes would yield incomparable BM25 scores and a meaningless merge). `manual` and `recipe` stay independent data providers; `find` only composes them, depending on both (no circular dependency the other way). Schema: `title` (TEXT, 4x boosted; for a recipe it carries name + title + aliases so any handle the user types ranks the recipe high), `content` (TEXT, the body), plus stored `kind` ("manual"/"recipe") and `handle`. Public API: `find(q, limit) -> Vec<Hit>` where `Hit { kind: HitKind, handle, breadcrumb, score }`; the `handle` re-keys a hit to its source (manual 8-hex id -> `manual::by_id` -> `po show`; recipe name -> `recipe::lookup` -> `po recipe`) and the breadcrumb is re-resolved live (manual `breadcrumb()` or `recipe / category / Title`). `manual::search` (manual-only) is retained for the `manual` module's own API/tests; `po search` uses `find`.
- `recipe`: the authored Pine-cookbook surface, backing `po recipe`. Unlike `manual` (scraped from TV) and `behavior` (pine-data exports), this corpus is hand-authored in-repo: anything with no TradingView builtin and no manual page that fits a self-contained recipe card - indicators *and* general helpers (easing curves, risk metrics). Scope is the broad cookbook (a deliberate user decision; it began TA-only because pandas-ta-classic was the first source). Library-shaped upstreams (external `import`s + UDTs) are not ingested wholesale; only their self-contained leaves are harvested and independently reimplemented. One markdown file per entry under `assets/recipes/<category>/<name>.md` with `title` + optional comma-separated `aliases` frontmatter and a freeform body (prose + a Pine v6 code fence); embedded via `include_dir`, rendered through `render` exactly like the manual (the difference is provenance, not shape). `name` = file stem (the lookup key), `category` = the subdir. Public API: `lookup(name) -> Option<&Recipe>` (exact by name or alias, case-insensitive, O(1) via an `OnceLock` HashMap that scales to a large corpus), `list(category, grep)`, `categories()`, `count()`, and `suggest(q, limit)` (a scored linear scan for the miss path - the corpus is name-keyed, so this beats standing up a second BM25 index; swap to tantivy-over-body later if full-text recipe search is wanted). `recipe` keeps no SPDX/vendor ceremony - it is our content, not third-party, and must stay gremlin-clean (it is NOT in `brokkr.toml`'s gremlin exclude, unlike the manual).
- `render`: markdown -> ANSI terminal renderer for `po search -1` / `po show` / `po recipe` text output. `render(markdown, no_color) -> String`. Adapted from markdown-peek (MIT) into this tree; renders headings / tables / lists / code fences / blockquotes / inline styles via `pulldown_cmark` + `owo_colors`. Inline link URLs are dropped (the link text stays) and images collapse to a bare `[image]` marker - the manual's prose is link- and screenshot-heavy and the URLs are noise in a terminal. `no_color` (from `--no-color` / `NO_COLOR` / non-TTY) strips all ANSI from the result.
- `behavior`: structured signature + polymorphism lookup over pine-tools' JSON exports (`vendor/pine-data/v6/{functions,variables,constants,keywords,types,annotations,operators}.json`). This is a data-layer module - there is no `po behavior` command; `po lookup` consumes it. Public API: `lookup(name) -> Option<Behavior>` (highest-precedence single match), `lookup_all(name) -> Vec<Behavior>` (every catalog match - the same name can resolve in several), `prefix_search(name)`, `list(kind, grep)`, `kind_catalog()`, `search_entries()`, and `snapshot()`. `Behavior` is one of `Function` / `Variable` / `Constant` / `Keyword` / `Type` / `Annotation` / `Operator`. Polymorphism is sourced entirely from each function's `flags` object (`polymorphic` = "input" | "element" | "numeric", plus `returnTypeParam`) since upstream retired the separate `function-behavior.json`; `FunctionFlags::is_polymorphic()` and `Behavior::is_polymorphic()` read it. `flags` also carries `historyDependent` (true on the whole ta.* namespace plus `fixnan` and `math.sum`: functions whose conditional/iterative calls build an inconsistent series, TV's CW10003 criterion); `po lookup` surfaces it as a flag note. Function entries also carry per-parameter `default` / `allowedValues` / `min` / `max`, a per-overload `overloads[]` array, and an optional `deprecated` note. Every catalog that documents them also carries the prose sub-sections `remarks` / `seeAlso` / `returnsDescription` (the "Returns" sentence, distinct from the typed return) - these are the fields that let `po lookup` render the full reference card with no markdown source. The `types` catalog carries `classification` + object `fields`; the `annotations` catalog carries `syntax` + examples; the `operators` catalog (+, -, ?:, [], +=, ...) carries `syntax` + `description` + the prose sub-sections (operators have no typed return). ~29 names resolve in more than one catalog (cast functions vs primitive types like `int`; variable/function pairs like `time`, `dayofmonth`; `na` is function + variable + keyword). `po lookup` renders every match via `lookup_all`; `lookup` (single) keeps first-hit precedence function > variable > constant > type > annotation > operator > keyword. Snapshot metadata (`version`, `generated_at`) is baked (`PINE_DATA_VERSION` / `PINE_DATA_SNAPSHOT`) since the JSON files are bare arrays with no envelope. Lenient deserialization (serde defaults on optional fields) so pine-tools schema tweaks don't break the binary.
- `verdict`: measured TradingView behavior, backing `po verdict` (user-facing reference: `docs/verdict.md`). The only surface NOT baked into the binary and the only one po writes: records live in a caller-named directory (in practice piners' git) passed with `--records` on every verb, never defaulted, never discovered, no env var. One `<id>.toml` per question (8-hex id generated by `add`, the file stem) holding `kind` (compile / runtime), question, answer, pine-data `identifiers`, the three question relations (`follow_up_to` lineage, `basis` premises, an optional `[retired]` table with `reason` + `replaced_by`), and `[[observation]]`s; fixtures are copied into a content-addressed `fixtures/<sha256>.pine` store, hashed by po from the stored bytes. Sources are editor = chart > endpoint (no local-validator source: pine-lint is not a measurement). Inconclusive and crashed observations are shown but never count; `Question::own_status` derives settled / conflict / open from counting top-strength observations (compile agreement = outcome + error-code set + warning-code set; runtime conflict = a candidate selected and refuted, or differing error codes; all-undecided runtime runs stay open), `Store::resolve` returns a `Resolved` disposition: `Measured(status)`, `Inferred` (a non-empty `basis`: settled when every premise is, otherwise open naming the worst premise, `inferred (open via <id>)` - a conflicting premise blocks the inference rather than making it a conflict), or `Retired` (no status at all; `replaced_by` is where the investigation continued, never an inherited answer). The three relations never stand in for each other: `derived_from` once meant status inheritance while its only consumer used it as lineage, so it was split, and a populated legacy `derived_from` fails validation (the empty list every old record carries is accepted and dropped on rewrite). `Question::ranked` orders strongest-first with "weaker source; confirmed by / same outcome as ..., different codes / ... disagrees" annotations. Identifiers are pine-data names or qualified parameters `function(parameter)`. Validation (`validate.rs`) is shared by the write path (`add` / `observe` / `retire` refuse and write nothing; the write path also runs the per-relation cycle check over the store as it would be after the write) and the strict read path (`load` fails on any invalid record, so any read verb is a CI gate). `search` is a per-call RAM tantivy index. Submodules: `spec` (CLI diagnostic / candidate spec parsing), `validate`, `load`, `write`, `search`. Deliberately separate from `lookup` / `search`: those never read verdicts. Tests write into `target/verdict-tests/` and pin against the real piners run 24 / run 26 fixtures + editor-probe JSON copied into `testdata/verdict/`.
**`--kind` note.** Only `po lookup --kind` (used with `--list`) takes a `--kind`; it accepts the behavior entry kinds: "function", "variable", "constant", "keyword", "type", "annotation", "operator". Pass `--kind ?` to list the catalog.

## Vendoring

`research/` is gitignored. It is third-party source we consult, not ship. Anything we ship goes under `vendor/<source>/` with:

- A `LICENSE` copy of the upstream license.
- A `NOTICE` naming the upstream, the path lifted, and the snapshot date / git ref.
- Per-file `SPDX-License-Identifier` header on every lifted source file that has comment syntax. Files with no comment syntax (e.g. JSON) satisfy attribution via the adjacent LICENSE and NOTICE instead.

Current vendors:

- `vendor/pine-data/v6/`: structured JSON snapshots from `../pine-tools/pine-data/v6/` (MIT, folknor owns pine-tools). Seven files: `functions.json`, `variables.json`, `constants.json`, `keywords.json`, `types.json`, `annotations.json`, `operators.json` (keywords graduated to objects carrying prose; every catalog carries `remarks` / `seeAlso` / `returnsDescription` where TV documents them). Polymorphism lives in each function's `flags` (the separate `function-behavior.json` was retired upstream). The files are bare arrays with no `generatedAt` envelope, so the snapshot ref/date is baked into `behavior::PINE_DATA_SNAPSHOT`; bump it on refresh. Refresh by re-running pine-tools' `pnpm run scrape`, copying ALL the JSON files in (the scrape regenerates the whole set, not one file), and updating the counts / snapshot ref in `vendor/pine-data/v6/NOTICE`. `scripts/vendor-refresh.py` does the diff-and-copy (dry run by default, `--apply` to write) for both vendors at once; `scripts/vendor-json-delta.py [<catalog>...]` summarizes a catalog's semantic delta vs. git HEAD, separating whitespace-only rescrape noise from real content changes so the NOTICE entry can be written accurately (no argument = every catalog; bare names like `functions.json` resolve under `vendor/pine-data/v6`). `--key <field>` and `--entry <name>` drill into the actual before/after values, which is what the NOTICE bullets need. Upstream also emits `libraries*.json` (exported symbols / history-dependence / UDT fields of popular published TV libraries); those are pine-lint import-validation data with no consumer here and are deliberately not vendored.
- `vendor/pine-manual/v6/`: the Pine User Manual as a per-page markdown tree mirroring the doc URL path (`language/operators.md`, `concepts/time.md`, ...), scraped from TradingView's docs via pine-tools. TradingView's documentation content (trademarks / copyright theirs); vendored as data for `po search` / `po show`. Each page carries YAML frontmatter (`title`, `source` URL, `section`) and `## Heading {#anchor}` section anchors that match the real URL fragments. The scraper emits the web page's Tip/Note/Notice/etc. callouts as GFM alerts (`> [!TIP]` / `> [!NOTE]`; TradingView's "Notice" folds into `[!IMPORTANT]`) and omits image embeds (screenshots whose alt text and hashed `_astro` URLs carry no text value). Gremlins (curly quotes, em-dashes, the registered mark) are legitimate typography here, so the `brokkr` gremlin scan skips this path via `brokkr.toml`'s `[gremlins] exclude`. Embedded with `include_dir`. Refresh by re-scraping and copying the tree in (`scripts/vendor-refresh.py` copies both this tree and pine-data). NOTE: this vendor lacks a LICENSE/NOTICE pair (pre-existing gap; the other vendor has them).

## Rules

### General rules

- Don't use gremlins! Em-dash, en-dash, strange quotes, whatever - they're all verboten.
- Don't remind the user of the rules. They wrote them, so they know them.
- The user can exempt you from any rule at any time.

**Exit-code convention.** Failure paths use `bail!` (anyhow renders the error and exits 1 with a message).

### Bash rules

- Never read or write from `/tmp`. All data lives in the project.
- Never run raw `cargo`, `curl`, `pkill`. Use `brokkr`.

### Vendoring rules

- New vendored sources go under `vendor/<name>/` with a LICENSE copy and a NOTICE file describing source path + snapshot date.
- Every lifted source file carries an `SPDX-License-Identifier` header pointing back to the upstream. Exception: binary or structured-data files with no comment syntax (e.g. JSON, compiled assets) cannot carry an inline header; the adjacent `LICENSE` file and the vendor `NOTICE` satisfy attribution for those files.
- The `research/` tree is read-only consultation material. Never edit it, never depend on its paths at runtime.

### Testing rules

- Tests are small and technical. Markdown parsing pinning, lookup-table sanity, pine-data JSON deserialization shape.
- Do not add tests that hit live TradingView or any network.
- Vendored data is the test fixture: pin behavior against the real `vendor/pine-data/v6/*.json` entries, not against hand-crafted toy data.
- When in doubt, write the smallest deterministic unit test that pins the behavior.

## Commands

Use `brokkr` (not `cargo`) for check/test. Output is never capped or scoped: every diagnostic prints every time, and errors in files with unstaged changes are listed first.

- `brokkr check` - gremlins + clippy + all tests
- `brokkr test <NAME>` - release-mode focused single-test runner. `<NAME>` is a case-sensitive substring filter. Streams the test's own stdout/stderr live.
  - `-N, --repeat <N>` - run the test N times (flaky-test hunting).
  - `--raw` - bypass output filtering.
  - `--debug` - build/run in dev profile (faster compile, when release-LTO time dominates).

Single-crate workspace, so `-p` is unnecessary.

Current Pine lint source of truth:

- `pine-lint` (the pine-tools CLI, installed on PATH) is the source of truth for Pine validation behavior; treat feature parity with it as the target. It reads a file path, an inline `--code/-c '<pine source>'`, or stdin via `-`, and emits JSON by default.
  - Prefer `pine-lint -H <file-or-`-c`>` for routine validity checks: human-readable one-line-per-finding output plus a summary, exit 1 on errors. This is how recipe Pine snippets are checked clean before shipping.
  - `pine-lint --tv` forwards the source to TradingView's `translate_light` endpoint instead of running locally - use it when a local result looks wrong or to confirm an error code.

## Subcommand status

| Subcommand | Status |
|---|---|
| `po lookup <name>` | done (identifier view rendered straight from pine-data: signature with params (default / allowedValues / min / max + per-argument prose), per-overload signatures, polymorphism + deprecation flags, and the prose sub-sections `remarks` / `seeAlso` / `returnsDescription`; operators are a first-class kind; multi-catalog names (`na`, `time`, ...) dump every meaning; on a miss, BM25 "did you mean ...?" suggestions via the `suggest` module. Text-only. `--list` + case-insensitive `--kind` + `--grep` browse the behavior catalog (functions / variables / constants / keywords / types / annotations / operators); `po lookup TEXT --list` treats `TEXT` as an implicit grep; pass `--kind ?` for the catalog) |
| `po search <query>` | done as a **unified finder** (BM25 over both the Pine User Manual prose, H1-H3 granularity, and the authored TA recipes, via the single `find` index). Prints a menu, no prose: a manual row is `<8-hex id>  page / H2 / H3` (feed the id to `po show`), a recipe row is `recipe <name>  recipe / category / Title` (run it directly). `--limit` defaults to 8. `-1` / `--top` shortcuts to rendering the top hit directly - a manual section subtree or a recipe body, via `render`, honouring `--no-color`. Text-only |
| `po show <id> [<id>...]` | done (renders Pine User Manual section(s) by the 8-hex id `po search` prints. Each id resolves via `manual::by_id` and renders the section **plus all its subsections** (`subtree_markdown`) through the `render` module, with the canonical TradingView URL as a trailing provenance line; multiple ids render in order. Unknown id errors; an ambiguous id (hash collision) errors and lists the matches. Hash-only addressing; text-only) |
| `po recipe <name>` | done (a Pine cookbook - how to build anything with no TradingView builtin, indicators + general helpers like easing/risk - from the authored `recipe` corpus: renders the recipe markdown body (prose + Pine v6 snippet) via `render`, with a `recipe: <category>/<name>` provenance trailer. Exact match by name or alias (case-insensitive); `--list` + `--category` + `--grep` browse the corpus (`po recipe TEXT --list` treats `TEXT` as an implicit grep; pass `--category ?` for the catalog); on a miss, fuzzy "did you mean ...?" suggestions. Text-only. The corpus is hand-authored under `assets/recipes/<category>/<name>.md`, baked via `include_dir`) |
| `po verdict add\|observe\|retire\|search\|show\|list` | done (records measured TradingView behavior per oracle source and derives whether each question is settled; `--records <DIR>` required on every verb, repeatable on read verbs, never defaulted. Full reference in `docs/verdict.md`) |
| `po version` | done (binary version + pine-data bake counts incl. operators + manual page/section counts + recipe entry/category counts) |

## Document folders

The standing layout, across every project. Three live folders plus one retired,
split by durability first, subject second.

| Folder | Contents | Rule |
|---|---|---|
| `reference/` | Durable in-repo reference for anyone working on or with the code - how the thing is built and why: `architecture.md`, `technical-implementation-spec.md`, `performance.md` (the durable record of measured numbers over time), invariants, protocol contracts | Citable from source as a source of truth. What it says must be true. |
| `docs/` | Durable in-repo documentation of how the thing is used - guides, CLI reference, the consumer-facing API surface. Sometimes exposed as a hand-edited VitePress gh-pages site | Same must-be-true rule. |
| `notes/` | Transient - work items (`todo.md`), future plans, hypotheticals, bug reports, research, analysis. Things that will die | No truth guarantee. Nothing durable cites it. |
| `plans/` | Retired | Plan documents are transient: they go in `notes/`. |

`reference/` and `docs/` are both durable and both binding. The difference is
subject, not audience: `reference/` covers how the thing is built and why - what
you need in order to change it safely - while `docs/` covers how it is used. A
developer or library consumer reads both. Where a project publishes a site,
`docs/` is what gets published; the folder means the same thing either way.
`notes/` is neither durable nor binding, which is the whole point of keeping it
separate: a document that may be wrong must not sit where a document that must
be right is expected.

The dependency direction is therefore one-way. `notes/` may cite `docs/` and
`reference/`; nothing durable may cite `notes/` - not a code comment, not
`docs/`, not `reference/`. A code comment must carry its full context, because
it outlives the note.

**Root-level convention files are exempt.** `AGENTS.md`, `CLAUDE.md`,
`README.md`, `LICENSE`, `CHANGELOG.md` and their kin are found by tooling and by
convention at the repository root, and stay there. These folders govern
documents we chose where to put, not files whose location is dictated.

In `notes/`, `docs/` and `reference/` alike, avoid citing source line numbers -
they drift fast.
