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
                    [--identifier X]... [--follow-up-to <id>]... [--basis <id>]...
                    [--key K]
po verdict observe  <id> --records <DIR> --source editor|endpoint|chart [--key K]
                    (--date YYYY-MM-DD | --date-before YYYY-MM-DD)
                    (--fixture <file.pine> | --no-fixture)
                    compile: (--accepted | --rejected | --crashed "<message>")
                             [--error SPEC]... [--warning SPEC]...
                    runtime: --result "..." --settings "..." [--error SPEC]...
                             [--candidate "name|model"]... [--selected name]... [--refuted name]...
                    [--inconclusive "<reason>"] [--evidence <path>]...
                    [--environment "..."] [--note "..."]
po verdict retire   <id> --records <DIR> --reason "..." [--replaced-by <id>]...
po verdict search   <query> --records <DIR>... [--limit N]
po verdict show     <id>... --records <DIR>...
po verdict list     --records <DIR>... [--identifier X] [--code C]
                    [--status settled|conflict|open] [--inferred] [--retired]
                    [--kind compile|runtime]
```

- `add` prints the generated 8-hex question id (or, under a capture key
  already held, the existing one). Every other verb addresses the question by
  it.
- `observe` appends one observation. Run it once per measurement: eight chart
  exports that differ only in settings are eight observations. Every
  observation under one question must test the same claim under comparable
  conditions; status compares them against each other, so two fixtures
  testing different claims belong to two questions.
- `retire` withdraws a question (see below). It refuses a question that is
  already retired, or that an active question uses as a premise.
- The write verbs take one `--records` directory, which must already exist.
  The read verbs accept `--records` more than once.
- `search` and `list` print `<id>  <status>  <kind>  <question>` rows. `show`
  prints the full question, strongest observation first, each observation
  headed by `#N`, its 1-based position in the file (display order is by
  strength, so `#N` is what stays put). `search` treats
  its query as plain words: Pine and query syntax (`strategy.exit(`, `?:`,
  `NASDAQ:AAPL`) is split into terms, never parsed.
- Each write verb holds an exclusive lock on `<records>/.lock` from its read
  of the directory through its last write, and each file write is atomic
  (temp file + rename), so concurrent writers serialize instead of losing
  each other's work. The lock file stays in place; leave it (or gitignore
  it).

### Capture keys

A script that records captures in a loop must be safe to rerun. `--key`
gives a question (unique in its records directory) or an observation (unique
in its question) a caller-chosen name, such as the capture's run and probe
name. Keys match exactly, byte for byte. Adding or observing under a key that is already held compares the
payload with the stored record:

- identical: nothing is written. `add` prints the existing id, `observe`
  reports the existing `#N` on stderr (unless `--quiet`);
- different: the command fails and lists every differing field, so a script
  that drifted from the record (or still emits a value since amended) is
  caught instead of silently accepted;
- a retired question: `add` fails, naming what replaced it, rather than
  handing a withdrawn id to the observations that follow.

Unkeyed writes always append: two identical unkeyed chart runs are two
measurements. There is no key-less deduplication by question text, because
two experiments can ask the same sentence.

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

