// BM25 search across the vendored v6 reference, baked PineForge corpus,
// PineForge docs, and structured pine-data behavior surface.
//
// The index is built lazily into a RAMDirectory on first query, cached via
// OnceLock. The current corpus is a few thousand compact documents; build
// cost stays in low milliseconds.
//
// Scoring: name field carries a 5x boost over content. A query like
// "rsi" therefore puts `ta.rsi` ahead of any prose paragraph that
// happens to mention RSI. Probes get indexed with their slug as `name` and
// their author-extracted summary (or slug-as-fallback when no summary is
// available) as `content`. A query like `pine search oca` surfaces both
// the reference's `oca_name=` parameter docs and the corpus's OCA probes.

use anyhow::{Result, bail};
use comrak::nodes::{AstNode, NodeValue};
use comrak::{Arena, Options, parse_document};
use include_dir::{Dir, include_dir};
use serde::Serialize;
use std::sync::OnceLock;
use tantivy::collector::TopDocs;
use tantivy::query::{BooleanQuery, BoostQuery, Query, QueryParser, TermQuery};
use tantivy::schema::{Field, IndexRecordOption, STORED, STRING, Schema, TEXT};
use tantivy::{Index, IndexReader, ReloadPolicy, TantivyDocument, Term};

use crate::{behavior, corpus, reference};

const AUDIT_MARKDOWN: &str = include_str!("../vendor/pineforge-docs/pine_v6_audit_master.md");
static DOCS_PAGES: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/vendor/pineforge-docs/pages");
const KIND_REFERENCE: &str = "reference";
const KIND_PROBE: &str = "probe";
const KIND_AUDIT: &str = "audit";
const KIND_DOCS: &str = "docs";
const KIND_BEHAVIOR: &str = "behavior";
const SEARCH_KIND_NAMES: [&str; 5] = [
    KIND_REFERENCE,
    KIND_PROBE,
    KIND_AUDIT,
    KIND_DOCS,
    KIND_BEHAVIOR,
];

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

    // Source 1: vendored v6 reference (941 entries).
    for entry in reference::all_entries() {
        let mut doc = TantivyDocument::default();
        doc.add_text(name_field, &entry.name);
        doc.add_text(category_field, &entry.category);
        doc.add_text(kind_field, KIND_REFERENCE);
        doc.add_text(content_field, &entry.content);
        doc.add_text(content_query_field, &entry.content);
        writer.add_document(doc)?;
    }

    // Source 2: baked PineForge corpus probes (239 entries). Probes without
    // an extractable header summary fall back to the slug as content so they
    // remain discoverable by their slug tokens (`oca`, `multi`, `bracket`,
    // ...). list_probes(None, None) is the same path `pine probes` uses.
    if let Ok(probes) = corpus::list_probes(None, None) {
        for p in probes {
            let mut doc = TantivyDocument::default();
            doc.add_text(name_field, &p.slug);
            doc.add_text(category_field, "Corpus");
            doc.add_text(kind_field, KIND_PROBE);
            let content = p.summary.unwrap_or(&p.slug);
            doc.add_text(content_field, content);
            doc.add_text(content_query_field, content);
            writer.add_document(doc)?;
        }
    }

    // Source 3: vendored PineForge audit doc. Each H2 / H3 section becomes
    // one doc so a query like `pine search fallthrough` surfaces the exact
    // class of divergence the section discusses. Category="Audit",
    // kind="audit". The doc-level table-of-contents H2 ("Headline" etc.) is
    // indexed too; those sections tend to score lower because their content
    // is shorter, which matches their intent as navigation rather than
    // forensic substance.
    for (title, body) in parse_md_sections(AUDIT_MARKDOWN) {
        let mut doc = TantivyDocument::default();
        doc.add_text(name_field, &title);
        doc.add_text(category_field, "Audit");
        doc.add_text(kind_field, KIND_AUDIT);
        doc.add_text(content_field, &body);
        doc.add_text(content_query_field, &body);
        writer.add_document(doc)?;
    }

    // Source 4: vendored PineForge narrative pages. 18 markdown files
    // covering Pine v6 concepts in depth (magnifier, mtf, timeframes,
    // lifecycle, report schema, examples, tutorials). Each H2 / H3 section
    // becomes one doc. Category="Docs", kind="docs".
    for (title, body) in pages_sections() {
        let mut doc = TantivyDocument::default();
        doc.add_text(name_field, &title);
        doc.add_text(category_field, "Docs");
        doc.add_text(kind_field, KIND_DOCS);
        doc.add_text(content_field, &body);
        doc.add_text(content_query_field, &body);
        writer.add_document(doc)?;
    }

    // Source 5: structured pine-data behavior exports. Exact lookup remains
    // `pine behavior <name>`; search indexes signatures, param prose,
    // examples, and polymorphism notes so users can discover a symbol when
    // they only remember a behavior or concept.
    for entry in behavior::search_entries() {
        let mut doc = TantivyDocument::default();
        doc.add_text(name_field, &entry.name);
        doc.add_text(category_field, entry.category);
        doc.add_text(kind_field, KIND_BEHAVIOR);
        doc.add_text(content_field, &entry.content);
        doc.add_text(content_query_field, &entry.content);
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
        kind_field,
        content_field,
        content_query_field,
    })
}

