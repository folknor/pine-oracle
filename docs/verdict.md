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
                    [--negative] [--exclusive] [--key K]
po verdict observe  <id> --records <DIR> --source editor|endpoint|chart [--key K]
                    (--date YYYY-MM-DD | --date-before YYYY-MM-DD)
                    (--fixture <file.pine> | --no-fixture)
                    compile: (--accepted | --rejected | --crashed "<message>")
                             [--error SPEC]... [--warning SPEC]...
                    runtime: --result "..." --settings "..." [--error SPEC]...
                             [--candidate "name|model"]... [--selected name]...
                             [--consistent name]... [--refuted name]...
                    [--inconclusive "<reason>"] [--evidence <path>]...
                    [--evidence-raw <path>]...
                    [--environment "..."] [--note "..."] [--dry-run]
po verdict retire   <id> --records <DIR> --reason "..." [--replaced-by <id>]...
po verdict close    <id> --records <DIR> --reason "..."
po verdict reopen   <id> --records <DIR> --reason "..."
po verdict amend    <id> --records <DIR> --reason "..."
                    ([--question "..."] [--answer "..."] [--negative | --clear negative]
                     [--exclusive | --clear exclusive]
                     | --observation N (--note "..." | --result "..."
                     | --settings "..." | --environment "..." | --inconclusive "..."
                     | --clear note|environment|inconclusive))
po verdict cite     <id> --records <DIR> --section <manual id | page#anchor>
                    --quote "..." [--quote "..."]... [--note "..."]
po verdict void     <id> --records <DIR> (--observation N | --citation N) --reason "..."
po verdict evidence move   <old> <new> --records <DIR> --reason "..."
                           [--old-raw] [--new-raw]
po verdict evidence attest [<path>...] --records <DIR> --reason "..." [--raw]
po verdict evidence check  [<id>...] --records <DIR>...
po verdict search   <query> --records <DIR>... [--limit N]
po verdict show     <id>... --records <DIR>...
po verdict list     --records <DIR>... [--identifier X] [--code C]
                    [--status settled|documented|conflict|open] [--closed] [--negative]
                    [--inferred] [--retired] [--kind compile|runtime]
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
- `close` / `reopen` stop and resume the investigation of an open question,
  and `--negative` marks an answer that identifies nothing (see Negative
  answers and closed investigations).
- The write verbs take one `--records` directory, which must already exist.
  The read verbs accept `--records` more than once.
- `search` and `list` print `<id>  <status>  <kind>  <question>` rows. `show`
  prints the full question, strongest observation first, each observation
  headed by `#N`, its 1-based position in the file (display order is by
  strength, so `#N` is what stays put). `search` treats
  its query as plain words: Pine and query syntax (`strategy.exit(`, `?:`,
  `NASDAQ:AAPL`) is split into terms, never parsed, and uppercase `AND`,
  `OR` and `NOT` are words like any other. A dotted name (`ta.sma`) is
  matched as a phrase. A query that is exactly a loaded question's id (any
  case) lists that question first, in one of the `--limit` rows and never
  twice, followed by the questions whose text matches it. An 8-hex token that
  is no loaded id is a plain word.
- Each write verb holds an exclusive lock on `<records>/.lock` from its read
  of the directory through its last write, and each file write is atomic
  (temp file + rename), so concurrent writers serialize instead of losing
  each other's work. The lock file stays in place; leave it (or gitignore
  it).
- Each read (the read verbs and `observe --dry-run`) holds `.lock` shared
  for its whole load, creating the file if it can, so it never reads a
  directory halfway through a write. Where it can neither create nor open
  `.lock` (a read-only checkout or mount of a directory never written
  through that path), it reads unlocked: reads are coordinated with writers
  only when the reader can open the lock file. A `.lock` that exists but
  cannot be opened or locked fails the read.
- A write verb that reports success has synced every file it wrote and (on
  Unix; elsewhere directory entries are left to the filesystem) the
  directory entries that publish them, and a fixture is synced before the
  record referencing it is put in place, so a machine crash cannot leave a
  record whose fixture is missing. A no-op (a keyed rerun, an identical
  citation) writes and syncs nothing. If a write fails after its record was
  put in place, the error says so: check with `show` before retrying an
  unkeyed write, which would otherwise land twice.

### Capture keys

A script that records captures in a loop must be safe to rerun. `--key`
gives a question (unique in its records directory) or an observation (unique
in its question) a caller-chosen name, such as the capture's run and probe
name. Keys match exactly, byte for byte. Adding or observing under a key that is already held compares the
payload with the stored record:

