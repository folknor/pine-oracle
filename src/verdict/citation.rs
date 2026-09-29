// Manual citations: resolving a section handle, matching quotes against the
// section text, and deciding whether a stored citation is still current
// against the manual baked into this binary.
//
// A quote is matched on a plain-text projection of the markdown, applied the
// same way to both sides (so `*first*` copied from `po show` and `first` in
// the source compare equal), whitespace-collapsed, and it must lie inside ONE
// block (paragraph, heading, list-item text, table cell, code block), so a
// phrase stitched across unrelated paragraphs never matches. Drift is decided
// on the source markdown of the cited subtree instead (its sha256 at cite
// time), so any change to the section - a new qualifier, a changed link -
// makes the citation stale and forces a review.

use anyhow::{Result, bail};
use pulldown_cmark::{Event, Options, Parser, Tag, TagEnd};
use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock, PoisonError};

use super::Citation;
use super::validate::sha256_hex;
use crate::manual::{self, Section};

/// The manual section `handle` names: the 8-hex id `po search` prints, or
/// `page#anchor`.
pub(super) fn resolve_section(handle: &str) -> Result<&'static Section> {
    let handle = handle.trim();
    if let Some((page, anchor)) = handle.split_once('#') {
        return match manual::get_section(page, anchor) {
            Some(s) => Ok(s),
            None => bail!("no manual section `{handle}`"),
        };
    }
    match manual::by_id(&handle.to_ascii_lowercase()).as_slice() {
        [s] => Ok(s),
        [] => bail!(
            "no manual section with id `{handle}` (use the id `po search` prints, or page#anchor)"
        ),
        many => bail!(
            "manual id `{handle}` is ambiguous: {}",
            many.iter().map(|s| key(s)).collect::<Vec<_>>().join(", ")
        ),
    }
}

/// `page#anchor`, the stored citation key of a section.
pub(super) fn key(s: &Section) -> String {
    format!("{}#{}", s.page, s.anchor)
}

/// The section a stored `page#anchor` key names, if the manual still has it.
pub(super) fn section(key: &str) -> Option<&'static Section> {
    let (page, anchor) = key.split_once('#')?;
    manual::get_section(page, anchor)
}

/// A section subtree as citations need it: its source digest and the plain
/// text of its blocks.
struct Projected {
    digest: String,
    blocks: Vec<String>,
}

/// The projection of `s`, computed once per section per process. The manual
/// is baked into the binary, so it never changes under a running `po`, and
/// every `resolve` of a cited question consults it.
fn projected(s: &Section) -> Arc<Projected> {
    static CACHE: OnceLock<Mutex<HashMap<String, Arc<Projected>>>> = OnceLock::new();
    let cache = CACHE.get_or_init(Mutex::default);
    let mut map = cache.lock().unwrap_or_else(PoisonError::into_inner);
    Arc::clone(map.entry(key(s)).or_insert_with(|| {
        let markdown = manual::subtree_markdown(s);
        Arc::new(Projected {
            digest: sha256_hex(markdown.as_bytes()),
            blocks: blocks(&markdown),
        })
    }))
}

/// sha256 of the cited subtree's source markdown.
pub(super) fn digest(s: &Section) -> String {
    projected(s).digest.clone()
}

/// Whether `quote` lies inside one block of the section subtree's text. The
/// quote must itself be one block: joining several would let a passage
/// stitched from separate paragraphs match.
pub(super) fn quote_found(s: &Section, quote: &str) -> bool {
    let quote_blocks = blocks(quote);
    let [needle] = quote_blocks.as_slice() else {
        return false;
    };
    projected(s)
        .blocks
        .iter()
        .any(|b| b.contains(needle.as_str()))
}

/// Why a stored citation no longer counts against this binary's manual, or
/// `None` while it is current. Quotes are checked even when the digest
/// matches, so a hand-edited quote cannot ride on an unchanged section.
pub(super) fn staleness(c: &Citation) -> Option<String> {
    let Some(s) = section(&c.section) else {
        return Some("the section is no longer in the manual".to_string());
    };
    if let Some(q) = c.quotes.iter().find(|q| !quote_found(s, q)) {
        return Some(format!("the manual no longer says \"{q}\""));
    }
    if projected(s).digest != c.digest {
        return Some("the section changed since it was cited; review and cite again".to_string());
    }
    None
}

/// The whitespace-collapsed plain text of each block in `markdown`. Inline
/// markup (emphasis, code spans, links) contributes its text only; every
/// other tag boundary ends a block.
fn blocks(markdown: &str) -> Vec<String> {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_GFM);
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_HEADING_ATTRIBUTES);
    let mut out = Vec::new();
    let mut current = String::new();
    let mut flush = |current: &mut String| {
        let text = current.split_whitespace().collect::<Vec<_>>().join(" ");
        if !text.is_empty() {
            out.push(text);
        }
        current.clear();
    };
    for event in Parser::new_ext(markdown, options) {
        match event {
            Event::Text(t) | Event::Code(t) => current.push_str(&t),
            Event::SoftBreak | Event::HardBreak => current.push(' '),
            Event::Start(tag) if !inline_start(&tag) => flush(&mut current),
            Event::End(tag) if !inline_end(tag) => flush(&mut current),
            _ => {}
        }
    }
    flush(&mut current);
    out
}

fn inline_start(tag: &Tag<'_>) -> bool {
    matches!(
        tag,
        Tag::Emphasis
            | Tag::Strong
            | Tag::Strikethrough
            | Tag::Link { .. }
            | Tag::Image { .. }
            | Tag::Superscript
            | Tag::Subscript
    )
}

fn inline_end(tag: TagEnd) -> bool {
    matches!(
        tag,
        TagEnd::Emphasis
            | TagEnd::Strong
            | TagEnd::Strikethrough
            | TagEnd::Link
            | TagEnd::Image
            | TagEnd::Superscript
            | TagEnd::Subscript
    )
}
