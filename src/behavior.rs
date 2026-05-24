// Structured behaviour lookup for the public Pine v6 surface.
//
// Inputs are pine-tools' JSON exports under vendor/pine-data/v6/:
//   - functions.json         (signature, params, returns, flags)
//   - variables.json         (type + qualifier for built-ins)
//   - constants.json         (typed constants like color.red)
//   - keywords.json          (reserved keyword list)
//   - function-behavior.json (polymorphism + argument-ordering markers)
//
// The five files are parsed once via OnceLock into a `BehaviorIndex` keyed
// by symbol name. `lookup(name)` returns the first match across functions,
// variables, constants, and keywords (in that order), or `None`.
//
// The pine-tools upstream is mid-rescrape; deserialization is lenient
// (serde ignores unknown fields by default, plus serde(default) on
// optional fields) so a schema tweak upstream doesn't break the binary.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::OnceLock;

const FUNCTIONS_JSON: &str = include_str!("../vendor/pine-data/v6/functions.json");
const VARIABLES_JSON: &str = include_str!("../vendor/pine-data/v6/variables.json");
const CONSTANTS_JSON: &str = include_str!("../vendor/pine-data/v6/constants.json");
const KEYWORDS_JSON: &str = include_str!("../vendor/pine-data/v6/keywords.json");
const BEHAVIOR_JSON: &str = include_str!("../vendor/pine-data/v6/function-behavior.json");

