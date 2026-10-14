// Tallies: the machine-readable fit counts a capture reader computed, landed
// with a runtime observation by `observe --tally` so its candidate states are
// checked against numbers rather than against prose.
//
// A tally says, per declared candidate, how many of the reader's evaluated
// predictions matched what TradingView did (`matched` of `of`). What counts as
// a valid evaluated prediction (missing rows, tolerances, exclusions) is the
// reader's, and is pinned by the tally's inputs, never re-derived here. po
// checks the bookkeeping only: the states the observation declares must be
// compatible with the counts. Fit never establishes selection: a candidate
// that matched every prediction is consistent or selected, and choosing
// selected stays the author's claim, as it is without a tally.
//
// The tally file is an evidence entry of the observation, read once by its
// leading bytes (a compressed file decoded) at `observe`: the digest and the
// parsed facts come from that one read, and the facts are stored in the
// record, so validation and `show` never reopen the file (which may later be
// compressed, moved, unfetched or released). Each input the tally names must
// be an evidence entry of the same observation holding that digest, so the
// derivation's manifest binds to pinned evidence. That binding is what the
// reader asserted, not proof that it read those inputs.

use anyhow::{Result, bail};
use serde::de::{self, MapAccess, Visitor};
use serde::{Deserialize, Deserializer, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use super::evidence::{Evidence, owner};
use super::validate::is_sha256;
use super::{CandidateState, Observation};

/// The tally file format this po reads, and the rules its stored facts are
/// judged by.
pub const SCHEMA: u32 = 1;

/// One candidate's fit: of the `of` predictions evaluated, `matched` agreed
/// with the run.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Fit {
    pub matched: u64,
    pub of: u64,
}

/// An input the reader derived the tally from: an evidence location and the
/// digest of what it read there.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Input {
    pub path: String,
    pub sha256: String,
}

/// The facts of a tally as stored on its observation. Field order is the
/// on-disk order: plain values, then the candidate table, then the inputs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Tally {
    pub schema: u32,
    /// The reader's name and version, as it reported them.
    pub reader: String,
    /// The tally file's location (an evidence location of the observation).
    pub path: String,
    /// Its digest, the identity of that evidence entry.
    pub sha256: String,
    pub candidates: BTreeMap<String, Fit>,
    #[serde(default, rename = "input", skip_serializing_if = "Vec::is_empty")]
    pub inputs: Vec<Input>,
}

/// A tally file as parsed: everything but its own location and digest.
#[derive(Debug, Clone)]
pub(super) struct Parsed {
    pub(super) reader: String,
    pub(super) candidates: BTreeMap<String, Fit>,
    pub(super) inputs: Vec<Input>,
}

#[derive(Deserialize)]
struct Head {
    schema: u32,
}

/// Schema 1. Keys po does not read (the reader's own counts, a query name)
/// are allowed and stay in the file; a key given twice is refused.
#[derive(Deserialize)]
struct File {
    #[serde(rename = "schema")]
    _schema: u32,
    reader: String,
    #[serde(default)]
    inputs: Vec<FileInput>,
    candidates: Candidates,
}

#[derive(Deserialize)]
struct FileInput {
    path: String,
    sha256: String,
}

#[derive(Deserialize)]
struct FileFit {
    matched: u64,
    of: u64,
}

/// The `candidates` object, refusing a name given twice (a JSON map would
/// otherwise keep the last silently).
struct Candidates(Vec<(String, FileFit)>);

impl<'de> Deserialize<'de> for Candidates {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        struct V;
        impl<'de> Visitor<'de> for V {
            type Value = Candidates;
            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str("an object of candidate name -> {matched, of}")
            }
            fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Candidates, A::Error> {
                let mut out: Vec<(String, FileFit)> = Vec::new();
                while let Some((name, fit)) = map.next_entry::<String, FileFit>()? {
                    if out.iter().any(|(n, _)| *n == name) {
                        return Err(de::Error::custom(format!(
                            "candidate `{name}` is given twice"
                        )));
                    }
                    out.push((name, fit));
                }
                Ok(Candidates(out))
            }
        }
        d.deserialize_map(V)
    }
}

/// Any JSON value, refusing an object that gives a member twice at any
/// depth: JSON parsers disagree on which one wins, so the pinned file would
/// mean different things to po and to whoever reads it next.
struct Unambiguous;

