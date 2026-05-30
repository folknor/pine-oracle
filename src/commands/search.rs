use anyhow::Result;
use pine_oracle::search;

use crate::output::Style;

/// `po search <query>` prints a ranked, deduplicated list of identifier names -
/// the index into `po lookup`. One line per name: `score name`. There is no
/// JSON form (the names are the payload; the structured data lives in lookup)
/// and no `--kind` filter (both sources are the same "name" axis).
pub(crate) fn run(query: &str, limit: usize, style: Style, quiet: bool) -> Result<()> {
    let hits = search::query(query, limit)?;
    if hits.is_empty() {
        if !quiet {
            eprintln!("no matches");
        }
        return Ok(());
    }
    for h in &hits {
        let score = style.dim(&format!("{:>6.2}", h.score));
        println!("{score}  {}", h.name);
    }
    Ok(())
}
