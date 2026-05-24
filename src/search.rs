// BM25 search across the vendored v6 reference and the baked PineForge
// corpus.
//
// The index is built lazily into a RAMDirectory on first query, cached via
// OnceLock. ~941 reference entries + ~235 corpus probes = ~1.18k documents;
// build cost is single-digit milliseconds.
//
// Scoring: name field carries a 5x boost over content. A query like
// "rsi" therefore puts `ta.rsi` ahead of any prose paragraph that
// happens to mention RSI. Probes get indexed with their slug as `name` and
// their author-extracted summary (or slug-as-fallback when no summary is
// available) as `content`. A query like `pine search oca` surfaces both
// the reference's `oca_name=` parameter docs and the corpus's OCA probes.

use anyhow::Result;
use serde::Serialize;
use std::sync::OnceLock;
use tantivy::collector::TopDocs;
use tantivy::query::{BooleanQuery, BoostQuery, Query, QueryParser};
use tantivy::schema::{Field, Schema, STORED, STRING, TEXT};
use tantivy::{Index, IndexReader, ReloadPolicy, TantivyDocument};

use crate::{corpus, reference};

#[derive(Debug, Clone, Serialize)]
pub struct SearchHit {
    pub kind: String,
    pub category: String,
    pub name: String,
    pub score: f32,
}

struct Engine {
    index: Index,
    reader: IndexReader,
    name_field: Field,
    category_field: Field,
    kind_field: Field,
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
    let kind_field = schema_builder.add_text_field("kind", STRING | STORED);
    let content_field = schema_builder.add_text_field("content", TEXT);
    let schema = schema_builder.build();

    let index = Index::create_in_ram(schema);
    let mut writer = index.writer(15_000_000)?;

    // Source 1: vendored v6 reference (941 entries).
    for entry in reference::all_entries() {
        let mut doc = TantivyDocument::default();
        doc.add_text(name_field, &entry.name);
        doc.add_text(category_field, &entry.category);
        doc.add_text(kind_field, "reference");
        doc.add_text(content_field, &entry.content);
        writer.add_document(doc)?;
    }

    // Source 2: baked PineForge corpus probes (235 entries). Probes without
    // an extractable header summary fall back to the slug as content so they
    // remain discoverable by their slug tokens (`oca`, `multi`, `bracket`,
    // ...). list_probes(None) is the same path `pine probes` uses.
    if let Ok(probes) = corpus::list_probes(None) {
        for p in probes {
            let mut doc = TantivyDocument::default();
            doc.add_text(name_field, &p.slug);
            doc.add_text(category_field, "Corpus");
            doc.add_text(kind_field, "probe");
            let content = p.summary.unwrap_or(&p.slug);
            doc.add_text(content_field, content);
            writer.add_document(doc)?;
        }
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
        kind_field,
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
        let kind = first_text(&doc, e.kind_field).unwrap_or_default();
        hits.push(SearchHit {
            kind,
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
        assert_eq!(hits[0].kind, "reference");
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

    #[test]
    fn corpus_probes_appear_in_search() {
        // `oca` is a strong slug token across multiple corpus probes; it
        // also appears in the v6 reference docs (function parameter
        // `oca_name`), so we expect a mix of kinds.
        let hits = query("oca", 25).expect("search must succeed");
        assert!(!hits.is_empty());
        assert!(
            hits.iter().any(|h| h.kind == "probe"),
            "at least one probe-kind hit expected for `oca`, got {:?}",
            hits.iter()
                .map(|h| (h.kind.as_str(), h.name.as_str()))
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn probe_search_returns_corpus_category() {
        let hits = query("anomaly", 25).expect("search must succeed");
        let probe_hit = hits.iter().find(|h| h.kind == "probe");
        assert!(
            probe_hit.is_some(),
            "expected at least one anomaly-related probe in search results, got {:?}",
            hits.iter()
                .map(|h| (h.kind.as_str(), h.name.as_str()))
                .collect::<Vec<_>>()
        );
        assert_eq!(probe_hit.unwrap().category, "Corpus");
    }
}
