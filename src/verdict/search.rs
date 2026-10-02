// BM25 over a loaded verdict store (tantivy, RAM-backed, built per call - the
// store is loaded per invocation from a caller-named directory, so there is
// nothing to cache across runs). The title field (question, identifiers,
// codes) is boosted over the body (answer, results, messages, notes,
// citation quotes, fixture source), matching the manual / recipe finder. Void
// observations and citations are left out: they were withdrawn.

use anyhow::Result;
use tantivy::collector::TopDocs;
use tantivy::query::{BooleanQuery, BoostQuery, Query};
use tantivy::schema::{STORED, STRING, Schema, TEXT, Value};
use tantivy::{Index, TantivyDocument};

use super::validate::fixture_path;
use super::{Question, Store};

const TITLE_BOOST: f32 = 4.0;

/// One ranked question.
#[derive(Debug, Clone)]
pub struct SearchHit {
    pub id: String,
    pub score: f32,
}

impl Store {
    /// Ranked questions for `q`. An empty query returns nothing.
    pub fn search(&self, q: &str, limit: usize) -> Result<Vec<SearchHit>> {
        // tantivy's TopDocs panics on a zero limit.
        if q.trim().is_empty() || limit == 0 || self.questions().is_empty() {
            return Ok(Vec::new());
        }
        let mut builder = Schema::builder();
        let title = builder.add_text_field("title", TEXT);
        let content = builder.add_text_field("content", TEXT);
        let id = builder.add_text_field("id", STRING | STORED);
        let index = Index::create_in_ram(builder.build());
        let mut writer = index.writer(15_000_000)?;
        for question in self.questions() {
            let mut doc = TantivyDocument::default();
            doc.add_text(title, title_text(question));
            doc.add_text(content, content_text(question));
            doc.add_text(id, &question.id);
            writer.add_document(doc)?;
        }
        writer.commit()?;

        let searcher = index.reader()?.searcher();
        // Plain words: verdict queries are full of Pine syntax
        // (`strategy.exit(`, `?:`, `a:b`) and never mean query operators.
        let title_q = crate::query::field_query(&index, title, q)?;
        let content_q = crate::query::field_query(&index, content, q)?;
        let boosted: Box<dyn Query> = Box::new(BoostQuery::new(title_q, TITLE_BOOST));
        let query = BooleanQuery::union(vec![boosted, content_q]);
        let top = searcher.search(&query, &TopDocs::with_limit(limit).order_by_score())?;
        let mut hits = Vec::with_capacity(top.len());
        for (score, addr) in top {
            let doc: TantivyDocument = searcher.doc(addr)?;
            let hit_id = doc
                .get_first(id)
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string();
            hits.push(SearchHit { id: hit_id, score });
        }
        Ok(hits)
    }
}

fn title_text(q: &Question) -> String {
    let codes: Vec<&str> = q.codes().into_iter().collect();
    format!(
        "{} {} {}",
        q.question,
        q.identifiers.join(" "),
        codes.join(" ")
    )
}

fn content_text(q: &Question) -> String {
    let mut parts: Vec<String> = vec![q.answer.clone()];
    parts.extend(q.retired.as_ref().map(|r| r.reason.clone()));
    // Void records were withdrawn; a search must not find a question by them.
    for c in q.citations.iter().filter(|c| c.void.is_none()) {
        parts.extend(c.quotes.iter().cloned());
        parts.extend(c.note.clone());
    }
    let mut fixtures = std::collections::BTreeSet::new();
    for o in q.observations.iter().filter(|o| o.void.is_none()) {
        parts.extend(
            [
                &o.result,
                &o.settings,
                &o.note,
                &o.inconclusive,
                &o.crash,
                &o.environment,
            ]
            .into_iter()
            .flatten()
            .cloned(),
        );
        for d in o.errors.iter().chain(&o.warnings) {
            parts.extend(d.message.clone());
            parts.extend(d.ctx.values().cloned());
        }
        for c in &o.candidates {
            parts.push(c.name.clone());
            parts.extend(c.model.clone());
        }
        if let Some(sha) = &o.fixture {
            fixtures.insert(sha.clone());
        }
    }
    // Fixture source makes a question findable by the Pine it exercised.
    // Validation already proved each file exists and matches its hash.
    for sha in fixtures {
        if let Ok(src) = std::fs::read_to_string(fixture_path(&q.root, &sha)) {
            parts.push(src);
        }
    }
    parts.join("\n")
}
