use anyhow::{Result, bail};
use pine_oracle::{manual, render};

use crate::output::{ResolvedFormat, Style, print_json};

/// `po search` over the Pine User Manual prose. The positional argument is
/// interpreted by shape:
///
/// - `page#anchor` (contains `#`) -> render that exact section
/// - a known page path -> render the whole page
/// - anything else -> BM25 query; render the best section, list alternates as refs
///
/// Text output renders the section markdown to the terminal (`crate::render`,
/// honouring `--no-color`); JSON returns the raw markdown plus provenance.
pub(crate) fn run(
    input: &str,
    limit: usize,
    format: ResolvedFormat,
    style: Style,
    quiet: bool,
) -> Result<()> {
    let no_color = !style.enabled();

    if let Some((page, anchor)) = input.split_once('#') {
        let Some(section) = manual::get_section(page, anchor) else {
            bail!("no manual section `{page}#{anchor}`");
        };
        return emit_markdown(&section.body, &section.url, format, style, quiet);
    }

    if let Some(page_md) = manual::get_page(input) {
        // Page base URL = any section's url with the `#anchor` dropped.
        let url = manual::sections()
            .iter()
            .find(|s| s.page == input)
            .map(|s| s.url.split('#').next().unwrap_or_default().to_string())
            .unwrap_or_default();
        return emit_markdown(&page_md, &url, format, style, quiet);
    }

    let hits = manual::search(input, limit)?;
    let Some(best) = hits.first() else {
        bail!("no manual match for `{input}`");
    };
    match format {
        ResolvedFormat::Json => print_json(&serde_json::json!({
            "query": input,
            "matches": hits,
        }))?,
        ResolvedFormat::Text => {
            print!("{}", render::render(&best.section.body, no_color));
            if !quiet {
                provenance(&best.section.url, &style);
                if hits.len() > 1 {
                    println!("\nmore sections:");
                    for h in &hits[1..] {
                        let r#ref = format!("{}#{}", h.section.page, h.section.anchor);
                        println!("  {}  {}", style.bold(&r#ref), style.dim(&h.section.title));
                    }
                }
            }
        }
    }
    Ok(())
}

/// Render a single section or whole page: styled markdown to the terminal, or
/// raw markdown + URL as JSON.
fn emit_markdown(
    md: &str,
    url: &str,
    format: ResolvedFormat,
    style: Style,
    quiet: bool,
) -> Result<()> {
    match format {
        ResolvedFormat::Json => print_json(&serde_json::json!({
            "url": url,
            "markdown": md,
        }))?,
        ResolvedFormat::Text => {
            print!("{}", render::render(md, !style.enabled()));
            if !quiet {
                provenance(url, &style);
            }
        }
    }
    Ok(())
}

/// Print the canonical TradingView URL as a dim trailing provenance line.
fn provenance(url: &str, style: &Style) {
    if !url.is_empty() {
        println!("{}", style.dim(url));
    }
}
