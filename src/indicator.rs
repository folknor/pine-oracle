// SPDX-License-Identifier: MPL-2.0
//
// Per-bar indicator parity fixtures.
//
// The `indicators/` tree is embedded into the binary. Each fixture provides
// Pine source, OHLCV bars, and expected per-output values captured from a
// baseline. The current command substrate runs the source through piners'
// runner and diffs runner outputs against the baked expectation.

use std::collections::{BTreeMap, HashMap};
use std::fmt;
use std::path::{Component, Path};
use std::sync::Arc;

use anyhow::{Context, Result, anyhow, bail};
use include_dir::{Dir, include_dir};
use piners_runner::types::BarSeriesContext;
use piners_runner::{Bar, BarSeries, Engine, RunConfig, ScriptKind, run_single};
use serde::de::Error as _;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

static INDICATORS: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/indicators");

#[derive(Debug, Clone, Serialize)]
pub struct IndicatorListing {
    pub slug: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct IndicatorReport {
    pub slug: String,
    pub ok: bool,
    pub bar_count: usize,
    pub output_count: usize,
    pub mismatch_count: usize,
    pub tolerance: f64,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MismatchReason {
    MissingOutput,
    UnexpectedOutput,
    LengthMismatch,
    ValueMismatch,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OutputValue {
    kind: OutputValueKind,
    number: f64,
    bool_value: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OutputValueKind {
    Number,
    Bool,
    Na,
    PosInfinity,
    NegInfinity,
    Undefined,
}

impl OutputValue {
    fn number(value: f64) -> Self {
        Self {
            kind: OutputValueKind::Number,
            number: value,
            bool_value: false,
        }
    }

    fn bool(value: bool) -> Self {
        Self {
            kind: OutputValueKind::Bool,
            number: 0.0,
            bool_value: value,
        }
    }

    fn special(kind: OutputValueKind) -> Self {
        Self {
            kind,
            number: 0.0,
            bool_value: false,
        }
    }

    fn from_f64(value: f64) -> Self {
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

    fn undefined() -> Self {
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
            OutputValueKind::Na => serializer.serialize_str("__NaN__"),
            OutputValueKind::PosInfinity => serializer.serialize_str("__Infinity__"),
            OutputValueKind::NegInfinity => serializer.serialize_str("__-Infinity__"),
            OutputValueKind::Undefined => serializer.serialize_str("__undefined__"),
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
                "__NaN__" => Ok(Self::special(OutputValueKind::Na)),
                "__Infinity__" => Ok(Self::special(OutputValueKind::PosInfinity)),
                "__-Infinity__" => Ok(Self::special(OutputValueKind::NegInfinity)),
                "__undefined__" => Ok(Self::undefined()),
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
            OutputValueKind::Na => f.write_str("__NaN__"),
            OutputValueKind::PosInfinity => f.write_str("__Infinity__"),
            OutputValueKind::NegInfinity => f.write_str("__-Infinity__"),
            OutputValueKind::Undefined => f.write_str("__undefined__"),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
struct BarsFile {
    #[serde(default)]
    symbol: Option<String>,
    #[serde(default)]
    timeframe: Option<String>,
    #[serde(default)]
    source: Option<String>,
    #[serde(default)]
    context: Option<BarSeriesContext>,
    bars: Vec<Bar>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
enum BarsDocument {
    Full(Box<BarsFile>),
    Bare(Vec<Bar>),
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct ExpectFile {
    schema_version: u32,
    indicator_slug: String,
    #[serde(default)]
    pine_version: Option<String>,
    #[serde(default)]
    tv_snapshot: Option<String>,
    #[serde(default = "default_tolerance")]
    tolerance: f64,
    outputs: BTreeMap<String, Vec<OutputValue>>,
    #[serde(default)]
    test_range: Option<TestRange>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct TestRange {
    start: String,
    end: String,
}

#[derive(Default, Debug, Clone, Deserialize, Serialize)]
struct MetadataFile {
    #[serde(default)]
    pine_version: Option<String>,
    #[serde(default)]
    tv_snapshot: Option<String>,
    #[serde(default)]
    notes: Option<String>,
}

#[derive(Debug, Clone)]
struct IndicatorFixture {
    slug: String,
    source: String,
    bars: BarsFile,
    expect: ExpectFile,
    metadata: MetadataFile,
}

pub fn list_fixtures() -> Result<Vec<IndicatorListing>> {
    let entries = INDICATORS
        .find("**/source.pine")
        .map_err(|e| anyhow!("walking indicator fixtures: {e}"))?;
    let mut out = entries
        .filter_map(|entry| {
            let file = entry.as_file()?;
            let parent = file.path().parent()?;
            let slug = parent.to_str()?;
            (!slug.is_empty()).then(|| IndicatorListing {
                slug: slug.to_string(),
            })
        })
        .collect::<Vec<_>>();
    out.sort_by(|a, b| a.slug.cmp(&b.slug));
    Ok(out)
}

pub fn run_strict(slug: &str) -> Result<IndicatorReport> {
    let fixture = load_fixture(slug)?;
    run_fixture(&fixture)
}

fn load_fixture(slug: &str) -> Result<IndicatorFixture> {
    let slug = sanitise_slug(slug)?;
    let source = require_utf8(slug, "source.pine")?.to_string();
    let bars_json = require_utf8(slug, "bars.json")?;
    let expect_json = require_utf8(slug, "expect.json")?;
    let metadata_json = optional_utf8(slug, "metadata.json")?;
    parse_fixture(slug, source, bars_json, expect_json, metadata_json)
}

fn parse_fixture(
    slug: &str,
    source: String,
    bars_json: &str,
    expect_json: &str,
    metadata_json: Option<&str>,
) -> Result<IndicatorFixture> {
    let bars = parse_bars(bars_json).with_context(|| format!("parsing {slug}/bars.json"))?;
    let expect: ExpectFile =
        serde_json::from_str(expect_json).with_context(|| format!("parsing {slug}/expect.json"))?;
    if expect.schema_version != 1 {
        bail!(
            "{slug}/expect.json has unsupported schema_version {}; expected 1",
            expect.schema_version
        );
    }
    if expect.indicator_slug != slug {
        bail!(
            "{slug}/expect.json indicator_slug is `{}`, expected `{slug}`",
            expect.indicator_slug
        );
    }
    if !expect.tolerance.is_finite() || expect.tolerance < 0.0 {
        bail!("{slug}/expect.json tolerance must be a finite non-negative number");
    }
    let metadata = metadata_json.map_or_else(
        || Ok(MetadataFile::default()),
        |json| serde_json::from_str(json).with_context(|| format!("parsing {slug}/metadata.json")),
    )?;
    Ok(IndicatorFixture {
        slug: slug.to_string(),
        source,
        bars,
        expect,
        metadata,
    })
}

fn parse_bars(json: &str) -> Result<BarsFile> {
    match serde_json::from_str(json)? {
        BarsDocument::Full(file) => Ok(*file),
        BarsDocument::Bare(bars) => Ok(BarsFile {
            symbol: None,
            timeframe: None,
            source: None,
            context: None,
            bars,
        }),
    }
}

fn run_fixture(fixture: &IndicatorFixture) -> Result<IndicatorReport> {
    if fixture.bars.bars.is_empty() {
        bail!("{}/bars.json contains no bars", fixture.slug);
    }
    let engine = Engine::new();
    let program = Arc::new(
        engine
            .compile_source(&fixture.source, None)
            .with_context(|| format!("compiling {}/source.pine", fixture.slug))?,
    );
    if !matches!(program.script_kind, ScriptKind::Indicator) {
        bail!("{} is not an indicator fixture", fixture.slug);
    }
    let bars = Arc::new(fixture.bars.to_series());
    let result = run_single(RunConfig::new(program, bars));
    let mut mismatches = diff_outputs(
        &fixture.expect.outputs,
        &result.indicator_outputs,
        fixture.expect.tolerance,
    );
    let runtime_error = result.runtime_error.map(|e| e.to_string());
    if runtime_error.is_some() {
        mismatches.push(IndicatorMismatch {
            output: "<runtime>".to_string(),
            bar_index: None,
            reason: MismatchReason::ValueMismatch,
            expected: None,
            actual: None,
            expected_len: 0,
            actual_len: 0,
        });
    }
    let mismatch_count = mismatches.len();
    let ok = mismatch_count == 0;
    Ok(IndicatorReport {
        slug: fixture.slug.clone(),
        ok,
        bar_count: fixture.bars.bars.len(),
        output_count: fixture.expect.outputs.len(),
        mismatch_count,
        tolerance: fixture.expect.tolerance,
        pine_version: fixture
            .expect
            .pine_version
            .clone()
            .or_else(|| fixture.metadata.pine_version.clone()),
        tv_snapshot: fixture
            .expect
            .tv_snapshot
            .clone()
            .or_else(|| fixture.metadata.tv_snapshot.clone()),
        runtime_error,
        stub_dependencies: result.stub_dependencies,
        mismatches,
    })
}

impl BarsFile {
    fn to_series(&self) -> BarSeries {
        let mut series = BarSeries::new(
            self.bars.clone(),
            self.symbol.as_deref().unwrap_or("fixture"),
            self.timeframe.as_deref().unwrap_or("1D"),
            self.source.as_deref().unwrap_or("indicator-fixture"),
        );
        if let Some(context) = &self.context {
            series = series.with_context(context.clone());
        }
        series
    }
}

fn diff_outputs(
    expected: &BTreeMap<String, Vec<OutputValue>>,
    actual: &HashMap<String, Vec<f64>>,
    tolerance: f64,
) -> Vec<IndicatorMismatch> {
    let mut mismatches = Vec::new();
    for (name, expected_values) in expected {
        let actual_values = actual.get(name);
        match actual_values {
            Some(actual_values) => {
                if expected_values.len() != actual_values.len() {
                    mismatches.push(IndicatorMismatch {
                        output: name.clone(),
                        bar_index: None,
                        reason: MismatchReason::LengthMismatch,
                        expected: None,
                        actual: None,
                        expected_len: expected_values.len(),
                        actual_len: actual_values.len(),
                    });
                }
                let len = expected_values.len().max(actual_values.len());
                for index in 0..len {
                    let expected_value = expected_values
                        .get(index)
                        .copied()
                        .unwrap_or_else(OutputValue::undefined);
                    let actual_value = actual_values
                        .get(index)
                        .map_or_else(OutputValue::undefined, |value| {
                            OutputValue::from_f64(*value)
                        });
                    if !values_match(expected_value, actual_value, tolerance) {
                        mismatches.push(IndicatorMismatch {
                            output: name.clone(),
                            bar_index: Some(index),
                            reason: MismatchReason::ValueMismatch,
                            expected: Some(expected_value),
                            actual: Some(actual_value),
                            expected_len: expected_values.len(),
                            actual_len: actual_values.len(),
                        });
                    }
                }
            }
            None => mismatches.push(IndicatorMismatch {
                output: name.clone(),
                bar_index: None,
                reason: MismatchReason::MissingOutput,
                expected: None,
                actual: None,
                expected_len: expected_values.len(),
                actual_len: 0,
            }),
        }
    }
    let mut actual_names = actual.keys().collect::<Vec<_>>();
    actual_names.sort();
    for name in actual_names {
        if name == "close" && !expected.contains_key(name) {
            continue;
        }
        if !expected.contains_key(name) {
            mismatches.push(IndicatorMismatch {
                output: name.clone(),
                bar_index: None,
                reason: MismatchReason::UnexpectedOutput,
                expected: None,
                actual: None,
                expected_len: 0,
                actual_len: actual.get(name).map_or(0, Vec::len),
            });
        }
    }
    mismatches
}

fn values_match(expected: OutputValue, actual: OutputValue, tolerance: f64) -> bool {
    if expected.kind != actual.kind {
        return false;
    }
    match expected.kind {
        OutputValueKind::Number => (expected.number - actual.number).abs() <= tolerance,
        OutputValueKind::Bool => expected.bool_value == actual.bool_value,
        OutputValueKind::Na
        | OutputValueKind::PosInfinity
        | OutputValueKind::NegInfinity
        | OutputValueKind::Undefined => true,
    }
}

fn require_utf8(slug: &str, file_name: &str) -> Result<&'static str> {
    optional_utf8(slug, file_name)?.ok_or_else(|| {
        let available = list_fixtures().map(|items| items.len()).unwrap_or_default();
        anyhow!(
            "indicator fixture `{slug}` is missing {file_name}; {available} fixture(s) are baked"
        )
    })
}

fn optional_utf8(slug: &str, file_name: &str) -> Result<Option<&'static str>> {
    let path = format!("{slug}/{file_name}");
    INDICATORS
        .get_file(&path)
        .map(|file| {
            file.contents_utf8()
                .ok_or_else(|| anyhow!("{path} is not valid UTF-8"))
        })
        .transpose()
}

fn sanitise_slug(slug: &str) -> Result<&str> {
    let slug = slug
        .trim()
        .strip_prefix("indicators/")
        .unwrap_or(slug.trim());
    if slug.is_empty() {
        bail!("indicator slug must not be empty");
    }
    let path = Path::new(slug);
    for component in path.components() {
        match component {
            Component::Normal(part) if !part.is_empty() => {}
            _ => bail!("invalid indicator slug `{slug}`"),
        }
    }
    Ok(slug)
}

fn default_tolerance() -> f64 {
    1e-9
}

#[cfg(test)]
mod tests {
    use super::*;

    const BARS: &str = r#"{
        "symbol": "NASDAQ:SPY",
        "timeframe": "1D",
        "source": "test",
        "bars": [
            {"timestamp": 1, "open": 10.0, "high": 11.0, "low": 9.0, "close": 10.0, "volume": 100.0},
            {"timestamp": 2, "open": 10.0, "high": 12.0, "low": 9.0, "close": 11.0, "volume": 110.0}
        ]
    }"#;

    #[test]
    fn strict_fixture_matches_runner_outputs() {
        let fixture = parse_fixture(
            "close-plus-one",
            "indicator(\"fixture\")\nplot(close + 1)\n".to_string(),
            BARS,
            r#"{
                "schema_version": 1,
                "indicator_slug": "close-plus-one",
                "pine_version": "6.0.0",
                "tv_snapshot": "self-test",
                "outputs": {"plot": [11.0, 12.0]}
            }"#,
            None,
        )
        .expect("fixture");
        let report = run_fixture(&fixture).expect("run");
        assert!(report.ok, "{:?}", report.mismatches);
        assert_eq!(report.output_count, 1);
        assert_eq!(report.bar_count, 2);
    }

    #[test]
    fn strict_fixture_reports_value_mismatch() {
        let fixture = parse_fixture(
            "close-plus-one",
            "indicator(\"fixture\")\nplot(close + 1)\n".to_string(),
            BARS,
            r#"{
                "schema_version": 1,
                "indicator_slug": "close-plus-one",
                "outputs": {"plot": [11.0, 99.0]}
            }"#,
            None,
        )
        .expect("fixture");
        let report = run_fixture(&fixture).expect("run");
        assert!(!report.ok);
        assert_eq!(report.mismatch_count, 1);
        assert_eq!(report.mismatches[0].bar_index, Some(1));
        assert_eq!(report.mismatches[0].reason, MismatchReason::ValueMismatch);
    }

    #[test]
    fn missing_output_is_reported_once() {
        let mut actual = HashMap::new();
        actual.insert("plot".to_string(), vec![1.0]);
        let expected = BTreeMap::from([("plot#1".to_string(), vec![OutputValue::number(2.0)])]);
        let mismatches = diff_outputs(&expected, &actual, 0.0);
        assert_eq!(mismatches.len(), 2);
        assert_eq!(mismatches[0].reason, MismatchReason::MissingOutput);
        assert_eq!(mismatches[1].reason, MismatchReason::UnexpectedOutput);
    }

    #[test]
    fn special_tokens_match_non_finite_values() {
        assert!(values_match(
            OutputValue::special(OutputValueKind::Na),
            OutputValue::from_f64(f64::NAN),
            0.0
        ));
        assert!(values_match(
            OutputValue::special(OutputValueKind::PosInfinity),
            OutputValue::from_f64(f64::INFINITY),
            0.0
        ));
        assert!(values_match(
            OutputValue::special(OutputValueKind::NegInfinity),
            OutputValue::from_f64(f64::NEG_INFINITY),
            0.0
        ));
    }

    #[test]
    fn rejects_traversal_slug() {
        let err = sanitise_slug("../x").expect_err("must reject");
        assert!(err.to_string().contains("invalid indicator slug"));
    }

    #[test]
    fn output_value_serializes_tokens() {
        assert_eq!(
            serde_json::to_string(&OutputValue::undefined()).expect("json"),
            r#""__undefined__""#
        );
    }
}
