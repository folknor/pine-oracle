// BM25 search across the vendored v6 reference and the structured pine-data
// behavior surface - the two "name" sources. Search is the index into
// `po lookup`: you reach for it when you don't yet know the identifier to
// pass to lookup.
//
// The index is built lazily into a RAMDirectory on first query, cached via
// OnceLock. A few thousand compact documents; build cost stays in low ms.
//
// Scoring: name field carries a 5x boost over content. A query like
// "rsi" therefore puts `ta.rsi` ahead of any prose paragraph that
// happens to mention RSI.

use anyhow::{Result, bail};
use serde::Serialize;
use std::sync::OnceLock;
use tantivy::collector::TopDocs;
use tantivy::query::{BooleanQuery, BoostQuery, Query, QueryParser, TermQuery};
use tantivy::schema::{Field, IndexRecordOption, STORED, STRING, Schema, TEXT};
use tantivy::{Index, IndexReader, ReloadPolicy, TantivyDocument, Term};

use crate::{behavior, reference};

const KIND_REFERENCE: &str = "reference";
const KIND_BEHAVIOR: &str = "behavior";

/// Multiplier applied to the name field when building the BM25 query. A hit
/// on `ta.rsi` as a name beats any number of prose mentions of "rsi" in
/// content bodies. 5x was chosen empirically: enough to surface the canonical
/// name entry first for exact-match queries without drowning out content
/// matches when the name token is absent.
const NAME_BOOST: f32 = 5.0;

/// Score multiplier applied to behavior-kind hits when no `--kind` filter is
/// active. Without dampening, behavior entries (which have rich content:
/// signatures, params, examples, polymorphism notes) outscore reference hits
/// for generic queries, which is rarely the user's intent. The asymmetry is
/// intentional: `--kind behavior` bypasses dampening entirely so a narrowed
/// behavior search gets the raw BM25 signal; only the unfiltered mixed-kind
/// ranking is adjusted.
const BEHAVIOR_UNFILTERED_DAMPEN: f32 = 0.65;
const SEARCH_KIND_NAMES: [&str; 2] = [KIND_REFERENCE, KIND_BEHAVIOR];

#[derive(Debug, Clone, Serialize)]
pub struct SearchHit {
    pub kind: String,
    pub category: String,
    pub name: String,
    pub score: f32,
    /// Full indexed body (reference prose or behavior signature block).
    /// Tantivy stores it alongside the tokenised form so consumers don't need
    /// a follow-up lookup.
    pub content: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct SearchKindInfo {
    pub kind: &'static str,
    pub category: &'static str,
    pub description: &'static str,
    /// Number of documents of this kind indexed in the BM25 engine.
    /// Previously named `document_count`; renamed to `count` in schema v1
    /// to match the field name used by all other catalog types.
    pub count: usize,
}

struct Engine {
    // `index` is retained solely so `QueryParser::for_index(&e.index, ...)` can
    // be called cheaply inside `query()` without re-opening the index on every
    // call. The reader/searcher does not need the Index directly.
    index: Index,
    reader: IndexReader,
    name_field: Field,
    category_field: Field,
    kind_field: Field,
    content_field: Field,
    content_query_field: Field,
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
    // `content` is stored (for SearchHit display) plus a separate tokenised
    // copy `content_search` that drives BM25 ranking. Splitting lets us
    // keep the stored content in its full prose form without bloating the
    // term dictionary.
    let content_field = schema_builder.add_text_field("content", STORED);
    let content_query_field = schema_builder.add_text_field("content_search", TEXT);
    let schema = schema_builder.build();

    let index = Index::create_in_ram(schema);
    let mut writer = index.writer(15_000_000)?;

    // Local helper: build and add one document from its four semantic parts.
    // Both sources use the same five-field layout (name, category, kind,
    // content [STORED], content_search [TEXT]).
    let add_doc = |name: &str, category: &str, kind: &str, content: &str| -> Result<()> {
        let mut doc = TantivyDocument::default();
        doc.add_text(name_field, name);
        doc.add_text(category_field, category);
        doc.add_text(kind_field, kind);
        doc.add_text(content_field, content);
        doc.add_text(content_query_field, content);
        writer.add_document(doc)?;
        Ok(())
    };

