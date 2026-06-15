use anyhow::{Result, bail};
use pine_oracle::find::{self, HitKind};
use pine_oracle::{manual, recipe, render};

use crate::output::Style;

/// `po search <query>` is the unified finder: BM25 over both the Pine User
/// Manual prose (`manual`) and the authored TA recipes (`recipe`), returning a
/// menu of candidates rather than dumping prose. Each row is a runnable handle
/// plus a breadcrumb:
///   - a manual hit shows its 8-hex section id and `page / H2 / H3`; feed the id
///     to `po show`.
///   - a recipe hit shows `recipe <name>` and `recipe / category / Title`; run
///     that directly.
///
/// `--top`/`-1` skips the menu and renders the best hit directly (a manual
/// section subtree, or a recipe body), for the "I just want the top answer"
/// case. Text-only.
pub(crate) fn run(query: &str, limit: usize, top: bool, style: Style, quiet: bool) -> Result<()> {
    let hits = find::find(query, limit)?;
    let Some(best) = hits.first() else {
        bail!("no match for `{query}`");
    };

    if top {
        let (markdown, provenance) = render_target(best)?;
        print!("{}", render::render(&markdown, !style.enabled()));
        if !quiet {
            println!("{}", style.dim(&provenance));
        }
        return Ok(());
    }

    if !quiet {
        println!("Run 'po show <hash>...' or 'po recipe <name>' to print a result.");
    }
    // Pad the plain handle labels to a common width before styling so the
    // breadcrumb column lines up (padding the styled string would count ANSI
    // bytes and misalign).
    let width = hits
        .iter()
        .map(|h| handle_label(h).len())
        .max()
        .unwrap_or(8);
    for h in &hits {
        let label = format!("{:<width$}", handle_label(h));
        println!("{}  {}", style.bold(&label), h.breadcrumb);
    }
    Ok(())
}

/// The runnable handle shown in a menu row: a bare 8-hex id for a manual hit
/// (fed to `po show`), or `recipe <name>` for a recipe hit (run as-is).
fn handle_label(hit: &find::Hit) -> String {
    match hit.kind {
        HitKind::Manual => hit.handle.clone(),
        HitKind::Recipe => format!("recipe {}", hit.handle),
    }
}

/// Resolve a hit to the markdown to render plus its provenance trailer line.
fn render_target(hit: &find::Hit) -> Result<(String, String)> {
    match hit.kind {
        HitKind::Manual => {
            let section = manual::by_id(&hit.handle)
                .into_iter()
                .next()
                .ok_or_else(|| anyhow::anyhow!("manual hit `{}` no longer resolves", hit.handle))?;
            Ok((manual::subtree_markdown(section), section.url.clone()))
        }
        HitKind::Recipe => {
            let r = recipe::lookup(&hit.handle)
                .ok_or_else(|| anyhow::anyhow!("recipe hit `{}` no longer resolves", hit.handle))?;
            Ok((r.body.clone(), format!("recipe: {}/{}", r.category, r.name)))
        }
    }
}
