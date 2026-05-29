// SPDX-License-Identifier: MPL-2.0
//
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

use anyhow::{Result, bail};
use piners_syntax::{
    BuiltinsTable, FunctionParameter as SyntaxFunctionParameter, FunctionSignature,
    PolymorphismRule, ValueType,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::sync::OnceLock;

const FUNCTIONS_JSON: &str = include_str!("../../vendor/pine-data/v6/functions.json");
const VARIABLES_JSON: &str = include_str!("../../vendor/pine-data/v6/variables.json");
const CONSTANTS_JSON: &str = include_str!("../../vendor/pine-data/v6/constants.json");
const KEYWORDS_JSON: &str = include_str!("../../vendor/pine-data/v6/keywords.json");
const BEHAVIOR_JSON: &str = include_str!("../../vendor/pine-data/v6/function-behavior.json");

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
    #[serde(rename = "topLevelOnly", default)]
    pub top_level_only: bool,
    /// True when the function accepts a variable number of trailing arguments
    /// (e.g. `array.from`). Upstream key: `variadic`.
    #[serde(default)]
    pub variadic: bool,
    /// Minimum number of arguments required for variadic functions.
    /// Upstream key: `minArgs`.
    #[serde(rename = "minArgs", default)]
    pub min_args: Option<u32>,
    /// Polymorphism hint from the functions.json flags object. Distinct from
    /// the richer `RawBehaviorEntry::polymorphic` field in function-behavior.json.
    /// Example values: `"element"`. Upstream key: `polymorphic`.
    #[serde(default)]
    pub polymorphic: Option<String>,
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
    /// Human-readable description from pine-data. Empty string when the source
    /// JSON carries no description.
    pub description: String,
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
    /// Human-readable description from pine-data. Empty string when the source
    /// JSON carries no description.
    pub description: String,
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

impl Behavior {
    /// Return the `BehaviorKind` tag for this variant.
    #[must_use]
    pub fn kind(&self) -> BehaviorKind {
        match self {
            Self::Function(_) => BehaviorKind::Function,
            Self::Variable(_) => BehaviorKind::Variable,
            Self::Constant(_) => BehaviorKind::Constant,
            Self::Keyword(_) => BehaviorKind::Keyword,
        }
    }

    /// Return `true` if this is a function that carries polymorphism markers.
    ///
    /// Equivalent to `if let Behavior::Function(f) = self { f.behavior.as_ref().is_some_and(|b| b.polymorphic.is_polymorphic()) }`.
    #[must_use]
    pub fn is_polymorphic(&self) -> bool {
        if let Self::Function(f) = self {
            f.behavior
                .as_ref()
                .is_some_and(|b| b.polymorphic.is_polymorphic())
        } else {
            false
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum BehaviorKind {
    Function,
    Variable,
    Constant,
    Keyword,
}

impl BehaviorKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Function => "function",
            Self::Variable => "variable",
            Self::Constant => "constant",
            Self::Keyword => "keyword",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            Self::Function => "Built-in functions with signatures and behavior metadata",
            Self::Variable => "Built-in variables such as OHLCV series",
            Self::Constant => "Typed named constants and enum-like values",
            Self::Keyword => "Reserved Pine keywords",
        }
    }
}

impl fmt::Display for BehaviorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct BehaviorListing {
    pub kind: BehaviorKind,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub namespace: Option<String>,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub detail: String,
    pub polymorphic: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct BehaviorKindInfo {
    pub kind: BehaviorKind,
    pub description: &'static str,
    pub count: usize,
}

#[derive(Debug, Clone)]
pub struct BehaviorSearchEntry {
    pub category: &'static str,
    pub name: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct PineDataSnapshot {
    pub version: String,
    pub generated_at: String,
    pub function_count: usize,
    pub variable_count: usize,
    pub constant_count: usize,
    pub keyword_count: usize,
    pub function_behavior_count: usize,
}

// ---------- indexed lookup ----------

struct BehaviorIndex {
    functions: HashMap<String, RawFunction>,
    variables: HashMap<String, RawVariable>,
    constants: HashMap<String, RawConstant>,
    keywords: Vec<String>,
    function_behaviors: HashMap<String, RawBehaviorEntry>,
    behavior_version: String,
    behavior_generated_at: String,
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
    let behavior_version = behavior_file.version;
    let behavior_generated_at = behavior_file.generated_at;

    Ok(BehaviorIndex {
        functions: functions.into_iter().map(|f| (f.name.clone(), f)).collect(),
        variables: variables.into_iter().map(|v| (v.name.clone(), v)).collect(),
        constants: constants.into_iter().map(|c| (c.name.clone(), c)).collect(),
        keywords,
        function_behaviors: behavior_file.functions,
        behavior_version,
        behavior_generated_at,
    })
}

/// Case-insensitive scan over a `HashMap<String, V>`. Returns the first
/// value whose key matches `name` under ASCII case folding.
fn map_get_ci<'a, V>(map: &'a HashMap<String, V>, name: &str) -> Option<&'a V> {
    // Fast path: exact key hit (the common case for correctly-cased input).
    if let Some(v) = map.get(name) {
        return Some(v);
    }
    // Slow path: linear scan for a case-insensitive match.
    let lower = name.to_ascii_lowercase();
    map.iter()
        .find(|(k, _)| k.to_ascii_lowercase() == lower)
        .map(|(_, v)| v)
}

/// First-hit lookup across functions, variables, constants, keywords.
/// Case-insensitive: `lookup("CLOSE")` and `lookup("close")` both work.
///
/// Generic-placeholder fallback: pine-data stores generic constructor functions
/// under names like `array.new<type>`, `matrix.new<type>`, `map.new<type,type>`.
/// When an exact-name miss occurs, `lookup` probes the two conventional suffixes
/// so that `lookup("array.new")` resolves to `array.new<type>`.
pub fn lookup(name: &str) -> Option<Behavior> {
    let idx = index();
    // Primary lookup: functions / variables / constants / keywords.
    if let Some(result) = lookup_in(idx, name) {
        return Some(result);
    }
    if idx.keywords.iter().any(|k| k.eq_ignore_ascii_case(name)) {
        return Some(Behavior::Keyword(KeywordBehavior {
            name: name.to_string(),
        }));
    }
    // Generic-placeholder fallback: try conventional suffixes in decreasing
    // specificity order. `map.new<type,type>` must be tried before `<type>`.
    for suffix in &["<type,type>", "<type>"] {
        let candidate = format!("{name}{suffix}");
        if let Some(result) = lookup_in(idx, &candidate) {
            return Some(result);
        }
    }
    None
}

/// Inner lookup over a pre-fetched index: functions, variables, constants only
/// (not keywords, which require the caller to own the keyword name string).
fn lookup_in(idx: &BehaviorIndex, name: &str) -> Option<Behavior> {
    if let Some(f) = map_get_ci(&idx.functions, name) {
        let behavior = map_get_ci(&idx.function_behaviors, &f.name).cloned();
        return Some(Behavior::Function(FunctionBehavior {
            name: f.name.clone(),
            namespace: f.namespace.clone(),
            syntax: f.syntax.clone(),
            returns: f.returns.clone(),
            description: f.description.clone(),
            parameters: f.parameters.clone(),
            examples: f.examples.clone(),
            flags: f.flags.clone().unwrap_or(FunctionFlags {
                top_level_only: false,
                variadic: false,
                min_args: None,
                polymorphic: None,
            }),
            behavior,
        }));
    }
    if let Some(v) = map_get_ci(&idx.variables, name) {
        return Some(Behavior::Variable(VariableBehavior {
            name: v.name.clone(),
            ty: v.ty.clone(),
            qualifier: v.qualifier.clone(),
            description: v.description.clone(),
        }));
    }
    if let Some(c) = map_get_ci(&idx.constants, name) {
        return Some(Behavior::Constant(ConstantBehavior {
            name: c.name.clone(),
            namespace: c.namespace.clone(),
            short_name: c.short_name.clone(),
            ty: c.ty.clone(),
        }));
    }
    None
}

pub fn snapshot() -> PineDataSnapshot {
    let idx = index();
    PineDataSnapshot {
        version: idx.behavior_version.clone(),
        generated_at: idx.behavior_generated_at.clone(),
        function_count: idx.functions.len(),
        variable_count: idx.variables.len(),
        constant_count: idx.constants.len(),
        keyword_count: idx.keywords.len(),
        function_behavior_count: idx.function_behaviors.len(),
    }
}

pub fn kind_catalog() -> Vec<BehaviorKindInfo> {
    let snapshot = snapshot();
    vec![
        BehaviorKindInfo {
            kind: BehaviorKind::Function,
            description: BehaviorKind::Function.description(),
            count: snapshot.function_count,
        },
        BehaviorKindInfo {
            kind: BehaviorKind::Variable,
            description: BehaviorKind::Variable.description(),
            count: snapshot.variable_count,
        },
        BehaviorKindInfo {
            kind: BehaviorKind::Constant,
            description: BehaviorKind::Constant.description(),
            count: snapshot.constant_count,
        },
        BehaviorKindInfo {
            kind: BehaviorKind::Keyword,
            description: BehaviorKind::Keyword.description(),
            count: snapshot.keyword_count,
        },
    ]
}

/// Returns `true` when `kind` is the catalog sentinel `"?"`.
/// Thin delegate kept for library consumers that import `pine_oracle::behavior`
/// directly; the binary uses `output::is_catalog_request` instead.
pub fn is_kind_catalog_request(kind: &str) -> bool {
    kind == "?"
}

pub fn list(kind_filter: Option<&str>, grep: Option<&str>) -> Result<Vec<BehaviorListing>> {
    let filter = kind_filter.map(parse_behavior_kind).transpose()?;
    let needle = grep.map(str::to_ascii_lowercase);
    let idx = index();
    let mut out = Vec::new();

    if filter.is_none_or(|kind| kind == BehaviorKind::Function) {
        for function in idx.functions.values() {
            out.push(BehaviorListing {
                kind: BehaviorKind::Function,
                name: function.name.clone(),
                namespace: function.namespace.clone(),
                detail: function.syntax.clone(),
                polymorphic: idx
                    .function_behaviors
                    .get(&function.name)
                    .is_some_and(|behavior| behavior.polymorphic.is_polymorphic()),
            });
        }
    }
    if filter.is_none_or(|kind| kind == BehaviorKind::Variable) {
        for variable in idx.variables.values() {
            out.push(BehaviorListing {
                kind: BehaviorKind::Variable,
                name: variable.name.clone(),
                namespace: None,
                detail: type_detail(&variable.ty, &variable.qualifier),
                polymorphic: false,
            });
        }
    }
    if filter.is_none_or(|kind| kind == BehaviorKind::Constant) {
        for constant in idx.constants.values() {
            out.push(BehaviorListing {
                kind: BehaviorKind::Constant,
                name: constant.name.clone(),
                namespace: constant.namespace.clone(),
                detail: constant.ty.clone(),
                polymorphic: false,
            });
        }
    }
    if filter.is_none_or(|kind| kind == BehaviorKind::Keyword) {
        for keyword in &idx.keywords {
            out.push(BehaviorListing {
                kind: BehaviorKind::Keyword,
                name: keyword.clone(),
                namespace: None,
                detail: String::new(),
                polymorphic: false,
            });
        }
    }

    if let Some(needle) = needle {
        out.retain(|entry| behavior_listing_matches(entry, &needle));
    }
    // HashMap iteration order is non-deterministic (hashbrown random seed).
    // The sort below makes both text and JSON output stable regardless of
    // which order the entries were pushed into `out`.
    out.sort_by(|a, b| a.kind.cmp(&b.kind).then_with(|| a.name.cmp(&b.name)));
    Ok(out)
}

fn type_detail(ty: &str, qualifier: &str) -> String {
    if qualifier.is_empty() {
        ty.to_string()
    } else {
        format!("{qualifier} {ty}")
    }
}

fn behavior_listing_matches(entry: &BehaviorListing, needle: &str) -> bool {
    entry.name.to_ascii_lowercase().contains(needle)
        || entry
            .namespace
            .as_ref()
            .is_some_and(|namespace| namespace.to_ascii_lowercase().contains(needle))
        || entry.detail.to_ascii_lowercase().contains(needle)
}

fn parse_behavior_kind(raw: &str) -> Result<BehaviorKind> {
    match raw.to_ascii_lowercase().as_str() {
        "function" | "functions" => Ok(BehaviorKind::Function),
        "variable" | "variables" => Ok(BehaviorKind::Variable),
        "constant" | "constants" => Ok(BehaviorKind::Constant),
        "keyword" | "keywords" => Ok(BehaviorKind::Keyword),
        _ => bail!(
            "unknown behavior kind `{raw}`; expected one of: function, variable, constant, keyword"
        ),
    }
}

pub fn search_entries() -> Vec<BehaviorSearchEntry> {
    let idx = index();
    let mut out = Vec::new();
    for function in idx.functions.values() {
        out.push(BehaviorSearchEntry {
            category: "Function",
            name: function.name.clone(),
            content: function_search_content(function, idx.function_behaviors.get(&function.name)),
        });
    }
    for variable in idx.variables.values() {
        out.push(BehaviorSearchEntry {
            category: "Variable",
            name: variable.name.clone(),
            content: variable_search_content(variable),
        });
    }
    for constant in idx.constants.values() {
        out.push(BehaviorSearchEntry {
            category: "Constant",
            name: constant.name.clone(),
            content: constant_search_content(constant),
        });
    }
    for keyword in &idx.keywords {
        out.push(BehaviorSearchEntry {
            category: "Keyword",
            name: keyword.clone(),
            content: "Reserved Pine keyword.".to_string(),
        });
    }
    out.sort_by(|a, b| a.category.cmp(b.category).then_with(|| a.name.cmp(&b.name)));
    out
}

fn function_search_content(function: &RawFunction, behavior: Option<&RawBehaviorEntry>) -> String {
    let mut parts = Vec::new();
    if !function.syntax.is_empty() {
        parts.push(format!("Syntax: {}", function.syntax));
    }
    if !function.returns.is_empty() {
        parts.push(format!("Returns: {}", function.returns));
    }
    if !function.description.is_empty() {
        parts.push(function.description.clone());
    }
    for param in &function.parameters {
        let required = if param.required {
            "required"
        } else {
            "optional"
        };
        let mut line = format!("Parameter {}: {} ({required})", param.name, param.ty);
        if !param.description.is_empty() {
            line.push_str(". ");
            line.push_str(&param.description);
        }
        parts.push(line);
    }
    if let Some(flags) = function.flags.as_ref() {
        if flags.top_level_only {
            parts.push("Top-level only.".to_string());
        }
        if flags.variadic {
            let min = flags
                .min_args
                .map(|n| format!(" (minimum {n} argument(s))"))
                .unwrap_or_default();
            parts.push(format!("Variadic{min}."));
        }
        if let Some(poly_hint) = &flags.polymorphic {
            parts.push(format!("Flags polymorphic: {poly_hint}."));
        }
    }
    if let Some(behavior) = behavior {
        if let Some(detail) = behavior.polymorphic.detail() {
            parts.push(format!(
                "Polymorphic return: {}",
                detail.strategy.as_deref().unwrap_or("dependent-on-input")
            ));
            if let Some(param) = &detail.return_type_param {
                parts.push(format!("Return type depends on parameter: {param}"));
            }
            if !detail.allowed_types.is_empty() {
                parts.push(format!(
                    "Allowed types: {}",
                    detail.allowed_types.join(", ")
                ));
            }
        }
        if let Some(ordering) = &behavior.argument_ordering {
            parts.push(format!("Argument ordering: {ordering}"));
        }
        if !behavior.observed_return_types.is_empty() {
            parts.push(format!(
                "Observed return types: {}",
                behavior.observed_return_types.join(", ")
            ));
        }
        if let Some(reason) = &behavior.reason {
            parts.push(reason.clone());
        }
    }
    for example in &function.examples {
        parts.push(format!("Example:\n{example}"));
    }
    parts.join("\n")
}

fn variable_search_content(variable: &RawVariable) -> String {
    let mut parts = vec![
        format!("Type: {}", variable.ty),
        format!("Qualifier: {}", variable.qualifier),
    ];
    if !variable.description.is_empty() {
        parts.push(variable.description.clone());
    }
    parts.join("\n")
}

fn constant_search_content(constant: &RawConstant) -> String {
    let mut parts = vec![format!("Type: {}", constant.ty)];
    if let Some(namespace) = &constant.namespace {
        parts.push(format!("Namespace: {namespace}"));
    }
    if let Some(short_name) = &constant.short_name {
        parts.push(format!("Short name: {short_name}"));
    }
    parts.join("\n")
}

/// Built-in surface for piners-syntax validation. piners-runtime is the
/// primary authority; pine-oracle's vendored pine-tools JSON fills symbols
/// not represented by the runtime table yet.
pub fn syntax_builtins() -> &'static BuiltinsTable {
    static BUILTINS: OnceLock<BuiltinsTable> = OnceLock::new();
    BUILTINS.get_or_init(build_syntax_builtins)
}

