use anyhow::{Result, bail};
use pine_oracle::{behavior, suggest};

use crate::output::{Style, is_catalog_request, print_catalog};

/// How many "did you mean ...?" suggestions to offer on a lookup miss.
const SUGGEST_LIMIT: usize = 8;

/// `po lookup` is the single "describe this identifier" verb. It reads the
/// structured pine-data `behavior` surface: signatures, typed params with
/// per-argument prose, polymorphism + deprecation flags, plus the prose
/// sub-sections (`remarks`, `returnsDescription`, `seeAlso`) and the operator
/// catalog. One source - no markdown.
///
/// `--list` / `--kind` / `--grep` browse the behavior catalog.
pub(crate) fn run(
    name: Option<&str>,
    list: bool,
    kind: Option<&str>,
    grep: Option<&str>,
    _style: Style,
    quiet: bool,
) -> Result<()> {
    if is_catalog_request(kind) {
        return print_kind_catalog(quiet);
    }
    if list {
        let grep = resolve_list_grep(name, grep)?;
        return print_behavior_list(kind, grep);
    }
    if kind.is_some() || grep.is_some() {
        bail!("`po lookup --kind/--grep` requires `--list`");
    }
    let Some(name) = name else {
        bail!("`po lookup` requires a name or `--list`");
    };

    // A name can resolve in several catalogs at once (e.g. `na` is a function,
    // a variable, and a keyword; `time` is a function and a variable). Render
    // every meaning rather than silently picking one.
    let matches = behavior::lookup_all(name);
    if matches.is_empty() {
        return did_you_mean(name, quiet);
    }

    for (i, b) in matches.iter().enumerate() {
        if i > 0 {
            println!();
        }
        print_behavior_text(b);
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

fn print_kind_catalog(quiet: bool) -> Result<()> {
    let kinds = behavior::kind_catalog();
    print_catalog(
        &kinds,
        |k| format!("{:<9} {:>5}  {}", k.kind, k.count, k.description),
        |k| k.kind.to_string(),
        quiet,
    )
}

fn print_behavior_list(kind: Option<&str>, grep: Option<&str>) -> Result<()> {
    let entries = behavior::list(kind, grep)?;
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
    Ok(())
}

/// No exact hit: offer the closest identifier names via the BM25 suggestion
/// engine ("did you mean ...?"). This is the former standalone name-search,
/// demoted to lookup's recovery path.
fn did_you_mean(name: &str, quiet: bool) -> Result<()> {
    let hits = suggest::suggest(name, SUGGEST_LIMIT)?;
    if hits.is_empty() {
        bail!("no match for `{name}`");
    }
    if !quiet {
        eprintln!("no exact match for `{name}`. did you mean:");
    }
    for s in &hits {
        println!("{}", s.name);
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
            if let Some(prose) = &f.returns_description {
                println!("      {prose}");
            }
            if let Some(deprecated) = &f.deprecated {
                println!("  deprecated: {deprecated}");
            }
            if !f.parameters.is_empty() {
                println!("  parameters:");
                for p in &f.parameters {
                    let req = if p.required { "required" } else { "optional" };
                    println!("    - {} : {} ({req})", p.name, p.ty);
                    if !p.description.is_empty() {
                        println!("        {}", p.description);
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
            print_prose(&f.remarks, &f.see_also);
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
            print_prose(&v.remarks, &v.see_also);
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
            print_prose(&c.remarks, &c.see_also);
        }
        behavior::Behavior::Keyword(k) => {
            println!("keyword {}", k.name);
            if !k.description.is_empty() {
                println!("  description: {}", k.description);
            }
            if let Some(prose) = &k.returns_description {
                println!("  returns: {prose}");
            }
            print_prose(&k.remarks, &k.see_also);
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
            print_prose(&t.remarks, &t.see_also);
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
            print_prose(&a.remarks, &a.see_also);
        }
        behavior::Behavior::Operator(o) => {
            println!("operator {}", o.name);
            if let Some(syntax) = &o.syntax {
                println!("  syntax: {syntax}");
            }
            if !o.description.is_empty() {
                println!("  description: {}", o.description);
            }
            if let Some(prose) = &o.returns_description {
                println!("  returns: {prose}");
            }
            print_examples(&o.examples);
            print_prose(&o.remarks, &o.see_also);
        }
    }
}

/// Shared "examples" renderer used by every variant.
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

/// Shared Remarks / See-also renderer - the prose sub-sections every catalog
/// can carry, now sourced straight from pine-data.
fn print_prose(remarks: &Option<String>, see_also: &[String]) {
    if let Some(remarks) = remarks {
        println!("  remarks: {remarks}");
    }
    if !see_also.is_empty() {
        println!("  see also: {}", see_also.join(", "));
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
