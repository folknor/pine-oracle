// Unified finder backing `po search`. Builds ONE BM25 index over both prose
// corpora - the Pine User Manual sections (`manual`) and the authored TA
// recipes (`recipe`) - so a single query ranks across both with comparable
// scores and shared corpus statistics (separate per-corpus indexes would give
// incomparable BM25 scores and a meaningless merge). `manual` and `recipe` stay
// independent data providers; this module composes them.
//
// A hit carries the corpus `kind` and a `handle` that re-keys it back to its
// source: an 8-hex section id (resolved by `manual::by_id` -> `po show`) for a
// manual hit, or a recipe name (resolved by `recipe::lookup` -> `po recipe`) for
// a recipe hit. The command turns those into menu rows and the `-1` top render.

use anyhow::Result;
use std::sync::OnceLock;
use tantivy::collector::TopDocs;
use tantivy::query::{BooleanQuery, BoostQuery, Query};
use tantivy::schema::{Field, STORED, STRING, Schema, TEXT, Value};
use tantivy::{Index, IndexReader, ReloadPolicy, TantivyDocument};

use crate::query::field_query;
use crate::{manual, recipe};

/// Multiplier on the title field so a heading / recipe-name match outranks a
/// body-prose match. Matches the manual finder's boost.
const TITLE_BOOST: f32 = 4.0;

/// Which corpus a hit came from. Drives the follow-up verb (`po show` vs
/// `po recipe`) and how the command renders the `-1` top hit.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HitKind {
    Manual,
    Recipe,
}

/// One unified search result: the corpus it came from, the `handle` that
/// re-keys it to its source, a human breadcrumb, and the BM25 score.
#[derive(Debug, Clone)]
pub struct Hit {
    pub kind: HitKind,
    /// Manual: the 8-hex section id (`po show <handle>`). Recipe: the recipe
    /// name (`po recipe <handle>`).
    pub handle: String,
    /// Manual: `page / H2 / H3`. Recipe: `recipe / category / Title`.
    pub breadcrumb: String,
    pub score: f32,
}

/// Ranked hits across both corpora for `q` (BM25, title 4x boosted). An empty
/// query returns an empty vec without touching the index.
pub fn find(q: &str, limit: usize) -> Result<Vec<Hit>> {
    // tantivy's TopDocs panics on a zero limit.
    if q.trim().is_empty() || limit == 0 {
        return Ok(Vec::new());
    }
    let e = engine();
    let searcher = e.reader.searcher();

    let title_q = field_query(&e.index, e.title_field, q)?;
    let content_q = field_query(&e.index, e.content_field, q)?;
    let boosted: Box<dyn Query> = Box::new(BoostQuery::new(title_q, TITLE_BOOST));
    let query: Box<dyn Query> = Box::new(BooleanQuery::union(vec![boosted, content_q]));

    let collector = TopDocs::with_limit(limit).order_by_score();
    let top = searcher.search(&query, &collector)?;
    let mut hits = Vec::with_capacity(top.len());
    for (score, addr) in top {
        let doc: TantivyDocument = searcher.doc(addr)?;
        let kind_str = stored(&doc, e.kind_field);
        let handle = stored(&doc, e.handle_field);
        let Some((kind, breadcrumb)) = resolve(&kind_str, &handle) else {
            continue;
        };
        hits.push(Hit {
            kind,
            handle,
            breadcrumb,
            score,
        });
    }
    Ok(hits)
}

/// Re-key a stored (kind, handle) pair back to a live breadcrumb. Returns `None`
/// if the source row vanished (e.g. a stale index), so the hit is skipped.
fn resolve(kind: &str, handle: &str) -> Option<(HitKind, String)> {
    match kind {
        "manual" => manual::by_id(handle)
            .into_iter()
            .next()
            .map(|s| (HitKind::Manual, s.breadcrumb())),
        "recipe" => recipe::lookup(handle).map(|r| {
            (
                HitKind::Recipe,
                format!("recipe / {} / {}", r.category, r.title),
            )
        }),
        _ => None,
    }
}

// ---------- index ----------

struct Engine {
    index: Index,
    reader: IndexReader,
    title_field: Field,
    content_field: Field,
    kind_field: Field,
    handle_field: Field,
}

fn engine() -> &'static Engine {
    static ENGINE: OnceLock<Engine> = OnceLock::new();
    ENGINE.get_or_init(|| build_index().expect("unified find index build must succeed"))
}

fn build_index() -> Result<Engine> {
    let mut schema_builder = Schema::builder();
    let title_field = schema_builder.add_text_field("title", TEXT);
    let content_field = schema_builder.add_text_field("content", TEXT);
    // kind + handle are stored only; they re-key a hit to its source row.
    let kind_field = schema_builder.add_text_field("kind", STRING | STORED);
    let handle_field = schema_builder.add_text_field("handle", STRING | STORED);
    let schema = schema_builder.build();

    let index = Index::create_in_ram(schema);
    let mut writer = index.writer(15_000_000)?;

    for s in manual::sections() {
        let mut doc = TantivyDocument::default();
        doc.add_text(title_field, &s.title);
        doc.add_text(content_field, &s.body);
        doc.add_text(kind_field, "manual");
        doc.add_text(handle_field, &s.id);
        writer.add_document(doc)?;
    }
    for r in recipe::recipes() {
        // Name + title + aliases all live in the boosted title field so any
        // handle a user might type ranks the recipe highly.
        let title = format!("{} {} {}", r.name, r.title, r.aliases.join(" "));
        let mut doc = TantivyDocument::default();
        doc.add_text(title_field, &title);
        doc.add_text(content_field, &r.body);
        doc.add_text(kind_field, "recipe");
        doc.add_text(handle_field, &r.name);
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
        kind_field,
        handle_field,
    })
}

fn stored(doc: &TantivyDocument, field: Field) -> String {
    doc.get_first(field)
        .and_then(|v| v.as_str())
        .map(String::from)
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_manual_section() {
        let hits = find("operator precedence", 8).expect("search ok");
        let manual_hit = hits
            .iter()
            .find(|h| h.kind == HitKind::Manual)
            .expect("a manual hit");
        // Manual handles are 8-hex section ids.
        assert_eq!(manual_hit.handle.len(), 8);
        assert!(manual_hit.handle.bytes().all(|b| b.is_ascii_hexdigit()));
    }

    #[test]
    fn finds_recipe_by_title_and_alias() {
        // A query that exists only in the recipe corpus must surface it.
        let hits = find("double exponential moving average", 8).expect("search ok");
        let hit = hits
            .iter()
            .find(|h| h.kind == HitKind::Recipe)
            .expect("a recipe hit");
        assert_eq!(hit.handle, "dema");
        assert!(
            hit.breadcrumb.starts_with("recipe / "),
            "recipe breadcrumb shape: {}",
            hit.breadcrumb
        );
    }

    #[test]
    fn empty_query_returns_nothing() {
        assert!(find("   ", 8).expect("ok").is_empty());
    }
}
