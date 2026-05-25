use anyhow::Result;
use pine_cli::corpus;

use crate::output::{ResolvedFormat, print_json};

pub(crate) fn run(grep: Option<&str>, feature: Option<&str>, format: ResolvedFormat) -> Result<()> {
    if matches!(feature, Some("?")) {
        return print_feature_catalog(format);
    }
    let probes = corpus::list_probes(grep, feature)?;
    match format {
        ResolvedFormat::Json => {
            print_json(&probes)?;
        }
        ResolvedFormat::Text => {
            if probes.is_empty() {
                eprintln!("no probes matched");
                return Ok(());
            }
            for p in &probes {
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
    Ok(())
}

fn print_feature_catalog(format: ResolvedFormat) -> Result<()> {
    let catalog = corpus::feature_catalog();
    match format {
        ResolvedFormat::Json => {
            let items: Vec<_> = catalog
                .iter()
                .map(|(name, desc)| serde_json::json!({"name": name, "description": desc}))
                .collect();
            print_json(&items)?;
        }
        ResolvedFormat::Text => {
            for (name, desc) in &catalog {
                println!("{name:<28}  {desc}");
            }
        }
    }
    Ok(())
}
