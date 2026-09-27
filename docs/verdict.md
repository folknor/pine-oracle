# `po verdict`: measured TradingView behavior

`po verdict` records what TradingView measurably did when a Pine question was
put to it, per oracle source, and answers whether each question is settled.
It records TradingView's behavior only, never whether some engine matches it.

Verdicts are not baked into the binary. Every verb takes `--records <DIR>`,
which is required and never defaulted: no environment variable, no discovery,
no user-level folder. An answer is reproducible from the command line that
produced it.

## Commands

```
po verdict add      --records <DIR> --kind compile|runtime --question "..." --answer "..."
                    [--identifier X]... [--derived-from <id>]...
po verdict observe  <id> --records <DIR> --source editor|endpoint|chart
                    (--date YYYY-MM-DD | --date-before YYYY-MM-DD)
                    (--fixture <file.pine> | --no-fixture)
                    compile: (--accepted | --rejected | --crashed "<message>")
                             [--error SPEC]... [--warning SPEC]...
                    runtime: --result "..." --settings "..." [--error SPEC]...
                             [--candidate "name|model"]... [--selected name]... [--refuted name]...
                    [--inconclusive "<reason>"] [--evidence <path>]...
                    [--environment "..."] [--note "..."]
po verdict search   <query> --records <DIR>... [--limit N]
po verdict show     <id>... --records <DIR>...
po verdict list     --records <DIR>... [--identifier X] [--code C]
                    [--status settled|conflict|open|derived] [--kind compile|runtime]
```

- `add` prints the generated 8-hex question id. Every other verb addresses the
  question by it.
- `observe` appends one observation. Run it once per measurement: eight chart
  exports that differ only in settings are eight observations.
- The write verbs take one `--records` directory, which must already exist.
  The read verbs accept `--records` more than once.
- `search` and `list` print `<id>  <status>  <kind>  <question>` rows. `show`
  prints the full question, strongest observation first. `search` treats
  its query as plain words: Pine and query syntax (`strategy.exit(`, `?:`,
  `NASDAQ:AAPL`) is split into terms, never parsed.
- Record sequentially. Each write is atomic (temp file + rename), but two
  concurrent `observe` calls on the same question race, and the last writer
  wins.

`--date` or `--date-before` is required. Use `--date-before` for a capture
whose date was never recorded, bounded by what is known (for example the date
of the commit that added the export). `show` prints it as `before
YYYY-MM-DD`, and it orders like a date.

`--fixture` or `--no-fixture` is required, so a missing fixture is always a
deliberate statement. Use `--no-fixture` when the observation was not taken on
a byte-exact file (for example inline code that predates the fixture): a
recorded hash would claim a source nobody measured.

### Diagnostic spec

`--error` and `--warning` take `CODE`, `CODE|message` or
`CODE|message|detail`. Leave the message out when the source recorded only
the code (`CE10271`, or `CE10271||9:1-9:4` with detail); `show` then prints
`(message not recorded)`. Detail is a comma-separated list of:

- a source span, `line:col-line:col` (`9:1-10:5`);
- `bar=N`, the bar a runtime error fired on;
- any other `key=value`, kept as the template context some errors carry
  instead of a span (`typeKindName=const`).

A value may be double-quoted to hold commas: `possibleValues="a, b"`. Inside
quotes, `\"` and `\\` escape a quote and a backslash. A message cannot
contain `|`.

Codes are checked against the question kind: compile errors are `CE`, compile
warnings `CW`, runtime errors `RE`, each followed by five digits. Runtime
observations take errors only.

```
--error "CE10099|Only libraries can contain exported functions.|9:1-10:5"
--error "CE10260|Cannot use the {typeKindName} keyword ...|typeKindName=const"
--error "RE10044|<banner text>|bar=100"
--error 'CE10079|<message>|possibleValues="a, b"'
--error "CE10271"
```

## Sources, strength and status

| Source | Surface | Strength |
|---|---|---|
| `editor` | The Pine editor's own compile (Save / Add to chart) | top |
| `chart` | A script run on a chart | top |
| `endpoint` | The `translate_light` compile endpoint | weaker |

The endpoint ranks lower because it accepts some scripts the editor rejects.
Runtime observations must come from `chart`. A compile question may also take
a `chart` observation: a script that ran on a chart was accepted, so such an
observation records `--accepted`. There is no local-validator source:
pine-lint is not TradingView, so its verdict is not a measurement.

An observation **counts** unless it is `--inconclusive` or `--crashed`. Both
kinds are shown, marked, but never decide status.

Status is decided by the counting editor/chart observations (the top tier):

| Status | Compile question | Runtime question |
|---|---|---|
| `settled` | Top observations agree on the outcome, the set of error codes and the set of warning codes | Top runs don't conflict, and at least one decides: it declares no candidates, or it selects or refutes one |
| `conflict` | Top observations differ on the outcome, the error codes or the warning codes | A candidate selected by one run and refuted by another, or runs whose error codes differ (one halts, one runs clean) |
| `open` | No top observation (endpoint only, only crashes, or nothing yet) | No top run, or only runs whose candidates are all undecided |