Messages never matter for agreement, because TradingView rewords them. A
reject that recorded no codes at all means "not recorded", not "different":
it is compared on the outcome alone, so it neither conflicts with nor
confirms a coded reject's codes (a weaker one reads `same outcome as editor;
codes not recorded`). A clean accept is different: no codes there means no
warning, and it does compare. Two
editor rejects with different codes are a conflict on purpose: TradingView
changed between the observations or the fixtures differ, and someone should
look. Two runs that differ in how they halt are a conflict for the same
reason. If the difference is intended, the runs had different inputs and
belong to separate questions. Nothing supersedes automatically: a stale
observation is removed from the TOML in a commit.

## Relations between questions

Questions relate to each other in three separate ways. None of them stands in
for another.

| Field | Meaning | Effect on status |
|---|---|---|
| `follow_up_to` | Lineage: this investigation grew out of that one | None |
| `basis` | Premises: the answer is concluded from these questions | Makes the question inferred |
| `[retired]` `replaced_by` | Where the investigation continued after this question was withdrawn | The question has no status |

An **inferred** question has a non-empty `basis`. It is not measured, so it
may not carry a counting editor or chart observation: answer it either by
measurement (drop `basis`) or by inference, never silently both. Weaker
(endpoint) and non-counting observations may sit on it and are shown
unannotated. Its answer must say why the premises jointly establish it: po
checks that the premises are settled, not that they imply the answer.

An inferred question is `inferred (settled)` when every premise resolves
settled. Otherwise it is open, naming the worst premise (conflict before
open, first in `basis` order on a tie): `inferred (open via 3fa91c02)`. A
conflicting premise blocks the inference; it does not make the inferred
question a conflict, since nobody measured it. Premises may be inferred
themselves.

A **retired** question was withdrawn, with a required reason: for example
ill-posed (no valid form can answer it), mis-filed (its observations test
different claims), reworded, or abandoned. It keeps its observations and its
answer as history, shows the answer as the former answer, takes no more
observations, and has no status:
`retired (replaced by 9fa2f155, 120605ec)`, or `retired` when nothing
replaced it. `replaced_by` records where the investigation continued, not a
claim that the replacements answer the retired question. A replacement may
itself be retired later (a question reworded, then split); `show` then also
prints where the chain continues now, each current successor with its
status. A retired question's observations are never annotated against each
other (`confirmed by editor` and the like): comparing them is what
retirement withdraws. An active
question may not rest on a retired premise, so `retire` refuses a question
an active one uses in `basis`: rework that inference first.

`list --status` matches the resolved status of questions that are not
retired, so `open` and `conflict` are the owed-work views. `--inferred` lists
inferred questions and combines with `--status`. `--retired` lists retired
questions and combines with neither. An unfiltered `list` shows every
question, retired ones included. `show` prints each relation both ways
(follows up / follow-ups, basis / premise of, replaced by / replaces), each
linked question with its own disposition.

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
matches its file and observations sharing a fixture share one copy. Passing a
stored fixture again by its store path (`fixtures/<sha256>.pine`) keeps the
original file name the store already recorded for it. Evidence
is not copied: each `--evidence` path must be an existing file (pass each
export separately, not a directory) and is stored relative to the
records directory (`../fieldwork/...`). Keep evidence inside the same git
repository, or the stored relative path will not resolve on another checkout.

Besides question files and `fixtures/`, a records directory may hold markdown
notes (`*.md`) and dot-files (the write lock, in-flight temp files). Anything else is an error,
so nothing in the directory is silently skipped.

```toml
kind = "compile"
question = "Can an indicator script declare an exported function?"
answer = "No. Only libraries can contain exported functions (CE10099)."
identifiers = ["export"]
follow_up_to = []
basis = []

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

A retired question carries a table after the plain fields:

```
[retired]
reason = "The editor refuses the method declaration (CE10236) before the length question arises; no valid Pine form asks it."
replaced_by = ["9fa2f155", "120605ec", "00524fc4"]
```

po has no edit or delete verb. A wrong observation or a reworded question is
fixed by editing the TOML in a commit; git keeps the history. A question that
was ill-posed or mis-filed is retired rather than deleted, so its id and its
evidence stay resolvable.

Records written before the relations split carry `derived_from = []`. The
empty list is accepted and dropped the next time po rewrites the file. A
populated `derived_from` is refused: it meant status inheritance but was used
as lineage, so each one needs a decision, usually `follow_up_to`.

## Validation

The same rules guard both directions. `add`, `observe` and `retire` refuse to
write an invalid record and leave every record and fixture untouched (only the
persistent `.lock` may be created). Every read verb loads strictly: one
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
- a `follow_up_to`, `basis` or `replaced_by` id that does not exist in the
  same directory, names the question itself, or is listed twice; a chain
  through any one of the three that leads back to the question (each is
  checked on its own: a question replaced by its own follow-up is fine);
- an active inferred question with a retired premise, or with a counting
  editor or chart observation;
- a retired question with an empty `reason`;
- a populated legacy `derived_from`;
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
- an empty `key`, a question key held by two questions of one directory, or
  an observation key used twice in one question;
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
