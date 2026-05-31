// Pine User Manual prose search. The vendored manual (vendor/pine-manual/v6, a
// per-page markdown tree scraped from TradingView's docs) is embedded and split
// into sections by its `## Heading {#anchor}` boundaries. Each section is one
// BM25 document keyed `page#anchor` - the same string is the index key, the
// result provenance, and (joined with the page's `source` URL) a real clickable
// link.
//
// This is the "how does X work" half of the oracle, complementing `po lookup`'s
// "what is X". `po search` queries it; the section markdown is rendered to the
// terminal by `crate::render`.

use anyhow::Result;
use include_dir::{Dir, include_dir};
use serde::Serialize;
use std::collections::BTreeSet;
use std::sync::OnceLock;
use tantivy::collector::TopDocs;
use tantivy::query::{BooleanQuery, BoostQuery, Query, QueryParser};
use tantivy::schema::{Field, STORED, STRING, Schema, TEXT, Value};
use tantivy::{Index, IndexReader, ReloadPolicy, TantivyDocument};

static MANUAL: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/vendor/pine-manual/v6");

/// Multiplier on the section-title field so a query matching a heading outranks
/// one that only matches body prose.
const TITLE_BOOST: f32 = 4.0;

/// One manual section: an H1/H2 heading and the prose beneath it (down to the
/// next H1/H2), addressed by `page#anchor`.
#[derive(Debug, Clone, Serialize)]
pub struct Section {
    /// URL-path page id, e.g. `language/operators`.
    pub page: String,
    /// Heading anchor / URL fragment, e.g. `operator-precedence`.
    pub anchor: String,
    /// Heading text.
    pub title: String,
    /// Canonical TradingView URL (`source` + `#anchor`).
    pub url: String,
    /// Section markdown, heading line included.
    pub body: String,
}

/// A search result: a section plus its BM25 score.
#[derive(Debug, Clone, Serialize)]
pub struct SectionHit {
    #[serde(flatten)]
    pub section: Section,
    pub score: f32,
}

/// All parsed sections across every manual page, in stable (page, document)
/// order. Built once, cached.
pub fn sections() -> &'static [Section] {
    static SECTIONS: OnceLock<Vec<Section>> = OnceLock::new();
    SECTIONS.get_or_init(build_sections).as_slice()
}

/// Number of distinct manual pages indexed.
pub fn page_count() -> usize {
    sections()
        .iter()
        .map(|s| s.page.as_str())
        .collect::<BTreeSet<_>>()
        .len()
}

/// Number of indexed sections.
pub fn section_count() -> usize {
    sections().len()
}

/// Exact section by `page` + `anchor` (case-sensitive on both).
pub fn get_section(page: &str, anchor: &str) -> Option<&'static Section> {
    sections()
        .iter()
        .find(|s| s.page == page && s.anchor == anchor)
}

/// The whole page as markdown: every section of `page` joined in order. `None`
/// if the page id is unknown.
pub fn get_page(page: &str) -> Option<String> {
    let bodies: Vec<&str> = sections()
        .iter()
        .filter(|s| s.page == page)
        .map(|s| s.body.as_str())
        .collect();
    if bodies.is_empty() {
        None
    } else {
        Some(bodies.join("\n\n"))
    }
}

/// Ranked sections matching `q` (BM25, title 4x boosted). Empty query returns
/// an empty vec without building the index.
pub fn search(q: &str, limit: usize) -> Result<Vec<SectionHit>> {
    if q.trim().is_empty() {
        return Ok(Vec::new());
    }
    let e = engine();
    let searcher = e.reader.searcher();

    let title_parser = QueryParser::for_index(&e.index, vec![e.title_field]);
    let content_parser = QueryParser::for_index(&e.index, vec![e.content_field]);
    let title_q = title_parser.parse_query(q)?;
    let content_q = content_parser.parse_query(q)?;
    let boosted: Box<dyn Query> = Box::new(BoostQuery::new(title_q, TITLE_BOOST));
    let query: Box<dyn Query> = Box::new(BooleanQuery::union(vec![boosted, content_q]));

    let collector = TopDocs::with_limit(limit).order_by_score();
    let top = searcher.search(&query, &collector)?;
    let mut hits = Vec::with_capacity(top.len());
    for (score, addr) in top {
        let doc: TantivyDocument = searcher.doc(addr)?;
        let page = stored(&doc, e.page_field);
        let anchor = stored(&doc, e.anchor_field);
        if let Some(section) = get_section(&page, &anchor) {
            hits.push(SectionHit {
                section: section.clone(),
                score,
            });
        }
    }
    Ok(hits)
}