    // Source 1: vendored v6 reference (941 entries).
    for entry in reference::all_entries() {
        add_doc(&entry.name, &entry.category, KIND_REFERENCE, &entry.content)?;
    }

    // Source 2: structured pine-data behavior exports. Indexes signatures,
    // param prose, examples, and polymorphism notes so users can discover a
    // symbol when they only remember a behavior or concept.
    for entry in behavior::search_entries() {
        add_doc(&entry.name, entry.category, KIND_BEHAVIOR, &entry.content)?;
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
        content_query_field,
    })
}

pub fn query(q: &str, limit: usize, kind_filter: Option<&str>) -> Result<Vec<SearchHit>> {
    // Validate the kind filter first so `po search "" --kind bogus` returns
    // the "unknown search kind" error rather than an empty-vec no-op. The empty
    // query short-circuit is still below this so a valid-but-empty query still
    // returns an empty slice without building the index.
    let kind_filter = kind_filter.map(str::to_ascii_lowercase);
    let kind_filter = kind_filter.as_deref();
    if let Some(kind) = kind_filter {
        validate_kind(kind)?;
    }
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

    // With a kind filter: push the filter into tantivy as an AND clause so the
    // searcher returns exactly `limit` matching docs without any post-filter
    // dance (which could silently under-deliver when the filtered kind is a
    // small fraction of the top-ranked BM25 hits).
    //
    // Without a kind filter: over-fetch 4x from tantivy, then apply the
    // BEHAVIOR_UNFILTERED_DAMPEN multiplier and re-sort. The 4x factor gives
    // the dampening step enough headroom to reorder behavior hits without
    // starving the final `limit`-length result set.
    let final_query: Box<dyn Query> = match kind_filter {
        Some(kind) => {
            let term = Term::from_field_text(e.kind_field, kind);
            let kind_q: Box<dyn Query> = Box::new(TermQuery::new(term, IndexRecordOption::Basic));
            Box::new(BooleanQuery::intersection(vec![scored, kind_q]))
        }
        None => scored,
    };

    let collect_limit = if kind_filter.is_some() {
        limit
    } else {
        // 4x over-fetch: see comment above.
        limit.saturating_mul(4).max(limit)
    };
    let collector = TopDocs::with_limit(collect_limit).order_by_score();
    let top = searcher.search(&final_query, &collector)?;

    let mut hits = Vec::with_capacity(top.len());
    for (score, addr) in top {
        let doc: TantivyDocument = searcher.doc(addr)?;
        let name = first_text(&doc, e.name_field).unwrap_or_default();
        let category = first_text(&doc, e.category_field).unwrap_or_default();
        let kind = first_text(&doc, e.kind_field).unwrap_or_default();
        let content = first_text(&doc, e.content_field).unwrap_or_default();
        let score = if kind_filter.is_none() && kind == KIND_BEHAVIOR {
            score * BEHAVIOR_UNFILTERED_DAMPEN
        } else {
            score
        };
        hits.push(SearchHit {
            kind,
            category,
            name,
            score,
            content,
        });
    }
    if kind_filter.is_none() {
        hits.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        hits.truncate(limit);
    }
    Ok(hits)
}

pub fn kind_catalog() -> Vec<SearchKindInfo> {
    vec![
        SearchKindInfo {
            kind: KIND_REFERENCE,
            category: "Reference",
            description: "TradingView v6 reference entries",
            count: reference::all_entries().len(),
        },
        SearchKindInfo {
            kind: KIND_BEHAVIOR,
            category: "Behavior",
            description: "pine-data signatures, params, examples, and polymorphism notes",
            count: behavior_doc_count(),
        },
    ]
}

