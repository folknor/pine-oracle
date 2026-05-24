// BM25 search over the vendored v6 reference.
//
// The index is built lazily into a RAMDirectory on first query, cached via
// OnceLock. ~941 documents; build cost is single-digit milliseconds.
//
// Scoring: name field carries a 5x boost over content. A query like
// "rsi" therefore puts `ta.rsi` ahead of any prose paragraph that
// happens to mention RSI.

use anyhow::Result;
use serde::Serialize;
use std::sync::OnceLock;
use tantivy::collector::TopDocs;
use tantivy::query::{BooleanQuery, BoostQuery, Query, QueryParser};
use tantivy::schema::{Field, Schema, STORED, STRING, TEXT};
use tantivy::{Index, IndexReader, ReloadPolicy, TantivyDocument};

use crate::reference;

#[derive(Debug, Clone, Serialize)]
pub struct SearchHit {
    pub category: String,
    pub name: String,
    pub score: f32,
}

struct Engine {
    index: Index,
    reader: IndexReader,
    name_field: Field,
    category_field: Field,
    content_field: Field,
}

fn engine() -> &'static Engine {
    static ENGINE: OnceLock<Engine> = OnceLock::new();
    ENGINE.get_or_init(|| build().expect("BM25 index build must succeed"))
}

fn build() -> Result<Engine> {
    let mut schema_builder = Schema::builder();
    let name_field = schema_builder.add_text_field("name", TEXT | STORED);
    let category_field = schema_builder.add_text_field("category", STRING | STORED);
    let content_field = schema_builder.add_text_field("content", TEXT);
    let schema = schema_builder.build();

    let index = Index::create_in_ram(schema);
    let mut writer = index.writer(15_000_000)?;

    for entry in reference::all_entries() {
        let mut doc = TantivyDocument::default();
        doc.add_text(name_field, &entry.name);
        doc.add_text(category_field, &entry.category);
        doc.add_text(content_field, &entry.content);
        writer.add_document(doc)?;
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
        category_field,
        content_field,
    })
}

pub fn query(q: &str, limit: usize) -> Result<Vec<SearchHit>> {
    if q.trim().is_empty() {
        return Ok(Vec::new());
    }
    let e = engine();
    let searcher = e.reader.searcher();

    let name_parser = QueryParser::for_index(&e.index, vec![e.name_field]);
    let content_parser = QueryParser::for_index(&e.index, vec![e.content_field]);

    let name_q = name_parser.parse_query(q)?;
    let content_q = content_parser.parse_query(q)?;

    let combined: Box<dyn Query> = Box::new(BooleanQuery::union(vec![
        Box::new(BoostQuery::new(name_q, 5.0)) as Box<dyn Query>,
        content_q,
    ]));

    let top = searcher.search(&combined, &TopDocs::with_limit(limit))?;

    let mut hits = Vec::with_capacity(top.len());
    for (score, addr) in top {
        let doc: TantivyDocument = searcher.doc(addr)?;
        let name = first_text(&doc, e.name_field).unwrap_or_default();
        let category = first_text(&doc, e.category_field).unwrap_or_default();
        hits.push(SearchHit {
            category,
            name,
            score,
        });
    }
    Ok(hits)
}

fn first_text(doc: &TantivyDocument, field: Field) -> Option<String> {
    use tantivy::schema::Value;
    doc.get_first(field)
        .and_then(|v| v.as_str().map(|s| s.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rsi_query_ranks_ta_rsi_first() {
        let hits = query("rsi", 5).expect("search must succeed");
        assert!(!hits.is_empty(), "expected at least one hit for rsi");
        assert_eq!(
            hits[0].name,
            "ta.rsi",
            "ta.rsi should rank first, got {:?}",
            hits.iter().map(|h| &h.name).collect::<Vec<_>>()
        );
    }

    #[test]
    fn empty_query_returns_empty() {
        let hits = query("", 10).expect("empty query must not error");
        assert!(hits.is_empty());
    }

    #[test]
    fn math_max_finds_the_function() {
        let hits = query("math max", 10).expect("search must succeed");
        assert!(
            hits.iter().any(|h| h.name == "math.max"),
            "math max must surface math.max, got {:?}",
            hits.iter().map(|h| &h.name).collect::<Vec<_>>()
        );
    }
}
