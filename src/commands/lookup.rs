use anyhow::{Result, bail};
use pine_cli::reference;

use crate::output::{ResolvedFormat, print_json};

pub(crate) fn run(name: &str, format: ResolvedFormat, quiet: bool) -> Result<()> {
    if let Some(entry) = reference::lookup(name) {
        match format {
            ResolvedFormat::Json => {
                // Exact-hit JSON uses the same envelope as the prefix path so
                // script consumers don't need to detect the result shape by
                // presence/absence of a field. An exact hit is a 1-element
                // matches array with `"exact": true`.
                print_json(&serde_json::json!({
                    "query": name,
                    "exact": true,
                    "matches": [{
                        "category": entry.category,
                        "name": entry.name,
                        "content": entry.content,
                    }],
                }))?;
            }
            ResolvedFormat::Text => {
                if !quiet {
                    println!("{} ({})\n", entry.name, entry.category);
                }
                println!("{}", entry.content);
            }
        }
        return Ok(());
    }

    let prefix_hits = reference::prefix_search(name);
    if prefix_hits.is_empty() {
        bail!("no match for `{name}`");
    }

    match format {
        ResolvedFormat::Json => {
            // Prefix-hit JSON: same envelope as the exact path, with
            // `"exact": false` and the full entry objects so callers don't
            // need follow-up lookups to get category and content.
            let matches: Vec<_> = prefix_hits
                .iter()
                .map(|e| {
                    serde_json::json!({
                        "category": e.category,
                        "name": e.name,
                        "content": e.content,
                    })
                })
                .collect();
            print_json(&serde_json::json!({
                "query": name,
                "exact": false,
                "matches": matches,
            }))?;
        }
        ResolvedFormat::Text => {
            if !quiet {
                eprintln!("no exact match; {} prefix hit(s):", prefix_hits.len());
            }
            for e in &prefix_hits {
                println!("{}  ({})", e.name, e.category);
            }
        }
    }
    Ok(())
}
