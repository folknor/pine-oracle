// SPDX-License-Identifier: MPL-2.0
//
// Structured behaviour lookup for the public Pine v6 surface.
//
// Inputs are pine-tools' JSON exports under vendor/pine-data/v6/:
//   - functions.json    (signature, params, returns, flags, overloads)
//   - variables.json    (type + qualifier for built-ins)
//   - constants.json    (typed constants like color.red)
//   - keywords.json      (reserved keyword list)
//   - types.json         (built-in types: chart.point, line, array, ...)
//   - annotations.json   (compiler annotations: @version=, @param, ...)
//   - operators.json     (operators: +, -, ?:, [], +=, ...)
//
// The seven files are parsed once via OnceLock into a `BehaviorIndex` keyed by
// symbol name. `lookup(name)` returns the first match across functions,
// variables, constants, keywords, types, annotations, and operators (in that
// order), or `None`.
//
// Prose sub-sections (`remarks`, `seeAlso`, `returnsDescription`) are carried
// on every catalog that documents them - these are the fields that let
// `po lookup` render the full reference card straight from pine-data, with no
// markdown source.
//
// Polymorphism note: an earlier upstream shipped a separate
// `function-behavior.json` with rich polymorphism markers. Upstream collapsed
// that data into the `functions.json` `flags` object (`polymorphic` =
// "input" | "element" | "numeric", plus `returnTypeParam`), so the separate
// file is gone. The richer-but-redundant fields (observedMappings,
// argumentOrdering, observedReturnTypes) went away with it.
//
// Deserialization is lenient (serde ignores unknown fields by default, plus
// serde(default) on optional fields) so a schema tweak upstream doesn't break
// the binary.

use anyhow::{Result, bail};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::sync::OnceLock;

const FUNCTIONS_JSON: &str = include_str!("../../vendor/pine-data/v6/functions.json");
const VARIABLES_JSON: &str = include_str!("../../vendor/pine-data/v6/variables.json");
const CONSTANTS_JSON: &str = include_str!("../../vendor/pine-data/v6/constants.json");
const KEYWORDS_JSON: &str = include_str!("../../vendor/pine-data/v6/keywords.json");
const TYPES_JSON: &str = include_str!("../../vendor/pine-data/v6/types.json");
const ANNOTATIONS_JSON: &str = include_str!("../../vendor/pine-data/v6/annotations.json");
const OPERATORS_JSON: &str = include_str!("../../vendor/pine-data/v6/operators.json");

