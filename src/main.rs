use anyhow::Result;
use clap::{Parser, Subcommand};
use pine_oracle::{behavior, manual, recipe, suggest};
use std::io::IsTerminal;

mod commands;
mod output;

use output::Style;

/// pine: Pine v6 oracle CLI. Answers semantic questions about Pine script
/// across every Pine-adjacent project. Vendors the pine-data behavior surface
/// (structured Pine v6 signatures + prose); exposes it as one-shot subcommands.
#[derive(Parser)]
#[command(name = "po", version, about = "Pine v6 oracle CLI", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Command,

    /// Suppress ANSI styling in text mode. Honoured automatically when
    /// `NO_COLOR` is set or stdout is not a TTY. The flag reaches every
    /// command; only those that emit colored text act on it visibly.
    #[arg(long, global = true)]
    no_color: bool,

    /// Suppress non-data status/note text. The exact effect is per-subcommand.
    #[arg(long, global = true)]
    quiet: bool,
}

#[derive(Subcommand)]
enum Command {
    /// Describe an identifier from pine-data: structured signature with
    /// per-argument prose, remarks, see-also, and operators.
    Lookup {
        name: Option<String>,
        /// List behavior catalog entries instead of looking up one name.
        #[arg(long)]
        list: bool,
        /// Restrict `--list` to function, variable, constant, keyword, type,
        /// or annotation. Pass `?` to list the catalog.
        #[arg(long)]
        kind: Option<String>,
        /// Restrict `--list` entries by name, namespace, or detail text.
        #[arg(long)]
        grep: Option<String>,
    },

    /// Find Pine User Manual sections (how does X work). Prints a menu of
    /// matching sections as `<hash>  page / H2 / H3`; feed a hash to `po show`.
    Search {
        /// Free-text query.
        query: String,
        /// Max sections to list.
        #[arg(long, default_value_t = 8)]
        limit: usize,
        /// Skip the menu and render the top hit's section directly.
        #[arg(long, short = '1')]
        top: bool,
    },

    /// Print Pine User Manual section(s) by their `po search` hash id. Each id
    /// renders the section plus all its subsections; pass several to print many.
    Show {
        /// One or more 8-hex section ids from `po search`.
        #[arg(required = true)]
        refs: Vec<String>,
    },

    /// Describe a TA instrument with no TradingView builtin (custom moving
    /// averages, composite indicators, candlestick patterns, structure
    /// concepts): authored prose + a Pine v6 recipe.
    Recipe {
        name: Option<String>,
        /// List recipe corpus entries instead of describing one name.
        #[arg(long)]
        list: bool,
        /// Restrict `--list` to one category. Pass `?` to list the catalog.
        #[arg(long)]
        category: Option<String>,
        /// Restrict `--list` entries by name, title, alias, or category text.
        #[arg(long)]
        grep: Option<String>,
    },

    /// Measured TradingView behavior: what each oracle (editor, endpoint,
    /// chart) did, and whether a question is settled. Every verb names its
    /// records directory with `--records`; nothing is read by default.
    Verdict {
        #[command(subcommand)]
        command: commands::verdict::VerdictCommand,
    },

    /// pine-data snapshot date + behavior bake counts
    Version,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let stdout_is_tty = std::io::stdout().is_terminal();
    // Every command is text-only; `Style` is `Copy` and threaded into each
    // `run`. Commands that don't emit colored text take it as `_style`.
    let text_style = Style::resolve(cli.no_color, stdout_is_tty);

    match cli.command {
        Command::Lookup {
            name,
            list,
            kind,
            grep,
        } => commands::lookup::run(
            name.as_deref(),
            list,
            kind.as_deref(),
            grep.as_deref(),
            text_style,
            cli.quiet,
        ),
        Command::Search { query, limit, top } => {
            commands::search::run(&query, limit, top, text_style, cli.quiet)
        }
        Command::Show { refs } => commands::show::run(&refs, text_style, cli.quiet),
        Command::Recipe {
            name,
            list,
            category,
            grep,
        } => commands::recipe::run(
            name.as_deref(),
            list,
            category.as_deref(),
            grep.as_deref(),
            text_style,
            cli.quiet,
        ),
        Command::Verdict { command } => commands::verdict::run(command, text_style, cli.quiet),
        Command::Version => cmd_version(cli.quiet),
    }
}

fn cmd_version(quiet: bool) -> Result<()> {
    let binary = env!("CARGO_PKG_VERSION");
    let indexed_names = suggest::indexed_name_count();
    let pine_data = behavior::snapshot();
    let manual_pages = manual::page_count();
    let manual_sections = manual::section_count();
    let recipe_count = recipe::count();
    let recipe_categories = recipe::categories().len();
    println!("po {binary}");
    if quiet {
        return Ok(());
    }
    println!(
        "pine-data:      v{} generated {}",
        pine_data.version, pine_data.generated_at
    );
    println!(
        "behavior:       {} functions ({} polymorphic), {} variables, {} constants, {} keywords, {} types, {} annotations, {} operators, {indexed_names} indexed names",
        pine_data.function_count,
        pine_data.polymorphic_function_count,
        pine_data.variable_count,
        pine_data.constant_count,
        pine_data.keyword_count,
        pine_data.type_count,
        pine_data.annotation_count,
        pine_data.operator_count
    );
    println!("manual:         {manual_pages} pages, {manual_sections} sections");
    println!("recipes:        {recipe_count} entries across {recipe_categories} categories");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lookup_list_parses_without_name() {
        let cli = Cli::try_parse_from([
            "pine",
            "lookup",
            "--list",
            "--kind",
            "function",
            "--grep",
            "plotshape",
        ])
        .expect("lookup list args should parse");

        match cli.command {
            Command::Lookup {
                name,
                list,
                kind,
                grep,
            } => {
                assert_eq!(name, None);
                assert!(list);
                assert_eq!(kind.as_deref(), Some("function"));
                assert_eq!(grep.as_deref(), Some("plotshape"));
            }
            _ => panic!("expected lookup command"),
        }
    }

    #[test]
    fn lookup_kind_catalog_parses_without_name() {
        let cli = Cli::try_parse_from(["pine", "lookup", "--kind", "?"])
            .expect("lookup kind catalog args should parse");

        match cli.command {
            Command::Lookup {
                name,
                list,
                kind,
                grep,
            } => {
                assert_eq!(name, None);
                assert!(!list);
                assert_eq!(kind.as_deref(), Some("?"));
                assert_eq!(grep, None);
            }
            _ => panic!("expected lookup command"),
        }
    }

    /// Pin that `--no-color` sets the flag.
    #[test]
    fn no_color_flag_sets_field() {
        let cli = Cli::try_parse_from(["pine", "--no-color", "lookup", "plot"])
            .expect("--no-color should parse");
        assert!(cli.no_color);
    }

    /// Pin that `--quiet` sets the flag.
    #[test]
    fn quiet_flag_sets_field() {
        let cli = Cli::try_parse_from(["pine", "--quiet", "lookup", "plot"])
            .expect("--quiet should parse");
        assert!(cli.quiet);
    }
}