fn build_syntax_builtins() -> BuiltinsTable {
    let idx = index();
    let mut table = piners_runtime::build_builtins_table();

    for function in idx.functions.values() {
        if function.name.contains('<') {
            continue;
        }
        for signature in syntax_function_signatures(function) {
            let signatures = table.functions.entry(function.name.clone()).or_default();
            if !signature_shape_exists(signatures, &signature) {
                signatures.push(signature);
            }
        }
    }

    for variable in idx.variables.values() {
        table
            .variables
            .entry(variable.name.clone())
            .or_insert_with(|| parse_value_type(&variable.ty));
    }
    for constant in idx.constants.values() {
        table
            .constants
            .entry(constant.name.clone())
            .or_insert_with(|| parse_value_type(&constant.ty));
    }
    for keyword in &idx.keywords {
        table.keywords.insert(keyword.clone());
    }
    for (name, behavior) in &idx.function_behaviors {
        if let Some(rule) = syntax_polymorphism_rule(behavior) {
            table.polymorphism.entry(name.clone()).or_insert(rule);
        }
    }

    table
}

fn signature_shape_exists(signatures: &[FunctionSignature], signature: &FunctionSignature) -> bool {
    signatures.iter().any(|existing| {
        existing.params == signature.params && existing.return_type == signature.return_type
    })
}