// ---------- index ----------

struct Engine {
    index: Index,
    reader: IndexReader,
    title_field: Field,
    content_field: Field,
    page_field: Field,
    anchor_field: Field,
}

fn engine() -> &'static Engine {
    static ENGINE: OnceLock<Engine> = OnceLock::new();
    ENGINE.get_or_init(|| build_index().expect("manual BM25 index build must succeed"))
}

fn build_index() -> Result<Engine> {
    let mut schema_builder = Schema::builder();
    let title_field = schema_builder.add_text_field("title", TEXT);
    let content_field = schema_builder.add_text_field("content", TEXT);
    // page + anchor are stored only; they re-key the hit back to its Section.
    let page_field = schema_builder.add_text_field("page", STRING | STORED);
    let anchor_field = schema_builder.add_text_field("anchor", STRING | STORED);
    let schema = schema_builder.build();

    let index = Index::create_in_ram(schema);
    let mut writer = index.writer(15_000_000)?;
    for s in sections() {
        let mut doc = TantivyDocument::default();
        doc.add_text(title_field, &s.title);
        doc.add_text(content_field, &s.body);
        doc.add_text(page_field, &s.page);
        doc.add_text(anchor_field, &s.anchor);
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
        title_field,
        content_field,
        page_field,
        anchor_field,
    })
}

fn stored(doc: &TantivyDocument, field: Field) -> String {
    doc.get_first(field)
        .and_then(|v| v.as_str())
        .map(String::from)
        .unwrap_or_default()
}

// ---------- parsing ----------

fn build_sections() -> Vec<Section> {
    let mut files: Vec<_> = MANUAL
        .find("**/*.md")
        .into_iter()
        .flatten()
        .filter_map(|entry| entry.as_file())
        .collect();
    files.sort_by_key(|f| f.path().to_path_buf());
    let mut out = Vec::new();
    for file in files {
        let Some(path) = file.path().to_str() else {
            continue;
        };
        let Some(text) = file.contents_utf8() else {
            continue;
        };
        // page id = path minus the `.md` suffix (e.g. `language/operators`).
        let page = path.strip_suffix(".md").unwrap_or(path);
        out.extend(parse_page(page, text));
    }
    out
}

/// Parse one page into sections. Pages without YAML frontmatter (e.g. the nav
/// `README.md`) are skipped - they carry no `source` URL and no anchored
/// headings.
fn parse_page(page: &str, text: &str) -> Vec<Section> {
    let Some((front, body)) = split_frontmatter(text) else {
        return Vec::new();
    };
    let Some(source) = front_value(front, "source") else {
        return Vec::new();
    };

    let mut sections = Vec::new();
    let mut current: Option<(String, String, Vec<&str>)> = None;
    for line in body.lines() {
        if let Some((anchor, title)) = heading_boundary(line) {
            if let Some((a, t, lines)) = current.take() {
                sections.push(make_section(page, source, a, t, &lines));
            }
            current = Some((anchor, title, vec![line]));
        } else if let Some((_, _, lines)) = current.as_mut() {
            lines.push(line);
        }
        // Lines before the first anchored heading (rare) are dropped.
    }
    if let Some((a, t, lines)) = current.take() {
        sections.push(make_section(page, source, a, t, &lines));
    }
    sections
}

