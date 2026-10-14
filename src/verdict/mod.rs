// Measured TradingView behavior ("verdicts"), backing `po verdict`. Unlike the
// baked corpora (`behavior`, `manual`, `recipe`), verdicts are NOT embedded:
// they live in the caller's git repository (in practice piners), in the
// `verdicts/` directory at its root, which po owns (`locate`).
//
// The verdicts directory holds `records/` (one `<id>.toml` per question),
// the content-addressed `fixtures/<sha256>.pine` store, and `evidence/`, the
// capture artifacts observations cite. po writes the first two (the write
// verbs in `write`); the read side (`load`) is strict, so any command that
// succeeds has validated every record. Evidence files are the caller's and
// are not part of that guarantee: `evidence` records their identity and
// audits their content separately, and refuses any outside `evidence/`.
//
// A question records what TradingView did, per oracle source, and never
// whether any engine matches it. Source strength is editor = chart >
// endpoint; `Question::own_status` derives settled / conflict / open from the
// observations that count (not void, not inconclusive, not crashed). Manual
// citations are not observations: they only lift a question nothing
// measured decides from open to documented, and only while every active
// citation is current against the manual baked into this binary.
//
// Questions relate to each other in three separate ways, and none of them
// stands in for another: `follow_up_to` is lineage (this investigation grew
// out of that one; never affects status), `basis` makes a question inferred
// (its answer is concluded from premise questions rather than measured), and
// `retired` withdraws an ill-posed or mis-filed question (`replaced_by` names
// where the investigation continued). A retired question has no status at
// all: its successors' statuses are theirs, never its answer.
//
// Two authored facts sit beside the derived status and never change it.
// `negative` classifies the answer: it reports that the behavior asked about
// was not identified (what it excludes is scoped to what was tested), so a
// `settled` question that only ruled models out does not read as a positive
// answer. Polarity cannot be derived from candidates: candidates often model
// why a behavior occurs while the question asks what occurs, and the same
// knowledge can be encoded either way. `[[closure]]` records that the
// investigation of an open question was stopped (and reopened); it is a
// workflow fact, not evidence, so the question stays open and shows as
// `open, closed`.

use serde::{Deserialize, Serialize};
use std::cmp::Reverse;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::path::PathBuf;
use std::str::FromStr;
use toml::value::Datetime;

mod citation;
pub mod cited;
mod evidence;
pub mod json;
mod load;
mod search;
mod spec;
#[cfg(test)]
mod tests;
mod validate;
mod write;

pub use evidence::{
    Attestation, Attested, Audit, AuditRow, Comparison, Encoding, Evidence, Fault, Inspection,
    Inspector, Merge, Merged, Move, NewEvidence, Readings, Release, ReleaseScope, Released,
    Relocated, attest, audit, merge, release, relocate,
};
pub use load::{ListFilter, Relation, Store, load};
pub use search::SearchHit;
pub use spec::{parse_candidate, parse_diag};
pub use validate::identifier_known;
pub use write::{
    Added, AmendTarget, Amended, Cited, NewObservation, NewQuestion, Observed, VoidTarget, add,
    amend, amend_all, cite, close, observe, observe_dry_run, reopen, retire, void,
};

/// The verdicts directory of the git repository the working directory is in:
/// `verdicts/` beside the nearest enclosing `.git` (a directory, or the file
/// a linked worktree or submodule has). It need not exist yet; the first
/// write creates it.
pub fn locate() -> anyhow::Result<PathBuf> {
    let cwd = std::env::current_dir()
        .map_err(|e| anyhow::anyhow!("reading the working directory: {e}"))?;
    locate_from(&cwd)
}

/// `locate` from directory `start`.
fn locate_from(start: &std::path::Path) -> anyhow::Result<PathBuf> {
    match start.ancestors().find(|dir| dir.join(".git").exists()) {
        Some(repo) => Ok(repo.join("verdicts")),
        None => anyhow::bail!(
            "{} is not inside a git repository: verdicts live in `verdicts/` at the repository root",
            start.display()
        ),
    }
}

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