fn syntax_function_signatures(function: &RawFunction) -> Vec<FunctionSignature> {
    let return_type = parse_value_type(&function.returns);
    let mut variants = vec![Vec::new()];
    for param in &function.parameters {
        let alternatives = parse_param_value_type_alternatives(&param.ty);
        let mut next = Vec::new();
        for existing in &variants {
            for value_type in &alternatives {
                let mut params = existing.clone();
                params.push(SyntaxFunctionParameter {
                    name: param.name.clone(),
                    value_type: value_type.clone(),
                    optional: !param.required,
                });
                next.push(params);
            }
        }
        variants = next;
    }
    variants
        .into_iter()
        .map(|params| FunctionSignature {
            params,
            return_type: return_type.clone(),
            stateful: false,
            stub: false,
        })
        .collect()
}

fn syntax_polymorphism_rule(behavior: &RawBehaviorEntry) -> Option<PolymorphismRule> {
    let detail = behavior.polymorphic.detail()?;
    match detail.strategy.as_deref() {
        Some("dependent-on-input") | None => Some(PolymorphismRule::Identity),
        Some("numeric") => Some(PolymorphismRule::Numeric),
        Some("collection-element") => Some(PolymorphismRule::CollectionElement),
        Some(strategy @ ("array_new" | "map_keys" | "map_values")) => {
            Some(PolymorphismRule::Custom(strategy.to_string()))
        }
        // piners-syntax treats unknown custom rules as fallback-to-static.
        // Do not install inert rules from pine-data until the checker knows
        // how to interpret them.
        Some(_) => None,
    }
}

