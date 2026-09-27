// BM25 "did you mean ...?" suggestions over the pine-data name surface.
//
// This is the engine behind `po lookup`'s miss path: when an exact (or
// multi-catalog) lookup finds nothing, `suggest` returns the closest identifier
// names so the user can retry. It is NOT a user-facing command - the standalone
// name-search verb was retired; `po search` is reserved for manual prose search.
//
// The index is built lazily into a RAMDirectory on first call, cached via
// OnceLock. A few thousand compact documents; build cost stays in low ms.
//
// Scoring: name field carries a 5x boost over content, so `rsi` puts `ta.rsi`
// ahead of any prose paragraph that merely mentions RSI.

use anyhow::Result;
use serde::Serialize;
use std::sync::OnceLock;
use tantivy::collector::TopDocs;
use tantivy::query::{BooleanQuery, BoostQuery, Query, QueryParser};
use tantivy::schema::{Field, STORED, Schema, TEXT};
use tantivy::{Index, IndexReader, ReloadPolicy, TantivyDocument};

use crate::behavior;

/// Multiplier applied to the name field when building the BM25 query. A hit
/// on `ta.rsi` as a name beats any number of prose mentions of "rsi" in
/// content bodies. 5x was chosen empirically: enough to surface the canonical
/// name entry first for exact-match queries without drowning out content
/// matches when the name token is absent.
const NAME_BOOST: f32 = 5.0;

/// One suggested identifier name, with its BM25 score.
#[derive(Debug, Clone, Serialize)]
pub struct Suggestion {
    pub name: String,
    pub score: f32,
}

struct Engine {
    // `index` is retained solely so `QueryParser::for_index(&e.index, ...)` can
    // be called cheaply inside `suggest()` without re-opening the index.
    index: Index,
    reader: IndexReader,
    name_field: Field,
    content_query_field: Field,
}

fn engine() -> &'static Engine {
    static ENGINE: OnceLock<Engine> = OnceLock::new();
    ENGINE.get_or_init(|| build().expect("BM25 index build must succeed"))
}

fn build() -> Result<Engine> {
    let mut schema_builder = Schema::builder();
    // `name` is stored (to retrieve the result name) and tokenised (for the
    // name-boosted ranking). `content_search` is tokenised only - it drives
    // content ranking but is never surfaced, since the name is the payload.
    let name_field = schema_builder.add_text_field("name", TEXT | STORED);
    let content_query_field = schema_builder.add_text_field("content_search", TEXT);
    let schema = schema_builder.build();

    let index = Index::create_in_ram(schema);
    let mut writer = index.writer(15_000_000)?;

    let add_doc = |name: &str, content: &str| -> Result<()> {
        let mut doc = TantivyDocument::default();
        doc.add_text(name_field, name);
        doc.add_text(content_query_field, content);
        writer.add_document(doc)?;
        Ok(())
    };

    // pine-data behavior exports - one doc per symbol (signatures, param prose,
    // examples, polymorphism notes, remarks / see-also). Names are distinct.
    for entry in behavior::search_entries() {
        add_doc(&entry.name, &entry.content)?;
    }

    writer.commit()?;

    let reader = index
        .reader_builder()
        .reload_policy(ReloadPolicy::Manual)
        .try_into()?;

    Ok(Engine {
        index,
        reader,
        name_field,
        content_query_field,
    })
}

/// The top `limit` identifier names closest to `q`, ranked by BM25 (name field
/// 5x boosted). Used for `po lookup`'s "did you mean ...?" miss path. An empty /
/// whitespace query returns an empty vec without building the index.
pub fn suggest(q: &str, limit: usize) -> Result<Vec<Suggestion>> {
    // tantivy's TopDocs panics on a zero limit.
    let q = &crate::query::plain_terms(q);
    if q.trim().is_empty() || limit == 0 {
        return Ok(Vec::new());
    }
    let e = engine();
    let searcher = e.reader.searcher();

    let name_parser = QueryParser::for_index(&e.index, vec![e.name_field]);
    let content_parser = QueryParser::for_index(&e.index, vec![e.content_query_field]);

    // Lenient: a mistyped Pine name (`strategy.exit(`, `a:b`) must still get
    // suggestions rather than a parse error.
    let (name_q, _) = name_parser.parse_query_lenient(q);
    let (content_q, _) = content_parser.parse_query_lenient(q);

    let boosted_name: Box<dyn Query> = Box::new(BoostQuery::new(name_q, NAME_BOOST));
    let scored: Box<dyn Query> = Box::new(BooleanQuery::union(vec![boosted_name, content_q]));

    let collector = TopDocs::with_limit(limit).order_by_score();
    let top = searcher.search(&scored, &collector)?;

    let mut out = Vec::with_capacity(top.len());
    for (score, addr) in top {
        let doc: TantivyDocument = searcher.doc(addr)?;
        if let Some(name) = first_text(&doc, e.name_field) {
            out.push(Suggestion { name, score });
        }
    }
    Ok(out)
}

fn first_text(doc: &TantivyDocument, field: Field) -> Option<String> {
    use tantivy::schema::Value;
    doc.get_first(field)
        .and_then(|v| v.as_str())
        .map(String::from)
}

/// Count of pine-data names indexed in the suggestion engine.
/// Computed once and cached; `po version` calls this on every invocation.
pub fn indexed_name_count() -> usize {
    static COUNT: OnceLock<usize> = OnceLock::new();
    *COUNT.get_or_init(|| behavior::search_entries().len())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rsi_suggests_ta_rsi_first() {
        let hits = suggest("rsi", 5).expect("suggest must succeed");
        assert!(!hits.is_empty(), "expected at least one suggestion for rsi");
        assert_eq!(
            hits[0].name,
            "ta.rsi",
            "ta.rsi should rank first, got {:?}",
            hits.iter().map(|h| &h.name).collect::<Vec<_>>()
        );
    }

    #[test]
    fn empty_query_returns_empty() {
        let hits = suggest("", 10).expect("empty query must not error");
        assert!(hits.is_empty());
    }

    #[test]
    fn math_max_is_suggested() {
        let hits = suggest("math max", 10).expect("suggest must succeed");
        assert!(
            hits.iter().any(|h| h.name == "math.max"),
            "math max must surface math.max, got {:?}",
            hits.iter().map(|h| &h.name).collect::<Vec<_>>()
        );
    }

    // No name may repeat - one doc per symbol.
    #[test]
    fn suggestions_are_distinct() {
        let hits = suggest("lower", 25).expect("suggest must succeed");
        let mut seen = std::collections::HashSet::new();
        for h in &hits {
            assert!(seen.insert(&h.name), "duplicate name: {}", h.name);
        }
    }

    // The 5x NAME_BOOST must seat the exact-name hit first. `ta.sma` is the
    // canonical entry whose name IS "ta.sma"; "sma" also appears in many bodies.
    #[test]
    fn name_boost_seats_exact_name_first() {
        let hits = suggest("sma", 10).expect("suggest must succeed");
        assert_eq!(
            hits[0].name,
            "ta.sma",
            "ta.sma (exact name match) must rank first; got {:?}",
            hits.iter().map(|h| &h.name).collect::<Vec<_>>()
        );
    }

    // A name reachable only via its rich body content (not the name token) must
    // still surface.
    #[test]
    fn body_only_match_surfaces() {
        let hits = suggest("polymorphic return", 25).expect("suggest must succeed");
        assert!(!hits.is_empty(), "expected hits for a body-flavored query");
    }
}
