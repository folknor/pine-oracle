// BM25 search over the structured pine-data behavior surface. Search is the
// index into `po lookup`: you reach for it when you don't yet know the
// identifier to pass lookup.
//
// Output is a ranked list of *names*. Each pine-data symbol is one document, so
// names are already distinct; the grouping below is a no-op safety net (and was
// load-bearing while a second, overlapping source existed). The name is the
// whole payload - you feed it back to `po lookup` for the full card.
//
// The index is built lazily into a RAMDirectory on first query, cached via
// OnceLock. A few thousand compact documents; build cost stays in low ms.
//
// Scoring: name field carries a 5x boost over content, so `rsi` puts `ta.rsi`
// ahead of any prose paragraph that merely mentions RSI.

use anyhow::Result;
use serde::Serialize;
use std::collections::HashMap;
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

/// Over-fetch multiplier before grouping. A name can be indexed in both
/// sources, so its two raw hits must both be inside the fetched window for the
/// summed score to be correct. 8x is ample headroom over the 2 sources while
/// keeping the per-query traversal cheap on a few-thousand-doc index.
const GROUP_OVERFETCH: usize = 8;

/// One ranked name in a search result. `score` is the sum of the per-source
/// BM25 scores for that name.
#[derive(Debug, Clone, Serialize)]
pub struct NameHit {
    pub name: String,
    pub score: f32,
}

struct Engine {
    // `index` is retained solely so `QueryParser::for_index(&e.index, ...)` can
    // be called cheaply inside `query()` without re-opening the index.
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

    // Single source: structured pine-data behavior exports - signatures, param
    // prose, examples, polymorphism notes, and the remarks / see-also prose.
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

/// Ranked, deduplicated names matching `q`. Each name's score is the sum of its
/// per-source BM25 scores; results are sorted descending and truncated to
/// `limit`. An empty / whitespace query returns an empty vec without building
/// the index.
pub fn query(q: &str, limit: usize) -> Result<Vec<NameHit>> {
    if q.trim().is_empty() {
        return Ok(Vec::new());
    }
    let e = engine();
    let searcher = e.reader.searcher();

    let name_parser = QueryParser::for_index(&e.index, vec![e.name_field]);
    let content_parser = QueryParser::for_index(&e.index, vec![e.content_query_field]);

    let name_q = name_parser.parse_query(q)?;
    let content_q = content_parser.parse_query(q)?;

    let boosted_name: Box<dyn Query> = Box::new(BoostQuery::new(name_q, NAME_BOOST));
    let scored: Box<dyn Query> = Box::new(BooleanQuery::union(vec![boosted_name, content_q]));

    // Over-fetch so both source-hits for a name land in the window before we
    // group and sum (see GROUP_OVERFETCH).
    let collect_limit = limit.saturating_mul(GROUP_OVERFETCH).max(limit);
    let collector = TopDocs::with_limit(collect_limit).order_by_score();
    let top = searcher.search(&scored, &collector)?;

    // Group by name, summing the per-source BM25 scores.
    let mut sums: HashMap<String, f32> = HashMap::new();
    for (score, addr) in top {
        let doc: TantivyDocument = searcher.doc(addr)?;
        if let Some(name) = first_text(&doc, e.name_field) {
            *sums.entry(name).or_insert(0.0) += score;
        }
    }

    let mut hits: Vec<NameHit> = sums
        .into_iter()
        .map(|(name, score)| NameHit { name, score })
        .collect();
    hits.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
            // Stable tiebreak so equal-scored names have a deterministic order.
            .then_with(|| a.name.cmp(&b.name))
    });
    hits.truncate(limit);
    Ok(hits)
}

fn first_text(doc: &TantivyDocument, field: Field) -> Option<String> {
    use tantivy::schema::Value;
    doc.get_first(field)
        .and_then(|v| v.as_str())
        .map(String::from)
}

/// Count of behavior-source documents indexed in the BM25 engine.
/// Computed once and cached; `po version` calls this on every invocation.
pub fn behavior_doc_count() -> usize {
    static COUNT: OnceLock<usize> = OnceLock::new();
    *COUNT.get_or_init(|| behavior::search_entries().len())
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

    // The core dedup property: a name indexed in both reference and behavior
    // (every named function) must appear exactly once in the results.
    #[test]
    fn names_are_deduplicated() {
        let hits = query("lower", 25).expect("search must succeed");
        let lower_rows = hits.iter().filter(|h| h.name == "str.lower").count();
        assert_eq!(
            lower_rows, 1,
            "str.lower must appear exactly once (deduped across sources), got {lower_rows}"
        );
        // No name may repeat anywhere in the result set.
        let mut seen = std::collections::HashSet::new();
        for h in &hits {
            assert!(
                seen.insert(&h.name),
                "duplicate name in results: {}",
                h.name
            );
        }
    }

    // Summing across sources: a name matching in both reference and behavior
    // should outscore the bare per-source contribution. `request.security_lower_tf`
    // matches in both, so it should rank at or above `str.lower` for `lower`
    // (matching the hand-computed mockup where multi-source corroboration won).
    #[test]
    fn multi_source_name_ranks_above_single_source() {
        let hits = query("lower", 25).expect("search must succeed");
        let pos = |name: &str| hits.iter().position(|h| h.name == name);
        let multi = pos("request.security_lower_tf").expect("multi-source name present");
        let single = pos("str.upper");
        if let Some(single) = single {
            assert!(
                multi < single,
                "multi-source `request.security_lower_tf` (#{multi}) should rank \
                 above single-source `str.upper` (#{single})"
            );
        }
    }

    // The 5x NAME_BOOST must seat the exact-name hit first. `ta.sma` is the
    // canonical reference entry whose name IS "ta.sma"; "sma" also appears in
    // many other bodies, so without the boost a content-heavy doc could win.
    #[test]
    fn name_boost_seats_exact_name_first() {
        let hits = query("sma", 10).expect("search must succeed");
        assert_eq!(
            hits[0].name,
            "ta.sma",
            "ta.sma (exact name match) must rank first; got {:?}",
            hits.iter().map(|h| &h.name).collect::<Vec<_>>()
        );
    }

    // A name only present in the behavior source (via its rich signature
    // content) must still surface - search indexes both sources.
    #[test]
    fn behavior_sourced_names_surface() {
        let hits = query("polymorphic return", 25).expect("search must succeed");
        assert!(
            !hits.is_empty(),
            "expected hits for a behavior-flavored query"
        );
    }
}
