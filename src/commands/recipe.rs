use anyhow::{Result, bail};
use pine_oracle::{recipe, render};

use crate::output::{Style, is_catalog_request, print_catalog};

/// How many "did you mean ...?" suggestions to offer on a recipe miss.
const SUGGEST_LIMIT: usize = 8;

/// `po recipe <name>` describes a technical-analysis instrument that has no
/// TradingView builtin and no manual page (custom moving averages, composite
/// indicators, candlestick patterns, market-structure concepts). Each entry is
/// authored markdown (prose + a Pine v6 snippet) from the `recipe` corpus,
/// rendered through `crate::render` - the recipe analogue of `po lookup`.
///
/// `--list` / `--category` / `--grep` browse the corpus; a miss offers BM25-free
/// "did you mean ...?" suggestions.
pub(crate) fn run(
    name: Option<&str>,
    list: bool,
    category: Option<&str>,
    grep: Option<&str>,
    style: Style,
    quiet: bool,
) -> Result<()> {
    if is_catalog_request(category) {
        return print_category_catalog(quiet);
    }
    if list {
        let grep = resolve_list_grep(name, grep)?;
        return print_recipe_list(category, grep);
    }
    if category.is_some() || grep.is_some() {
        bail!("`po recipe --category/--grep` requires `--list`");
    }
    let Some(name) = name else {
        bail!("`po recipe` requires a name or `--list`");
    };

    match recipe::lookup(name) {
        Some(r) => {
            print!("{}", render::render(&r.body, !style.enabled()));
            if !quiet {
                println!(
                    "{}",
                    style.dim(&format!("recipe: {}/{}", r.category, r.name))
                );
            }
            Ok(())
        }
        None => did_you_mean(name, quiet),
    }
}

/// A bare `po recipe TEXT --list` treats TEXT as an implicit grep, mirroring
/// `po lookup`. An explicit `--grep` alongside the positional is a conflict.
fn resolve_list_grep<'a>(name: Option<&'a str>, grep: Option<&'a str>) -> Result<Option<&'a str>> {
    match (name, grep) {
        (Some(_), Some(_)) => bail!("`po recipe <name> --list` cannot combine with `--grep`"),
        (Some(name), None) => Ok(Some(name)),
        (None, grep) => Ok(grep),
    }
}

fn print_category_catalog(quiet: bool) -> Result<()> {
    let cats = recipe::categories();
    print_catalog(
        &cats,
        |c| format!("{:<16} {:>4}", c.category, c.count),
        |c| c.category.clone(),
        quiet,
    )
}

fn print_recipe_list(category: Option<&str>, grep: Option<&str>) -> Result<()> {
    let entries = recipe::list(category, grep);
    if entries.is_empty() {
        println!("no recipes");
    } else {
        for r in &entries {
            println!("{:<16} {:<22} {}", r.category, r.name, r.title);
        }
    }
    Ok(())
}

/// No exact hit: offer the closest recipe names via the fuzzy suggestion engine.
fn did_you_mean(name: &str, quiet: bool) -> Result<()> {
    let hits = recipe::suggest(name, SUGGEST_LIMIT);
    if hits.is_empty() {
        bail!("no recipe for `{name}` (run `po recipe --list`)");
    }
    if !quiet {
        eprintln!("no recipe named `{name}`. did you mean:");
    }
    for r in &hits {
        println!("{:<22} {}", r.name, r.title);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn list_name_becomes_implicit_grep() {
        assert_eq!(
            resolve_list_grep(Some("hma"), None).expect("valid"),
            Some("hma")
        );
    }

    #[test]
    fn list_rejects_name_and_explicit_grep() {
        let err = resolve_list_grep(Some("hma"), Some("hull")).expect_err("must reject");
        assert!(err.to_string().contains("cannot combine"));
    }
}
