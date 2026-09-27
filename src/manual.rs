// Pine User Manual prose search. The vendored manual (vendor/pine-manual/v6, a
// per-page markdown tree scraped from TradingView's docs) is embedded and split
// into sections at its anchored H1-H3 headings (`## Heading {#anchor}` and
// finer). Each section is one BM25 document keyed `page#anchor` - the same
// string is the index key, the result provenance, and (joined with the page's
// `source` URL) a real clickable link. Each section also gets a stable 8-hex
// `id` (`po search` prints it, `po show` resolves it via `by_id`).
//
// This is the "how does X work" half of the oracle, complementing `po lookup`'s
// "what is X". `po search` finds candidate sections (a menu of id + breadcrumb)
// and `po show` renders a chosen section's subtree (`subtree_markdown`) via
// `crate::render`.

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

/// One manual section: an H1-H3 heading and the prose beneath it (down to the
/// next H1-H3 heading), addressed by `page#anchor`.
#[derive(Debug, Clone, Serialize)]
pub struct Section {
    /// Stable 8-hex section id: the first 8 hex chars of FNV-1a(`page#anchor`).
    /// The handle `po search` prints and `po show` resolves.
    pub id: String,
    /// URL-path page id, e.g. `language/operators`.
    pub page: String,
    /// Heading anchor / URL fragment, e.g. `operator-precedence`.
    pub anchor: String,
    /// Heading text (the leaf heading of this section).
    pub title: String,
    /// Heading level: 1, 2, or 3. Drives the `po show` subtree walk (a section
    /// owns every following section of deeper level until the next same-or-
    /// shallower heading).
    pub level: usize,
    /// Ancestor heading titles from H2 down to and including this section's own
    /// heading, H1 excluded (the page path stands in for H1). Empty for an
    /// H1/page-intro section. Joined with the page path to form the breadcrumb.
    pub trail: Vec<String>,
    /// Canonical TradingView URL (`source` + `#anchor`).
    pub url: String,
    /// Section markdown, heading line included.
    pub body: String,
}

impl Section {
    /// Human breadcrumb shown in `po search` rows: `page-path / H2 / H3`,
    /// collapsing to `page-path / H2` for an H2 leaf and just `page-path` for
    /// an H1/page-intro section.
    pub fn breadcrumb(&self) -> String {
        if self.trail.is_empty() {
            self.page.clone()
        } else {
            format!("{} / {}", self.page, self.trail.join(" / "))
        }
    }
}

/// The stable section id: first 8 hex chars of FNV-1a(`key`), where `key` is the
/// section's `page#anchor`. FNV-1a is deterministic and dependency-free; 8 hex
/// (32 bits) is collision-free across the current corpus with comfortable
/// headroom (`all_section_ids_are_unique` guards against a future clash).
fn short_id(key: &str) -> String {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for &b in key.as_bytes() {
        h ^= b as u64;
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    format!("{h:016x}")[..8].to_string()
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

/// Every section whose 8-hex id equals `id`. Normally one; a length > 1 means a
/// hash collision (guarded against by `all_section_ids_are_unique`), which
/// `po show` reports rather than silently picking one.
pub fn by_id(id: &str) -> Vec<&'static Section> {
    sections().iter().filter(|s| s.id == id).collect()
}

/// Markdown for `target` plus all its descendant sections: every following
/// section on the same page whose heading is deeper than `target`'s, stopping at
/// the next same-or-shallower heading. So `po show` of an H2 yields the H2 and
/// all its H3 children; `po show` of an H3 leaf yields just that section.
pub fn subtree_markdown(target: &Section) -> String {
    let secs = sections();
    let Some(start) = secs.iter().position(|s| s.id == target.id) else {
        return target.body.clone();
    };
    let mut bodies = vec![secs[start].body.as_str()];
    for s in &secs[start + 1..] {
        if s.page != target.page || s.level <= target.level {
            break;
        }
        bodies.push(s.body.as_str());
    }
    bodies.join("\n\n")
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
    // tantivy's TopDocs panics on a zero limit.
    let q = &crate::query::plain_terms(q);
    if q.trim().is_empty() || limit == 0 {
        return Ok(Vec::new());
    }
    let e = engine();
    let searcher = e.reader.searcher();

    let title_parser = QueryParser::for_index(&e.index, vec![e.title_field]);
    let content_parser = QueryParser::for_index(&e.index, vec![e.content_field]);
    // Lenient: Pine syntax in a query (`strategy.exit(`, `?:`, `a:b`) must
    // not be a parse error.
    let (title_q, _) = title_parser.parse_query_lenient(q);
    let (content_q, _) = content_parser.parse_query_lenient(q);
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
    let mut current: Option<Pending<'_>> = None;
    // Ancestor heading stack: (level, title) from the page H1 down to the
    // current heading. Trails are derived from it (H2+ only).
    let mut stack: Vec<(usize, String)> = Vec::new();
    for line in body.lines() {
        if let Some((level, anchor, title)) = heading_boundary(line) {
            if let Some((lvl, a, t, trail, lines)) = current.take() {
                sections.push(make_section(page, source, a, t, lvl, trail, &lines));
            }
            // Drop any siblings/deeper headings, then push this one so the
            // stack holds exactly this heading's ancestors plus itself.
            while stack.last().is_some_and(|(l, _)| *l >= level) {
                stack.pop();
            }
            stack.push((level, title.clone()));
            // Breadcrumb trail = the H2..leaf titles (the page path stands in
            // for H1, so H1 is excluded).
            let trail: Vec<String> = stack
                .iter()
                .filter(|(l, _)| *l >= 2)
                .map(|(_, t)| t.clone())
                .collect();
            current = Some((level, anchor, title, trail, vec![line]));
        } else if let Some((_, _, _, _, lines)) = current.as_mut() {
            lines.push(line);
        }
        // Lines before the first anchored heading (rare) are dropped.
    }
    if let Some((lvl, a, t, trail, lines)) = current.take() {
        sections.push(make_section(page, source, a, t, lvl, trail, &lines));
    }
    sections
}

/// The in-progress section while parsing a page: its heading level, anchor,
/// title, breadcrumb trail, and the raw lines accumulated so far.
type Pending<'a> = (usize, String, String, Vec<String>, Vec<&'a str>);

