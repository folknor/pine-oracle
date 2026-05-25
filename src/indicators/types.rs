// SPDX-License-Identifier: MPL-2.0

use std::collections::BTreeMap;
use std::fmt;

use serde::de::Error as _;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

pub const EXPECT_SCHEMA_VERSION: u32 = 1;
pub(super) const DEFAULT_RUNNER_EXPECT_TOLERANCE: f64 = 0.0;

// Interchange tokens for special OutputValue variants. Defined once here so
// Serialize, Deserialize, and Display all reference the same literal.
pub(super) const TOKEN_NA: &str = "__NaN__";
pub(super) const TOKEN_POS_INF: &str = "__Infinity__";
pub(super) const TOKEN_NEG_INF: &str = "__-Infinity__";
pub(super) const TOKEN_UNDEFINED: &str = "__undefined__";

#[derive(Debug, Clone, Serialize)]
pub struct IndicatorListing {
    pub slug: String,
    pub baseline: BaselineKind,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub symbol: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeframe: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bar_count: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_count: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub test_range: Option<TestRange>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pine_version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tv_snapshot: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct IndicatorFixtureCounts {
    pub total: usize,
    pub smoke: usize,
    pub tv: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct IndicatorBaselineInfo {
    pub baseline: BaselineKind,
    pub description: &'static str,
    pub count: usize,
}

#[must_use]
#[derive(Debug, Clone, Serialize)]
pub struct IndicatorBatchReport {
    pub ok: bool,
    pub fixture_count: usize,
    pub passed_count: usize,
    pub failed_count: usize,
    pub reports: Vec<IndicatorReport>,
}

#[must_use]
#[derive(Debug, Clone, Serialize)]
pub struct IndicatorFixtureDetail {
    pub slug: String,
    pub baseline: BaselineKind,
    pub source_pine: String,
    pub bar_count: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub symbol: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeframe: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data_source: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub first_bar_timestamp: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_bar_timestamp: Option<i64>,
    pub output_count: usize,
    pub expected_outputs: Vec<IndicatorExpectedOutput>,
    pub actual_outputs_checked: bool,
    pub actual_output_keys: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub missing_expected_output_keys: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub unexpected_actual_output_keys: Vec<String>,
    pub tolerance: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub test_range: Option<TestRange>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pine_version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tv_snapshot: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub runtime_error: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub stub_dependencies: Vec<piners_runner::StubDependency>,
}

#[must_use]
#[derive(Debug, Clone, Serialize)]
pub struct IndicatorActualReport {
    pub slug: String,
    pub baseline: BaselineKind,
    pub bar_count: usize,
    pub output_count: usize,
    pub output_keys: Vec<String>,
    pub outputs: BTreeMap<String, Vec<OutputValue>>,
    pub runner_expect: IndicatorGeneratedExpect,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub runtime_error: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub stub_dependencies: Vec<piners_runner::StubDependency>,
}

#[must_use]
#[derive(Debug, Clone, Serialize)]
pub struct IndicatorGeneratedExpect {
    pub schema_version: u32,
    pub indicator_slug: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pine_version: Option<String>,
    pub tolerance: f64,
    pub outputs: BTreeMap<String, Vec<OutputValue>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub test_range: Option<TestRange>,
}

#[derive(Debug, Clone, Serialize)]
pub struct IndicatorExpectedOutput {
    pub key: String,
    pub value_count: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub first_value: Option<OutputValue>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_value: Option<OutputValue>,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq, Serialize)]
pub struct TestRange {
    pub start: String,
    pub end: String,
}

#[must_use]
#[derive(Debug, Clone, Serialize)]
pub struct IndicatorReport {
    pub slug: String,
    pub baseline: BaselineKind,
    pub ok: bool,
    pub bar_count: usize,
    pub compared_bar_count: usize,
    pub output_count: usize,
    pub expected_output_keys: Vec<String>,
    pub actual_output_keys: Vec<String>,
    pub mismatch_count: usize,
    pub tolerance: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub test_range: Option<TestRange>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pine_version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tv_snapshot: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub runtime_error: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub stub_dependencies: Vec<piners_runner::StubDependency>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub mismatches: Vec<IndicatorMismatch>,
}

#[must_use]
#[derive(Debug, Clone, Serialize)]
pub struct IndicatorMismatch {
    pub output: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bar_index: Option<usize>,
    pub reason: MismatchReason,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expected: Option<OutputValue>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub actual: Option<OutputValue>,
    pub expected_len: usize,
    pub actual_len: usize,
}

// New failure modes (e.g. key-name drift, tolerance class) can be added without breaking callers.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MismatchReason {
    MissingOutput,
    UnexpectedOutput,
    LengthMismatch,
    ValueMismatch,
}