- identical: nothing is written. `add` prints the existing id, `observe`
  reports the existing `#N` on stderr (unless `--quiet`);
- different: the command fails and lists every differing field, so a script
  that drifted from the record is caught instead of silently accepted;
- an amended field (see Corrections) matches any value it has held: the
  value first landed, or any amendment of it. Amending a keyed record
  therefore never breaks the script that landed it, nor one updated to the
  corrected wording, and the rerun leaves the amended value in place. A
  rerun that gave a superseded value warns (stderr, unless `--quiet`): the
  script is likely the template for the next landing, so fix it there too.
  Each field is matched on its own, so a mix of values from different
  points in the history is accepted as well;
- except the question text and the exclusive mark: an `add` rerun giving a
  superseded value of either fails, naming the amendment and the current
  value (the current value is matched first, so a mark set and cleared
  again accepts its absence). The text is the claim's identity, so matching
  the old wording would hand a script written for the old claim (both
  halves of a since-split question, say) the reworded question's id, and
  its observations would land under a claim they do not test. Exclusivity
  likewise decides how the script's next runs are compared;
- a retired question: `add` fails, naming what replaced it, rather than
  handing a withdrawn id to the observations that follow;
- a void observation: `observe` fails. The key stays with the voided record,
  so the corrected measurement needs a new key;
- evidence is compared as a set, not in order, and by location: each given
  path must be a location of exactly one stored entry (its current one or
  one it moved from, see Evidence), and every entry must be named. Where the
  named file exists, it must decode to the entry's digest under the
  interpretation recorded for that location; a missing file, or an
  unfetched Git LFS pointer in its place, is matched by its location alone,
  since the landing script may predate a move or run on a fresh clone. A former
  location warns, like a superseded value. A rerun never gives an entry a
  digest it lacks. Reusing a moved-away path for other content therefore
  fails the old script's rerun, which is the point: the location names the
  evidence, the digest proves it.

Unkeyed writes always append: two identical unkeyed chart runs are two
measurements. There is no key-less deduplication by question text, because
two experiments can ask the same sentence.

### Corrections