fn parse_param_value_type_alternatives(raw: &str) -> Vec<ValueType> {
    let ty = strip_qualifier(raw.trim());
    let parts = split_top_level(ty, '/');
    if parts.len() > 1 {
        let mut values = Vec::new();
        for part in parts {
            let Some(value_type) = parse_primitive_value_type(part) else {
                return vec![parse_value_type(raw)];
            };
            if !values.contains(&value_type) {
                values.push(value_type);
            }
        }
        return values;
    }
    vec![parse_value_type(raw)]
}

fn parse_value_type(raw: &str) -> ValueType {
    let ty = raw.trim();
    if ty.is_empty() || ty.eq_ignore_ascii_case("void") {
        return ValueType::Unknown;
    }

    let ty = strip_qualifier(ty);
    if let Some((head, inner)) = split_generic(ty) {
        return match head {
            "series" => parse_value_type(inner),
            "array" => ValueType::Array(Box::new(parse_value_type(inner))),
            "matrix" => ValueType::Matrix(Box::new(parse_value_type(inner))),
            "map" => {
                let args = split_top_level(inner, ',');
                if args.len() == 2 {
                    ValueType::Map(
                        Box::new(parse_value_type(args[0])),
                        Box::new(parse_value_type(args[1])),
                    )
                } else {
                    ValueType::Map(Box::new(ValueType::Unknown), Box::new(ValueType::Unknown))
                }
            }
            _ => ValueType::Unknown,
        };
    }

    if ty.contains('/') {
        let parts = split_top_level(ty, '/');
        if parts.iter().all(|part| matches!(*part, "int" | "float")) {
            return ValueType::Float;
        }
        return ValueType::Unknown;
    }

    if matches!(ty, "source" | "series" | "literal") {
        return ValueType::Unknown;
    }

    match ty {
        "int" | "float" | "bool" | "string" | "color" | "line" | "label" | "box" | "table"
        | "polyline" | "linefill" | "chart" | "chart.point" => {
            parse_primitive_value_type(ty).unwrap_or(ValueType::Unknown)
        }
        "array" => ValueType::Array(Box::new(ValueType::Unknown)),
        "matrix" => ValueType::Matrix(Box::new(ValueType::Unknown)),
        "map" => ValueType::Map(Box::new(ValueType::Unknown), Box::new(ValueType::Unknown)),
        _ => ValueType::Unknown,
    }
}

