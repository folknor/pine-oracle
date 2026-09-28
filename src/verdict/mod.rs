// Measured TradingView behavior ("verdicts"), backing `po verdict`. Unlike the
// baked corpora (`behavior`, `manual`, `recipe`), verdicts are NOT embedded:
// they live in a caller-named records directory (in practice piners' git) and
// every command names it with `--records`. There is no default location, no
// env var and no discovery - an answer must be reproducible from the command
// line that produced it.
//
// A records directory holds one `<id>.toml` per question plus a
// content-addressed `fixtures/<sha256>.pine` store. po writes both (`add`,
// `observe`); the read side (`load`) is strict, so any command that succeeds
// has validated every record under the directories it was given.
//
// A question records what TradingView did, per oracle source, and never
// whether any engine matches it. Source strength is editor = chart >
// endpoint; `Question::own_status` derives settled / conflict / open from the
// observations that count (not inconclusive, not crashed).
//
// Questions relate to each other in three separate ways, and none of them
// stands in for another: `follow_up_to` is lineage (this investigation grew
// out of that one; never affects status), `basis` makes a question inferred
// (its answer is concluded from premise questions rather than measured), and
// `retired` withdraws an ill-posed or mis-filed question (`replaced_by` names
// where the investigation continued). A retired question has no status at
// all: its successors' statuses are theirs, never its answer.

use serde::{Deserialize, Serialize};
use std::cmp::Reverse;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::path::PathBuf;
use std::str::FromStr;
use toml::value::Datetime;

mod load;
mod search;
mod spec;
#[cfg(test)]
mod tests;
mod validate;
mod write;

pub use load::{ListFilter, Relation, Store, load};
pub use search::SearchHit;
pub use spec::{parse_candidate, parse_diag};
pub use write::{Added, NewObservation, NewQuestion, add, observe, retire};

/// Whether a question is about compilation (accept / reject) or about what a
/// script does when it runs on a chart.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Kind {
    Compile,
    Runtime,
}

/// Which TradingView surface produced an observation. There is deliberately
/// no local-validator source: pine-lint is our own tool, and its verdict is a
/// guess about TradingView, not a measurement of it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Source {
    /// The Pine editor's own compile (Save / Add to chart).
    Editor,
    /// The `translate_light` compile endpoint. Known to accept some scripts
    /// the editor rejects, so it ranks below the editor.
    Endpoint,
    /// A script run on a chart (strategy report / data-window / log exports).
    Chart,
}

/// The result of a compile observation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Outcome {
    Accepted,
    Rejected,
    /// The oracle itself failed (e.g. `translate_light` throwing). A real
    /// observation, but neither accept nor reject; never counts toward status.
    Crashed,
}

/// What a runtime observation's evidence did to one competing model.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CandidateState {
    Selected,
    Refuted,
    Undecided,
}

/// Question status. For a measured question only observations that count
/// (not inconclusive, not crashed) at the top source strength decide it.
/// Ordered worst to best, so an inferred question's blocking premise is the
/// minimum over its premises.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Status {
    /// Counting editor/chart observations of this question disagree.
    /// Compile: on the outcome, the set of error codes or the set of warning
    /// codes. Runtime: a candidate selected by one run and refuted by
    /// another, or runs whose error codes differ (one halts, one runs clean).
    /// Never the status of an inferred question: nobody measured it.
    Conflict,
    /// Nothing decides it yet: no counting editor/chart observation, only
    /// chart runs whose candidates are all undecided, or (inferred) a premise
    /// that is not settled.
    Open,
    /// Measured: counting editor/chart observations exist, agree, and
    /// decide. Inferred: every premise is settled.
    Settled,
}

/// A question's disposition as shown by every read verb.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Resolved {
    /// Answered by its own observations.
    Measured(Status),
    /// Answered from `basis` premises: settled only when every premise is.
    /// Otherwise open, naming the worst premise (first in `basis` order on a
    /// tie); a conflicting premise blocks the inference, it does not make
    /// the inferred question a conflict.
    Inferred { status: Status, via: Option<String> },
    /// Withdrawn. Has no status; `replaced_by` is where the investigation
    /// continued.
    Retired { replaced_by: Vec<String> },
}