`amend` corrects what was written down, keeping the history beside it: the
question's text, `answer`, `negative` and `exclusive` marks, or an
observation's `note`, `result`, `settings`, `environment` or `inconclusive`
reason. `--question`, `--answer`, `--negative` or `--clear negative` and
`--exclusive` or `--clear exclusive` combine, so a question and its answer
and classification change in one write (one amendment each, with the same
reason), never through a contradictory intermediate record. Each amendment
records the UTC time, the reason, and the value before and after (either
absent when the field was added or cleared; a mark's value is `"true"`). `show` prints the history under the current value. Search
indexes current values only.

What was measured is not amendable: source, dates, fixture, outcome, codes
and candidates. A wrong measurement is withdrawn with `void`, which keeps the
observation visible, marks it `void, does not count: <reason>`, and never
lets it count again; the corrected measurement is then a new `observe`. A
void observation's codes no longer match `list --code`, and search skips it.
Marking a run inconclusive after the fact (`amend --inconclusive`) is for a
run that was measured correctly but did not test the claim; `--clear
inconclusive` makes it count again. Reviving a run, or marking the
question exclusive, can add disagreements, even to a pair that already
conflicted; `amend` then warns as `observe` does (see Sources, strength and
status), led by what it did (`after clearing inconclusive on #2`, `after
marking the question exclusive`) and naming each pair with only its new
axes (`observations newly contradict: #1 ... and #2 ... [...]`). Status is recomputed
from the result, and the write is validated like any other, so clearing a
runtime `result` is refused.

**Rewording a question.** `amend --question` is for a wording correction:
every retained observation still tests the reworded claim, and the retained
citations or premises still support the answer to it. It is the fix when
splitting a question off left the parent's text describing both halves and
all of the parent's evidence is about the half it keeps. When the evidence
splits too, the claim changed: retire the question with `--replaced-by` and
re-observe under the new questions. Rewording is refused on a retired
question, whose text is what its replacements replaced (the whole write is
refused, an answer amended alongside included). Search indexes the current
text; see Capture keys for keyed `add` reruns.

An amended question or answer is an assertion like the original: po never
checks that the observations, citations or premises supporting a question
imply its answer, so a question-level `amend` prints (stderr, unless
`--quiet`) what the answer now rests on, computed from the state it wrote:
the counting editor / chart observations and weaker counting ones by `#N`
plus the active citations (stale ones marked), or the premises with their
dispositions; the resulting status; and the questions resting on this one
(the active ones using it as a premise, with their dispositions, and the
ones it replaces). A counting editor / chart observation need not decide: a
runtime run whose candidates are all consistent or undecided counts without
settling anything. Amendments are allowed on retired questions, except for
the question text (they correct the record, they do not measure); there the
former answer rests on nothing and the summary says so.

A field's amendments must chain (each `was` is the previous `now`) and end
at the current value, so an amended field edited by hand without its history
fails validation. Git remains the audit boundary: the in-file history guards
against isolated edits, not against rewriting the history too.

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

An observation **counts** unless it is `--inconclusive`, `--crashed` or void.
All three are shown, marked, but never decide status.

Status is decided by the counting editor/chart observations (the top tier):

| Status | Compile question | Runtime question |
|---|---|---|
| `settled` | Top observations agree on the outcome, the set of error codes and the set of warning codes | Top runs don't conflict, and at least one decides: it declares no candidates, or it selects or refutes one |
| `conflict` | Top observations differ on the outcome, the error codes or the warning codes | A candidate selected by one run and refuted by another, runs whose error codes differ (one halts, one runs clean), or, under an exclusive question, runs selecting different candidates |
| `open` | No top observation (endpoint only, only crashes, or nothing yet) | No top run, or only runs whose candidates are all consistent or undecided |

### Candidates

A runtime observation may declare candidates (`--candidate "name|model"`),
competing models of what TradingView does, and say what its run did to each.
Every state is about that one run, never about the evidence as a whole:

| State | Flag | The run's result | Decides | Opposes |
|---|---|---|---|---|
| `selected` | `--selected` | establishes the model | yes | a run refuting it |
| `consistent` | `--consistent` | fits the model without establishing it: it fits another candidate too | no | nothing |
| `refuted` | `--refuted` | rules the model out | yes | a run selecting it |
| `undecided` | (neither) | does not speak to the model | no | nothing |

A run that cannot separate A from B marks both `consistent`, not both
`selected`: a later run that does separate them (selects A, refutes B) then
settles the question, where two selections would make it a conflict. A
`consistent` A and a later run refuting A are both true, so they do not
conflict. A run whose candidates are all `consistent` or `undecided` still
counts, so its halt codes are compared with the other runs' as usual.

**Exclusive candidates.** By default candidates are not assumed mutually
exclusive: one run may select two (a run establishing "adds left to right"
and "skips na" at once), and runs selecting different candidates do not
conflict. When the candidates are true alternatives (which bar stamp rule,
which fill timing, which rounding), declare it: `add --exclusive`, or
`amend --exclusive` on an existing runtime question (`--clear exclusive`
undoes it; refused on compile questions). Then:

- two counting top runs selecting different candidates conflict (`#2
  selects B where #1 selects A (exclusive)`), whether or not each run
  declared the other's candidate;
- a counting run may select at most one candidate. A void or inconclusive
  run is out of the comparison, so a run that selected two never blocks
  marking the question exclusive once it is voided or marked inconclusive,
  but clearing its inconclusive is then refused;
- the alternatives are not assumed exhaustive (the true rule may be none of
  them), so a run refuting all but one does not select the last;
- `consistent` and `undecided` mean what they always do: selecting A in
  one run and being consistent with B in another, or in the same run, is no
  conflict, since a false model can fit a run's results.

`show` prints `exclusive: the candidates are mutually exclusive
alternatives` with the mark's history.

A question that would be `open` is `documented` instead when its manual
citations hold (see Manual citations). `documented` is never a measurement:
a measured question's `settled` and `conflict` come from observations only,
and override any citation (an inferred question settles through its
premises, see Relations between questions).

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
belong to separate questions. Nothing supersedes automatically: a stale or
wrongly recorded observation is voided (see Corrections).

`observe` warns (stderr, unless `--quiet`) when the observation it just
wrote contradicts counting observations of the same strength already under
the question, by the agreement rules above (`amend --clear inconclusive`
warns the same way, see Corrections). The warning names each
contradicted `#N` with its outcome and codes (runtime: how the run halts and
what it selects, is consistent with or refutes), then says in brackets what
differs, from the same comparison that decides the conflict: `the outcome
differs`, `error codes differ (only in #3: CE10147; only in #1: CE10099)`,
the same for warning codes, or `#2 selects A that #1 refutes`. It gives the
question's disposition as `show` prints it, before and after: `is now conflict (was settled)` (or `was open`:
runs with only undecided candidates still contradict when they halt
differently), `stays conflict`, and, between two endpoint observations,
`weaker source only` with whatever the question stays (`settled`,
`documented`, `inferred (...)`). The write still happens, since a real
conflict is a legitimate record. Mixed outcomes or codes are usually
fixtures testing different claims, though (a control filed under the
question it controls, most often: see Controls): void the new observation
and file it under its own question, before more pile up. An endpoint
disagreeing with the editor is the known gap and is not warned about (`show`
annotates it).

