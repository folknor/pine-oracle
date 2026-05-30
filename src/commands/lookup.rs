use anyhow::{Result, bail};
use pine_oracle::{behavior, reference};

use crate::output::{ResolvedFormat, Style, is_catalog_request, print_catalog, print_json};

/// `po lookup` is the single "describe this identifier" verb. It joins two
/// datasets keyed on the same name:
///
///   - pine-data `behavior`: the structured signature (typed params with
///     defaults / ranges, per-overload arrays, polymorphism + deprecation
///     flags).
///   - the vendored v6 `reference`: the prose the structured surface drops -
///     per-argument descriptions, `Remarks`, and the `See also` cross-refs.
///
/// `--list` / `--kind` / `--grep` browse the behavior catalog (these moved here
/// when the standalone `behavior` command was folded in).
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
        bail!("`po lookup --kind/--grep` requires `--list`");
    }
    let Some(name) = name else {
        bail!("`po lookup` requires a name or `--list`");
    };

    let bhv = behavior::lookup(name);
    let refent = reference::lookup(name);
    let enrich = reference::enrichment(name);

    if bhv.is_none() && refent.is_none() {
        return prefix_fallback(name, format, quiet);
    }

    match format {
        ResolvedFormat::Json => print_merged_json(name, bhv.as_ref(), enrich.as_ref())?,
        ResolvedFormat::Text => {
            print_merged_text(bhv.as_ref(), refent.as_ref(), enrich.as_ref(), quiet);
        }
    }
    Ok(())
}