impl Resolved {
    /// The answer status, `None` for a retired question.
    pub fn status(&self) -> Option<Status> {
        match self {
            Resolved::Measured(s) | Resolved::Inferred { status: s, .. } => Some(*s),
            Resolved::Retired { .. } => None,
        }
    }
}

impl fmt::Display for Resolved {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Resolved::Measured(s) => f.write_str(s.as_str()),
            Resolved::Inferred { status, via: None } => write!(f, "inferred ({status})"),
            Resolved::Inferred {
                status,
                via: Some(via),
            } => write!(f, "inferred ({status} via {via})"),
            Resolved::Retired { replaced_by } if replaced_by.is_empty() => f.write_str("retired"),
            Resolved::Retired { replaced_by } => {
                write!(f, "retired (replaced by {})", replaced_by.join(", "))
            }
        }
    }
}

/// A withdrawn question, with the reason it was withdrawn: for example
/// ill-posed (no valid form can answer it), mis-filed (observations that
/// test different claims), reworded, or abandoned. Its observations stay as
/// history and never count.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Retired {
    pub reason: String,
    /// Where the investigation continued - not a claim that these questions
    /// answer the retired one. May be empty (abandoned) and may name a
    /// question that was itself retired later (a chain of splits).
    #[serde(default)]
    pub replaced_by: Vec<String>,
}

/// One diagnostic TradingView reported: a compile error / warning or a
/// runtime halt. `span` is a `line:col-line:col` source range, `bar` the bar
/// index a runtime error fired on, `ctx` the template arguments some errors
/// carry instead of a span (CE10260: `typeKindName = "const"`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Diag {
    pub code: String,
    /// Absent when the source recorded the code but never its text.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub span: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bar: Option<u64>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub ctx: BTreeMap<String, String>,
}

/// One competing model a runtime fixture was built to separate.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Candidate {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    pub state: CandidateState,
}

/// One measurement of a question by one TradingView source. Compile
/// observations carry `outcome` (+ `crash` when crashed) and diagnostics;
/// runtime observations carry `result`, `settings`, candidates and runtime
/// errors. Field order is the on-disk order: TOML needs plain values before
/// arrays of tables.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Observation {
    pub source: Source,
    /// When the observation was taken. Exactly one of `date` and
    /// `date_before` is set: `date_before` is an upper bound for captures
    /// whose date was never recorded (e.g. the commit that added an export).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub date: Option<Datetime>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub date_before: Option<Datetime>,
    /// sha256 of the fixture as stored under `fixtures/<sha256>.pine`. Absent
    /// when the observation was not taken on a byte-exact file (inline code).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fixture: Option<String>,
    /// The fixture's original file name, for humans.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fixture_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub outcome: Option<Outcome>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub crash: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub result: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub settings: Option<String>,
    /// Why this observation adjudicates nothing (confounded, fixture bug,
    /// the rule never armed). Shown, but never counts toward status.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub inconclusive: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub environment: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
    /// Evidence paths, relative to the records directory.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub evidence: Vec<String>,
    #[serde(default, rename = "error", skip_serializing_if = "Vec::is_empty")]
    pub errors: Vec<Diag>,
    #[serde(default, rename = "warning", skip_serializing_if = "Vec::is_empty")]
    pub warnings: Vec<Diag>,
    #[serde(default, rename = "candidate", skip_serializing_if = "Vec::is_empty")]
    pub candidates: Vec<Candidate>,
}