### Controls

A control is a fixture that differs from the experiment in the one thing
the question is about, so it is expected to behave differently: the script
without the construct that the experiment's reject is blamed on. Filed
under the experiment's question it is a contradicting observation, and the
question becomes a conflict. Marking it inconclusive or void would hide it,
and a hidden control proves nothing: the question would then settle on the
experiment alone, even if the control rejected too.

A control is a claim of its own, so it is a question of its own. Record:

1. the experiment E (`fixture with X rejects with CE10099`), measured;
2. the control C (`the same fixture without X accepts`), measured;
3. the conclusion Q (`X causes CE10099`), inferred: `add --basis E --basis C`.

Q is `inferred (settled)` only once both E and C are settled, and `inferred
(open via C)` while the control is outstanding, so an unrun or conflicting
control blocks the conclusion rather than disappearing. po checks the
premises' dispositions, not that they imply the answer, nor that C really
controls E: Q's answer must say why (same symbol, timeframe and settings,
fixtures that differ only in X). A control whose answer came out unexpected
(C rejects too) still settles; reading it is the author's job, through Q's
answer, like any other evidence. If C turns out mis-scoped, rework Q's
`basis` first (`retire` refuses a premise of an active question), then
retire C with `--replaced-by` the corrected control.

`observe --dry-run` makes every check and warning `observe` makes and
writes no record and no fixture. It is a read: it loads under the shared
lock like the read verbs (creating `.lock` if it can), and works on a
read-only checkout. It prints the
question's resulting disposition on stdout, even under `--quiet`: `dry run:
observation #3 would make question d4892d70 conflict (is settled)`, `dry
run: observation #3 would leave question d4892d70 settled`, or, under a key
already held, `dry run: already recorded as observation #2; question
d4892d70 stays settled`. The contradiction warning speaks in the
conditional (`would be conflict (is settled)`) and, since nothing was
written, advises correcting the observation before landing rather than
voiding it. A refused observation fails the dry run as it would the write.
A landing script can dry-run each observation first and stop on a stray
conflict (say, a run landed without the error code its siblings carry)
before it becomes a record to void. The exit status is 0 whatever the
predicted disposition.

## Negative answers and closed investigations

Status says whether the evidence agrees, never what the answer says. Two
authored facts sit beside it and never change it; `search`, `list` and
`show` print them after the status: `settled, negative`, `open, closed`.

**Negative.** `add --negative` or `amend --negative` marks an answer that
reports that the behavior asked about was **not identified**: "none of
three running-sum orders, four fresh sums, Kahan or Neumaier reproduces
ta.sma". What it excludes is scoped to what was tested. Without the mark a
`settled` question that only ruled models out reads like a positive answer.
The mark is the author's classification of the answer, as the answer is the
author's: po does not derive it from candidates, because candidates often
model why a behavior occurs while the question asks what occurs (a run that
refutes every model of a halt still measured the halt bar), and the same
knowledge can be recorded as a refuted candidate or a selected one. A No to
a yes/no question ("does TradingView use X?") is a positive fact, not a
negative answer. Neither the mark nor its absence says whether an answer
licenses an implementation: po never checks that evidence implies an answer.
Absence asserts nothing (it is not a claim that the answer identifies the
behavior). The mark is allowed whatever the status, and an inferred question
on a negative premise resolves through the premise's status as usual.
`amend --clear negative` removes it. `list --negative` lists marked
questions that are not retired. On a retired question the mark is history,
like the answer: the disposition carries no `negative` qualifier, and `show`
prints the mark under the former answer.

`show` also prints `candidates: every declared candidate refuted, none
selected` when the counting editor/chart runs do not conflict, declare at
least one candidate, select none and refute every declared one. It is a
fact about the runs, not a status and not a classification of the answer.
It is never printed for a retired question, whose observations are not
compared.