// The pine-data JSON files are bare arrays with no `generatedAt` envelope (the
// removed function-behavior.json used to carry one). The snapshot ref/date is
// baked here from the vendoring pass; see vendor/pine-data/v6/NOTICE.
const PINE_DATA_VERSION: &str = "6";
const PINE_DATA_SNAPSHOT: &str = "2026-08-15T00:00:00+02:00";

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
    /// Documented default value as the Pine expression from the docs (e.g. "0",
    /// "true", "na"). Dynamic/inherited defaults use a magic sentinel
    /// (CHART_SYMBOL, SCRIPT_FORMAT, "ARG:<sibling>", ...). Absent when no
    /// default is documented. Upstream key: `default`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default: Option<String>,
    /// Fixed set of accepted values when the parameter is enumerated
    /// (namespaced constants like "display.all" or quoted-string literals).
    /// Empty when the parameter is not enumerated. Upstream key: `allowedValues`.
    #[serde(
        rename = "allowedValues",
        default,
        skip_serializing_if = "Vec::is_empty"
    )]
    pub allowed_values: Vec<String>,
    /// Inclusive lower bound of an accepted numeric range, when documented.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub min: Option<f64>,
    /// Inclusive upper bound of an accepted numeric range, when documented.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max: Option<f64>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct FunctionFlags {
    #[serde(rename = "topLevelOnly", default)]
    pub top_level_only: bool,
    /// True when the function's return is a series type. Upstream key:
    /// `seriesReturning`.
    #[serde(rename = "seriesReturning", default)]
    pub series_returning: bool,
    /// True when the function accepts a variable number of trailing arguments
    /// (e.g. `array.from`). Upstream key: `variadic`.
    #[serde(default)]
    pub variadic: bool,
    /// Minimum number of arguments required for variadic functions.
    /// Upstream key: `minArgs`.
    #[serde(rename = "minArgs", default)]
    pub min_args: Option<u32>,
    /// Maximum number of arguments for variadic functions (None = unlimited).
    /// Upstream key: `maxArgs`.
    #[serde(rename = "maxArgs", default)]
    pub max_args: Option<u32>,
    /// Polymorphic return-type class, the single source of truth for
    /// polymorphism since function-behavior.json was retired:
    /// - "input":   return follows the first argument's type (nz, fixnan, input)
    /// - "element": return is the element type of a collection argument (array.get)
    /// - "numeric": return is the common numeric type of the arguments (math.max)
    ///
    /// None for monomorphic functions. Upstream key: `polymorphic`.
    #[serde(default)]
    pub polymorphic: Option<String>,
    /// Name of the parameter whose type the return type follows, for
    /// return-follows-source functions (e.g. ta.valuewhen -> "source"). Present
    /// on its own (no `polymorphic`) for the ta.* return-follows-source set, and
    /// alongside `polymorphic` for input/nz/fixnan/math.abs/math.round.
    /// Upstream key: `returnTypeParam`.
    #[serde(rename = "returnTypeParam", default)]
    pub return_type_param: Option<String>,
    /// True when the function relies on values from past executions of its
    /// own scope (the `[]` operator or internal state), so conditional or
    /// iterative calls build an inconsistent series - TV's CW10003 criterion.
    /// Covers the whole ta.* namespace plus `fixnan` and `math.sum`.
    /// Upstream key: `historyDependent`.
    #[serde(rename = "historyDependent", default)]
    pub history_dependent: bool,
}

impl FunctionFlags {
    /// True when the function carries a polymorphic return-type class.
    #[must_use]
    pub fn is_polymorphic(&self) -> bool {
        self.polymorphic.is_some()
    }
}

/// A single overload of an overloaded function. The top-level `parameters` /
/// `returns` on `RawFunction` are a merged view (param types unioned, returns
/// frozen to the first form); each overload preserves its exact, non-unioned
/// parameter types and its own return type.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RawOverload {
    #[serde(default)]
    pub parameters: Vec<FunctionParameter>,
    #[serde(default)]
    pub returns: String,
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
    #[serde(default)]
    pub flags: Option<FunctionFlags>,
    /// Per-overload signatures, present only for overloaded functions (>1 form).
    #[serde(default)]
    pub overloads: Vec<RawOverload>,
    /// Deprecation note when the reference flags the function as deprecated
    /// (rare in v6 - e.g. request.quandl). Absent otherwise.
    #[serde(default)]
    pub deprecated: Option<String>,
    /// Code examples preserving original newlines + indentation. TV's docs ship
    /// one or more sibling `<pre>` blocks per function.
    #[serde(default)]
    pub examples: Vec<String>,
    /// Prose "Returns" sentence, distinct from the typed `returns`. Upstream
    /// key: `returnsDescription`.
    #[serde(rename = "returnsDescription", default)]
    pub returns_description: Option<String>,
    /// Free-text "Remarks" caveats. Upstream key: `remarks`.
    #[serde(default)]
    pub remarks: Option<String>,
    /// "See also" cross-references as bare symbol names. Upstream key: `seeAlso`.
    #[serde(rename = "seeAlso", default)]
    pub see_also: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RawVariable {
    pub name: String,
    #[serde(default)]
    pub namespace: Option<String>,
    #[serde(rename = "type", default)]
    pub ty: String,
    #[serde(default)]
    pub qualifier: String,
    #[serde(default)]
    pub description: String,
    #[serde(rename = "returnsDescription", default)]
    pub returns_description: Option<String>,
    #[serde(default)]
    pub remarks: Option<String>,
    #[serde(rename = "seeAlso", default)]
    pub see_also: Vec<String>,
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
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub remarks: Option<String>,
    #[serde(rename = "seeAlso", default)]
    pub see_also: Vec<String>,
}

