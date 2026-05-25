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

#[derive(Debug, Clone, Serialize)]
pub struct IndicatorBatchReport {
    pub ok: bool,
    pub fixture_count: usize,
    pub passed_count: usize,
    pub failed_count: usize,
    pub reports: Vec<IndicatorReport>,
}

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

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OutputValue {
    pub(super) kind: OutputValueKind,
    pub(super) number: f64,
    pub(super) bool_value: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum OutputValueKind {
    Number,
    Bool,
    Na,
    PosInfinity,
    NegInfinity,
    Undefined,
}

impl OutputValue {
    pub(super) fn number(value: f64) -> Self {
        Self {
            kind: OutputValueKind::Number,
            number: value,
            bool_value: false,
        }
    }

    pub(super) fn bool(value: bool) -> Self {
        Self {
            kind: OutputValueKind::Bool,
            number: 0.0,
            bool_value: value,
        }
    }

    pub(super) fn special(kind: OutputValueKind) -> Self {
        Self {
            kind,
            number: 0.0,
            bool_value: false,
        }
    }

    pub(super) fn from_f64(value: f64) -> Self {
        if value.is_nan() {
            Self::special(OutputValueKind::Na)
        } else if value == f64::INFINITY {
            Self::special(OutputValueKind::PosInfinity)
        } else if value == f64::NEG_INFINITY {
            Self::special(OutputValueKind::NegInfinity)
        } else {
            Self::number(value)
        }
    }

    pub(super) fn undefined() -> Self {
        Self::special(OutputValueKind::Undefined)
    }
}

impl Serialize for OutputValue {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self.kind {
            OutputValueKind::Number => serializer.serialize_f64(self.number),
            OutputValueKind::Bool => serializer.serialize_bool(self.bool_value),
            OutputValueKind::Na => serializer.serialize_str(TOKEN_NA),
            OutputValueKind::PosInfinity => serializer.serialize_str(TOKEN_POS_INF),
            OutputValueKind::NegInfinity => serializer.serialize_str(TOKEN_NEG_INF),
            OutputValueKind::Undefined => serializer.serialize_str(TOKEN_UNDEFINED),
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
                .map(Self::number)
                .ok_or_else(|| D::Error::custom("expected finite JSON number")),
            serde_json::Value::Bool(value) => Ok(Self::bool(value)),
            serde_json::Value::String(token) => match token.as_str() {
                TOKEN_NA => Ok(Self::special(OutputValueKind::Na)),
                TOKEN_POS_INF => Ok(Self::special(OutputValueKind::PosInfinity)),
                TOKEN_NEG_INF => Ok(Self::special(OutputValueKind::NegInfinity)),
                TOKEN_UNDEFINED => Ok(Self::undefined()),
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
        match self.kind {
            OutputValueKind::Number => write!(f, "{}", self.number),
            OutputValueKind::Bool => write!(f, "{}", self.bool_value),
            OutputValueKind::Na => f.write_str(TOKEN_NA),
            OutputValueKind::PosInfinity => f.write_str(TOKEN_POS_INF),
            OutputValueKind::NegInfinity => f.write_str(TOKEN_NEG_INF),
            OutputValueKind::Undefined => f.write_str(TOKEN_UNDEFINED),
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
            (OutputValue::special(OutputValueKind::Na), TOKEN_NA),
            (
                OutputValue::special(OutputValueKind::PosInfinity),
                TOKEN_POS_INF,
            ),
            (
                OutputValue::special(OutputValueKind::NegInfinity),
                TOKEN_NEG_INF,
            ),
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
}