/// What one runtime observation's evidence did to one candidate model. Every
/// state is local to its run: candidates are not assumed exclusive, so two
/// may be selected by one run, and exclusivity is said only by refuting.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CandidateState {
    /// The run establishes the model.
    Selected,
    /// The run's result matches the model's prediction without establishing
    /// it (it fits another candidate too). Never decides and never opposes
    /// anything: a later run refuting the model is no contradiction.
    Consistent,
    /// The run's result rules the model out.
    Refuted,
    /// The run does not speak to the model.
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
    /// chart runs whose candidates are all consistent or undecided, or (inferred) a premise
    /// that is neither settled nor documented.
    Open,
    /// Not measured, but stated by the manual: no deciding measurement, and
    /// at least one active citation with every active citation current.
    /// Inferred: the worst premise is documented. Never produced by
    /// `Question::own_status`, which is measurement-only.
    Documented,
    /// Measured: counting editor/chart observations exist, agree, and
    /// decide. Inferred: every premise is settled.
    Settled,
}

/// A question's disposition as shown by every read verb.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Resolved {
    /// Answered by its own observations (settled, conflict, or open).
    Measured(Status),
    /// Nothing measured decides it, and the manual states it: at least one
    /// active citation, every active citation current.
    Documented,
    /// Answered from `basis` premises: settled only when every premise is,
    /// documented when the worst premise is documented (named). Otherwise
    /// open, naming the worst premise (first in `basis` order on a tie); a
    /// conflicting premise blocks the inference, it does not make the
    /// inferred question a conflict.
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
            Resolved::Documented => Some(Status::Documented),
            Resolved::Retired { .. } => None,
        }
    }
}

impl fmt::Display for Resolved {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Resolved::Measured(s) => f.write_str(s.as_str()),
            Resolved::Documented => f.write_str(Status::Documented.as_str()),
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

/// What every read verb shows for a question: its resolved disposition plus
/// the two authored qualifiers, `open, closed` / `settled, negative`. The
/// qualifiers never change the status: negative classifies the answer, and a
/// closure is in effect only while the question resolves open.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Disposition {
    pub resolved: Resolved,
    /// The answer is marked negative (`Question::negative`).
    pub negative: bool,
    /// The investigation is closed and the question resolves open.
    pub closed: bool,
}

impl fmt::Display for Disposition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.resolved)?;
        if self.negative {
            f.write_str(", negative")?;
        }
        if self.closed {
            f.write_str(", closed")?;
        }
        Ok(())
    }
}

/// One step of a question's investigation history: `close` stops the
/// investigation of an open question, `reopen` resumes it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ClosureAction {
    Close,
    Reopen,
}

/// A closure or reopening, kept as history. Entries alternate starting with
/// a close; the last one is the current state. A reopen po appended itself
/// (a write left the question decided) is `automatic`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Closure {
    /// UTC timestamp.
    pub at: Datetime,
    pub action: ClosureAction,
    pub reason: String,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub automatic: bool,
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

/// A field `po verdict amend` may correct. The question's `answer` and its
/// `negative` classification, and an observation's transcribed prose. What
/// was measured (source, dates, fixture, outcome, codes, candidates) is not
/// amendable: a wrong measurement is voided and the corrected one observed
/// again.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AmendField {
    /// The question text: a wording correction, never a change of claim. A
    /// keyed `add` rerun giving a superseded text fails rather than matching.
    Question,
    Answer,
    /// History values are `"true"` (marked) or absent (not marked).
    Negative,
    /// History values as for `Negative`. Like the question text it changes
    /// how future measurements are read, so a keyed `add` rerun giving a
    /// superseded value fails rather than matching.
    Exclusive,
    Note,
    Result,
    Settings,
    Environment,
    Inconclusive,
}

/// One correction, stored beside what it corrects. `was` / `now` are absent
/// when the field was absent before / cleared. Consecutive amendments of one
/// field chain (`now` of one is `was` of the next) and the last `now` is the
/// current value; validation checks both, so an amended field is not edited
/// by hand without its history. Git remains the audit boundary.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Amendment {
    /// UTC timestamp of the amendment.
    pub at: Datetime,
    pub field: AmendField,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub was: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub now: Option<String>,
    pub reason: String,
}

/// A withdrawn record: shown, never counts. Not reversible by po.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Void {
    /// UTC timestamp of the voiding.
    pub at: Datetime,
    pub reason: String,
}