pub fn query(q: &str, limit: usize, kind_filter: Option<&str>) -> Result<Vec<SearchHit>> {
    if q.trim().is_empty() {
        return Ok(Vec::new());
    }
    let kind_filter = kind_filter.map(str::to_ascii_lowercase);
    let kind_filter = kind_filter.as_deref();
    if let Some(kind) = kind_filter {
        validate_kind(kind)?;
    }
    let e = engine();
    let searcher = e.reader.searcher();

    let name_parser = QueryParser::for_index(&e.index, vec![e.name_field]);
    let content_parser = QueryParser::for_index(&e.index, vec![e.content_query_field]);

    let name_q = name_parser.parse_query(q)?;
    let content_q = content_parser.parse_query(q)?;

    let boosted_name: Box<dyn Query> = Box::new(BoostQuery::new(name_q, 5.0));
    let scored: Box<dyn Query> = Box::new(BooleanQuery::union(vec![boosted_name, content_q]));

    // Push the kind filter down into tantivy as an AND clause so the
    // searcher returns exactly `limit` matching docs - no over-fetch +
    // post-filter dance (which could silently under-deliver when the
    // filtered kind is a small fraction of top-ranked hits).
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
            score * 0.65
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
            kind: KIND_PROBE,
            category: "Corpus",
            description: "Baked PineForge validation probes",
            count: corpus::list_probes(None, None).map_or(0, |items| items.len()),
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
/// library consumers that depend on the `pine_cli::search` surface don't
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

/// Count of indexed sections in the vendored audit doc. Cheap accessor for
/// `pine version`; re-parses the markdown each call (~sub-millisecond).
pub fn audit_section_count() -> usize {
    parse_md_sections(AUDIT_MARKDOWN).len()
}

/// Count of indexed sections across the vendored narrative pages.
pub fn docs_section_count() -> usize {
    pages_sections().len()
}

pub fn behavior_doc_count() -> usize {
    behavior::search_entries().len()
}

/// Split markdown into `(heading_text, body_text)` pairs for each H2 / H3
/// section. Body is everything from the heading line to (but not including)
/// the next heading at any level.
fn parse_md_sections(markdown: &str) -> Vec<(String, String)> {
    let arena = Arena::new();
    let opts = Options::default();
    let root = parse_document(&arena, markdown, &opts);
    let lines: Vec<&str> = markdown.lines().collect();

    // Depth-first walk that concatenates all Text and Code literals that are
    // descendants of `node`. Recurses into Emph, Strong, Link, etc. so that
    // inline markup inside headings (e.g. backtick code spans) is not dropped.
    fn inline_text<'a>(node: &'a AstNode<'a>) -> String {
        let mut out = String::new();
        for child in node.children() {
            match &child.data.borrow().value {
                NodeValue::Text(t) => out.push_str(t),
                NodeValue::Code(code) => out.push_str(&code.literal),
                _ => out.push_str(&inline_text(child)),
            }
        }
        out
    }

    // Each entry stores (title, level, heading_end_0) where heading_end_0 is
    // the 0-based index of the heading's last source line (comrak sourcepos is
    // 1-based, so heading_end_0 = sourcepos.end.line - 1). The body of a
    // section starts at heading_end_0 + 1. For single-line headings (the
    // normal case) heading_end_0 == sourcepos.start.line - 1.
    fn collect<'a>(node: &'a AstNode<'a>, out: &mut Vec<(String, u8, usize)>) {
        if let NodeValue::Heading(h) = &node.data.borrow().value
            && (h.level == 2 || h.level == 3)
        {
            let text = inline_text(node);
            let heading_end_0 = node.data.borrow().sourcepos.end.line - 1;
            out.push((text, h.level, heading_end_0));
        }
        for child in node.children() {
            collect(child, out);
        }
    }

    let mut headings = Vec::new();
    collect(root, &mut headings);

    let mut out = Vec::with_capacity(headings.len());
    for (i, (title, _, heading_end_0)) in headings.iter().enumerate() {
        // Body ends just before the next heading's first line. Since we store
        // heading_end_0 (0-based last line of the heading), for single-line
        // headings that equals the 0-based start line, which is exactly the
        // exclusive upper bound we need for the preceding section's body.
        let end = if i + 1 < headings.len() {
            headings[i + 1].2
        } else {
            lines.len()
        };
        // Body starts at the line immediately after the heading ends.
        let body: String = lines[heading_end_0 + 1..end]
            .to_vec()
            .join("\n")
            .trim()
            .to_string();
        out.push((title.trim().to_string(), body));
    }
    out
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
    fn corpus_probes_appear_in_search() {
        // `oca` is a strong slug token across multiple corpus probes; it
        // also appears in the v6 reference docs (function parameter
        // `oca_name`), so we expect a mix of kinds.
        let hits = query("oca", 25, None).expect("search must succeed");
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
        let hits = query("anomaly", 25, None).expect("search must succeed");
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

    #[test]
    fn audit_sections_parse() {
        let s = parse_md_sections(AUDIT_MARKDOWN);
        assert!(
            s.len() >= 5,
            "expected several audit sections, got {}",
            s.len()
        );
        // Every section should have a non-empty title.
        assert!(s.iter().all(|(t, _)| !t.is_empty()));
    }

    #[test]
    fn pages_sections_yield_multiple_files() {
        let s = pages_sections();
        assert!(
            s.len() >= 30,
            "expected dozens of page sections across 18 files, got {}",
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
            vec!["reference", "probe", "audit", "docs", "behavior"]
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
        // Pre-fix: `--kind probe --limit 25` over-fetched 100 hits, then
        // post-filtered; if probes were <25% of the top 100 for a popular
        // query the caller would silently get fewer than 25 hits.
        // Post-fix: tantivy filters during retrieval so the limit is honored
        // whenever the underlying index has enough matching docs.
        let hits = query("strategy", 20, Some("probe")).expect("search must succeed");
        assert!(
            hits.len() >= 15,
            "kind filter under-delivered: got {} probe hits for `strategy`, expected >=15",
            hits.len()
        );
        assert!(hits.iter().all(|h| h.kind == "probe"));
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
}