**Closed.** `close` records that the investigation of an open question
stopped without an answer: every capture was inconclusive, say, and nobody
will pursue it. Nothing was learned, so the question stays open and shows as
`open, closed`. `list --status open` is the owed-work view and leaves closed
questions out; `list --closed` lists them. This differs from retiring: a
retired question is withdrawn (its answer becomes history), while a closed
one still stands, its record of what was tried still citable. `close` is
refused on a retired question, on an inferred one (its openness names the
premise that owes the work: close or pursue that premise instead), on one
already closed, and on one that does not resolve open. `reopen` resumes the
investigation; it is allowed whenever the last entry is a close, and refused
on a retired question, whose closure is history.

A closure is in effect only while the question resolves open. A write by
`observe`, `amend`, `void` or `cite` that leaves a closed, active, measured
question settled, conflicting or documented appends an automatic reopen in
the same write (`the question is now settled after adding observation #2`),
so voiding what decided it later leaves the question open, owed work again,
rather than silently closed. A no-op (a keyed rerun, an identical citation)
writes nothing, and `observe --dry-run` predicts the reopen. A disposition
that changes without a write (a citation turning current or stale under
another baked manual, a hand edit) only suspends the closure: `show` prints
`closure suspended`, and the closure applies again once the question is
open. Retiring a closed question, or amending a retired one, appends
nothing. `show` prints the closure history (`closed`, `reopened`,
`reopened automatically`) under the answer.

## Manual citations

When the Pine User Manual states a behavior outright, `cite` records the
passage as support for the answer, where a live capture would be impractical
or is simply not owed. It is not an observation: the manual is not a
measurement of TradingView, and a citation never makes a question `settled`.

```
po verdict cite 127addd2 --records <DIR> --section b3541569 \
    --quote "barmerge.lookahead_on returns the *first* intrabar of the period historically, but the *last* intrabar in realtime." \
    --quote "barmerge.lookahead_off always returns the last intrabar for both historical and realtime data."
```

`--section` takes the 8-hex id `po search` prints or the section's
`page#anchor`; po stores `page#anchor`. Each `--quote` must be a verbatim
passage of the section (its subsections included) in the manual baked into
this po binary, lying inside one paragraph, list item, heading, table cell
or code block. Quotes are compared on the plain text of the markdown with
whitespace collapsed, so text copied from `po show` (emphasis as `*first*`,
links without their URL) matches the source. Angle brackets are text, not
markup: `array<type>` and `<identifier>` must appear in the section as
written.

All active citations of a question form one support set. Whoever cites
asserts that they jointly establish the whole answer: po checks provenance
(the section says the quotes), never support, just as it never checks that
observations or premises imply an answer. Cite every passage the answer
needs; context that does not support the answer belongs in a note.

A question resolves `documented` when nothing measured decides it (its
measured status would be `open`), it has at least one active citation, and
every active citation is **current**. po stores the sha256 of the cited
subtree's source markdown, and a citation goes **stale** when the baked
manual no longer has the section, no longer says a quote, or the subtree
changed at all (a new qualifier elsewhere in the section, a changed link).
Staleness is decided at read time and is not a validation error: a manual
re-vendor never fails a load or blocks a write. A stale citation is shown as
`stale, does not count: <why>` and drops its question back to `open` until
someone reviews the section, voids the stale citation (`void --citation N`)
and cites again. `void --citation` also withdraws a citation applied in
error.

Citing the same section with the same quotes while an identical active
citation is current is a no-op, so a script may rerun it; with a different
`--note` it is an error (void the old citation to cite anew). Against a
stale citation it appends a new one, and the stale one keeps the question
`open` until it is voided. `cite` reports the question's resulting status on
stderr (unless `--quiet`), naming every active citation that is stale. A citation is not
allowed on an active inferred question (it is an answer route of its own,
like measurement) or on a retired one. An inferred question whose worst
premise is `documented` is `inferred (documented via <id>)`.

`show` lists citations before the observations, each headed `#N  manual
<section id>  <time>` with the section, its breadcrumb, the quotes and the
note. Search indexes the quotes and notes of active citations.

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
checks the premises' dispositions, not that they imply the answer.

An inferred question is `inferred (settled)` when every premise resolves
settled, and `inferred (documented via 5c0ffee1)` when the worst premise is
documented. Otherwise it is open, naming the worst premise (conflict before
open before documented, first in `basis` order on a tie):
`inferred (open via 3fa91c02)`. A
conflicting premise blocks the inference; it does not make the inferred
question a conflict, since nobody measured it. Premises may be inferred
themselves.

