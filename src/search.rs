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
use comrak::nodes::{AstNode, NodeValue};
use comrak::{Arena, Options, parse_document};
use include_dir::{Dir, include_dir};
use serde::Serialize;
use std::sync::OnceLock;
use tantivy::collector::TopDocs;
use tantivy::query::{BooleanQuery, BoostQuery, Query, QueryParser};
use tantivy::schema::{Field, STORED, STRING, Schema, TEXT};
use tantivy::{Index, IndexReader, ReloadPolicy, TantivyDocument};

use crate::{corpus, reference};

const AUDIT_MARKDOWN: &str = include_str!("../vendor/pineforge-docs/pine_v6_audit_master.md");
static DOCS_PAGES: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/vendor/pineforge-docs/pages");

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
        doc.add_text(kind_field, "reference");
        doc.add_text(content_field, &entry.content);
        doc.add_text(content_query_field, &entry.content);
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
        doc.add_text(kind_field, "audit");
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
        doc.add_text(kind_field, "docs");
        doc.add_text(content_field, &body);
        doc.add_text(content_query_field, &body);
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

pub fn query(q: &str, limit: usize) -> Result<Vec<SearchHit>> {
    if q.trim().is_empty() {
        return Ok(Vec::new());
    }
    let e = engine();
    let searcher = e.reader.searcher();

    let name_parser = QueryParser::for_index(&e.index, vec![e.name_field]);
    let content_parser = QueryParser::for_index(&e.index, vec![e.content_query_field]);

    let name_q = name_parser.parse_query(q)?;
    let content_q = content_parser.parse_query(q)?;

    let boosted_name: Box<dyn Query> = Box::new(BoostQuery::new(name_q, 5.0));
    let combined: Box<dyn Query> = Box::new(BooleanQuery::union(vec![boosted_name, content_q]));

    let collector = TopDocs::with_limit(limit).order_by_score();
    let top = searcher.search(&combined, &collector)?;

    let mut hits = Vec::with_capacity(top.len());
    for (score, addr) in top {
        let doc: TantivyDocument = searcher.doc(addr)?;
        let name = first_text(&doc, e.name_field).unwrap_or_default();
        let category = first_text(&doc, e.category_field).unwrap_or_default();
        let kind = first_text(&doc, e.kind_field).unwrap_or_default();
        let content = first_text(&doc, e.content_field).unwrap_or_default();
        hits.push(SearchHit {
            kind,
            category,
            name,
            score,
            content,
        });
    }
    Ok(hits)
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

/// Split markdown into `(heading_text, body_text)` pairs for each H2 / H3
/// section. Body is everything from the heading line to (but not including)
/// the next heading at any level.
fn parse_md_sections(markdown: &str) -> Vec<(String, String)> {
    let arena = Arena::new();
    let opts = Options::default();
    let root = parse_document(&arena, markdown, &opts);
    let lines: Vec<&str> = markdown.lines().collect();

    fn collect<'a>(node: &'a AstNode<'a>, out: &mut Vec<(String, u8, usize)>) {
        if let NodeValue::Heading(h) = &node.data.borrow().value
            && (h.level == 2 || h.level == 3)
        {
            let mut text = String::new();
            for child in node.children() {
                if let NodeValue::Text(t) = &child.data.borrow().value {
                    text.push_str(t);
                }
            }
            let start = node.data.borrow().sourcepos.start.line;
            out.push((text, h.level, start));
        }
        for child in node.children() {
            collect(child, out);
        }
    }

    let mut headings = Vec::new();
    collect(root, &mut headings);

    let mut out = Vec::with_capacity(headings.len());
    for (i, (title, _, start)) in headings.iter().enumerate() {
        let end = if i + 1 < headings.len() {
            headings[i + 1].2 - 1
        } else {
            lines.len()
        };
        let body: String = lines[*start..end]
            .iter()
            .skip(1)
            .copied()
            .collect::<Vec<_>>()
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
        let hits = query("magnifier", 25).expect("search must succeed");
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
        let hits = query("fallthrough", 25).expect("search must succeed");
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
        let hits = query("rsi", 3).expect("search must succeed");
        assert!(!hits.is_empty());
        let ta_rsi = hits.iter().find(|h| h.name == "ta.rsi").expect("ta.rsi");
        assert!(
            !ta_rsi.content.is_empty(),
            "ta.rsi hit must carry stored content"
        );
        assert!(ta_rsi.content.len() > 30);
    }
}