/// What an observation replaces: an earlier observation of the same
/// question (`#N`, 1-based file order), void since, typically a reanalysis of
/// the same capture with a new reader. Its own time and reason, apart from
/// the predecessor's void (which may be older). One successor per
/// predecessor: a later correction supersedes the replacement.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Supersedes {
    pub observation: usize,
    /// UTC timestamp.
    pub at: Datetime,
    pub reason: String,
}

/// A Pine User Manual passage cited as support for the answer. Not an
/// observation: the manual is not a measurement of TradingView, so a
/// citation never makes a question settled, only `documented`. All active
/// citations of a question form one support set, asserted by whoever cites
/// to establish the whole answer jointly; po checks provenance (the section
/// exists and says the quotes), never support.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Citation {
    /// `page#anchor` of the cited manual section.
    pub section: String,
    /// Verbatim passages from the section subtree, each inside one block.
    pub quotes: Vec<String>,
    /// sha256 of the cited subtree's source markdown at cite time. When the
    /// baked manual differs, the citation is stale and stops counting.
    pub digest: String,
    /// UTC timestamp of the citation.
    pub at: Datetime,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
    /// Set when the citation was withdrawn (stale and reviewed, or wrongly
    /// applied): it stays visible and never counts.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub void: Option<Void>,
}

impl Citation {
    /// Why this citation does not count, or `None` when it counts: void,
    /// or stale against the manual baked into this binary.
    pub fn not_counting(&self) -> Option<String> {
        match &self.void {
            Some(v) => Some(format!("void, does not count: {}", v.reason)),
            None => citation::staleness(self).map(|why| format!("stale, does not count: {why}")),
        }
    }

    /// The manual section's 8-hex id and breadcrumb, if it still exists.
    pub fn section_info(&self) -> Option<(String, String)> {
        citation::section(&self.section).map(|s| (s.id.clone(), s.breadcrumb()))
    }
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
    /// Caller-chosen capture key, unique within the question: re-observing
    /// under an existing key is a no-op when the payload matches and an error
    /// when it differs, so capture scripts can be rerun.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub key: Option<String>,
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
    /// Set when the observation was withdrawn (a mis-transcribed outcome,
    /// code or candidate): it stays visible and never counts.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub void: Option<Void>,
    /// The earlier observation this one replaces; see `Supersedes`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub supersedes: Option<Supersedes>,
    /// The supporting artifacts (exports, probe JSON): a set; the stored
    /// order carries no meaning. See `Evidence`.
    #[serde(
        default,
        deserialize_with = "evidence::deserialize_entries",
        skip_serializing_if = "Vec::is_empty"
    )]
    pub evidence: Vec<Evidence>,
    #[serde(default, rename = "error", skip_serializing_if = "Vec::is_empty")]
    pub errors: Vec<Diag>,
    #[serde(default, rename = "warning", skip_serializing_if = "Vec::is_empty")]
    pub warnings: Vec<Diag>,
    #[serde(default, rename = "candidate", skip_serializing_if = "Vec::is_empty")]
    pub candidates: Vec<Candidate>,
    #[serde(default, rename = "amendment", skip_serializing_if = "Vec::is_empty")]
    pub amendments: Vec<Amendment>,
}