/// A field of a non-opaque built-in object type (e.g. chart.point's index /
/// time / price). Opaque ID types (line, label, box, ...) expose no fields.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RawTypeField {
    pub name: String,
    #[serde(rename = "type", default)]
    pub ty: String,
    #[serde(default)]
    pub description: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RawType {
    pub name: String,
    #[serde(default)]
    pub namespace: Option<String>,
    /// Classification: "primitive" | "qualifier" | "container" | "object".
    #[serde(default)]
    pub kind: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub examples: Vec<String>,
    #[serde(default)]
    pub fields: Vec<RawTypeField>,
    #[serde(default)]
    pub remarks: Option<String>,
    #[serde(rename = "seeAlso", default)]
    pub see_also: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RawAnnotation {
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub syntax: Option<String>,
    #[serde(default)]
    pub examples: Vec<String>,
    #[serde(default)]
    pub remarks: Option<String>,
    #[serde(rename = "seeAlso", default)]
    pub see_also: Vec<String>,
}

/// A keyword entry. `keywords.json` is transitioning from a bare `string[]` to
/// objects carrying prose sub-sections; this untagged form accepts both so the
/// binary works against either schema. A bare string normalises to a
/// `RawKeyword` with no prose.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum RawKeywordEntry {
    Bare(String),
    Rich(RawKeyword),
}