/// One question and every observation of it. `id` (the file stem) and `root`
/// (the records directory it was loaded from) are not stored in the file.
/// Field order is the on-disk order: `retired` is a table, so it follows the
/// plain values and precedes the observation array of tables.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Question {
    #[serde(skip)]
    pub id: String,
    #[serde(skip)]
    pub root: PathBuf,
    pub kind: Kind,
    pub question: String,
    /// For an inferred question, why the premises jointly establish it. For
    /// a retired one, the former answer, kept as history.
    pub answer: String,
    #[serde(default)]
    pub identifiers: Vec<String>,
    /// Lineage: the questions this investigation grew out of.
    #[serde(default)]
    pub follow_up_to: Vec<String>,
    /// Premises: a non-empty basis makes the question inferred.
    #[serde(default)]
    pub basis: Vec<String>,
    /// The retired field `derived_from`, which meant status inheritance but
    /// was used as lineage. Never written, so po drops it on the next rewrite.
    /// The empty list every old record carries is accepted; a populated one
    /// fails validation with a message naming the split, rather than a bare
    /// unknown-field error.
    #[serde(default, rename = "derived_from", skip_serializing)]
    pub legacy_derived_from: Option<toml::Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub retired: Option<Retired>,
    #[serde(default, rename = "observation", skip_serializing_if = "Vec::is_empty")]
    pub observations: Vec<Observation>,
}

/// An observation in display order plus how it relates to the strongest
/// counting observation.
#[derive(Debug, Clone)]
pub struct Ranked<'a> {
    pub observation: &'a Observation,
    pub annotation: Option<String>,
}

impl Kind {
    pub fn as_str(self) -> &'static str {
        match self {
            Kind::Compile => "compile",
            Kind::Runtime => "runtime",
        }
    }
}

impl Source {
    pub fn as_str(self) -> &'static str {
        match self {
            Source::Editor => "editor",
            Source::Endpoint => "endpoint",
            Source::Chart => "chart",
        }
    }

    /// Editor and chart are TradingView's authoritative surfaces; the
    /// endpoint is weaker because it accepts scripts the editor rejects.
    pub fn strength(self) -> u8 {
        match self {
            Source::Editor | Source::Chart => 2,
            Source::Endpoint => 1,
        }
    }
}

/// The strength a question needs from a counting observation to be settled.
const TOP_STRENGTH: u8 = 2;

impl Outcome {
    pub fn as_str(self) -> &'static str {
        match self {
            Outcome::Accepted => "accepted",
            Outcome::Rejected => "rejected",
            Outcome::Crashed => "crashed",
        }
    }
}

impl CandidateState {
    pub fn as_str(self) -> &'static str {
        match self {
            CandidateState::Selected => "selected",
            CandidateState::Refuted => "refuted",
            CandidateState::Undecided => "undecided",
        }
    }
}

impl Status {
    pub fn as_str(self) -> &'static str {
        match self {
            Status::Settled => "settled",
            Status::Conflict => "conflict",
            Status::Open => "open",
        }
    }
}

impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Parse errors for the string forms of the enums above, used by the CLI.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnknownValue {
    pub what: &'static str,
    pub value: String,
    pub expected: &'static str,
}

impl fmt::Display for UnknownValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "unknown {} `{}` (expected {})",
            self.what, self.value, self.expected
        )
    }
}

impl std::error::Error for UnknownValue {}

fn unknown(what: &'static str, value: &str, expected: &'static str) -> UnknownValue {
    UnknownValue {
        what,
        value: value.to_string(),
        expected,
    }
}

impl FromStr for Kind {
    type Err = UnknownValue;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "compile" => Ok(Kind::Compile),
            "runtime" => Ok(Kind::Runtime),
            _ => Err(unknown("kind", s, "compile or runtime")),
        }
    }
}

impl FromStr for Source {
    type Err = UnknownValue;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "editor" => Ok(Source::Editor),
            "endpoint" => Ok(Source::Endpoint),
            "chart" => Ok(Source::Chart),
            _ => Err(unknown("source", s, "editor, endpoint or chart")),
        }
    }
}

