use anyhow::{Result, bail};
use pine_cli::reference;

use crate::output::{ResolvedFormat, print_json};

pub(crate) fn run(name: &str, format: ResolvedFormat, quiet: bool) -> Result<()> {
    if let Some(entry) = reference::lookup(name) {
        match format {
            ResolvedFormat::Json => {
                print_json(&entry)?;
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
            let names: Vec<&str> = prefix_hits.iter().map(|e| e.name.as_str()).collect();
            print_json(&serde_json::json!({
                "query": name,
                "exact": false,
                "matches": names,
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