A **retired** question was withdrawn, with a required reason: for example
ill-posed (no valid form can answer it), mis-filed (its observations test
different claims), reworded into a different claim (a wording fix that
keeps the claim is `amend --question`, see Corrections), or abandoned. It
keeps its observations and its
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
retired, so `open` (closed investigations left out) and `conflict` are the
owed-work views, and `documented`
lists questions with no deciding measurement that current manual citations
support, directly or (inferred) through a documented premise. `--inferred` lists
inferred questions and combines with `--status`. `--retired` lists retired
questions and combines with neither. An unfiltered `list` shows every
question, retired ones included. `show` prints each relation both ways
(follows up / follow-ups, basis / premise of, replaced by / replaces), each
linked question with its own disposition.

**Which id to cite.** A follow-up does not supersede its parent: it may ask
a finer point while the parent's answer stays valid, so po never computes a
"current answer" from lineage. Before citing a question id at a code site,
read its `follow-ups:` line and cite the question whose answer is the claim
the code relies on, which may be the parent, a follow-up, or both. When a
parent's question or answer is actually withdrawn, retire it with
`--replaced-by`: it then shows `retired (replaced by ...)` (and, once a
replacement is itself retired, `now continued in` with the live
successors), and it drops out of every `--status` view.
Do not rewrite a still-valid answer into a pointer at a follow-up.

`show` orders observations by strength, then newest first, and among
same-day observations the later-recorded first. Weaker compile observations
are annotated against the top ones:

- `weaker source; confirmed by editor`: same outcome and codes;
- `weaker source; same outcome as editor, different codes`;
- `weaker source; editor disagrees`: a different outcome.

So an endpoint accept under an editor reject reads as a weaker observation,
not a contradiction. When the top observations themselves conflict there is
nothing single to compare against, so weaker ones go unannotated. Runtime
candidates are shown as selected, consistent, refuted or undecided.

## Records directory

```
<records>/<id>.toml                   one question and all its observations
<records>/fixtures/<sha256>.pine      content-addressed fixture store
```

po writes both. `observe --fixture` copies the file into `fixtures/`, named by
the sha256 po computes from the stored bytes, so a recorded hash always
matches its file and observations sharing a fixture share one copy. Passing a
stored fixture again by its store path (`fixtures/<sha256>.pine`) never
records the hash as the file name: a keyed rerun carries the name its own
observation recorded, and a new observation takes the name the store's
non-void observations of those bytes agree on, or none when they recorded
different names. Evidence is not copied; see Evidence.

## Evidence

An observation's `--evidence` files (chart exports, probe JSON) stay where
they are, usually elsewhere in the same git repository. Each is recorded as
an entry with a location and an identity, kept apart so a file can be
compressed or moved without breaking the records citing it.

**Identity** is the sha256 of the file's decoded content. `observe` reads
each file once: one starting like a gzip stream is decoded (every member,
each trailer checked), one starting like a zstd stream (a zstd frame, or a
skippable frame) likewise (every frame, checksums checked), anything else is
hashed as it is. The interpretation is recorded as `encoding` (absent: raw)
and followed on every later read, never re-sniffed. A file that starts like
a compressed stream but does not decode completely (truncated, a bad
checksum, trailing bytes, a dictionary frame) is refused, never hashed raw
instead, as is a compressed stream that decodes to nothing and a Git LFS
pointer (an unfetched object), raw or decoded. `--evidence-raw` records an
artifact that merely starts like a compressed stream as it is. So
`record.json` and its `record.json.zst` are the same evidence, while the
same JSON rewritten compactly is not: whitespace is content.

A digest recorded by `observe` identifies the bytes supplied when the
observation was recorded. One established later is marked `attested` with
its time and reason (see `attest` and `move` below): it identifies the
content from then on, and says nothing about the capture before. Entries
recorded before digests existed (a bare path string in the file) have
neither digest nor interpretation.

**Location** is a path relative to the records directory, `/`-separated.
Paths are resolved lexically, never through the filesystem: a path given
relative to the working directory is made absolute, `.` is dropped and `..`
folds into the directory before it, and po opens exactly the spelling it
stores. `--records` is resolved the same way, so the directory loaded is the
one locations are relative to. Symlinks are therefore not resolved: records
written before digests stored symlink-resolved paths, so a landing script
that reaches its evidence through a symlinked directory now names a
different location, and its keyed rerun fails until it names the evidence
by the stored spelling. Entries are stored `--evidence` first, then
`--evidence-raw`; the order carries no meaning. Within one observation a location (current or former) belongs to
one entry. Keep evidence inside the same git repository, or the stored
relative path will not resolve on another checkout.