// Smoke is the implicit default so that a fixture with no metadata.json (or
// no `baseline` field) gets classified as the lower tier. TV baselines must
// be opted into explicitly because they additionally require pine_version +
// tv_snapshot. No "unknown" third tier exists; serde rejects any other value.
// Additional tiers (e.g. community-sourced captures) may be added later.
#[non_exhaustive]
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum BaselineKind {
    #[default]
    Smoke,
    Tv,
}

impl BaselineKind {
    pub fn description(self) -> &'static str {
        match self {
            Self::Smoke => "deterministic runner/differ substrate fixture",
            Self::Tv => "TradingView-captured oracle baseline",
        }
    }
}

impl fmt::Display for BaselineKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Smoke => f.write_str("smoke"),
            Self::Tv => f.write_str("tv"),
        }
    }
}

// OutputValue is a true enum so callers can construct and pattern-match
// variants directly without accessing private fields. The wire format (JSON
// number, bool, or token string) is unchanged.
//
// Variant size differences are expected: Number(f64) is 16 bytes while the
// unit variants are 1 byte. clippy::variant_size_differences is silenced
// because boxing the f64 would hurt every series-of-f64 use case to save
// 15 bytes per discriminant.
#[allow(variant_size_differences)]
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OutputValue {
    Number(f64),
    Bool(bool),
    Na,
    PosInfinity,
    NegInfinity,
    Undefined,
}

impl OutputValue {
    /// Wrap a finite f64 as `Number`. The caller asserts the value is finite;
    /// no auto-routing happens here. Use `from_f64` when the value may be
    /// NaN or infinite.
    pub fn number(value: f64) -> Self {
        Self::Number(value)
    }

    /// Wrap a bool as `Bool`.
    pub fn bool(value: bool) -> Self {
        Self::Bool(value)
    }

    /// Convert an f64, auto-routing NaN -> `Na`, +inf -> `PosInfinity`,
    /// -inf -> `NegInfinity`, finite -> `Number`.
    pub fn from_f64(value: f64) -> Self {
        if value.is_nan() {
            Self::Na
        } else if value == f64::INFINITY {
            Self::PosInfinity
        } else if value == f64::NEG_INFINITY {
            Self::NegInfinity
        } else {
            Self::Number(value)
        }
    }

    /// Construct the `Undefined` sentinel (piners runner produces this for
    /// outputs that have no value at a given bar).
    pub fn undefined() -> Self {
        Self::Undefined
    }
}

impl Serialize for OutputValue {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Self::Number(n) => serializer.serialize_f64(*n),
            Self::Bool(b) => serializer.serialize_bool(*b),
            Self::Na => serializer.serialize_str(TOKEN_NA),
            Self::PosInfinity => serializer.serialize_str(TOKEN_POS_INF),
            Self::NegInfinity => serializer.serialize_str(TOKEN_NEG_INF),
            Self::Undefined => serializer.serialize_str(TOKEN_UNDEFINED),
        }
    }
}