/// One question and every observation of it. `id` (the file stem) and `root`
/// (the verdicts directory it was loaded from) are not stored in the file.
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
    /// Caller-chosen capture key, unique within the store:
    /// adding under an existing key returns that question when the payload
    /// matches and fails when it differs, so capture scripts can be rerun.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub key: Option<String>,
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
    /// The answer reports that the behavior asked about was not identified;
    /// what it excludes is scoped to what was tested. Absent asserts
    /// nothing: it is not a claim that the answer identifies the behavior.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub negative: bool,
    /// Runtime: the declared candidates are mutually exclusive alternatives
    /// (not necessarily exhaustive). Two counting runs selecting different
    /// candidates then conflict, and a counting run may select at most one.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub exclusive: bool,
    /// The retired field `derived_from`, which meant status inheritance but
    /// was used as lineage. Never written, so po drops it on the next rewrite.
    /// The empty list every old record carries is accepted; a populated one
    /// fails validation with a message naming the split, rather than a bare
    /// unknown-field error.
    #[serde(default, rename = "derived_from", skip_serializing)]
    pub legacy_derived_from: Option<toml::Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub retired: Option<Retired>,
    /// Investigation history: closures and reopenings, oldest first.
    #[serde(default, rename = "closure", skip_serializing_if = "Vec::is_empty")]
    pub closures: Vec<Closure>,
    /// Corrections of `answer` and `negative`, oldest first.
    #[serde(default, rename = "amendment", skip_serializing_if = "Vec::is_empty")]
    pub amendments: Vec<Amendment>,
    /// Manual passages supporting the answer; see `Citation`.
    #[serde(default, rename = "citation", skip_serializing_if = "Vec::is_empty")]
    pub citations: Vec<Citation>,
    #[serde(default, rename = "observation", skip_serializing_if = "Vec::is_empty")]
    pub observations: Vec<Observation>,
}

/// An observation in display order plus how it relates to the strongest
/// counting observation.
#[derive(Debug, Clone)]
pub struct Ranked<'a> {
    /// 1-based position in file order: the stable `#N` that addresses the
    /// observation, since display order is by strength.
    pub number: usize,
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
            CandidateState::Consistent => "consistent",
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
            Status::Documented => "documented",
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

impl FromStr for AmendField {
    type Err = UnknownValue;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "question" => Ok(AmendField::Question),
            "answer" => Ok(AmendField::Answer),
            "negative" => Ok(AmendField::Negative),
            "exclusive" => Ok(AmendField::Exclusive),
            "note" => Ok(AmendField::Note),
            "result" => Ok(AmendField::Result),
            "settings" => Ok(AmendField::Settings),
            "environment" => Ok(AmendField::Environment),
            "inconclusive" => Ok(AmendField::Inconclusive),
            _ => Err(unknown(
                "field",
                s,
                "question, answer, negative, exclusive, note, result, settings, environment or inconclusive",
            )),
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
            "documented" => Ok(Status::Documented),
            _ => Err(unknown(
                "status",
                s,
                "settled, documented, conflict or open",
            )),
        }
    }
}

impl AmendField {
    pub const ALL: [AmendField; 9] = [
        AmendField::Question,
        AmendField::Answer,
        AmendField::Negative,
        AmendField::Exclusive,
        AmendField::Note,
        AmendField::Result,
        AmendField::Settings,
        AmendField::Environment,
        AmendField::Inconclusive,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            AmendField::Question => "question",
            AmendField::Answer => "answer",
            AmendField::Negative => "negative",
            AmendField::Exclusive => "exclusive",
            AmendField::Note => "note",
            AmendField::Result => "result",
            AmendField::Settings => "settings",
            AmendField::Environment => "environment",
            AmendField::Inconclusive => "inconclusive",
        }
    }
}

impl Observation {
    /// Whether this observation may decide status: void records,
    /// inconclusive runs and crashed oracles are shown but never count.
    pub fn counts(&self) -> bool {
        self.void.is_none() && self.inconclusive.is_none() && self.outcome != Some(Outcome::Crashed)
    }

    /// The current value of amendable field `field` (`None` for the question
    /// fields `Question`, `Answer`, `Negative` and `Exclusive`).
    pub fn field(&self, field: AmendField) -> Option<&Option<String>> {
        match field {
            AmendField::Question
            | AmendField::Answer
            | AmendField::Negative
            | AmendField::Exclusive => None,
            AmendField::Note => Some(&self.note),
            AmendField::Result => Some(&self.result),
            AmendField::Settings => Some(&self.settings),
            AmendField::Environment => Some(&self.environment),
            AmendField::Inconclusive => Some(&self.inconclusive),
        }
    }

    /// Amendable field `field`, for writing (`None` for the question fields).
    pub fn field_mut(&mut self, field: AmendField) -> Option<&mut Option<String>> {
        match field {
            AmendField::Question
            | AmendField::Answer
            | AmendField::Negative
            | AmendField::Exclusive => None,
            AmendField::Note => Some(&mut self.note),
            AmendField::Result => Some(&mut self.result),
            AmendField::Settings => Some(&mut self.settings),
            AmendField::Environment => Some(&mut self.environment),
            AmendField::Inconclusive => Some(&mut self.inconclusive),
        }
    }

