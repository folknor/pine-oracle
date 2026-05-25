use anyhow::{Result, bail};
use pine_cli::behavior;

use crate::output::{ResolvedFormat, print_json};

pub(crate) fn run(name: &str, format: ResolvedFormat) -> Result<()> {
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

fn print_behavior_text(b: &behavior::Behavior) {
    match b {
        behavior::Behavior::Function(f) => {
            println!("function {}", f.name);
            if let Some(ns) = &f.namespace {
                println!("  namespace: {ns}");
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