impl<'de> Deserialize<'de> for Unambiguous {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        struct V;
        impl<'de> Visitor<'de> for V {
            type Value = Unambiguous;
            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str("a JSON value")
            }
            fn visit_bool<E>(self, _: bool) -> Result<Unambiguous, E> {
                Ok(Unambiguous)
            }
            fn visit_i64<E>(self, _: i64) -> Result<Unambiguous, E> {
                Ok(Unambiguous)
            }
            fn visit_u64<E>(self, _: u64) -> Result<Unambiguous, E> {
                Ok(Unambiguous)
            }
            fn visit_f64<E>(self, _: f64) -> Result<Unambiguous, E> {
                Ok(Unambiguous)
            }
            fn visit_str<E>(self, _: &str) -> Result<Unambiguous, E> {
                Ok(Unambiguous)
            }
            fn visit_unit<E>(self) -> Result<Unambiguous, E> {
                Ok(Unambiguous)
            }
            fn visit_seq<A: de::SeqAccess<'de>>(self, mut seq: A) -> Result<Unambiguous, A::Error> {
                while seq.next_element::<Unambiguous>()?.is_some() {}
                Ok(Unambiguous)
            }
            fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Unambiguous, A::Error> {
                let mut keys = BTreeSet::new();
                while let Some(key) = map.next_key::<String>()? {
                    if !keys.insert(key.clone()) {
                        return Err(de::Error::custom(format!("member `{key}` is given twice")));
                    }
                    map.next_value::<Unambiguous>()?;
                }
                Ok(Unambiguous)
            }
        }
        d.deserialize_any(V)
    }
}

/// Parse a tally file's content. Refuses another schema, an empty reader or
/// candidate set, a count above its total, a malformed input digest, and a
/// member given twice anywhere; whether the facts fit the observation is
/// `validate`'s.
pub(super) fn parse(bytes: &[u8]) -> Result<Parsed> {
    serde_json::from_slice::<Unambiguous>(bytes)
        .map_err(|e| anyhow::anyhow!("not an unambiguous JSON tally: {e}"))?;
    let head: Head = serde_json::from_slice(bytes)
        .map_err(|e| anyhow::anyhow!("not a tally (a JSON object with `schema`): {e}"))?;
    if head.schema != SCHEMA {
        bail!(
            "tally schema {} is not one this po reads (it reads {SCHEMA})",
            head.schema
        );
    }
    let file: File =
        serde_json::from_slice(bytes).map_err(|e| anyhow::anyhow!("tally schema {SCHEMA}: {e}"))?;
    let reader = file.reader.trim().to_string();
    if reader.is_empty() {
        bail!("the tally names no `reader`");
    }
    if file.candidates.0.is_empty() {
        bail!("the tally has no candidates: a tally checks candidate states, so it needs one");
    }
    let mut candidates = BTreeMap::new();
    for (name, fit) in file.candidates.0 {
        if fit.matched > fit.of {
            bail!(
                "candidate `{name}` matched {} of {}: more than were evaluated",
                fit.matched,
                fit.of
            );
        }
        candidates.insert(
            name,
            Fit {
                matched: fit.matched,
                of: fit.of,
            },
        );
    }
    let mut inputs: Vec<Input> = Vec::new();
    for i in file.inputs {
        if !is_sha256(&i.sha256) {
            bail!(
                "tally input `{}`: `{}` is not a lowercase sha256",
                i.path,
                i.sha256
            );
        }
        if inputs.iter().any(|x| x.path == i.path) {
            bail!("tally input `{}` is given twice", i.path);
        }
        inputs.push(Input {
            path: i.path,
            sha256: i.sha256,
        });
    }
    Ok(Parsed {
        reader,
        candidates,
        inputs,
    })
}

/// The state a fit allows, as text for messages.
fn allowed(fit: Fit) -> &'static str {
    if fit.of == 0 {
        "undecided (nothing was evaluated)"
    } else if fit.matched < fit.of {
        "refuted (a prediction failed)"
    } else {
        "consistent or selected (every prediction matched)"
    }
}

/// Whether `state` is compatible with `fit`.
fn compatible(state: CandidateState, fit: Fit) -> bool {
    match state {
        CandidateState::Undecided => fit.of == 0,
        CandidateState::Refuted => fit.of > 0 && fit.matched < fit.of,
        CandidateState::Consistent | CandidateState::Selected => {
            fit.of > 0 && fit.matched == fit.of
        }
    }
}

/// Where the entry a tally location names is now: the location stored when
/// the tally landed stays the binding, and a move or merge since re-points
/// the entry, not the tally.
pub fn current<'a>(entries: &'a [Evidence], location: &'a str) -> &'a str {
    owner(entries, location).map_or(location, |i| entries[i].path.as_str())
}

/// Whether `entries` hold an entry answering to `location` with `sha256`.
fn bound(entries: &[Evidence], location: &str, sha256: &str) -> Result<(), String> {
    match owner(entries, location) {
        None => Err(format!("`{location}` is not evidence of the observation")),
        Some(i) if entries[i].sha256.as_deref() != Some(sha256) => Err(format!(
            "`{location}` is evidence of the observation with another digest"
        )),
        Some(_) => Ok(()),
    }
}

