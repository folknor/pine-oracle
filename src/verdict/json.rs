// `po verdict show --json` / `list --json`: questions as a consumer would
// otherwise read the record TOML for, resolved by po, so the consumer
// depends on the command line rather than on the file format or the store
// layout.
//
// The contract (docs/verdict.md, "JSON output"): one envelope
// `{"schema": SCHEMA, "questions": [...]}`; every key always present, null
// or empty when unrecorded, in the order written here; inner objects built
// field by field, so a change to the on-disk serde shape of `Diag` or
// `Candidate` cannot change this output by accident.

use serde_json::{Value, json};

use super::{Diag, Outcome, Question, Relation, Resolved, Store, tally};

/// Bumped on any change a consumer could notice other than an added key.
pub const SCHEMA: u32 = 1;

/// The envelope around `questions`.
pub fn envelope(questions: &[Value]) -> Value {
    json!({ "schema": SCHEMA, "questions": questions })
}

fn diag(d: &Diag) -> Value {
    json!({
        "code": d.code,
        "message": d.message,
        "span": d.span,
        "bar": d.bar,
        "ctx": d.ctx,
    })
}

impl Store {
    /// `q` as one entry of `questions`.
    pub fn json(&self, q: &Question) -> Value {
        let disposition = self.disposition(q);
        let resolved = &disposition.resolved;
        let (inferred, via) = match resolved {
            Resolved::Inferred { via, .. } => (true, via.clone()),
            _ => (false, None),
        };
        let observations: Vec<Value> = q
            .observations
            .iter()
            .enumerate()
            .map(|(i, o)| {
                json!({
                    "number": i + 1,
                    "source": o.source.as_str(),
                    "date": o.date.map(|d| d.to_string()),
                    "date_before": o.date_before.map(|d| d.to_string()),
                    "outcome": o.outcome.map(Outcome::as_str),
                    "crash": o.crash,
                    "counts": o.counts(),
                    "void": o.void.as_ref().map(|v| &v.reason),
                    "inconclusive": o.inconclusive,
                    "supersedes": o.supersedes.as_ref().map(|s| json!({
                        "observation": s.observation,
                        "at": s.at.to_string(),
                        "reason": s.reason,
                    })),
                    "errors": o.errors.iter().map(diag).collect::<Vec<_>>(),
                    "warnings": o.warnings.iter().map(diag).collect::<Vec<_>>(),
                    "result": o.result,
                    "settings": o.settings,
                    "candidates": o
                        .candidates
                        .iter()
                        .map(|c| json!({
                            "name": c.name,
                            "model": c.model,
                            "state": c.state.as_str(),
                        }))
                        .collect::<Vec<_>>(),
                    "tally": o.tally.as_ref().map(|t| json!({
                        "schema": t.schema,
                        "reader": t.reader,
                        "path": format!(
                            "verdicts/evidence/{}",
                            tally::current(&o.evidence, &t.path)
                        ),
                        "sha256": t.sha256,
                        "inputs": t
                            .inputs
                            .iter()
                            .map(|i| json!({
                                "path": format!(
                                    "verdicts/evidence/{}",
                                    tally::current(&o.evidence, &i.path)
                                ),
                                "sha256": i.sha256,
                            }))
                            .collect::<Vec<_>>(),
                        "candidates": t
                            .candidates
                            .iter()
                            .map(|(name, f)| json!({
                                "name": name,
                                "matched": f.matched,
                                "of": f.of,
                            }))
                            .collect::<Vec<_>>(),
                    })),
                    "fixture": o.fixture,
                    "fixture_path": o
                        .fixture
                        .as_ref()
                        .map(|sha| format!("verdicts/fixtures/{sha}.pine")),
                    "fixture_name": o.fixture_name,
                    "evidence": o
                        .evidence
                        .iter()
                        .map(|e| json!({
                            "path": format!("verdicts/evidence/{}", e.path),
                            "sha256": e.sha256,
                            "released": e.released.as_ref().map(|r| json!({
                                "at": r.at.to_string(),
                                "reason": r.reason,
                            })),
                        }))
                        .collect::<Vec<_>>(),
                    "environment": o.environment,
                    "note": o.note,
                })
            })
            .collect();
        json!({
            "id": q.id,
            "kind": q.kind.as_str(),
            "status": resolved.status().map_or("retired", |s| s.as_str()),
            "disposition": disposition.to_string(),
            "retired": q.is_retired(),
            "replaced_by": Relation::ReplacedBy.of(q),
            "inferred": inferred,
            "via": via,
            "basis": q.basis,
            "follow_up_to": q.follow_up_to,
            "negative": disposition.negative,
            "closed": disposition.closed,
            "question": q.question,
            "answer": q.answer,
            "identifiers": q.identifiers,
            "observations": observations,
        })
    }
}