fn make_section(
    page: &str,
    source: &str,
    anchor: String,
    title: String,
    lines: &[&str],
) -> Section {
    Section {
        url: format!("{source}#{anchor}"),
        page: page.to_string(),
        anchor,
        title,
        body: lines.join("\n").trim_end().to_string(),
    }
}

/// Split a `---\n...\n---\n` YAML frontmatter block off the front. Returns
/// `(frontmatter_lines, body)` or `None` when there is no frontmatter.
fn split_frontmatter(text: &str) -> Option<(&str, &str)> {
    let rest = text.strip_prefix("---\n")?;
    let end = rest.find("\n---")?;
    let front = &rest[..end];
    let body = rest[end + 4..].trim_start_matches('\n');
    Some((front, body))
}

/// Read a `key: value` line out of a frontmatter block.
fn front_value<'a>(front: &'a str, key: &str) -> Option<&'a str> {
    front.lines().find_map(|line| {
        let (k, v) = line.split_once(':')?;
        (k.trim() == key).then(|| v.trim())
    })
}

/// Recognise an H1/H2 section boundary `# Title {#anchor}` / `## Title
/// {#anchor}`. Returns `(anchor, title)`. H3+ and headings without an explicit
/// `{#anchor}` are not boundaries (they fold into the enclosing section).
fn heading_boundary(line: &str) -> Option<(String, String)> {
    let trimmed = line.trim_end();
    let hashes = trimmed.bytes().take_while(|&b| b == b'#').count();
    if !(1..=2).contains(&hashes) {
        return None;
    }
    if trimmed.as_bytes().get(hashes) != Some(&b' ') {
        return None;
    }
    let without_close = trimmed.strip_suffix('}')?;
    let open = without_close.rfind("{#")?;
    let anchor = without_close[open + 2..].trim();
    if anchor.is_empty() {
        return None;
    }
    // `open` indexes into `without_close`, a prefix of `trimmed`, so it indexes
    // `trimmed` too: title is everything between the `#`s and the `{#`.
    let title = trimmed[hashes..open].trim();
    Some((anchor.to_string(), title.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn heading_boundary_parses_anchored_h2() {
        let (anchor, title) =
            heading_boundary("## Operator precedence {#operator-precedence}").expect("boundary");
        assert_eq!(anchor, "operator-precedence");
        assert_eq!(title, "Operator precedence");
    }

    #[test]
    fn heading_boundary_rejects_unanchored_and_h3() {
        assert!(heading_boundary("## No anchor here").is_none());
        assert!(heading_boundary("### Sub {#sub}").is_none());
        assert!(heading_boundary("not a heading").is_none());
    }

    #[test]
    fn operators_page_splits_into_sections() {
        let ops: Vec<_> = sections()
            .iter()
            .filter(|s| s.page == "language/operators")
            .collect();
        assert!(!ops.is_empty(), "operators page must produce sections");
        let prec = ops
            .iter()
            .find(|s| s.anchor == "operator-precedence")
            .expect("operator-precedence section");
        assert_eq!(prec.title, "Operator precedence");
        assert!(
            prec.url
                .ends_with("/language/operators/#operator-precedence")
        );
        assert!(prec.body.contains("{#operator-precedence}"));
    }

    #[test]
    fn search_finds_operator_precedence() {
        let hits = search("operator precedence", 5).expect("search ok");
        assert!(
            hits.iter()
                .any(|h| h.section.anchor == "operator-precedence"),
            "expected an operator-precedence hit, got {:?}",
            hits.iter().map(|h| &h.section.anchor).collect::<Vec<_>>()
        );
    }

    #[test]
    fn get_section_and_page_resolve() {
        assert!(get_section("language/operators", "operator-precedence").is_some());
        assert!(get_section("language/operators", "nope").is_none());
        let page = get_page("language/operators").expect("page");
        assert!(page.contains("Operator precedence"));
    }

    #[test]
    fn counts_are_sane() {
        assert!(
            page_count() >= 70,
            "expected ~76 pages, got {}",
            page_count()
        );
        assert!(
            section_count() >= page_count(),
            "every page yields at least one section"
        );
    }
}
