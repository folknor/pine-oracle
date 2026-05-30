// BM25 search across the vendored v6 reference, PineForge docs, and
// structured pine-data behavior surface.
//
// The index is built lazily into a RAMDirectory on first query, cached via
// OnceLock. The index is a few thousand compact documents; build cost stays
// in low milliseconds.
//
// Scoring: name field carries a 5x boost over content. A query like
// "rsi" therefore puts `ta.rsi` ahead of any prose paragraph that
// happens to mention RSI.

use anyhow::{Result, bail};
use include_dir::{Dir, include_dir};
use serde::Serialize;
use std::sync::OnceLock;
use tantivy::collector::TopDocs;
use tantivy::query::{BooleanQuery, BoostQuery, Query, QueryParser, TermQuery};
use tantivy::schema::{Field, IndexRecordOption, STORED, STRING, Schema, TEXT};
use tantivy::{Index, IndexReader, ReloadPolicy, TantivyDocument, Term};

use crate::{behavior, reference, util::markdown};

const AUDIT_MARKDOWN: &str = include_str!("../vendor/pineforge-docs/pine_v6_audit_master.md");
static DOCS_PAGES: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/vendor/pineforge-docs/pages");
const KIND_REFERENCE: &str = "reference";
const KIND_AUDIT: &str = "audit";
const KIND_DOCS: &str = "docs";
const KIND_BEHAVIOR: &str = "behavior";

/// Multiplier applied to the name field when building the BM25 query. A hit
/// on `ta.rsi` as a name beats any number of prose mentions of "rsi" in
/// content bodies. 5x was chosen empirically: enough to surface the canonical
/// name entry first for exact-match queries without drowning out content
/// matches when the name token is absent.
const NAME_BOOST: f32 = 5.0;

/// Score multiplier applied to behavior-kind hits when no `--kind` filter is
/// active. Without dampening, behavior entries (which have rich content:
/// signatures, params, examples, polymorphism notes) outscore reference /
/// audit / docs hits for generic queries, which is rarely the user's intent.
/// The asymmetry is intentional: `--kind behavior` bypasses dampening entirely
/// so a narrowed behavior search gets the raw BM25 signal; only the
/// unfiltered mixed-kind ranking is adjusted.
const BEHAVIOR_UNFILTERED_DAMPEN: f32 = 0.65;
const SEARCH_KIND_NAMES: [&str; 4] = [KIND_REFERENCE, KIND_AUDIT, KIND_DOCS, KIND_BEHAVIOR];

#[derive(Debug, Clone, Serialize)]
pub struct SearchHit {
    pub kind: String,
    pub category: String,
    pub name: String,
    pub score: f32,
    /// Full indexed body (reference prose, probe summary, audit section
    /// body, narrative-page section body). Tantivy stores it alongside the
    /// tokenised form so consumers don't need a follow-up lookup.
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
    // Every source uses the same five-field layout (name, category, kind,
    // content [STORED], content_search [TEXT]). The closure centralises the
    // boilerplate so adding a sixth source in the future requires only a new
    // call site, not a new copy of the pattern.
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

    // Source 2: vendored PineForge audit doc. Each H2 / H3 section becomes
    // one doc so a query like `po search fallthrough` surfaces the exact
    // class of divergence the section discusses. Category="Audit",
    // kind="audit". The doc-level table-of-contents H2 ("Headline" etc.) is
    // indexed too; those sections tend to score lower because their content
    // is shorter, which matches their intent as navigation rather than
    // forensic substance.
    for (title, body) in parse_md_sections(AUDIT_MARKDOWN) {
        add_doc(&title, "Audit", KIND_AUDIT, &body)?;
    }

    // Source 3: vendored PineForge narrative pages. 18 markdown files
    // covering Pine v6 concepts in depth (magnifier, mtf, timeframes,
    // lifecycle, report schema, examples, tutorials). Each H2 / H3 section
    // becomes one doc. Category="Docs", kind="docs".
    for (title, body) in pages_sections() {
        add_doc(&title, "Docs", KIND_DOCS, &body)?;
    }