/// Errors in an observation's tally, judged from the stored facts alone:
/// its shape, its binding to the observation's evidence, and the declared
/// candidate states against the counts. Under `exclusive`, a counting run's
/// selected candidate must be the only one that matched everything (two
/// survivors are both consistent); void and inconclusive runs are out of
/// that comparison, as for every exclusive rule.
pub(super) fn validate(o: &Observation, exclusive: bool) -> Vec<String> {
    let Some(t) = &o.tally else {
        return Vec::new();
    };
    let mut errs = Vec::new();
    if t.schema != SCHEMA {
        errs.push(format!(
            "tally schema {} is not one this po reads (it reads {SCHEMA})",
            t.schema
        ));
        return errs;
    }
    if t.reader.trim().is_empty() {
        errs.push("tally `reader` is empty".to_string());
    }
    if !is_sha256(&t.sha256) {
        errs.push(format!(
            "tally `sha256` `{}` is not a lowercase sha256",
            t.sha256
        ));
    }
    if let Err(why) = bound(&o.evidence, &t.path, &t.sha256) {
        errs.push(format!("the tally file {why}"));
    }
    let mut seen = BTreeSet::new();
    for i in &t.inputs {
        if !seen.insert(i.path.as_str()) {
            errs.push(format!("tally input `{}` is listed twice", i.path));
        }
        if !is_sha256(&i.sha256) {
            errs.push(format!(
                "tally input `{}`: `{}` is not a lowercase sha256",
                i.path, i.sha256
            ));
        } else if let Err(why) = bound(&o.evidence, &i.path, &i.sha256) {
            errs.push(format!("tally input {why}"));
        }
    }
    if t.candidates.is_empty() {
        errs.push("the tally has no candidates".to_string());
    }
    let declared: BTreeSet<&str> = o.candidates.iter().map(|c| c.name.as_str()).collect();
    let tallied: BTreeSet<&str> = t.candidates.keys().map(String::as_str).collect();
    for name in declared.difference(&tallied) {
        errs.push(format!("declared candidate `{name}` is not in the tally"));
    }
    for name in tallied.difference(&declared) {
        errs.push(format!(
            "the tally counts `{name}`, which is not a declared candidate"
        ));
    }
    for (name, fit) in &t.candidates {
        if fit.matched > fit.of {
            errs.push(format!(
                "tally candidate `{name}` matched {} of {}",
                fit.matched, fit.of
            ));
        }
    }
    let survivors: Vec<&str> = t
        .candidates
        .iter()
        .filter(|(_, f)| f.of > 0 && f.matched == f.of)
        .map(|(n, _)| n.as_str())
        .collect();
    for c in &o.candidates {
        let Some(fit) = t.candidates.get(&c.name) else {
            continue;
        };
        if !compatible(c.state, *fit) {
            errs.push(format!(
                "candidate `{}` is {} but matched {} of {}: the tally allows {}",
                c.name,
                c.state.as_str(),
                fit.matched,
                fit.of,
                allowed(*fit)
            ));
        }
        if exclusive && o.counts() && c.state == CandidateState::Selected && survivors.len() > 1 {
            errs.push(format!(
                "candidate `{}` is selected, but {} matched every prediction too: \
                 exclusive alternatives this run cannot separate are consistent",
                c.name,
                survivors
                    .iter()
                    .filter(|n| **n != c.name)
                    .copied()
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }
    }
    errs
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_refuses_duplicates_and_other_schemas_but_keeps_extras_out() {
        let ok = br#"{"schema": 1, "reader": "r 1", "separating": 27,
            "candidates": {"a": {"matched": 2, "of": 2}, "b": {"matched": 1, "of": 2}}}"#;
        let p = parse(ok).expect("extras are allowed");
        assert_eq!(p.candidates.len(), 2);
        let dup = br#"{"schema": 1, "reader": "r", "candidates": {"a": {"matched": 1, "of": 1}, "a": {"matched": 0, "of": 1}}}"#;
        assert!(parse(dup).unwrap_err().to_string().contains("given twice"));
        for ambiguous in [
            &br#"{"schema": 1, "reader": "r", "separating": 0, "separating": 27, "candidates": {"a": {"matched": 1, "of": 1}}}"#[..],
            br#"{"schema": 1, "reader": "r", "extra": [{"x": 1, "x": 2}], "candidates": {"a": {"matched": 1, "of": 1}}}"#,
        ] {
            assert!(parse(ambiguous).unwrap_err().to_string().contains("given twice"));
        }
        let two = br#"{"schema": 2, "reader": "r", "candidates": {}}"#;
        assert!(parse(two).unwrap_err().to_string().contains("schema 2"));
        let over = br#"{"schema": 1, "reader": "r", "candidates": {"a": {"matched": 3, "of": 2}}}"#;
        assert!(parse(over).unwrap_err().to_string().contains("more than"));
        let none = br#"{"schema": 1, "reader": "r", "candidates": {}}"#;
        assert!(
            parse(none)
                .unwrap_err()
                .to_string()
                .contains("no candidates")
        );
    }

    #[test]
    fn fit_allows_states() {
        let fit = |matched, of| Fit { matched, of };
        assert!(compatible(CandidateState::Undecided, fit(0, 0)));
        assert!(!compatible(CandidateState::Consistent, fit(0, 0)));
        assert!(compatible(CandidateState::Refuted, fit(1, 2)));
        assert!(!compatible(CandidateState::Selected, fit(1, 2)));
        assert!(compatible(CandidateState::Selected, fit(2, 2)));
        assert!(compatible(CandidateState::Consistent, fit(2, 2)));
        assert!(!compatible(CandidateState::Refuted, fit(2, 2)));
        assert!(!compatible(CandidateState::Undecided, fit(2, 2)));
    }
}