**Availability is not validation.** A missing or unfetched evidence file
never fails a load or blocks a write: a store whose large artifacts are not
checked out still loads, and its repair is not blocked by the damage it
repairs. Evidence never affects a question's status: the measurement is
what the record says, and the file is the artifact behind it.

`po verdict evidence check` is the content audit. It reads every evidence
file of the given questions (all when none), void observations and retired
questions included, and prints one line per entry that fails, `<id> #N
<path>  <state>`: missing, unreadable, not decodable under its
interpretation, an unfetched Git LFS pointer, changed while being read, or
`CHANGED since its digest was recorded` (slimmed, rewritten, replaced). It
exits non-zero when any entry fails. Entries without a digest are listed on
stderr (unless `--quiet`) and fail only when unavailable. `show` prints each
entry's state the same way, read when shown, with its digest's provenance
and the locations it left.

`po verdict evidence move <old> <new>` re-points every entry in the records
directory whose current location is `<old>`: one file is usually cited by
many observations. `<new>` must decode to each entry's digest. An entry
without one is compared with `<old>` as it reads now (`--old-raw`: read it
as it is) and gains that digest, attested by the move. Every entry is
checked before any record is written, and the move is refused when `<new>`
is another evidence entry of the same observation (the same bytes recorded
twice stay two entries) or a former location of the entry read differently.
Each entry keeps a `[[moved]]` history (time, former location, its
encoding, reason). po never deletes evidence: once the move succeeds,
delete `<old>` yourself, after checking no other records directory cites
it. Each question file is replaced atomically, so an interrupted move
leaves a valid store with some entries moved; rerun it to move the rest (it
reports the entries it moved, and a no-op when none remain at `<old>`). An
`<old>` that no entry is at or has left is refused, as a likely typo.

`po verdict evidence attest [<path>...]` records the digest of entries that
have none, at the given locations or, with no path, everywhere in the
store, marked attested with `--reason`. Run it before slimming anything, so
the slimming is detectable. A given path must hold some entry; entries there
that already carry a digest are verified against it and skipped, so a rerun
after an interruption attests the rest. Every file is read before any record
is written, and one that cannot be read fails the whole run.

Every verb that reads evidence compares each file's device, inode, size
and modification time before and after reading it, and again before
writing: a change refuses the write. This detects a producer still writing;
it is not a lock, so publish evidence by atomic rename.

To shrink a store's evidence: `evidence attest` (the digests now identify
what is there), compress each file beside the original, `evidence move` the
original to the compressed copy, `evidence check`, then delete the
originals.

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

[[observation.evidence]]
path = "../fieldwork/capture-campaign-2026-09/editor-probes-2026-09-27/run26-export-outside-library.json.zst"
sha256 = "8d1b0c6a4f3e2d1c0b9a8f7e6d5c4b3a2f1e0d9c8b7a6f5e4d3c2b1a0f9e8d7c"
encoding = "zstd"

[[observation.evidence.moved]]
at = 2026-10-10T12:00:00Z
from = "../fieldwork/capture-campaign-2026-09/editor-probes-2026-09-27/run26-export-outside-library.json"
reason = "Compressed to keep the LFS store small."

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

po has no delete verb. A wrong answer, observation prose or question
wording (the claim and its evidence unchanged) is amended, a wrong
measurement is voided, and a question whose claim changed (reworded into a
different claim, split along its evidence), ill-posed or mis-filed is
retired rather than deleted, so its id and its evidence stay resolvable. An amended observation carries `[[observation.amendment]]`
tables (`at`, `field`, `was`, `now`, `reason`), a voided one an
`[observation.void]` table (`at`, `reason`); question, answer and negative
amendments are question-level `[[amendment]]` tables. Citations are question-level
`[[citation]]` tables (`section`, `quotes`, `digest`, `at`, optional `note`,
optional `[citation.void]`). Evidence entries are
`[[observation.evidence]]` tables (`path`, and with a digest `sha256`,
`encoding` when decoded, an `[observation.evidence.attested]` table (`at`,
`reason`) when established after the fact, and `[[observation.evidence.moved]]`
tables (`at`, `from`, `encoding`, `reason`)); an entry recorded before
digests may be a bare path string, which po rewrites as a table the next
time it writes the file. A negative answer carries `negative = true`
and an exclusive question `exclusive = true` among the plain fields (each
absent when unmarked), and the investigation history
is question-level `[[closure]]` tables (`at`, `action` = `close` or
`reopen`, `reason`, and `automatic = true` on a reopen po appended).

