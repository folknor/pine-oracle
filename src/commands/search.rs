use anyhow::Result;
use pine_cli::search;

use crate::output::{ResolvedFormat, Style, is_catalog_request, print_catalog, print_json};

pub(crate) fn run(
    query: &str,
    limit: usize,
    kind_filter: Option<&str>,
    format: ResolvedFormat,
    style: Style,
    quiet: bool,
) -> Result<()> {
    if is_catalog_request(kind_filter) {
        return print_kind_catalog(format, quiet);
    }
    let hits = search::query(query, limit, kind_filter)?;
    match format {
        ResolvedFormat::Json => {
            let matches: Vec<_> = hits
                .iter()
                .map(|h| {
                    serde_json::json!({
                        "kind": h.kind,
                        "name": h.name,
                        "category": h.category,
                        "score": h.score,
                        "content": h.content,
                    })
                })
                .collect();
            print_json(&serde_json::json!({
                "query": query,
                "matches": matches,
            }))?;
        }
        ResolvedFormat::Text => {
            if hits.is_empty() {
                if !quiet {
                    eprintln!("no matches");
                }
                return Ok(());
            }
            for h in &hits {
                let score = style.dim(&format!("{:>6.2}", h.score));
                let kind = style.cyan(&format!("[{:<9}]", h.kind));
                let name = style.bold(&h.name);
                let category = style.dim(&format!("({})", h.category));
                println!("{score}  {kind} {name}  {category}");
                if !quiet {
                    let snippet = snippet_first_line(&h.content, 120);
                    if !snippet.is_empty() {
                        println!("        {}", style.dim(&snippet));
                    }
                }
            }
        }
    }
    Ok(())
}

fn print_kind_catalog(format: ResolvedFormat, quiet: bool) -> Result<()> {
    let kinds = search::kind_catalog();
    print_catalog(
        "kinds",
        &kinds,
        |k| {
            format!(
                "{:<9} {:<9} {:>5}  {}",
                k.kind, k.category, k.count, k.description
            )
        },
        |k| k.kind.to_string(),
        format,
        quiet,
    )
}

fn snippet_first_line(content: &str, max_chars: usize) -> String {
    let first = content
        .lines()
        .map(str::trim)
        .find(|l| !l.is_empty())
        .unwrap_or("");
    let collected: String = first.chars().take(max_chars).collect();
    if first.chars().count() > max_chars {
        format!("{collected}...")
    } else {
        collected
    }
}