    /// Whether this observation may decide a measured question's status: it
    /// counts and comes from a top-strength source (editor or chart).
    pub fn decides(&self) -> bool {
        self.counts() && self.source.strength() == TOP_STRENGTH
    }

    /// Whether this observation and `other`, both under a question with
    /// `rules`, disagree. Two deciding observations that do make the question
    /// a conflict; whether either counts is the caller's concern.
    pub fn contradicts(&self, other: &Observation, rules: Rules) -> bool {
        !self.disagreements(other, rules).is_empty()
    }

    /// Every axis on which this observation and `other` disagree, from the
    /// one comparison that decides contradiction, so an explanation cannot
    /// drift from the verdict it explains. Compile: the outcome; with the
    /// same outcome, the error and warning code sets, unless either is a
    /// reject that recorded no codes ("not recorded", not "different").
    /// Runtime: the error code sets (how the runs halt), any candidate one
    /// selects and the other refutes, and, under an exclusive question,
    /// selections of different candidates (whether or not the other run
    /// declared them). Messages never matter.
    pub fn disagreements(&self, other: &Observation, rules: Rules) -> Vec<Disagreement> {
        let mut found = Vec::new();
        let mut differ = |ours: &[Diag], theirs: &[Diag], make: fn(Split) -> Disagreement| {
            let split = Split::of(&codes(ours), &codes(theirs));
            if !split.is_empty() {
                found.push(make(split));
            }
        };
        match rules.kind {
            Kind::Compile => {
                if self.outcome != other.outcome {
                    return vec![Disagreement::Outcome];
                }
                if codes_unrecorded(self) || codes_unrecorded(other) {
                    return Vec::new();
                }
                differ(&self.errors, &other.errors, Disagreement::ErrorCodes);
                differ(&self.warnings, &other.warnings, Disagreement::WarningCodes);
            }
            Kind::Runtime => {
                differ(&self.errors, &other.errors, Disagreement::ErrorCodes);
                let opposed = |x: &Observation, y: &Observation| -> Vec<String> {
                    x.candidates
                        .iter()
                        .filter(|c| c.state == CandidateState::Selected)
                        .filter(|c| {
                            y.candidates
                                .iter()
                                .any(|d| d.name == c.name && d.state == CandidateState::Refuted)
                        })
                        .map(|c| c.name.clone())
                        .collect()
                };
                let split = Split {
                    ours: opposed(self, other),
                    theirs: opposed(other, self),
                };
                if !split.is_empty() {
                    found.push(Disagreement::Opposed(split));
                }
                if rules.exclusive {
                    let ours = self.selected();
                    let theirs = other.selected();
                    // Only two selections can rival: a run selecting nothing
                    // asserts no alternative.
                    if !ours.is_empty() && !theirs.is_empty() {
                        let split = Split::of(&ours, &theirs);
                        if !split.is_empty() {
                            found.push(Disagreement::Rival(split));
                        }
                    }
                }
            }
        }
        found
    }

    /// The names of the candidates this run selects.
    pub fn selected(&self) -> BTreeSet<&str> {
        self.candidates
            .iter()
            .filter(|c| c.state == CandidateState::Selected)
            .map(|c| c.name.as_str())
            .collect()
    }

