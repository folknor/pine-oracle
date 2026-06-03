use anyhow::{Result, bail};
use pine_oracle::{manual, render};

use crate::output::Style;

/// `po search <query>` is the finder: BM25 over the Pine User Manual prose,
/// returning a menu of candidate sections rather than dumping prose. Each row is
/// a stable 8-hex id plus a `page-path / H2 / H3` breadcrumb; feed an id (or
/// several) to `po show` to read the section(s).
///
/// `--top`/`-1` skips the menu and renders the best hit's section directly (its
/// subtree), for the "I just want the top answer" case. Text-only.
pub(crate) fn run(query: &str, limit: usize, top: bool, style: Style, quiet: bool) -> Result<()> {
    let hits = manual::search(query, limit)?;
    let Some(best) = hits.first() else {
        bail!("no manual match for `{query}`");
    };

    if top {
        let md = manual::subtree_markdown(&best.section);
        print!("{}", render::render(&md, !style.enabled()));
        if !quiet {
            println!("{}", style.dim(&best.section.url));
        }
        return Ok(());
    }

    if !quiet {
        println!("Run 'po show <hash>...' to print one or more sections.");
    }
    for h in &hits {
        println!("{}  {}", style.bold(&h.section.id), h.section.breadcrumb());
    }
    Ok(())
}