impl RawKeywordEntry {
    fn into_keyword(self) -> RawKeyword {
        match self {
            Self::Bare(name) => RawKeyword {
                name,
                description: String::new(),
                returns_description: None,
                remarks: None,
                see_also: Vec::new(),
            },
            Self::Rich(k) => k,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RawKeyword {
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(rename = "returnsDescription", default)]
    pub returns_description: Option<String>,
    #[serde(default)]
    pub remarks: Option<String>,
    #[serde(rename = "seeAlso", default)]
    pub see_also: Vec<String>,
}

/// Raw operator entry from operators.json. Operators carry no namespace, no
/// typed return (only a prose `returnsDescription`), and no parameters.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RawOperator {
    pub name: String,
    #[serde(default)]
    pub syntax: Option<String>,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub examples: Vec<String>,
    #[serde(rename = "returnsDescription", default)]
    pub returns_description: Option<String>,
    #[serde(default)]
    pub remarks: Option<String>,
    #[serde(rename = "seeAlso", default)]
    pub see_also: Vec<String>,
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
    Type(TypeBehavior),
    Annotation(AnnotationBehavior),
    Operator(OperatorBehavior),
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
    /// Per-overload signatures, present only for overloaded functions.
    pub overloads: Vec<RawOverload>,
    /// Deprecation note, when the reference flags the function deprecated.
    pub deprecated: Option<String>,
    /// Prose "Returns" sentence (distinct from the typed `returns`).
    pub returns_description: Option<String>,
    /// Free-text "Remarks" caveats.
    pub remarks: Option<String>,
    /// "See also" cross-references as bare symbol names.
    pub see_also: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct VariableBehavior {
    pub name: String,
    pub namespace: Option<String>,
    pub ty: String,
    pub qualifier: String,
    /// Human-readable description from pine-data. Empty string when the source
    /// JSON carries no description.
    pub description: String,
    pub returns_description: Option<String>,
    pub remarks: Option<String>,
    pub see_also: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ConstantBehavior {
    pub name: String,
    pub namespace: Option<String>,
    pub short_name: Option<String>,
    pub ty: String,
    pub description: Option<String>,
    pub remarks: Option<String>,
    pub see_also: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct KeywordBehavior {
    pub name: String,
    pub description: String,
    pub returns_description: Option<String>,
    pub remarks: Option<String>,
    pub see_also: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct TypeBehavior {
    pub name: String,
    pub namespace: Option<String>,
    /// primitive | qualifier | container | object. Renamed from the JSON's
    /// `kind` key to avoid colliding with the `Behavior` serde tag, also `kind`.
    pub classification: String,
    pub description: String,
    pub examples: Vec<String>,
    pub fields: Vec<RawTypeField>,
    pub remarks: Option<String>,
    pub see_also: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AnnotationBehavior {
    pub name: String,
    pub description: String,
    pub syntax: Option<String>,
    pub examples: Vec<String>,
    pub remarks: Option<String>,
    pub see_also: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct OperatorBehavior {
    pub name: String,
    pub syntax: Option<String>,
    pub description: String,
    pub examples: Vec<String>,
    /// Operators carry no typed return - only a prose "Returns" sentence.
    pub returns_description: Option<String>,
    pub remarks: Option<String>,
    pub see_also: Vec<String>,
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
            Self::Type(_) => BehaviorKind::Type,
            Self::Annotation(_) => BehaviorKind::Annotation,
            Self::Operator(_) => BehaviorKind::Operator,
        }
    }

    /// Return `true` if this is a function that carries a polymorphic return.
    #[must_use]
    pub fn is_polymorphic(&self) -> bool {
        matches!(self, Self::Function(f) if f.flags.is_polymorphic())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum BehaviorKind {
    Function,
    Variable,
    Constant,
    Keyword,
    Type,
    Annotation,
    Operator,
}

impl BehaviorKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Function => "function",
            Self::Variable => "variable",
            Self::Constant => "constant",
            Self::Keyword => "keyword",
            Self::Type => "type",
            Self::Annotation => "annotation",
            Self::Operator => "operator",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            Self::Function => "Built-in functions with signatures and behavior metadata",
            Self::Variable => "Built-in variables such as OHLCV series",
            Self::Constant => "Typed named constants and enum-like values",
            Self::Keyword => "Reserved Pine keywords",
            Self::Type => "Built-in types (chart.point, line, array, ...) with fields",
            Self::Annotation => "Compiler annotations (@version=, @param, @type, ...)",
            Self::Operator => "Operators (+, -, ?:, [], +=, ...) with prose semantics",
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
    pub type_count: usize,
    pub annotation_count: usize,
    pub operator_count: usize,
    /// Number of functions carrying a polymorphic return-type class in their
    /// flags. Previously sourced from function-behavior.json's entry count.
    pub polymorphic_function_count: usize,
}

// ---------- indexed lookup ----------

struct BehaviorIndex {
    functions: HashMap<String, RawFunction>,
    variables: HashMap<String, RawVariable>,
    constants: HashMap<String, RawConstant>,
    keywords: Vec<RawKeyword>,
    types: HashMap<String, RawType>,
    annotations: HashMap<String, RawAnnotation>,
    operators: HashMap<String, RawOperator>,
}

fn index() -> &'static BehaviorIndex {
    static INDEX: OnceLock<BehaviorIndex> = OnceLock::new();
    INDEX.get_or_init(|| build_index().expect("vendored pine-data must parse"))
}

fn build_index() -> anyhow::Result<BehaviorIndex> {
    let functions: Vec<RawFunction> = serde_json::from_str(FUNCTIONS_JSON)?;
    let variables: Vec<RawVariable> = serde_json::from_str(VARIABLES_JSON)?;
    let constants: Vec<RawConstant> = serde_json::from_str(CONSTANTS_JSON)?;
    let keywords: Vec<RawKeywordEntry> = serde_json::from_str(KEYWORDS_JSON)?;
    let keywords: Vec<RawKeyword> = keywords
        .into_iter()
        .map(RawKeywordEntry::into_keyword)
        .collect();
    let types: Vec<RawType> = serde_json::from_str(TYPES_JSON)?;
    let annotations: Vec<RawAnnotation> = serde_json::from_str(ANNOTATIONS_JSON)?;
    let operators: Vec<RawOperator> = serde_json::from_str(OPERATORS_JSON)?;

    Ok(BehaviorIndex {
        functions: functions.into_iter().map(|f| (f.name.clone(), f)).collect(),
        variables: variables.into_iter().map(|v| (v.name.clone(), v)).collect(),
        constants: constants.into_iter().map(|c| (c.name.clone(), c)).collect(),
        keywords,
        types: types.into_iter().map(|t| (t.name.clone(), t)).collect(),
        annotations: annotations
            .into_iter()
            .map(|a| (a.name.clone(), a))
            .collect(),
        operators: operators.into_iter().map(|o| (o.name.clone(), o)).collect(),
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

/// First-hit lookup. Case-insensitive. Returns the highest-precedence catalog
/// match (function > variable > constant > type > annotation > operator >
/// keyword) - the single canonical entry. For names that live in several
/// catalogs at once (e.g. `na`, `time`), use `lookup_all` to get every meaning.
pub fn lookup(name: &str) -> Option<Behavior> {
    lookup_all(name).into_iter().next()
}

/// Every catalog match for `name`, in precedence order. A name can resolve in
/// more than one catalog: ~29 names collide (cast functions vs primitive types
/// like `int`; variable/function pairs like `time`, `dayofmonth`; `na` is all
/// of function, variable, and keyword). `po lookup` renders each.
///
/// Generic-placeholder fallback: pine-data stores generic constructors under
/// names like `array.new<type>`, `map.new<type,type>`. On a total miss, the two
/// conventional suffixes are probed so `lookup_all("array.new")` resolves.
pub fn lookup_all(name: &str) -> Vec<Behavior> {
    let idx = index();
    let out = collect_matches(idx, name);
    if !out.is_empty() {
        return out;
    }
    for suffix in &["<type,type>", "<type>"] {
        let candidate = format!("{name}{suffix}");
        let extra = collect_matches(idx, &candidate);
        if !extra.is_empty() {
            return extra;
        }
    }
    Vec::new()
}

/// Push every catalog match for `name` into a vec, in precedence order. Keyword
/// is last so `lookup`'s first-hit precedence (function/type win over keyword,
/// e.g. `int` -> cast function, `const` -> type) is preserved.
fn collect_matches(idx: &BehaviorIndex, name: &str) -> Vec<Behavior> {
    let mut out = Vec::new();
    if let Some(f) = map_get_ci(&idx.functions, name) {
        out.push(Behavior::Function(function_behavior(f)));
    }
    if let Some(v) = map_get_ci(&idx.variables, name) {
        out.push(Behavior::Variable(VariableBehavior {
            name: v.name.clone(),
            namespace: v.namespace.clone(),
            ty: v.ty.clone(),
            qualifier: v.qualifier.clone(),
            description: v.description.clone(),
            returns_description: v.returns_description.clone(),
            remarks: v.remarks.clone(),
            see_also: v.see_also.clone(),
        }));
    }
    if let Some(c) = map_get_ci(&idx.constants, name) {
        out.push(Behavior::Constant(ConstantBehavior {
            name: c.name.clone(),
            namespace: c.namespace.clone(),
            short_name: c.short_name.clone(),
            ty: c.ty.clone(),
            description: c.description.clone(),
            remarks: c.remarks.clone(),
            see_also: c.see_also.clone(),
        }));
    }
    if let Some(t) = map_get_ci(&idx.types, name) {
        out.push(Behavior::Type(TypeBehavior {
            name: t.name.clone(),
            namespace: t.namespace.clone(),
            classification: t.kind.clone(),
            description: t.description.clone(),
            examples: t.examples.clone(),
            fields: t.fields.clone(),
            remarks: t.remarks.clone(),
            see_also: t.see_also.clone(),
        }));
    }
    if let Some(a) = map_get_ci(&idx.annotations, name) {
        out.push(Behavior::Annotation(AnnotationBehavior {
            name: a.name.clone(),
            description: a.description.clone(),
            syntax: a.syntax.clone(),
            examples: a.examples.clone(),
            remarks: a.remarks.clone(),
            see_also: a.see_also.clone(),
        }));
    }
    if let Some(o) = map_get_ci(&idx.operators, name) {
        out.push(Behavior::Operator(OperatorBehavior {
            name: o.name.clone(),
            syntax: o.syntax.clone(),
            description: o.description.clone(),
            examples: o.examples.clone(),
            returns_description: o.returns_description.clone(),
            remarks: o.remarks.clone(),
            see_also: o.see_also.clone(),
        }));
    }
    if let Some(k) = idx
        .keywords
        .iter()
        .find(|k| k.name.eq_ignore_ascii_case(name))
    {
        out.push(Behavior::Keyword(KeywordBehavior {
            name: k.name.clone(),
            description: k.description.clone(),
            returns_description: k.returns_description.clone(),
            remarks: k.remarks.clone(),
            see_also: k.see_also.clone(),
        }));
    }
    out
}

fn function_behavior(f: &RawFunction) -> FunctionBehavior {
    FunctionBehavior {
        name: f.name.clone(),
        namespace: f.namespace.clone(),
        syntax: f.syntax.clone(),
        returns: f.returns.clone(),
        description: f.description.clone(),
        parameters: f.parameters.clone(),
        examples: f.examples.clone(),
        flags: f.flags.clone().unwrap_or_default(),
        overloads: f.overloads.clone(),
        deprecated: f.deprecated.clone(),
        returns_description: f.returns_description.clone(),
        remarks: f.remarks.clone(),
        see_also: f.see_also.clone(),
    }
}

pub fn snapshot() -> PineDataSnapshot {
    let idx = index();
    let polymorphic_function_count = idx
        .functions
        .values()
        .filter(|f| f.flags.as_ref().is_some_and(FunctionFlags::is_polymorphic))
        .count();
    PineDataSnapshot {
        version: PINE_DATA_VERSION.to_string(),
        generated_at: PINE_DATA_SNAPSHOT.to_string(),
        function_count: idx.functions.len(),
        variable_count: idx.variables.len(),
        constant_count: idx.constants.len(),
        keyword_count: idx.keywords.len(),
        type_count: idx.types.len(),
        annotation_count: idx.annotations.len(),
        operator_count: idx.operators.len(),
        polymorphic_function_count,
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
        BehaviorKindInfo {
            kind: BehaviorKind::Type,
            description: BehaviorKind::Type.description(),
            count: snapshot.type_count,
        },
        BehaviorKindInfo {
            kind: BehaviorKind::Annotation,
            description: BehaviorKind::Annotation.description(),
            count: snapshot.annotation_count,
        },
        BehaviorKindInfo {
            kind: BehaviorKind::Operator,
            description: BehaviorKind::Operator.description(),
            count: snapshot.operator_count,
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
                polymorphic: function
                    .flags
                    .as_ref()
                    .is_some_and(FunctionFlags::is_polymorphic),
            });
        }
    }
    if filter.is_none_or(|kind| kind == BehaviorKind::Variable) {
        for variable in idx.variables.values() {
            out.push(BehaviorListing {
                kind: BehaviorKind::Variable,
                name: variable.name.clone(),
                namespace: variable.namespace.clone(),
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
                name: keyword.name.clone(),
                namespace: None,
                detail: String::new(),
                polymorphic: false,
            });
        }
    }
    if filter.is_none_or(|kind| kind == BehaviorKind::Type) {
        for ty in idx.types.values() {
            out.push(BehaviorListing {
                kind: BehaviorKind::Type,
                name: ty.name.clone(),
                namespace: ty.namespace.clone(),
                detail: ty.kind.clone(),
                polymorphic: false,
            });
        }
    }
    if filter.is_none_or(|kind| kind == BehaviorKind::Annotation) {
        for annotation in idx.annotations.values() {
            out.push(BehaviorListing {
                kind: BehaviorKind::Annotation,
                name: annotation.name.clone(),
                namespace: None,
                detail: annotation.syntax.clone().unwrap_or_default(),
                polymorphic: false,
            });
        }
    }
    if filter.is_none_or(|kind| kind == BehaviorKind::Operator) {
        for operator in idx.operators.values() {
            out.push(BehaviorListing {
                kind: BehaviorKind::Operator,
                name: operator.name.clone(),
                namespace: None,
                detail: operator.syntax.clone().unwrap_or_default(),
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
        "type" | "types" => Ok(BehaviorKind::Type),
        "annotation" | "annotations" => Ok(BehaviorKind::Annotation),
        "operator" | "operators" => Ok(BehaviorKind::Operator),
        _ => bail!(
            "unknown behavior kind `{raw}`; expected one of: function, variable, constant, keyword, type, annotation, operator"
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
            content: function_search_content(function),
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
            name: keyword.name.clone(),
            content: keyword_search_content(keyword),
        });
    }
    for ty in idx.types.values() {
        out.push(BehaviorSearchEntry {
            category: "Type",
            name: ty.name.clone(),
            content: type_search_content(ty),
        });
    }
    for annotation in idx.annotations.values() {
        out.push(BehaviorSearchEntry {
            category: "Annotation",
            name: annotation.name.clone(),
            content: annotation_search_content(annotation),
        });
    }
    for operator in idx.operators.values() {
        out.push(BehaviorSearchEntry {
            category: "Operator",
            name: operator.name.clone(),
            content: operator_search_content(operator),
        });
    }
    out.sort_by(|a, b| a.category.cmp(b.category).then_with(|| a.name.cmp(&b.name)));
    out
}

fn function_search_content(function: &RawFunction) -> String {
    let mut parts = Vec::new();
    if !function.syntax.is_empty() {
        parts.push(format!("Syntax: {}", function.syntax));
    }
    if !function.returns.is_empty() {
        parts.push(format!("Returns: {}", function.returns));
    }
    if let Some(deprecated) = &function.deprecated {
        parts.push(format!("Deprecated: {deprecated}"));
    }
    if !function.description.is_empty() {
        parts.push(function.description.clone());
    }
    for param in &function.parameters {
        parts.push(param_search_line(param));
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
        if let Some(poly) = &flags.polymorphic {
            parts.push(format!("Polymorphic return: {poly}."));
        }
        if let Some(param) = &flags.return_type_param {
            parts.push(format!("Return type follows parameter: {param}"));
        }
    }
    if function.overloads.len() > 1 {
        parts.push(format!("Overloads: {}", function.overloads.len()));
        for overload in &function.overloads {
            parts.push(format!("Overload returns: {}", overload.returns));
        }
    }
    for example in &function.examples {
        parts.push(format!("Example:\n{example}"));
    }
    push_prose(
        &mut parts,
        &function.remarks,
        std::slice::from_ref(&function.returns_description),
        &function.see_also,
    );
    parts.join("\n")
}

fn param_search_line(param: &FunctionParameter) -> String {
    let required = if param.required {
        "required"
    } else {
        "optional"
    };
    let mut line = format!("Parameter {}: {} ({required})", param.name, param.ty);
    if let Some(default) = &param.default {
        line.push_str(&format!(", default {default}"));
    }
    if !param.allowed_values.is_empty() {
        line.push_str(&format!(", one of: {}", param.allowed_values.join(", ")));
    }
    match (param.min, param.max) {
        (Some(min), Some(max)) => line.push_str(&format!(", range {min}..{max}")),
        (Some(min), None) => line.push_str(&format!(", min {min}")),
        (None, Some(max)) => line.push_str(&format!(", max {max}")),
        (None, None) => {}
    }
    if !param.description.is_empty() {
        line.push_str(". ");
        line.push_str(&param.description);
    }
    line
}

fn variable_search_content(variable: &RawVariable) -> String {
    let mut parts = vec![
        format!("Type: {}", variable.ty),
        format!("Qualifier: {}", variable.qualifier),
    ];
    if !variable.description.is_empty() {
        parts.push(variable.description.clone());
    }
    push_prose(
        &mut parts,
        &variable.remarks,
        std::slice::from_ref(&variable.returns_description),
        &variable.see_also,
    );
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
    if let Some(description) = &constant.description
        && !description.is_empty()
    {
        parts.push(description.clone());
    }
    push_prose(&mut parts, &constant.remarks, &[], &constant.see_also);
    parts.join("\n")
}

fn type_search_content(ty: &RawType) -> String {
    let mut parts = vec![format!("Type kind: {}", ty.kind)];
    if !ty.description.is_empty() {
        parts.push(ty.description.clone());
    }
    for field in &ty.fields {
        parts.push(format!(
            "Field {}: {}. {}",
            field.name, field.ty, field.description
        ));
    }
    for example in &ty.examples {
        parts.push(format!("Example:\n{example}"));
    }
    push_prose(&mut parts, &ty.remarks, &[], &ty.see_also);
    parts.join("\n")
}

fn annotation_search_content(annotation: &RawAnnotation) -> String {
    let mut parts = Vec::new();
    if let Some(syntax) = &annotation.syntax {
        parts.push(format!("Syntax: {syntax}"));
    }
    if !annotation.description.is_empty() {
        parts.push(annotation.description.clone());
    }
    for example in &annotation.examples {
        parts.push(format!("Example:\n{example}"));
    }
    push_prose(&mut parts, &annotation.remarks, &[], &annotation.see_also);
    parts.join("\n")
}

fn keyword_search_content(keyword: &RawKeyword) -> String {
    let mut parts = Vec::new();
    if keyword.description.is_empty() {
        parts.push("Reserved Pine keyword.".to_string());
    } else {
        parts.push(keyword.description.clone());
    }
    push_prose(
        &mut parts,
        &keyword.remarks,
        std::slice::from_ref(&keyword.returns_description),
        &keyword.see_also,
    );
    parts.join("\n")
}

fn operator_search_content(operator: &RawOperator) -> String {
    let mut parts = Vec::new();
    if let Some(syntax) = &operator.syntax {
        parts.push(format!("Syntax: {syntax}"));
    }
    if !operator.description.is_empty() {
        parts.push(operator.description.clone());
    }
    for example in &operator.examples {
        parts.push(format!("Example:\n{example}"));
    }
    push_prose(
        &mut parts,
        &operator.remarks,
        std::slice::from_ref(&operator.returns_description),
        &operator.see_also,
    );
    parts.join("\n")
}

/// Append the shared prose sub-sections (Remarks, Returns-prose, See-also) to a
/// search-content part list. Centralised so every catalog folds the same fields
/// into its BM25 body - this is what preserves discoverability of remarks /
/// see-also terms now that the markdown reference source is gone.
fn push_prose(
    parts: &mut Vec<String>,
    remarks: &Option<String>,
    returns_descriptions: &[Option<String>],
    see_also: &[String],
) {
    if let Some(remarks) = remarks {
        parts.push(format!("Remarks: {remarks}"));
    }
    for returns in returns_descriptions.iter().flatten() {
        parts.push(format!("Returns: {returns}"));
    }
    if !see_also.is_empty() {
        parts.push(format!("See also: {}", see_also.join(", ")));
    }
}

#[cfg(test)]
mod tests {
    include!("tests.rs");
}