    /// What this observation measured, in the terms agreement compares:
    /// compile `rejected (CE10099)`, runtime `halts (RE10044); selects A`.
    pub fn verdict_summary(&self, kind: Kind) -> String {
        let listed = |diags: &mut dyn Iterator<Item = &Diag>| {
            diags
                .map(|d| d.code.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        };
        let mut parts = Vec::new();
        match kind {
            Kind::Compile => {
                let outcome = self.outcome.map_or("no outcome", Outcome::as_str);
                let all = listed(&mut self.errors.iter().chain(&self.warnings));
                if codes_unrecorded(self) {
                    parts.push(format!("{outcome} (codes not recorded)"));
                } else if all.is_empty() {
                    parts.push(outcome.to_string());
                } else {
                    parts.push(format!("{outcome} ({all})"));
                }
            }
            Kind::Runtime => {
                if self.errors.is_empty() {
                    parts.push("runs clean".to_string());
                } else {
                    parts.push(format!("halts ({})", listed(&mut self.errors.iter())));
                }
                for (state, verb) in [
                    (CandidateState::Selected, "selects"),
                    (CandidateState::Consistent, "consistent with"),
                    (CandidateState::Refuted, "refutes"),
                ] {
                    let names: Vec<&str> = self
                        .candidates
                        .iter()
                        .filter(|c| c.state == state)
                        .map(|c| c.name.as_str())
                        .collect();
                    if !names.is_empty() {
                        parts.push(format!("{verb} {}", names.join(", ")));
                    }
                }
            }
        }
        parts.join("; ")
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
    /// Every diagnostic code a non-void observation carries, sorted. A void
    /// observation was mis-recorded, so its codes describe nothing.
    pub fn codes(&self) -> BTreeSet<&str> {
        self.observations
            .iter()
            .filter(|o| o.void.is_none())
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

    /// Whether the last closure entry is a close. The closure is in effect
    /// only while the question also resolves open (`Store::disposition`).
    pub fn is_closed(&self) -> bool {
        self.closures
            .last()
            .is_some_and(|c| c.action == ClosureAction::Close)
    }

    /// `negative` in its amendment-history form: `"true"` or absent.
    pub fn negative_value(&self) -> Option<String> {
        self.negative.then(|| "true".to_string())
    }

    /// `exclusive` in its amendment-history form: `"true"` or absent.
    pub fn exclusive_value(&self) -> Option<String> {
        self.exclusive.then(|| "true".to_string())
    }

    /// What this question's observations are compared under.
    pub fn rules(&self) -> Rules {
        Rules {
            kind: self.kind,
            exclusive: self.exclusive,
        }
    }

    /// Whether the counting top runs, without conflicting, declare at least
    /// one candidate, select none and refute every declared one. A factual
    /// summary `show` prints; it is not a status and says nothing about the
    /// answer's polarity (candidates often model why, while the question
    /// asks what). Never for a retired question: comparing its observations
    /// is what retirement withdraws.
    pub fn every_candidate_refuted(&self) -> bool {
        if self.kind != Kind::Runtime || self.is_retired() || self.own_status() == Status::Conflict
        {
            return false;
        }
        let top = self.top_observations();
        let declared: BTreeSet<&str> = top
            .iter()
            .flat_map(|o| &o.candidates)
            .map(|c| c.name.as_str())
            .collect();
        let state = |s: CandidateState| -> BTreeSet<&str> {
            top.iter()
                .flat_map(|o| &o.candidates)
                .filter(|c| c.state == s)
                .map(|c| c.name.as_str())
                .collect()
        };
        !declared.is_empty()
            && state(CandidateState::Selected).is_empty()
            && state(CandidateState::Refuted) == declared
    }

    /// Whether the citations support the answer: at least one active (not
    /// void) citation, and every active one current. One stale member stops
    /// the whole set, since the citations jointly assert the answer: void it
    /// after review and cite again.
    pub fn citations_hold(&self) -> bool {
        let mut active = self
            .citations
            .iter()
            .filter(|c| c.void.is_none())
            .peekable();
        active.peek().is_some() && active.all(|c| citation::staleness(c).is_none())
    }

    /// Status from this question's own observations, ignoring derivation.
    /// `Store::resolve` gives the status to show.
    pub fn own_status(&self) -> Status {
        let top = self.top_observations();
        if top.is_empty() {
            return Status::Open;
        }
        // Pairwise, not against one reference: a code-less reject agrees
        // with rejects carrying CE10147 and CE10099 alike, but those two
        // still conflict with each other.
        let conflict = top
            .iter()
            .enumerate()
            .any(|(i, a)| top[i + 1..].iter().any(|b| a.contradicts(b, self.rules())));
        if conflict {
            return Status::Conflict;
        }
        match self.kind {
            Kind::Compile => Status::Settled,
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
            .map(|(i, o)| Ranked {
                number: i + 1,
                observation: o,
                annotation: annotate(self.kind, o, anchor),
            })
            .collect()
    }
}

fn annotate(kind: Kind, o: &Observation, anchor: Option<&Observation>) -> Option<String> {
    if let Some(void) = &o.void {
        return Some(format!("void, does not count: {}", void.reason));
    }
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
    !a.contradicts(
        b,
        Rules {
            kind: Kind::Compile,
            exclusive: false,
        },
    )
}

/// A reject with no diagnostics: the codes were not recorded. (An accept
/// with no diagnostics is a clean accept, which does compare.)
fn codes_unrecorded(o: &Observation) -> bool {
    o.outcome == Some(Outcome::Rejected) && o.errors.is_empty() && o.warnings.is_empty()
}

fn codes(diags: &[Diag]) -> BTreeSet<&str> {
    diags.iter().map(|d| d.code.as_str()).collect()
}

/// One axis on which two observations disagree (see
/// `Observation::disagreements`). Code and candidate lists are directional:
/// `ours` belongs to the observation compared, `theirs` to the other.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Disagreement {
    /// Compile: one accepted, the other rejected.
    Outcome,
    /// Error codes only one carries. Runtime: the runs halt differently
    /// (one may run clean).
    ErrorCodes(Split),
    /// Compile: warning codes only one carries.
    WarningCodes(Split),
    /// Runtime: candidates one run selects and the other refutes.
    Opposed(Split),
    /// Runtime, exclusive question: the runs select different candidates.
    Rival(Split),
}

/// What observations of one question are compared under: its kind, and
/// whether its candidates are exclusive alternatives.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rules {
    pub kind: Kind,
    pub exclusive: bool,
}

/// What only the compared observation has (`ours`) and only the other has
/// (`theirs`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Split {
    pub ours: Vec<String>,
    pub theirs: Vec<String>,
}

impl Split {
    fn of(ours: &BTreeSet<&str>, theirs: &BTreeSet<&str>) -> Split {
        let only = |a: &BTreeSet<&str>, b: &BTreeSet<&str>| {
            a.difference(b).map(ToString::to_string).collect()
        };
        Split {
            ours: only(ours, theirs),
            theirs: only(theirs, ours),
        }
    }