fn make_section(
    page: &str,
    source: &str,
    anchor: String,
    title: String,
    level: usize,
    trail: Vec<String>,
    lines: &[&str],
) -> Section {
    Section {
        id: short_id(&format!("{page}#{anchor}")),
        url: format!("{source}#{anchor}"),
        page: page.to_string(),
        anchor,
        title,
        level,
        trail,
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

/// Recognise an H1-H3 section boundary (`# Title {#anchor}` through `### Title
/// {#anchor}`). Returns `(level, anchor, title)`. H4+ and headings without an
/// explicit `{#anchor}` are not boundaries (they fold into the enclosing
/// section). The manual's anchored structure bottoms out at H3 in practice, so
/// that is where per-heading granularity (one tantivy document per section)
/// stops.
fn heading_boundary(line: &str) -> Option<(usize, String, String)> {
    let trimmed = line.trim_end();
    let hashes = trimmed.bytes().take_while(|&b| b == b'#').count();
    if !(1..=3).contains(&hashes) {
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
    Some((hashes, anchor.to_string(), title.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn heading_boundary_parses_anchored_h2() {
        let (level, anchor, title) =
            heading_boundary("## Operator precedence {#operator-precedence}").expect("boundary");
        assert_eq!(level, 2);
        assert_eq!(anchor, "operator-precedence");
        assert_eq!(title, "Operator precedence");
    }

    #[test]
    fn heading_boundary_accepts_h3_rejects_h4_and_unanchored() {
        let (level, anchor, title) =
            heading_boundary("### Changing case {#changing-case}").expect("h3 boundary");
        assert_eq!(level, 3);
        assert_eq!(anchor, "changing-case");
        assert_eq!(title, "Changing case");
        assert!(heading_boundary("## No anchor here").is_none());
        assert!(heading_boundary("#### Deep {#deep}").is_none());
        assert!(heading_boundary("not a heading").is_none());
    }

    #[test]
    fn all_section_ids_are_unique() {
        // The collision safety net: a re-vendor that introduces an 8-hex clash
        // fails here instead of shipping an ambiguous `po show` id.
        use std::collections::HashSet;
        let mut seen = HashSet::new();
        for s in sections() {
            assert_eq!(s.id.len(), 8, "id must be 8 hex chars: {:?}", s.id);
            assert!(
                s.id.bytes().all(|b| b.is_ascii_hexdigit()),
                "id must be hex: {}",
                s.id
            );
            assert!(
                seen.insert(s.id.as_str()),
                "duplicate id {} ({})",
                s.id,
                s.breadcrumb()
            );
        }
    }

    #[test]
    fn breadcrumb_is_page_path_plus_heading_trail() {
        let cc = get_section("concepts/strings", "changing-case").expect("changing-case");
        assert_eq!(cc.level, 3);
        assert_eq!(
            cc.breadcrumb(),
            "concepts/strings / Modifying strings / Changing case"
        );
        // An H2 leaf collapses to page-path / H2.
        let modifying = get_section("concepts/strings", "modifying-strings").expect("modifying");
        assert_eq!(modifying.level, 2);
        assert_eq!(
            modifying.breadcrumb(),
            "concepts/strings / Modifying strings"
        );
    }

    #[test]
    fn subtree_of_parent_includes_children_leaf_does_not() {
        // `po show` of an H2 pulls the H2 + all its H3 descendants ...
        let parent = get_section("concepts/strings", "modifying-strings").expect("parent");
        let md = subtree_markdown(parent);
        assert!(
            md.contains("{#changing-case}"),
            "subtree must include H3 child"
        );
        assert!(md.contains("{#repeating-sequences}"), "and the last child");
        assert!(md.contains("str.lower"));
        assert!(
            !md.contains("{#string-inspection-and-extraction}"),
            "subtree must stop at the next H2"
        );
        // ... while an H3 leaf yields only itself.
        let leaf = get_section("concepts/strings", "changing-case").expect("leaf");
        let leaf_md = subtree_markdown(leaf);
        assert!(leaf_md.contains("{#changing-case}"));
        assert!(
            !leaf_md.contains("{#trimming-whitespaces}"),
            "leaf must not swallow its sibling"
        );
    }

    #[test]
    fn by_id_round_trips_to_the_section() {
        let cc = get_section("concepts/strings", "changing-case").expect("changing-case");
        let found = by_id(&cc.id);
        assert_eq!(found.len(), 1, "id resolves to exactly one section");
        assert_eq!(found[0].anchor, "changing-case");
        assert!(by_id("zzzzzzzz").is_empty(), "unknown id resolves to none");
    }

    #[test]
    fn h3_splits_from_parent_h2() {
        let cc = get_section("concepts/strings", "changing-case").expect("changing-case section");
        assert_eq!(cc.title, "Changing case");
        assert!(cc.body.contains("str.lower"));
        // The parent H2 no longer swallows its H3 children.
        let parent = get_section("concepts/strings", "modifying-strings").expect("parent section");
        assert!(!parent.body.contains("{#changing-case}"));
        assert!(!parent.body.contains("{#trimming-whitespaces}"));
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