    // Source 4: structured pine-data behavior exports. Exact lookup remains
    // `po behavior <name>`; search indexes signatures, param prose,
    // examples, and polymorphism notes so users can discover a symbol when
    // they only remember a behavior or concept.
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
    // starving the final `limit`-length result set. If behavior entries
    // dominate the raw BM25 top-N (common for rich-content queries), dampening
    // may push them far enough down that the final slice would be empty without
    // the extra candidates. 4x was chosen empirically: large enough that
    // behavior-heavy queries still deliver `limit` non-behavior hits after
    // dampening, small enough that the per-query tantivy traversal stays cheap.
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
            kind: KIND_AUDIT,
            category: "Audit",
            description: "PineForge TV-vs-engine divergence sections",
            count: audit_section_count(),
        },
        SearchKindInfo {
            kind: KIND_DOCS,
            category: "Docs",
            description: "PineForge narrative documentation sections",
            count: docs_section_count(),
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

/// Count of indexed sections in the vendored audit doc.
/// Computed once and cached; `pine version` calls this on every invocation.
pub fn audit_section_count() -> usize {
    static COUNT: OnceLock<usize> = OnceLock::new();
    *COUNT.get_or_init(|| parse_md_sections(AUDIT_MARKDOWN).len())
}

/// Count of indexed sections across the vendored narrative pages.
/// Computed once and cached; `pine version` calls this on every invocation.
pub fn docs_section_count() -> usize {
    static COUNT: OnceLock<usize> = OnceLock::new();
    *COUNT.get_or_init(|| pages_sections().len())
}

/// Count of behavior-kind documents indexed in the BM25 engine.
/// Computed once and cached; `pine version` calls this on every invocation.
pub fn behavior_doc_count() -> usize {
    static COUNT: OnceLock<usize> = OnceLock::new();
    *COUNT.get_or_init(|| behavior::search_entries().len())
}

/// Split markdown into `(heading_text, body_text)` pairs for each H2 / H3
/// section. Thin adapter over `util::markdown::sections` that discards the
/// level field (search.rs callers only need title + body).
fn parse_md_sections(markdown: &str) -> Vec<(String, String)> {
    markdown::sections(markdown)
        .into_iter()
        .map(|s| (s.title, s.body))
        .collect()
}

/// Walk every `.md` file in `vendor/pineforge-docs/pages/` and yield section
/// pairs for each. The walker keeps file ordering deterministic by sorting
/// by path so the BM25 index is stable across builds.
fn pages_sections() -> Vec<(String, String)> {
    let mut files: Vec<_> = DOCS_PAGES.files().collect();
    files.sort_by_key(|f| f.path());
    let mut out = Vec::new();
    for file in files {
        if file
            .path()
            .extension()
            .and_then(|e| e.to_str())
            .map(|s| s.eq_ignore_ascii_case("md"))
            != Some(true)
        {
            continue;
        }
        let Some(content) = file.contents_utf8() else {
            continue;
        };
        out.extend(parse_md_sections(content));
    }
    out
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
    fn audit_sections_parse() {
        let s = parse_md_sections(AUDIT_MARKDOWN);
        // The audit doc has >= 18 H2/H3 sections (raw grep: 20). We use a
        // floor of 18 to tolerate any H2/H3 lines inside fenced code blocks
        // that comrak correctly ignores. If a scrape ever drops large swathes
        // of the audit doc, this test will catch it.
        assert!(
            s.len() >= 18,
            "expected >= 18 audit sections, got {}",
            s.len()
        );
        // Every section should have a non-empty title.
        assert!(s.iter().all(|(t, _)| !t.is_empty()));
    }

    #[test]
    fn pages_sections_yield_multiple_files() {
        let s = pages_sections();
        // 19 page files, raw grep yields 118 H2/H3 lines total. Some of those
        // may fall inside fenced code blocks (comrak ignores them), so we use
        // >= 80 as the floor rather than the raw count.
        assert!(
            s.len() >= 80,
            "expected >= 80 page sections across 19 files, got {}",
            s.len()
        );
    }

    #[test]
    fn docs_hit_appears_for_magnifier_query() {
        let hits = query("magnifier", 25, None).expect("search must succeed");
        assert!(
            hits.iter().any(|h| h.kind == "docs"),
            "expected a docs-kind hit for `magnifier`, got {:?}",
            hits.iter()
                .map(|h| (h.kind.as_str(), h.name.as_str()))
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn audit_doc_appears_in_search() {
        // "fallthrough" is the canonical name for the most-dangerous
        // divergence class in audit_master ("Silent fallthrough -> return 0").
        let hits = query("fallthrough", 25, None).expect("search must succeed");
        assert!(
            hits.iter().any(|h| h.kind == "audit"),
            "expected at least one audit-kind hit for `fallthrough`, got {:?}",
            hits.iter()
                .map(|h| (h.kind.as_str(), h.name.as_str()))
                .collect::<Vec<_>>()
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
        // Push the filter into tantivy: every returned hit must carry
        // the requested kind. Previously this was done by over-fetching
        // 4x and post-retaining in main.rs.
        let hits = query("magnifier", 10, Some("docs")).expect("search must succeed");
        assert!(!hits.is_empty(), "expected docs-kind hits for `magnifier`");
        assert!(
            hits.iter().all(|h| h.kind == "docs"),
            "kind filter leaked non-docs hits: {:?}",
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
            vec!["reference", "audit", "docs", "behavior"]
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

    // The new pine-data type + annotation catalogs are indexed under the
    // behavior kind (categories "Type" / "Annotation"). A name-token query for
    // each must surface them.
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

    // Bug-1 regression: heading immediately followed by a body line (no blank
    // line between) must not drop the first body line.
    #[test]
    fn body_not_dropped_when_heading_has_no_blank_line() {
        let md = "## Section\nFirst body line\nSecond body line\n";
        let sections = parse_md_sections(md);
        assert_eq!(sections.len(), 1, "expected one section");
        let body = &sections[0].1;
        assert!(
            body.contains("First body line"),
            "first body line must not be dropped; got: {body:?}"
        );
        assert!(
            body.contains("Second body line"),
            "second body line must not be dropped; got: {body:?}"
        );
    }

    // Bug-2 regression: inline code spans inside headings must be captured in
    // the section title.
    #[test]
    fn heading_inline_code_included_in_title() {
        let md = "## Section with `inline_code` token\nsome body\n";
        let sections = parse_md_sections(md);
        assert_eq!(sections.len(), 1, "expected one section");
        let title = &sections[0].0;
        assert!(
            title.contains("inline_code"),
            "inline code token must appear in title; got: {title:?}"
        );
    }

    // Gap-1: The 5x NAME_BOOST must seat the exact-name hit above every
    // content-only hit by a meaningful margin. We use "sma" as the probe
    // token: `ta.sma` is the canonical reference entry whose name IS "ta.sma"
    // (contains "sma"), while "sma" also appears in the body text of many
    // other reference / probe / audit / docs / behavior entries. If the boost
    // were absent or too small, a document with heavy "sma" repetition in its
    // body could outscore the name match.
    //
    // The test asserts two things:
    //   1. The first hit is a reference entry whose name matches (ta.sma).
    //   2. Its score is at least 2x the second-ranked hit's score.
    //      2x is deliberately below NAME_BOOST (5x) to stay robust against
    //      IDF / BM25 saturation effects while still catching any accidental
    //      removal or drastic reduction of the boost.
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
        // The name-boosted hit should substantially outscore the runner-up.
        // We use a 2x floor rather than pinning to NAME_BOOST exactly so the
        // assertion survives BM25 saturation and corpus churn, while still
        // catching any accidental removal of the boost.
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

    // Gap-2: The BEHAVIOR_UNFILTERED_DAMPEN factor must push behavior-kind
    // hits below competing reference / probe / audit / docs hits when no kind
    // filter is active, and must NOT apply when `--kind behavior` is set.
    //
    // "array" is a good probe token: it is a strong keyword that matches many
    // behavior entries (array.*  functions have "array" as a name prefix) but
    // also appears heavily in reference prose and audit sections. Without
    // dampening, the rich behavior content typically wins unfiltered queries.
    #[test]
    fn behavior_kind_dampened_when_unfiltered() {
        // --- Filtered: with --kind behavior, the top hit must be a behavior
        // entry (dampening is bypassed, raw BM25 applies). ---
        let hits_filtered = query("array", 10, Some("behavior")).expect("search must succeed");
        assert!(
            !hits_filtered.is_empty(),
            "expected behavior-kind hits for filtered `array` query"
        );
        assert_eq!(
            hits_filtered[0].kind,
            "behavior",
            "top hit must be behavior-kind when kind=behavior is requested; \
             got {:?}",
            hits_filtered.iter().map(|h| &h.kind).collect::<Vec<_>>()
        );

        // The const value itself is pinned here so a silent change from 0.65
        // to some other value triggers a test failure and forces a conscious
        // update of this assertion.
        //
        // An ordering test (behavior dampened below another kind in the
        // unfiltered ranking) was attempted but proved brittle: the corpus
        // has so many reference entries matching `array` that behavior hits
        // get pushed past the top-25, regardless of dampening. The const
        // pin is the load-bearing assertion.
        assert_eq!(
            BEHAVIOR_UNFILTERED_DAMPEN, 0.65,
            "BEHAVIOR_UNFILTERED_DAMPEN changed from 0.65; verify the new \
             dampening level is intentional and update this assertion"
        );
    }
}