Records written before the relations split carry `derived_from = []`. The
empty list is accepted and dropped the next time po rewrites the file. A
populated `derived_from` is refused: it meant status inheritance but was used
as lineage, so each one needs a decision, usually `follow_up_to`.

## Validation

The same rules guard both directions. Every write verb refuses to
write an invalid record and leave every record and fixture untouched (only the
persistent `.lock` may be created, as any read may create it). Every read verb loads strictly: one
invalid record anywhere under the given directories fails the command, listing
every problem with its file. A successful `po verdict list --records <DIR>`
therefore validates the whole directory and serves as a CI gate. It does
not read evidence files: `po verdict evidence check` is the content gate.

Errors:

- unknown TOML fields, unknown `kind` / `source` / `outcome`;
- an empty question, answer, free-text field, fixture name, candidate model,
  evidence path, or ctx key / value;
- an identifier that is empty, has surrounding whitespace, or holds
  parentheses without being `function(parameter)`, or the same identifier
  listed twice;
- on `add` creating a question only: an identifier that is not in the
  pine-data baked into this po. An identifier is any pine-data name
  (operators like `?:` and keywords like `for` resolve), or a qualified
  parameter `function(parameter)` such as
  `strategy(process_orders_on_close)`, whose parameter must belong to that
  function. A bare parameter name is refused, because the same name is a
  parameter of many functions. Stored identifiers are never re-checked, so a
  pine-data refresh that renames or drops a name cannot fail a load or block
  a write (a keyed `add` rerun included); `show` marks such an identifier
  `(not in pine-data snapshot ...)` and the record is fixed by hand;
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
- a `chart` observation of a compile question that is not `accepted`
  (void or inconclusive included: not counting does not change what the
  source means);
- compile fields on a runtime observation, or runtime fields on a compile one;
- accepted with errors, crashed with diagnostics or without a message;
- a code with the wrong prefix for its kind and severity;
- a span that is not an ordered, 1-based `line:col-line:col` range;
- a candidate that is undeclared, listed twice, or given two states,
  or a candidate name that means different models in different observations
  of one question (runs are compared by candidate name);
- `exclusive` on a compile question, or a counting observation of an
  exclusive question selecting more than one candidate;
- a citation whose section is not `page#anchor`, with no quotes or an empty
  one, a digest that is not a sha256, a non-UTC time or an empty note; an
  active citation on an active inferred question (a stale citation is not
  an error, see Manual citations);
- an amendment of a field not amendable at its level, with an empty reason,
  empty `was` / `now`, `was` equal to `now`, or a time that is not a UTC
  timestamp; a field whose amendments do not chain or do not end at its
  current value; a `negative` or `exclusive` amendment whose `was` / `now` is neither
  `"true"` nor absent; a `void` with an empty reason or a non-UTC time;
- a closure history that does not start with a close and alternate, an
  entry with an empty reason or a non-UTC time, an automatic close; an
  active closure on an active inferred question;
- an empty `key`, a question key held by two questions of one directory, or
  an observation key used twice in one question;
- a `fixture_name` without a `fixture`;
- a stored fixture (referenced or not) that is missing or does not hash to
  its name;
- an evidence location (current or former) that is empty, absolute, or not
  in the stored form (an empty or `.` component, `..` after a directory
  name), or that two entries of one observation share; an entry revisiting
  a location under another encoding, or moving in place; a `sha256` that is
  not a lowercase sha256, or `encoding`, `attested` or `moved` without one;
  an empty attestation or move reason, or a non-UTC time. Whether the files
  exist and still match is not validation: see Evidence;
- a directory entry that is not a question file, `fixtures/`, a markdown note
  or a dot-file (a dot-directory is an error, in `fixtures/` too); an id
  present in two directories.

Warnings are printed by `add` / `observe` only (stderr, suppressed by
`--quiet`): a question with no identifiers, and a rejection with no error
recorded. Reads stay quiet. Empty identifiers are allowed on purpose, so a
question about fill ordering or session boundaries is not padded with a
stretch identifier.
