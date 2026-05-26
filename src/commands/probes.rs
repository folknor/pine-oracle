use anyhow::Result;
use pine_cli::corpus;

use crate::output::{ResolvedFormat, Style, is_catalog_request, print_catalog, print_json};

pub(crate) fn run(
    grep: Option<&str>,
    feature: Option<&str>,
    format: ResolvedFormat,
    _style: Style,
    quiet: bool,
) -> Result<()> {
    if is_catalog_request(feature) {
        return print_feature_catalog(format, quiet);
    }
    let probes = corpus::list_probes(grep, feature)?;
    match format {
        ResolvedFormat::Json => {
            print_json(&probes)?;
        }
        ResolvedFormat::Text => {
            if probes.is_empty() {
                if !quiet {
                    eprintln!("no probes matched");
                }
                return Ok(());
            }
            for p in &probes {
                if quiet {
                    println!("{}", p.slug);
                } else {
                    match &p.summary {
                        Some(s) => {
                            let snippet: String = s.chars().take(80).collect();
                            println!("{}  -  {snippet}", p.slug);
                        }
                        None => println!("{}", p.slug),
                    }
                }
            }
        }
    }
    Ok(())
}

fn print_feature_catalog(format: ResolvedFormat, quiet: bool) -> Result<()> {
    use serde::Serialize;

    #[derive(Serialize)]
    struct FeatureInfo {
        name: &'static str,
        description: &'static str,
    }

    let catalog: Vec<FeatureInfo> = corpus::feature_catalog()
        .into_iter()
        .map(|(name, description)| FeatureInfo { name, description })
        .collect();

    print_catalog(
        "features",
        &catalog,
        |f| format!("{:<28}  {}", f.name, f.description),
        |f| f.name.to_string(),
        format,
        quiet,
    )
}