impl FromStr for Status {
    type Err = UnknownValue;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "settled" => Ok(Status::Settled),
            "conflict" => Ok(Status::Conflict),
            "open" => Ok(Status::Open),
            _ => Err(unknown("status", s, "settled, conflict or open")),
        }
    }
}

impl Observation {
    /// Whether this observation may decide status: inconclusive runs and
    /// crashed oracles are shown but never count.
    pub fn counts(&self) -> bool {
        self.inconclusive.is_none() && self.outcome != Some(Outcome::Crashed)
    }

    /// Whether this observation may decide a measured question's status: it
    /// counts and comes from a top-strength source (editor or chart).
    pub fn decides(&self) -> bool {
        self.counts() && self.source.strength() == TOP_STRENGTH
    }

    /// The date for display: `YYYY-MM-DD`, or `before YYYY-MM-DD` for an
    /// upper bound (validation guarantees exactly one, date-only).
    pub fn date_string(&self) -> String {
        match (&self.date, &self.date_before) {
            (Some(d), _) => d.to_string(),
            (None, Some(d)) => format!("before {d}"),
            (None, None) => "undated".to_string(),
        }
    }

    /// The date used for newest-first ordering: the date itself, or the
    /// upper bound. `YYYY-MM-DD` strings order chronologically.
    pub(crate) fn sort_date(&self) -> String {
        self.date
            .as_ref()
            .or(self.date_before.as_ref())
            .map(ToString::to_string)
            .unwrap_or_default()
    }
}

impl Question {
    /// Every diagnostic code any observation carries, sorted.
    pub fn codes(&self) -> BTreeSet<&str> {
        self.observations
            .iter()
            .flat_map(|o| o.errors.iter().chain(&o.warnings))
            .map(|d| d.code.as_str())
            .collect()
    }

    /// Whether the question is answered from `basis` premises. A retired
    /// question is not inferred: it asserts nothing, and its basis is history.
    pub fn is_inferred(&self) -> bool {
        !self.basis.is_empty() && self.retired.is_none()
    }

    pub fn is_retired(&self) -> bool {
        self.retired.is_some()
    }

    /// Status from this question's own observations, ignoring derivation.
    /// `Store::resolve` gives the status to show.
    pub fn own_status(&self) -> Status {
        let top = self.top_observations();
        if top.is_empty() {
            return Status::Open;
        }
        match self.kind {
            Kind::Compile => {
                // Pairwise, not against one reference: a code-less reject
                // agrees with rejects carrying CE10147 and CE10099 alike, but
                // those two still conflict with each other.
                let all_agree = top
                    .iter()
                    .enumerate()
                    .all(|(i, a)| top[i + 1..].iter().all(|b| compile_agree(a, b)));
                if all_agree {
                    Status::Settled
                } else {
                    Status::Conflict
                }
            }
            Kind::Runtime => runtime_status(&top),
        }
    }

    /// The counting observations at the top source strength.
    fn top_observations(&self) -> Vec<&Observation> {
        self.observations.iter().filter(|o| o.decides()).collect()
    }

    /// Observations strongest source first, newest first within a strength,
    /// and later-recorded first among same-day observations. Weaker counting
    /// compile observations are annotated against the top counting ones
    /// ("confirmed by editor" / "editor disagrees") so a weaker verdict reads
    /// as a weaker observation, not a contradiction. When the top observations
    /// themselves disagree there is no single verdict to compare against, so
    /// weaker ones go unannotated (the status already says `conflict`).
    /// Non-counting observations say why they do not count.
    pub fn ranked(&self) -> Vec<Ranked<'_>> {
        let mut obs: Vec<(usize, &Observation)> = self.observations.iter().enumerate().collect();
        obs.sort_by_key(|(i, o)| {
            (
                Reverse(o.source.strength()),
                Reverse(o.sort_date()),
                Reverse(*i),
            )
        });
        // A retired question's observations are history: comparing them is
        // exactly what retirement withdraws (a mis-filed question's fixtures
        // test different claims), so none is annotated against another.
        let anchor = if self.retired.is_none() && self.own_status() == Status::Settled {
            // Prefer a top observation that recorded its codes, so weaker
            // rows are compared against the most informative verdict.
            let top: Vec<&Observation> = obs
                .iter()
                .map(|(_, o)| *o)
                .filter(|o| o.decides())
                .collect();
            top.iter()
                .find(|o| !codes_unrecorded(o))
                .or(top.first())
                .copied()
        } else {
            None
        };
        obs.into_iter()
            .map(|(_, o)| o)
            .map(|o| Ranked {
                observation: o,
                annotation: annotate(self.kind, o, anchor),
            })
            .collect()
    }
}

