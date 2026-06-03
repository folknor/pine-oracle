use anyhow::{Result, bail};
use pine_oracle::{manual, render};

use crate::output::Style;

/// `po show <hash> [<hash>...]` renders manual sections selected by the 8-hex
/// ids `po search` prints. Each id resolves to one section, which is rendered
/// together with all of its descendant sections (the subtree: an H2 pulls in
/// its H3 children, an H3 leaf renders alone). Multiple ids render in order,
/// blank-line separated. Hash-only - there is no other addressing.
pub(crate) fn run(refs: &[String], style: Style, quiet: bool) -> Result<()> {
    let no_color = !style.enabled();
    for (i, raw) in refs.iter().enumerate() {
        let id = raw.trim().to_ascii_lowercase();
        let section = match manual::by_id(&id).as_slice() {
            [] => bail!("no manual section with id `{id}` (run `po search` for ids)"),
            [s] => *s,
            many => {
                // The id-uniqueness test makes this unreachable for the
                // vendored corpus; guard anyway so a future clash is reported,
                // never silently resolved to an arbitrary section.
                let list = many
                    .iter()
                    .map(|s| format!("  {}  {}", s.id, s.breadcrumb()))
                    .collect::<Vec<_>>()
                    .join("\n");
                bail!(
                    "ambiguous id `{id}` matches {} sections:\n{list}",
                    many.len()
                );
            }
        };
        if i > 0 {
            println!();
        }
        print!(
            "{}",
            render::render(&manual::subtree_markdown(section), no_color)
        );
        if !quiet {
            println!("{}", style.dim(&section.url));
        }
    }
    Ok(())
}