// ---------- raw types (mirror the JSON 1:1) ----------

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FunctionParameter {
    pub name: String,
    #[serde(rename = "type", default)]
    pub ty: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub required: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FunctionFlags {
    #[serde(default)]
    pub top_level_only: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RawFunction {
    pub name: String,
    #[serde(default)]
    pub namespace: Option<String>,
    #[serde(default)]
    pub syntax: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub parameters: Vec<FunctionParameter>,
    #[serde(default)]
    pub returns: String,
    /// Code examples preserving original newlines + indentation. Upstream
    /// recently switched from a single `example: string` to a multi-element
    /// `examples: string[]` after confirming TV's docs ship multiple sibling
    /// `<pre>` blocks per function.
    #[serde(default)]
    pub examples: Vec<String>,
    #[serde(default)]
    pub flags: Option<FunctionFlags>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RawVariable {
    pub name: String,
    #[serde(rename = "type", default)]
    pub ty: String,
    #[serde(default)]
    pub qualifier: String,
    #[serde(default)]
    pub description: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RawConstant {
    pub name: String,
    #[serde(default)]
    pub namespace: Option<String>,
    #[serde(rename = "shortName", default)]
    pub short_name: Option<String>,
    #[serde(rename = "type", default)]
    pub ty: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PolymorphicDetail {
    #[serde(rename = "returnTypeParam", default)]
    pub return_type_param: Option<String>,
    #[serde(default)]
    pub strategy: Option<String>,
    #[serde(rename = "observedMappings", default)]
    pub observed_mappings: HashMap<String, String>,
    #[serde(rename = "allowedTypes", default)]
    pub allowed_types: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum PolymorphicField {
    /// `false` in JSON
    Static(bool),
    /// `{ returnTypeParam, strategy, observedMappings, allowedTypes }`
    Dynamic(PolymorphicDetail),
}

impl PolymorphicField {
    pub fn is_polymorphic(&self) -> bool {
        matches!(self, PolymorphicField::Dynamic(_))
    }
    pub fn detail(&self) -> Option<&PolymorphicDetail> {
        match self {
            PolymorphicField::Dynamic(d) => Some(d),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RawBehaviorEntry {
    pub polymorphic: PolymorphicField,
    #[serde(rename = "argumentOrdering", default)]
    pub argument_ordering: Option<String>,
    #[serde(rename = "observedReturnTypes", default)]
    pub observed_return_types: Vec<String>,
    #[serde(default)]
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct RawBehaviorFile {
    #[serde(default)]
    version: String,
    #[serde(rename = "generatedAt", default)]
    generated_at: String,
    #[serde(default)]
    functions: HashMap<String, RawBehaviorEntry>,
}

// ---------- merged public view ----------

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
#[allow(clippy::large_enum_variant)] // FunctionBehavior dominates; size is fine for a CLI.
pub enum Behavior {
    Function(FunctionBehavior),
    Variable(VariableBehavior),
    Constant(ConstantBehavior),
    Keyword(KeywordBehavior),
}

#[derive(Debug, Clone, Serialize)]
pub struct FunctionBehavior {
    pub name: String,
    pub namespace: Option<String>,
    pub syntax: String,
    pub returns: String,
    pub parameters: Vec<FunctionParameter>,
    pub examples: Vec<String>,
    pub flags: FunctionFlags,
    /// Present only when behavior data covers this function.
    pub behavior: Option<RawBehaviorEntry>,
}

#[derive(Debug, Clone, Serialize)]
pub struct VariableBehavior {
    pub name: String,
    pub ty: String,
    pub qualifier: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ConstantBehavior {
    pub name: String,
    pub namespace: Option<String>,
    pub short_name: Option<String>,
    pub ty: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct KeywordBehavior {
    pub name: String,
}

// ---------- indexed lookup ----------

struct BehaviorIndex {
    functions: HashMap<String, RawFunction>,
    variables: HashMap<String, RawVariable>,
    constants: HashMap<String, RawConstant>,
    keywords: Vec<String>,
    function_behaviors: HashMap<String, RawBehaviorEntry>,
}

fn index() -> &'static BehaviorIndex {
    static INDEX: OnceLock<BehaviorIndex> = OnceLock::new();
    INDEX.get_or_init(|| build_index().expect("vendored pine-data must parse"))
}

fn build_index() -> anyhow::Result<BehaviorIndex> {
    let functions: Vec<RawFunction> = serde_json::from_str(FUNCTIONS_JSON)?;
    let variables: Vec<RawVariable> = serde_json::from_str(VARIABLES_JSON)?;
    let constants: Vec<RawConstant> = serde_json::from_str(CONSTANTS_JSON)?;
    let keywords: Vec<String> = serde_json::from_str(KEYWORDS_JSON)?;
    let behavior_file: RawBehaviorFile = serde_json::from_str(BEHAVIOR_JSON)?;
    // Track upstream snapshot metadata as eprintln-debuggable but otherwise unused.
    let _ = (&behavior_file.version, &behavior_file.generated_at);

    Ok(BehaviorIndex {
        functions: functions.into_iter().map(|f| (f.name.clone(), f)).collect(),
        variables: variables.into_iter().map(|v| (v.name.clone(), v)).collect(),
        constants: constants.into_iter().map(|c| (c.name.clone(), c)).collect(),
        keywords,
        function_behaviors: behavior_file.functions,
    })
}

/// First-hit lookup across functions, variables, constants, keywords.
pub fn lookup(name: &str) -> Option<Behavior> {
    let idx = index();

    if let Some(f) = idx.functions.get(name) {
        let behavior = idx.function_behaviors.get(name).cloned();
        return Some(Behavior::Function(FunctionBehavior {
            name: f.name.clone(),
            namespace: f.namespace.clone(),
            syntax: f.syntax.clone(),
            returns: f.returns.clone(),
            parameters: f.parameters.clone(),
            examples: f.examples.clone(),
            flags: f.flags.clone().unwrap_or(FunctionFlags {
                top_level_only: false,
            }),
            behavior,
        }));
    }
    if let Some(v) = idx.variables.get(name) {
        return Some(Behavior::Variable(VariableBehavior {
            name: v.name.clone(),
            ty: v.ty.clone(),
            qualifier: v.qualifier.clone(),
        }));
    }
    if let Some(c) = idx.constants.get(name) {
        return Some(Behavior::Constant(ConstantBehavior {
            name: c.name.clone(),
            namespace: c.namespace.clone(),
            short_name: c.short_name.clone(),
            ty: c.ty.clone(),
        }));
    }
    if idx.keywords.iter().any(|k| k == name) {
        return Some(Behavior::Keyword(KeywordBehavior {
            name: name.to_string(),
        }));
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn function_lookup_carries_behavior_when_present() {
        let b = lookup("input").expect("input must exist");
        match b {
            Behavior::Function(f) => {
                assert_eq!(f.name, "input");
                let beh = f.behavior.expect("input is polymorphic per behavior.json");
                assert!(beh.polymorphic.is_polymorphic());
            }
            other => panic!("expected Function, got {other:?}"),
        }
    }

    #[test]
    fn non_polymorphic_function_has_static_polymorphic_field() {
        let b = lookup("plot").expect("plot must exist");
        match b {
            Behavior::Function(f) => {
                if let Some(beh) = f.behavior {
                    assert!(!beh.polymorphic.is_polymorphic());
                }
            }
            other => panic!("expected Function, got {other:?}"),
        }
    }

    #[test]
    fn variable_lookup_carries_qualifier() {
        let b = lookup("close").expect("close must exist");
        match b {
            Behavior::Variable(v) => {
                assert_eq!(v.qualifier, "series");
                assert!(v.ty.contains("float"));
            }
            other => panic!("expected Variable, got {other:?}"),
        }
    }

    #[test]
    fn constant_lookup_works() {
        let b = lookup("color.red").or_else(|| lookup("adjustment.none"));
        assert!(b.is_some(), "at least one known constant should be found");
    }

    #[test]
    fn keyword_lookup_works() {
        let b = lookup("var").expect("var keyword must exist");
        assert!(matches!(b, Behavior::Keyword(_)));
    }

    #[test]
    fn unknown_name_is_none() {
        assert!(lookup("definitely-not-a-pine-symbol").is_none());
    }
}