Messages never matter for agreement, because TradingView rewords them. Two
editor rejects with different codes are a conflict on purpose: TradingView
changed between the observations or the fixtures differ, and someone should
look. Two runs that differ in how they halt are a conflict for the same
reason. If the difference is intended, the runs had different inputs and
belong to separate questions. Nothing supersedes automatically: a stale
observation is removed from the TOML in a commit.

A **derived** question has `--derived-from` sources and no counting
observations of its own. It takes the worst status among its sources,
ordered conflict, open, settled, and names the source responsible:
`derived (open via 3fa91c02)`. It is `derived (settled)` only when every
source is settled. Sources may themselves be derived. `list --status`
matches the resolved status; `list --status derived` lists derived questions.

`show` orders observations by strength, then newest first, and among
same-day observations the later-recorded first. Weaker compile observations
are annotated against the top ones:

- `weaker source; confirmed by editor`: same outcome and codes;
- `weaker source; same outcome as editor, different codes`;
- `weaker source; editor disagrees`: a different outcome.

So an endpoint accept under an editor reject reads as a weaker observation,
not a contradiction. When the top observations themselves conflict there is
nothing single to compare against, so weaker ones go unannotated. Runtime
candidates are shown as selected, refuted or undecided.

## Records directory

```
<records>/<id>.toml                   one question and all its observations
<records>/fixtures/<sha256>.pine      content-addressed fixture store
```

po writes both. `observe --fixture` copies the file into `fixtures/`, named by
the sha256 po computes from the stored bytes, so a recorded hash always
matches its file and observations sharing a fixture share one copy. Evidence
is not copied: each `--evidence` path must be an existing file (pass each
export separately, not a directory) and is stored relative to the
records directory (`../fieldwork/...`). Keep evidence inside the same git
repository, or the stored relative path will not resolve on another checkout.

Besides question files and `fixtures/`, a records directory may hold markdown
notes (`*.md`) and dot-files (in-flight temp files). Anything else is an error,
so nothing in the directory is silently skipped.

```toml
kind = "compile"
question = "Can an indicator script declare an exported function?"
answer = "No. Only libraries can contain exported functions (CE10099)."
identifiers = ["export"]
derived_from = []

[[observation]]
source = "editor"
date = 2026-09-27
fixture = "520b4a5046d1594580fbf346d728996449c17ee33ee193ea051a4c70826e28b8"
fixture_name = "editor-export-outside-library.pine"
outcome = "rejected"
environment = "TradingView Desktop 3.4.1"
evidence = ["../fieldwork/capture-campaign-2026-09/editor-probes-2026-09-27/run26-export-outside-library.json"]

[[observation.error]]
code = "CE10099"
message = "Only libraries can contain exported functions."
span = "9:1-10:5"

[[observation]]
source = "endpoint"
date = 2026-09-06
outcome = "accepted"
note = "Taken on inline `export f() => 1` before the fixture file existed."
```

po has no edit or delete verb. A wrong observation or a reworded question is
fixed by editing the TOML in a commit; git keeps the history.

## Validation

The same rules guard both directions. `add` and `observe` refuse to write an
invalid record and touch nothing on disk. Every read verb loads strictly: one
invalid record anywhere under the given directories fails the command, listing
every problem with its file. A successful `po verdict list --records <DIR>`
therefore validates the whole directory and serves as a CI gate.

Errors:

- unknown TOML fields, unknown `kind` / `source` / `outcome`;
- an empty question, answer, free-text field, fixture name, candidate model,
  evidence path, or ctx key / value;
- an identifier that is not in pine-data, or the same identifier listed
  twice. An identifier is any pine-data name (operators like `?:` and
  keywords like `for` resolve), or a qualified parameter
  `function(parameter)` such as `strategy(process_orders_on_close)`, whose
  parameter must belong to that function. A bare parameter name is refused,
  because the same name is a parameter of many functions;
- a `derived_from` id that does not exist in the same directory, or a
  derivation chain that leads back to the question itself. `add` cannot
  create a cycle, since sources must exist first; a hand edit that does is
  refused on read;
- a date or date bound that is not a plain `YYYY-MM-DD`, or an observation
  with both or neither of `date` and `date_before`;
- an explicitly empty diagnostic message (leave the field out instead);
- a runtime observation not from `chart`, or without `result` / `settings`;
- compile fields on a runtime observation, or runtime fields on a compile one;
- accepted with errors, crashed with diagnostics or without a message;
- a code with the wrong prefix for its kind and severity;
- a span that is not an ordered, 1-based `line:col-line:col` range;
- a candidate that is undeclared, listed twice, or both selected and refuted,
  or a candidate name that means different models in different observations
  of one question (runs are compared by candidate name);
- a `fixture_name` without a `fixture`;
- a stored fixture (referenced or not) that is missing or does not hash to
  its name;
- an evidence path that is not an existing file;
- a directory entry that is not a question file, `fixtures/`, a markdown note
  or a dot-file; an id present in two directories.

Warnings are printed by `add` / `observe` only (stderr, suppressed by
`--quiet`): a question with no identifiers, and a rejection with no error
recorded. Reads stay quiet. Empty identifiers are allowed on purpose, so a
question about fill ordering or session boundaries is not padded with a
stretch identifier.