fn annotate(kind: Kind, o: &Observation, anchor: Option<&Observation>) -> Option<String> {
    if let Some(reason) = &o.inconclusive {
        return Some(format!("inconclusive, does not count: {reason}"));
    }
    if o.outcome == Some(Outcome::Crashed) {
        return Some("oracle crashed, does not count".to_string());
    }
    let anchor = anchor?;
    if kind != Kind::Compile || o.source.strength() >= anchor.source.strength() {
        return None;
    }
    let top = anchor.source.as_str();
    let verb = if o.outcome != anchor.outcome {
        format!("{top} disagrees")
    } else if codes_unrecorded(o) && !codes_unrecorded(anchor) {
        format!("same outcome as {top}; codes not recorded")
    } else if compile_agree(o, anchor) {
        format!("confirmed by {top}")
    } else {
        format!("same outcome as {top}, different codes")
    };
    Some(format!("weaker source; {verb}"))
}

/// Whether two compile observations agree: the same outcome and the same
/// sets of error and warning codes. Messages may differ (TradingView rewords
/// them between releases). A reject that recorded no codes at all means "not
/// recorded", not "different", so it is compared on the outcome alone.
fn compile_agree(a: &Observation, b: &Observation) -> bool {
    if a.outcome != b.outcome {
        return false;
    }
    if codes_unrecorded(a) || codes_unrecorded(b) {
        return true;
    }
    codes(&a.errors) == codes(&b.errors) && codes(&a.warnings) == codes(&b.warnings)
}

/// A reject with no diagnostics: the codes were not recorded. (An accept
/// with no diagnostics is a clean accept, which does compare.)
fn codes_unrecorded(o: &Observation) -> bool {
    o.outcome == Some(Outcome::Rejected) && o.errors.is_empty() && o.warnings.is_empty()
}

fn codes(diags: &[Diag]) -> BTreeSet<&str> {
    diags.iter().map(|d| d.code.as_str()).collect()
}

/// Runtime status over the counting chart runs. Conflict when a candidate is
/// selected by one run and refuted by another (result text cannot be compared
/// automatically, so this is the contradiction that matters), or when the runs'
/// error codes differ (one halts, one runs clean: if intended, the runs had
/// different inputs and belong to separate questions). Otherwise settled only
/// if some run decides: it declares no candidates, or it selects or refutes
/// one. Runs whose candidates are all undecided leave the question open.
fn runtime_status(top: &[&Observation]) -> Status {
    let mut selected = BTreeSet::new();
    let mut refuted = BTreeSet::new();
    for o in top {
        for c in &o.candidates {
            match c.state {
                CandidateState::Selected => {
                    selected.insert(c.name.as_str());
                }
                CandidateState::Refuted => {
                    refuted.insert(c.name.as_str());
                }
                CandidateState::Undecided => {}
            }
        }
    }
    let halts_differ = top
        .iter()
        .any(|o| codes(&o.errors) != codes(&top[0].errors));
    if !selected.is_disjoint(&refuted) || halts_differ {
        return Status::Conflict;
    }
    let decides = top.iter().any(|o| {
        o.candidates.is_empty()
            || o.candidates
                .iter()
                .any(|c| c.state != CandidateState::Undecided)
    });
    if decides {
        Status::Settled
    } else {
        Status::Open
    }
}