fn resolve_list_grep<'a>(name: Option<&'a str>, grep: Option<&'a str>) -> Result<Option<&'a str>> {
    match (name, grep) {
        (Some(_), Some(_)) => bail!("`po lookup <name> --list` cannot combine with `--grep`"),
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

/// No exact behavior/reference hit: fall back to reference prefix matches so a
/// partial name (`math.`) still surfaces the namespace.
fn prefix_fallback(name: &str, format: ResolvedFormat, quiet: bool) -> Result<()> {
    let prefix_hits = reference::prefix_search(name);
    if prefix_hits.is_empty() {
        bail!("no match for `{name}`");
    }
    match format {
        ResolvedFormat::Json => {
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

/// Merged JSON: the structured behavior object plus a `reference` object that
/// folds the parsed enrichment together. Either side may be null.
fn print_merged_json(
    name: &str,
    bhv: Option<&behavior::Behavior>,
    enrich: Option<&reference::Enrichment>,
) -> Result<()> {
    let reference = enrich.map(|e| {
        serde_json::json!({
            "category": e.category,
            "remarks": e.remarks,
            "see_also": e.see_also,
            "arguments": e.arguments,
        })
    });
    print_json(&serde_json::json!({
        "query": name,
        "exact": true,
        "behavior": bhv,
        "reference": reference,
    }))
}

fn print_merged_text(
    bhv: Option<&behavior::Behavior>,
    refent: Option<&reference::Entry>,
    enrich: Option<&reference::Enrichment>,
    quiet: bool,
) {
    match bhv {
        Some(b) => {
            let arg_prose: &[reference::ArgProse] = match enrich {
                Some(e) => &e.arguments,
                None => &[],
            };
            print_behavior_text(b, arg_prose);
            // Append only what the structured block lacks: Remarks + See also.
            if let Some(e) = enrich {
                if let Some(remarks) = &e.remarks {
                    println!("  remarks: {remarks}");
                }
                if !e.see_also.is_empty() {
                    println!("  see also: {}", e.see_also.join(", "));
                }
            }
        }
        // No structured data (e.g. an Operators entry): print the reference
        // prose verbatim - it already contains its own Remarks / See also.
        None => {
            if let Some(entry) = refent {
                if !quiet {
                    println!("{} ({})\n", entry.name, entry.category);
                }
                println!("{}", entry.content);
            }
        }
    }
}

/// Prose lookup for a parameter name from the reference `Arguments` section.
fn prose_for<'a>(args: &'a [reference::ArgProse], name: &str) -> Option<&'a str> {
    args.iter()
        .find(|a| a.name == name)
        .map(|a| a.description.as_str())
}

fn print_behavior_text(b: &behavior::Behavior, arg_prose: &[reference::ArgProse]) {
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
            if let Some(deprecated) = &f.deprecated {
                println!("  deprecated: {deprecated}");
            }
            if !f.parameters.is_empty() {
                println!("  parameters:");
                for p in &f.parameters {
                    let req = if p.required { "required" } else { "optional" };
                    println!("    - {} : {} ({req})", p.name, p.ty);
                    if let Some(prose) = prose_for(arg_prose, &p.name) {
                        println!("        {prose}");
                    }
                    if let Some(default) = &p.default {
                        println!("        default: {default}");
                    }
                    if !p.allowed_values.is_empty() {
                        println!("        allowed: {}", p.allowed_values.join(", "));
                    }
                    match (p.min, p.max) {
                        (Some(min), Some(max)) => println!("        range: {min} .. {max}"),
                        (Some(min), None) => println!("        min: {min}"),
                        (None, Some(max)) => println!("        max: {max}"),
                        (None, None) => {}
                    }
                }
            }
            if !f.overloads.is_empty() {
                println!("  overloads: {}", f.overloads.len());
                for (i, o) in f.overloads.iter().enumerate() {
                    let sig = o
                        .parameters
                        .iter()
                        .map(|p| format!("{}: {}", p.name, p.ty))
                        .collect::<Vec<_>>()
                        .join(", ");
                    println!("    {}. ({sig}) -> {}", i + 1, o.returns);
                }
            }
            let mut flag_notes = Vec::new();
            if f.flags.top_level_only {
                flag_notes.push("top-level only".to_string());
            }
            if f.flags.series_returning {
                flag_notes.push("series-returning".to_string());
            }
            if f.flags.variadic {
                let bounds = match (f.flags.min_args, f.flags.max_args) {
                    (Some(min), Some(max)) => format!(" ({min}..{max} args)"),
                    (Some(min), None) => format!(" (min {min} args)"),
                    (None, Some(max)) => format!(" (max {max} args)"),
                    (None, None) => String::new(),
                };
                flag_notes.push(format!("variadic{bounds}"));
            }
            if !flag_notes.is_empty() {
                println!("  flags: {}", flag_notes.join(", "));
            }
            if let Some(poly) = &f.flags.polymorphic {
                println!("  polymorphic: yes ({poly})");
                if let Some(rtp) = &f.flags.return_type_param {
                    println!("    return-type follows parameter: {rtp}");
                }
            } else if let Some(rtp) = &f.flags.return_type_param {
                println!("  return-type follows parameter: {rtp}");
            }
            print_examples(&f.examples);
        }
        behavior::Behavior::Variable(v) => {
            println!("variable {}", v.name);
            if let Some(ns) = &v.namespace {
                println!("  namespace: {ns}");
            }
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
            if let Some(desc) = &c.description
                && !desc.is_empty()
            {
                println!("  description: {desc}");
            }
        }
        behavior::Behavior::Keyword(k) => {
            println!("keyword {}", k.name);
        }
        behavior::Behavior::Type(t) => {
            println!("type {}", t.name);
            if let Some(ns) = &t.namespace {
                println!("  namespace: {ns}");
            }
            println!("  classification: {}", t.classification);
            if !t.description.is_empty() {
                println!("  description: {}", t.description);
            }
            if !t.fields.is_empty() {
                println!("  fields:");
                for field in &t.fields {
                    println!("    - {} : {}", field.name, field.ty);
                    if !field.description.is_empty() {
                        println!("        {}", field.description);
                    }
                }
            }
            print_examples(&t.examples);
        }
        behavior::Behavior::Annotation(a) => {
            println!("annotation {}", a.name);
            if let Some(syntax) = &a.syntax {
                println!("  syntax: {syntax}");
            }
            if !a.description.is_empty() {
                println!("  description: {}", a.description);
            }
            print_examples(&a.examples);
        }
    }
}

/// Shared "examples" renderer used by the function / type / annotation views.
fn print_examples(examples: &[String]) {
    if examples.is_empty() {
        return;
    }
    let n = examples.len();
    let label = if n == 1 { "example" } else { "examples" };
    println!("  {label}: {n}");
    for (i, ex) in examples.iter().enumerate() {
        if n > 1 {
            println!("    --- example {} ---", i + 1);
        }
        for line in ex.lines() {
            println!("    {line}");
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

    #[test]
    fn prose_for_matches_by_name() {
        let args = vec![
            reference::ArgProse {
                name: "source".to_string(),
                description: "Series of values to process.".to_string(),
            },
            reference::ArgProse {
                name: "length".to_string(),
                description: "Number of bars.".to_string(),
            },
        ];
        assert_eq!(prose_for(&args, "length"), Some("Number of bars."));
        assert_eq!(prose_for(&args, "missing"), None);
    }
}