    fn is_empty(&self) -> bool {
        self.ours.is_empty() && self.theirs.is_empty()
    }
}

impl Disagreement {
    /// The disagreement in words, naming the compared observation `us` and
    /// the other `them` (e.g. `#3`, `#1`).
    pub fn describe(&self, us: &str, them: &str) -> String {
        let list = |v: &[String]| {
            if v.is_empty() {
                "none".to_string()
            } else {
                v.join(", ")
            }
        };
        let codes = |what: &str, s: &Split| {
            format!(
                "{what} codes differ (only in {us}: {}; only in {them}: {})",
                list(&s.ours),
                list(&s.theirs)
            )
        };
        match self {
            Disagreement::Outcome => "the outcome differs".to_string(),
            Disagreement::ErrorCodes(s) => codes("error", s),
            Disagreement::WarningCodes(s) => codes("warning", s),
            Disagreement::Opposed(s) => {
                let mut parts = Vec::new();
                if !s.ours.is_empty() {
                    parts.push(format!(
                        "{us} selects {} that {them} refutes",
                        s.ours.join(", ")
                    ));
                }
                if !s.theirs.is_empty() {
                    parts.push(format!(
                        "{them} selects {} that {us} refutes",
                        s.theirs.join(", ")
                    ));
                }
                parts.join("; ")
            }
            Disagreement::Rival(s) => format!(
                "{us} selects {} where {them} selects {} (exclusive)",
                list(&s.ours),
                list(&s.theirs)
            ),
        }
    }
}

/// Runtime status over counting chart runs that contradict nothing: settled
/// only if some run decides (it declares no candidates, or it selects or
/// refutes one). Runs whose candidates are all consistent or undecided leave
/// it open: neither separates the models.
fn runtime_status(top: &[&Observation]) -> Status {
    let decides = top.iter().any(|o| {
        o.candidates.is_empty()
            || o.candidates
                .iter()
                .any(|c| matches!(c.state, CandidateState::Selected | CandidateState::Refuted))
    });
    if decides {
        Status::Settled
    } else {
        Status::Open
    }
}