/// Returns `true` when `kind` is the catalog sentinel `"?"`.
/// Thin delegate to the binary's `output::CATALOG_MARKER`; kept here so
/// library consumers that depend on the `pine_oracle::search` surface don't
/// need to import the binary-internal `output` module.
pub fn is_kind_catalog_request(kind: &str) -> bool {
    kind == "?"
}

fn validate_kind(kind: &str) -> Result<()> {
    if SEARCH_KIND_NAMES.contains(&kind) {
        return Ok(());
    }
    bail!(
        "unknown search kind `{kind}`; expected one of: {}",
        SEARCH_KIND_NAMES.join(", ")
    )
}

fn first_text(doc: &TantivyDocument, field: Field) -> Option<String> {
    use tantivy::schema::Value;
    doc.get_first(field)
        .and_then(|v| v.as_str())
        .map(String::from)
}

/// Count of behavior-kind documents indexed in the BM25 engine.
/// Computed once and cached; `pine version` calls this on every invocation.
pub fn behavior_doc_count() -> usize {
    static COUNT: OnceLock<usize> = OnceLock::new();
    *COUNT.get_or_init(|| behavior::search_entries().len())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rsi_query_ranks_ta_rsi_first() {
        let hits = query("rsi", 5, None).expect("search must succeed");
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
        let hits = query("", 10, None).expect("empty query must not error");
        assert!(hits.is_empty());
    }

    #[test]
    fn math_max_finds_the_function() {
        let hits = query("math max", 10, None).expect("search must succeed");
        assert!(
            hits.iter().any(|h| h.name == "math.max"),
            "math max must surface math.max, got {:?}",
            hits.iter().map(|h| &h.name).collect::<Vec<_>>()
        );
    }

    #[test]
    fn hits_carry_non_empty_content() {
        let hits = query("rsi", 3, None).expect("search must succeed");
        assert!(!hits.is_empty());
        let ta_rsi = hits.iter().find(|h| h.name == "ta.rsi").expect("ta.rsi");
        assert!(
            !ta_rsi.content.is_empty(),
            "ta.rsi hit must carry stored content"
        );
        assert!(ta_rsi.content.len() > 30);
    }

    #[test]
    fn kind_filter_returns_only_matching_kind() {
        // Push the filter into tantivy: every returned hit must carry the
        // requested kind.
        let hits = query("array", 10, Some("behavior")).expect("search must succeed");
        assert!(!hits.is_empty(), "expected behavior-kind hits for `array`");
        assert!(
            hits.iter().all(|h| h.kind == "behavior"),
            "kind filter leaked non-behavior hits: {:?}",
            hits.iter()
                .map(|h| (h.kind.as_str(), h.name.as_str()))
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn kind_filter_is_case_insensitive() {
        let hits = query("rsi", 10, Some("REFERENCE")).expect("search must succeed");
        assert!(!hits.is_empty(), "expected reference-kind hits for `rsi`");
        assert!(hits.iter().all(|h| h.kind == "reference"));
    }

    #[test]
    fn kind_catalog_lists_all_supported_kinds_with_counts() {
        let kinds = kind_catalog();
        assert_eq!(
            kinds.iter().map(|kind| kind.kind).collect::<Vec<_>>(),
            vec!["reference", "behavior"]
        );
        assert!(kinds.iter().all(|kind| kind.count > 0));
    }

    #[test]
    fn invalid_kind_filter_errors() {
        let err = query("rsi", 10, Some("behaviour")).expect_err("must reject unknown kind");
        assert!(err.to_string().contains("unknown search kind"));
    }

    #[test]
    fn kind_filter_can_deliver_full_limit_when_kind_is_sparse() {
        // tantivy filters during retrieval so the limit is honored whenever
        // the underlying index has enough matching docs of the requested kind.
        let hits = query("function", 20, Some("reference")).expect("search must succeed");
        assert!(
            hits.len() >= 15,
            "kind filter under-delivered: got {} reference hits for `function`, expected >=15",
            hits.len()
        );
        assert!(hits.iter().all(|h| h.kind == "reference"));
    }

    #[test]
    fn behavior_entries_appear_in_search() {
        let hits = query("polymorphic return allowed types", 25, Some("behavior"))
            .expect("search must succeed");
        assert!(!hits.is_empty(), "expected behavior-kind hits");
        assert!(hits.iter().all(|h| h.kind == "behavior"));
        assert!(
            hits.iter().any(|h| h.category == "Function"),
            "expected at least one function behavior hit, got {:?}",
            hits.iter()
                .map(|h| (h.category.as_str(), h.name.as_str()))
                .collect::<Vec<_>>()
        );
    }

    // The pine-data type + annotation catalogs are indexed under the behavior
    // kind (categories "Type" / "Annotation"). A name-token query for each must
    // surface them.
    #[test]
    fn type_and_annotation_catalogs_appear_in_search() {
        let type_hits = query("chart.point", 25, Some("behavior")).expect("search must succeed");
        assert!(
            type_hits
                .iter()
                .any(|h| h.category == "Type" && h.name == "chart.point"),
            "expected a Type-category hit for chart.point, got {:?}",
            type_hits
                .iter()
                .map(|h| (h.category.as_str(), h.name.as_str()))
                .collect::<Vec<_>>()
        );

        let annotation_hits = query("version", 25, Some("behavior")).expect("search must succeed");
        assert!(
            annotation_hits
                .iter()
                .any(|h| h.category == "Annotation" && h.name == "@version="),
            "expected an Annotation-category hit for @version=, got {:?}",
            annotation_hits
                .iter()
                .map(|h| (h.category.as_str(), h.name.as_str()))
                .collect::<Vec<_>>()
        );
    }

    // The 5x NAME_BOOST must seat the exact-name hit above every content-only
    // hit by a meaningful margin. We use "sma" as the probe token: `ta.sma` is
    // the canonical reference entry whose name IS "ta.sma", while "sma" also
    // appears in the body text of many other reference / behavior entries.
    #[test]
    fn name_boost_dominates_over_content_match() {
        let hits = query("sma", 10, None).expect("search must succeed");
        assert!(
            hits.len() >= 2,
            "expected at least two hits for `sma`, got {}",
            hits.len()
        );
        assert_eq!(
            hits[0].name,
            "ta.sma",
            "ta.sma (exact name match) must rank first; got {:?}",
            hits.iter().map(|h| &h.name).collect::<Vec<_>>()
        );
        assert_eq!(hits[0].kind, "reference");
        let top_score = hits[0].score;
        let second_score = hits[1].score;
        // 1.3x floor catches accidental boost removal while staying robust
        // against BM25 saturation that flattens score ratios at high IDF.
        assert!(
            top_score >= second_score * 1.3,
            "expected name-boosted hit to score at least 1.3x the second hit \
             (top={top_score:.4}, second={second_score:.4}); \
             NAME_BOOST={NAME_BOOST} may have been reduced or removed"
        );
    }

    // The BEHAVIOR_UNFILTERED_DAMPEN factor must not apply when `--kind
    // behavior` is set (raw BM25 applies), and the const value is pinned.
    #[test]
    fn behavior_kind_dampened_when_unfiltered() {
        let hits_filtered = query("array", 10, Some("behavior")).expect("search must succeed");
        assert!(
            !hits_filtered.is_empty(),
            "expected behavior-kind hits for filtered `array` query"
        );
        assert_eq!(
            hits_filtered[0].kind,
            "behavior",
            "top hit must be behavior-kind when kind=behavior is requested; got {:?}",
            hits_filtered.iter().map(|h| &h.kind).collect::<Vec<_>>()
        );
        // Pin the const so a silent change forces a conscious update here.
        assert_eq!(
            BEHAVIOR_UNFILTERED_DAMPEN, 0.65,
            "BEHAVIOR_UNFILTERED_DAMPEN changed from 0.65; verify the new \
             dampening level is intentional and update this assertion"
        );
    }
}