fn parse_primitive_value_type(ty: &str) -> Option<ValueType> {
    match ty.trim() {
        "int" => Some(ValueType::Int),
        "float" => Some(ValueType::Float),
        "bool" => Some(ValueType::Bool),
        "string" => Some(ValueType::String),
        "color" => Some(ValueType::Color),
        "line" => Some(ValueType::Line),
        "label" => Some(ValueType::Label),
        "box" => Some(ValueType::Box),
        "table" => Some(ValueType::Table),
        "polyline" => Some(ValueType::Polyline),
        "linefill" => Some(ValueType::Linefill),
        "chart" => Some(ValueType::Chart),
        "chart.point" => Some(ValueType::ChartPoint),
        _ => None,
    }
}

fn strip_qualifier(ty: &str) -> &str {
    for qualifier in ["const ", "input ", "simple ", "series "] {
        if let Some(rest) = ty.strip_prefix(qualifier) {
            return rest.trim();
        }
    }
    ty
}

fn split_generic(ty: &str) -> Option<(&str, &str)> {
    let open = ty.find('<')?;
    let close = ty.rfind('>')?;
    if close <= open {
        return None;
    }
    let head = ty[..open].trim();
    let inner = ty[open + 1..close].trim();
    Some((head, inner))
}

fn split_top_level(input: &str, delimiter: char) -> Vec<&str> {
    let mut parts = Vec::new();
    let mut depth = 0usize;
    let mut start = 0usize;
    for (idx, ch) in input.char_indices() {
        match ch {
            '<' => depth += 1,
            '>' => depth = depth.saturating_sub(1),
            _ if ch == delimiter && depth == 0 => {
                parts.push(input[start..idx].trim());
                start = idx + ch.len_utf8();
            }
            _ => {}
        }
    }
    parts.push(input[start..].trim());
    parts
}

#[cfg(test)]
mod tests {
    include!("tests.rs");
}
