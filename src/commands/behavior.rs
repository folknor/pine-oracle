use anyhow::{Result, bail};
use pine_cli::behavior;

use crate::output::{ResolvedFormat, Style, is_catalog_request, print_catalog, print_json};

pub(crate) fn run(
    name: Option<&str>,
    list: bool,
    kind: Option<&str>,
    grep: Option<&str>,
    format: ResolvedFormat,
    _style: Style,
    quiet: bool,
) -> Result<()> {
    if is_catalog_request(kind) {
        return print_kind_catalog(format, quiet);
    }
    if list {
        let grep = resolve_list_grep(name, grep)?;
        return print_behavior_list(kind, grep, format);
    }
    if kind.is_some() || grep.is_some() {
        bail!("`pine behavior --kind/--grep` requires `--list`");
    }
    let Some(name) = name else {
        bail!("`pine behavior` requires a name or `--list`");
    };
    let Some(b) = behavior::lookup(name) else {
        bail!("no behavior data for `{name}`");
    };
    match format {
        ResolvedFormat::Json => {
            print_json(&b)?;
        }
        ResolvedFormat::Text => print_behavior_text(&b),
    }
    Ok(())
}

fn resolve_list_grep<'a>(name: Option<&'a str>, grep: Option<&'a str>) -> Result<Option<&'a str>> {
    match (name, grep) {
        (Some(_), Some(_)) => bail!("`pine behavior <name> --list` cannot combine with `--grep`"),
        (Some(name), None) => Ok(Some(name)),
        (None, grep) => Ok(grep),
    }
}

fn print_kind_catalog(format: ResolvedFormat, quiet: bool) -> Result<()> {
    let kinds = behavior::kind_catalog();
    print_catalog(
        "kinds",
        &kinds,
        |k| format!("{:<9} {:>5}  {}", k.kind, k.count, k.description),
        |k| k.kind.to_string(),
        format,
        quiet,
    )
}

fn print_behavior_list(
    kind: Option<&str>,
    grep: Option<&str>,
    format: ResolvedFormat,
) -> Result<()> {
    let entries = behavior::list(kind, grep)?;
    match format {
        ResolvedFormat::Json => {
            print_json(&entries)?;
        }
        ResolvedFormat::Text => {
            if entries.is_empty() {
                println!("no behavior entries");
            } else {
                for entry in &entries {
                    let marker = if entry.polymorphic { " poly" } else { "" };
                    if entry.detail.is_empty() {
                        println!("{:<9} {}{}", entry.kind, entry.name, marker);
                    } else {
                        println!(
                            "{:<9} {}{}  {}",
                            entry.kind, entry.name, marker, entry.detail
                        );
                    }
                }
            }
        }
    }
    Ok(())
}

fn print_behavior_text(b: &behavior::Behavior) {
    match b {
        behavior::Behavior::Function(f) => {
            println!("function {}", f.name);
            if let Some(ns) = &f.namespace {
                println!("  namespace: {ns}");
            }
            if !f.description.is_empty() {
                println!("  description: {}", f.description);
            }
            println!("  syntax: {}", f.syntax);
            if !f.returns.is_empty() {
                println!("  returns: {}", f.returns);
            }
            if !f.parameters.is_empty() {
                println!("  parameters:");
                for p in &f.parameters {
                    let req = if p.required { "required" } else { "optional" };
                    println!("    - {} : {} ({req})", p.name, p.ty);
                }
            }
            if f.flags.top_level_only {
                println!("  flags: top-level only");
            }
            if !f.examples.is_empty() {
                let n = f.examples.len();
                let label = if n == 1 { "example" } else { "examples" };
                println!("  {label}: {n}");
                for (i, ex) in f.examples.iter().enumerate() {
                    if n > 1 {
                        println!("    --- example {} ---", i + 1);
                    }
                    for line in ex.lines() {
                        println!("    {line}");
                    }
                }
            }
            if let Some(beh) = &f.behavior {
                let poly = if beh.polymorphic.is_polymorphic() {
                    "yes"
                } else {
                    "no"
                };
                println!("  polymorphic: {poly}");
                if let Some(detail) = beh.polymorphic.detail() {
                    if let Some(rtp) = &detail.return_type_param {
                        println!("    return-type-param: {rtp}");
                    }
                    if let Some(strat) = &detail.strategy {
                        println!("    strategy: {strat}");
                    }
                    if !detail.allowed_types.is_empty() {
                        println!("    allowed-types: {}", detail.allowed_types.join(", "));
                    }
                }
                if let Some(ord) = &beh.argument_ordering {
                    println!("  argument-ordering: {ord}");
                }
                if !beh.observed_return_types.is_empty() {
                    println!(
                        "  observed-return-types: {}",
                        beh.observed_return_types.join(", ")
                    );
                }
                if let Some(reason) = &beh.reason {
                    println!("  reason: {reason}");
                }
            }
        }
        behavior::Behavior::Variable(v) => {
            println!("variable {}", v.name);
            println!("  type: {}", v.ty);
            println!("  qualifier: {}", v.qualifier);
            if !v.description.is_empty() {
                println!("  description: {}", v.description);
            }
        }
        behavior::Behavior::Constant(c) => {
            println!("constant {}", c.name);
            if let Some(ns) = &c.namespace {
                println!("  namespace: {ns}");
            }
            if let Some(short) = &c.short_name {
                println!("  short-name: {short}");
            }
            println!("  type: {}", c.ty);
        }
        behavior::Behavior::Keyword(k) => {
            println!("keyword {}", k.name);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn list_name_becomes_implicit_grep() {
        assert_eq!(
            resolve_list_grep(Some("plotshape"), None).expect("valid"),
            Some("plotshape")
        );
    }

    #[test]
    fn list_rejects_name_and_explicit_grep() {
        let err = resolve_list_grep(Some("plotshape"), Some("plot")).expect_err("must reject");
        assert!(err.to_string().contains("cannot combine"));
    }
}