impl<'de> Deserialize<'de> for OutputValue {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = serde_json::Value::deserialize(deserializer)?;
        match value {
            serde_json::Value::Number(number) => number
                .as_f64()
                .map(Self::Number)
                .ok_or_else(|| D::Error::custom("expected finite JSON number")),
            serde_json::Value::Bool(b) => Ok(Self::Bool(b)),
            serde_json::Value::String(token) => match token.as_str() {
                TOKEN_NA => Ok(Self::Na),
                TOKEN_POS_INF => Ok(Self::PosInfinity),
                TOKEN_NEG_INF => Ok(Self::NegInfinity),
                TOKEN_UNDEFINED => Ok(Self::Undefined),
                _ => Err(D::Error::custom(format!(
                    "unknown indicator output token `{token}`"
                ))),
            },
            _ => Err(D::Error::custom(
                "expected number, bool, or indicator output token",
            )),
        }
    }
}

impl fmt::Display for OutputValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Number(n) => write!(f, "{n}"),
            Self::Bool(b) => write!(f, "{b}"),
            Self::Na => f.write_str(TOKEN_NA),
            Self::PosInfinity => f.write_str(TOKEN_POS_INF),
            Self::NegInfinity => f.write_str(TOKEN_NEG_INF),
            Self::Undefined => f.write_str(TOKEN_UNDEFINED),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Confirm that each special OutputValue token constant matches what
    // serde_json::to_string produces, so a typo in one constant is caught
    // at compile/test time rather than silently diverging at runtime.
    #[test]
    fn output_value_token_constants_round_trip() {
        let cases: &[(OutputValue, &str)] = &[
            (OutputValue::Na, TOKEN_NA),
            (OutputValue::PosInfinity, TOKEN_POS_INF),
            (OutputValue::NegInfinity, TOKEN_NEG_INF),
            (OutputValue::undefined(), TOKEN_UNDEFINED),
        ];

        for (value, expected_token) in cases {
            // Serialize produces a JSON string like `"__NaN__"` (with quotes).
            let serialized = serde_json::to_string(value).expect("serialize");
            let quoted = format!("\"{expected_token}\"");
            assert_eq!(
                serialized, quoted,
                "token constant mismatch for {value}: serialized to {serialized}, expected {quoted}"
            );

            // Deserialize round-trips back to the same value.
            let round_tripped: OutputValue =
                serde_json::from_str(&serialized).expect("deserialize");
            assert_eq!(
                &round_tripped, value,
                "round-trip mismatch for token {expected_token}"
            );
        }
    }

    // Confirm that the enum API is fully consumer-friendly: construction and
    // pattern-matching work without any `pub(super)` workaround.
    #[test]
    fn output_value_enum_consumer_api() {
        // Construction via named constructors.
        let n = OutputValue::number(1.5);
        let b = OutputValue::bool(true);
        let na = OutputValue::Na;
        let pos = OutputValue::PosInfinity;
        let neg = OutputValue::NegInfinity;
        let undef = OutputValue::undefined();

        // from_f64 routing.
        assert_eq!(OutputValue::from_f64(f64::NAN), OutputValue::Na);
        assert_eq!(
            OutputValue::from_f64(f64::INFINITY),
            OutputValue::PosInfinity
        );
        assert_eq!(
            OutputValue::from_f64(f64::NEG_INFINITY),
            OutputValue::NegInfinity
        );
        assert_eq!(OutputValue::from_f64(2.5), OutputValue::Number(2.5));

        // Pattern-matching: the whole point of the refactor.
        if let OutputValue::Number(v) = n {
            assert!((v - 1.5).abs() < f64::EPSILON);
        } else {
            panic!("expected Number variant");
        }
        assert!(matches!(b, OutputValue::Bool(true)));
        assert!(matches!(na, OutputValue::Na));
        assert!(matches!(pos, OutputValue::PosInfinity));
        assert!(matches!(neg, OutputValue::NegInfinity));
        assert!(matches!(undef, OutputValue::Undefined));

        // Serialize / Display contract unchanged.
        assert_eq!(
            serde_json::to_string(&OutputValue::Number(2.0)).unwrap(),
            "2.0"
        );
        assert_eq!(
            serde_json::to_string(&OutputValue::Bool(false)).unwrap(),
            "false"
        );
        assert_eq!(format!("{}", OutputValue::Na), TOKEN_NA);
    }
}
