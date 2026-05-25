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
use chrono::{DateTime, Utc};
use include_dir::{Dir, include_dir};
use piners_runner::types::BarSeriesContext;
use piners_runner::{
    Bar, BarSeries, Engine, PineOutput, PineOutputChannel, PineOutputEventKind, RunConfig,
    ScriptKind, run_single,
};
use serde::de::Error as _;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

static INDICATORS: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/indicators");
const BASELINE_CATALOG_MARKER: &str = "?";

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

#[derive(Debug, Clone, Deserialize, PartialEq, Eq, Serialize)]
pub struct TestRange {
    pub start: String,
    pub end: String,
}

#[derive(Default, Debug, Clone, Deserialize, Serialize)]
struct MetadataFile {
    #[serde(default)]
    baseline: BaselineKind,
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

#[derive(Debug, Clone)]
struct ComparisonPlan {
    bar_indices: Vec<usize>,
    expected_shape: ExpectedShape,
    windowed: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ExpectedShape {
    FullSeries,
    WindowSeries,
}

pub fn list_fixtures() -> Result<Vec<IndicatorListing>> {
    let entries = INDICATORS
        .find("**/source.pine")
        .map_err(|e| anyhow!("walking indicator fixtures: {e}"))?;
    let mut out = Vec::new();
    for entry in entries {
        let Some(file) = entry.as_file() else {
            continue;
        };
        let Some(parent) = file.path().parent() else {
            continue;
        };
        let Some(slug) = parent.to_str() else {
            continue;
        };
        if !slug.is_empty() {
            out.push(load_listing_lenient(slug));
        }
    }
    out.sort_by(|a, b| a.slug.cmp(&b.slug));
    Ok(out)
}

pub fn list_fixtures_filtered(
    grep: Option<&str>,
    baseline_filter: Option<&str>,
) -> Result<Vec<IndicatorListing>> {
    let filter = baseline_filter.map(parse_baseline_kind).transpose()?;
    let needle = grep.map(str::to_ascii_lowercase);
    let mut fixtures = list_fixtures()?;
    if let Some(filter) = filter {
        fixtures.retain(|fixture| fixture.baseline == filter);
    }
    if let Some(needle) = needle {
        fixtures.retain(|fixture| indicator_listing_matches(fixture, &needle));
    }
    Ok(fixtures)
}

pub fn fixture_counts() -> Result<IndicatorFixtureCounts> {
    let fixtures = list_fixtures()?;
    let total = fixtures.len();
    let smoke = fixtures
        .iter()
        .filter(|fixture| matches!(fixture.baseline, BaselineKind::Smoke))
        .count();
    let tv = fixtures
        .iter()
        .filter(|fixture| matches!(fixture.baseline, BaselineKind::Tv))
        .count();
    Ok(IndicatorFixtureCounts { total, smoke, tv })
}

pub fn baseline_catalog() -> Result<Vec<IndicatorBaselineInfo>> {
    let counts = fixture_counts()?;
    Ok(vec![
        IndicatorBaselineInfo {
            baseline: BaselineKind::Smoke,
            description: BaselineKind::Smoke.description(),
            count: counts.smoke,
        },
        IndicatorBaselineInfo {
            baseline: BaselineKind::Tv,
            description: BaselineKind::Tv.description(),
            count: counts.tv,
        },
    ])
}

pub fn is_baseline_catalog_request(baseline: &str) -> bool {
    baseline == BASELINE_CATALOG_MARKER
}

pub fn run_strict(slug: &str) -> Result<IndicatorReport> {
    let fixture = load_fixture(slug)?;
    run_fixture(&fixture)
}

pub fn run_strict_filtered(
    grep: Option<&str>,
    baseline_filter: Option<&str>,
) -> Result<IndicatorBatchReport> {
    let fixtures = list_fixtures_filtered(grep, baseline_filter)?;
    let mut reports = Vec::with_capacity(fixtures.len());
    for fixture in fixtures {
        reports.push(run_strict(&fixture.slug)?);
    }
    let fixture_count = reports.len();
    if fixture_count == 0 {
        bail!(
            "no indicator fixtures matched ({})",
            filter_description(grep, baseline_filter)
        );
    }
    let passed_count = reports.iter().filter(|report| report.ok).count();
    let failed_count = fixture_count - passed_count;
    Ok(IndicatorBatchReport {
        ok: failed_count == 0,
        fixture_count,
        passed_count,
        failed_count,
        reports,
    })
}

// Lenient listing: malformed metadata or expect.json downgrades to defaults
// rather than failing the whole catalog. Validation lives in `run_strict` (and
// the `baked_fixtures_validate_strictly` test) so a single bad fixture cannot
// silently zero out `pine version` or `pine indicator --list`.
fn load_listing_lenient(slug: &str) -> IndicatorListing {
    let metadata: MetadataFile = optional_utf8(slug, "metadata.json")
        .ok()
        .flatten()
        .and_then(|json| serde_json::from_str(json).ok())
        .unwrap_or_default();
    let bars = optional_utf8(slug, "bars.json")
        .ok()
        .flatten()
        .and_then(|json| parse_bars(json).ok());
    let (expect_pine, expect_tv, output_count, test_range) = optional_utf8(slug, "expect.json")
        .ok()
        .flatten()
        .and_then(|json| parse_expect(slug, json).ok())
        .map_or((None, None, None, None), |expect| {
            (
                expect.pine_version,
                expect.tv_snapshot,
                Some(expect.outputs.len()),
                expect.test_range,
            )
        });
    IndicatorListing {
        slug: slug.to_string(),
        baseline: metadata.baseline,
        symbol: bars.as_ref().and_then(|bars| bars.symbol.clone()),
        timeframe: bars.as_ref().and_then(|bars| bars.timeframe.clone()),
        bar_count: bars.as_ref().map(|bars| bars.bars.len()),
        output_count,
        test_range,
        pine_version: expect_pine.or(metadata.pine_version),
        tv_snapshot: expect_tv.or(metadata.tv_snapshot),
    }
}

fn indicator_listing_matches(fixture: &IndicatorListing, needle: &str) -> bool {
    fixture.slug.to_ascii_lowercase().contains(needle)
        || fixture
            .baseline
            .to_string()
            .to_ascii_lowercase()
            .contains(needle)
        || fixture
            .symbol
            .as_deref()
            .is_some_and(|value| value.to_ascii_lowercase().contains(needle))
        || fixture
            .timeframe
            .as_deref()
            .is_some_and(|value| value.to_ascii_lowercase().contains(needle))
        || fixture
            .pine_version
            .as_deref()
            .is_some_and(|value| value.to_ascii_lowercase().contains(needle))
        || fixture
            .tv_snapshot
            .as_deref()
            .is_some_and(|value| value.to_ascii_lowercase().contains(needle))
        || fixture.test_range.as_ref().is_some_and(|range| {
            range.start.to_ascii_lowercase().contains(needle)
                || range.end.to_ascii_lowercase().contains(needle)
        })
}

fn parse_baseline_kind(raw: &str) -> Result<BaselineKind> {
    match raw.to_ascii_lowercase().as_str() {
        "smoke" => Ok(BaselineKind::Smoke),
        "tv" => Ok(BaselineKind::Tv),
        other => bail!("unknown indicator baseline `{other}`; expected smoke or tv"),
    }
}

fn filter_description(grep: Option<&str>, baseline_filter: Option<&str>) -> String {
    let mut parts = Vec::new();
    if let Some(grep) = grep {
        parts.push(format!("grep={grep}"));
    }
    if let Some(baseline) = baseline_filter {
        parts.push(format!("baseline={baseline}"));
    }
    if parts.is_empty() {
        "no filters".to_string()
    } else {
        parts.join(" ")
    }
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
    validate_bars(slug, &bars)?;
    let expect = parse_expect(slug, expect_json)?;
    let metadata = parse_metadata(slug, metadata_json)?;
    validate_baseline_metadata(slug, &expect, &metadata)?;
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

fn parse_expect(slug: &str, json: &str) -> Result<ExpectFile> {
    let expect: ExpectFile =
        serde_json::from_str(json).with_context(|| format!("parsing {slug}/expect.json"))?;
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
    if expect.outputs.is_empty() {
        bail!("{slug}/expect.json must define at least one output");
    }
    // test_range validation happens in `comparison_plan` so the parsed seconds
    // are reused for window selection instead of being parsed twice.
    Ok(expect)
}

fn parse_metadata(slug: &str, json: Option<&str>) -> Result<MetadataFile> {
    json.map_or_else(
        || Ok(MetadataFile::default()),
        |json| serde_json::from_str(json).with_context(|| format!("parsing {slug}/metadata.json")),
    )
}

fn validate_baseline_metadata(
    slug: &str,
    expect: &ExpectFile,
    metadata: &MetadataFile,
) -> Result<()> {
    if matches!(metadata.baseline, BaselineKind::Tv) {
        if expect
            .pine_version
            .as_ref()
            .or(metadata.pine_version.as_ref())
            .is_none()
        {
            bail!("{slug}/metadata.json baseline `tv` requires pine_version");
        }
        if expect
            .tv_snapshot
            .as_ref()
            .or(metadata.tv_snapshot.as_ref())
            .is_none()
        {
            bail!("{slug}/metadata.json baseline `tv` requires tv_snapshot");
        }
    }
    Ok(())
}

fn validate_bars(slug: &str, bars: &BarsFile) -> Result<()> {
    let mut previous_timestamp = None;
    for (index, bar) in bars.bars.iter().enumerate() {
        if bar.has_nan() {
            bail!("{slug}/bars.json bar {index} contains NaN");
        }
        if let Some(previous) = previous_timestamp
            && bar.timestamp <= previous
        {
            bail!("{slug}/bars.json timestamps must be strictly increasing");
        }
        previous_timestamp = Some(bar.timestamp);
    }
    Ok(())
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
    let actual_outputs = actual_plot_outputs(&result.pine_outputs, fixture.bars.bars.len());
    let comparison = comparison_plan(fixture)?;
    let expected_output_keys = fixture.expect.outputs.keys().cloned().collect::<Vec<_>>();
    let actual_output_keys = actual_outputs.keys().cloned().collect::<Vec<_>>();
    let mut mismatches = diff_outputs(
        &fixture.expect.outputs,
        &actual_outputs,
        fixture.expect.tolerance,
        &comparison,
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
        baseline: fixture.metadata.baseline,
        ok,
        bar_count: fixture.bars.bars.len(),
        compared_bar_count: comparison.compared_bar_count(),
        output_count: fixture.expect.outputs.len(),
        expected_output_keys,
        actual_output_keys,
        mismatch_count,
        tolerance: fixture.expect.tolerance,
        test_range: fixture.expect.test_range.clone(),
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

fn comparison_plan(fixture: &IndicatorFixture) -> Result<ComparisonPlan> {
    let bar_count = fixture.bars.bars.len();
    let Some(range) = &fixture.expect.test_range else {
        return Ok(ComparisonPlan::full(bar_count));
    };
    let (start, end) = parse_test_range(&fixture.slug, range)?;
    let bar_indices = fixture
        .bars
        .bars
        .iter()
        .enumerate()
        .filter_map(|(index, bar)| {
            (bar.timestamp >= start && bar.timestamp <= end).then_some(index)
        })
        .collect::<Vec<_>>();
    if bar_indices.is_empty() {
        bail!(
            "{}/expect.json test_range selects no bars from bars.json",
            fixture.slug
        );
    }
    let expected_is_full_series = fixture
        .expect
        .outputs
        .values()
        .all(|values| values.len() == bar_count);
    let expected_is_window_series = fixture
        .expect
        .outputs
        .values()
        .all(|values| values.len() == bar_indices.len());
    // When the range covers every bar, full-series and window-series are
    // observationally identical (projecting through bar_indices yields the same
    // sequence as indexing the sliced array). Prefer FullSeries so the chosen
    // shape stays stable as the range narrows.
    let expected_shape = match (expected_is_full_series, expected_is_window_series) {
        (true, _) => ExpectedShape::FullSeries,
        (false, true) => ExpectedShape::WindowSeries,
        (false, false) => {
            bail!(
                "{}/expect.json output lengths must match either bars.json length or test_range length",
                fixture.slug
            );
        }
    };
    Ok(ComparisonPlan::window(bar_indices, expected_shape))
}

fn parse_test_range(slug: &str, range: &TestRange) -> Result<(i64, i64)> {
    let start = parse_range_endpoint(slug, "start", &range.start)?;
    let end = parse_range_endpoint(slug, "end", &range.end)?;
    if start > end {
        bail!("{slug}/expect.json test_range start must be before or equal to end");
    }
    Ok((start, end))
}

fn parse_range_endpoint(slug: &str, label: &str, value: &str) -> Result<i64> {
    let dt = DateTime::parse_from_rfc3339(value)
        .with_context(|| format!("parsing {slug}/expect.json test_range.{label}"))?;
    Ok(dt.with_timezone(&Utc).timestamp())
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
    actual: &BTreeMap<String, Vec<OutputValue>>,
    tolerance: f64,
    comparison: &ComparisonPlan,
) -> Vec<IndicatorMismatch> {
    let mut mismatches = Vec::new();
    for (name, expected_values) in expected {
        let actual_values = actual.get(name);
        match actual_values {
            Some(actual_values) => {
                let expected_len = comparison.expected_len(expected_values);
                let actual_len = comparison.actual_len(actual_values);
                if expected_len != actual_len {
                    mismatches.push(IndicatorMismatch {
                        output: name.clone(),
                        bar_index: None,
                        reason: MismatchReason::LengthMismatch,
                        expected: None,
                        actual: None,
                        expected_len,
                        actual_len,
                    });
                }
                let len = expected_len.max(actual_len);
                for index in 0..len {
                    let expected_value = comparison
                        .expected_value(expected_values, index)
                        .unwrap_or_else(OutputValue::undefined);
                    let actual_value = comparison
                        .actual_value(actual_values, index)
                        .unwrap_or_else(OutputValue::undefined);
                    if !values_match(expected_value, actual_value, tolerance) {
                        mismatches.push(IndicatorMismatch {
                            output: name.clone(),
                            bar_index: comparison.bar_index(index),
                            reason: MismatchReason::ValueMismatch,
                            expected: Some(expected_value),
                            actual: Some(actual_value),
                            expected_len,
                            actual_len,
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
                expected_len: comparison.expected_len(expected_values),
                actual_len: 0,
            }),
        }
    }
    for name in actual.keys() {
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

impl ComparisonPlan {
    fn full(bar_count: usize) -> Self {
        Self {
            bar_indices: (0..bar_count).collect(),
            expected_shape: ExpectedShape::FullSeries,
            windowed: false,
        }
    }

    fn window(bar_indices: Vec<usize>, expected_shape: ExpectedShape) -> Self {
        Self {
            bar_indices,
            expected_shape,
            windowed: true,
        }
    }

    fn compared_bar_count(&self) -> usize {
        self.bar_indices.len()
    }

    fn expected_len(&self, values: &[OutputValue]) -> usize {
        match (self.windowed, self.expected_shape) {
            (true, ExpectedShape::FullSeries) => self.accessible_window_values(values),
            _ => values.len(),
        }
    }

    fn actual_len(&self, values: &[OutputValue]) -> usize {
        if self.windowed {
            self.accessible_window_values(values)
        } else {
            values.len()
        }
    }

    fn expected_value(&self, values: &[OutputValue], index: usize) -> Option<OutputValue> {
        match (self.windowed, self.expected_shape) {
            (true, ExpectedShape::FullSeries) => self.window_value(values, index),
            _ => values.get(index).copied(),
        }
    }

    fn actual_value(&self, values: &[OutputValue], index: usize) -> Option<OutputValue> {
        if self.windowed {
            self.window_value(values, index)
        } else {
            values.get(index).copied()
        }
    }

    fn bar_index(&self, index: usize) -> Option<usize> {
        if self.windowed {
            self.bar_indices.get(index).copied()
        } else {
            Some(index)
        }
    }

    fn accessible_window_values(&self, values: &[OutputValue]) -> usize {
        self.bar_indices
            .iter()
            .filter(|index| values.get(**index).is_some())
            .count()
    }

    fn window_value(&self, values: &[OutputValue], index: usize) -> Option<OutputValue> {
        self.bar_indices
            .get(index)
            .and_then(|bar_index| values.get(*bar_index))
            .copied()
    }
}

fn actual_plot_outputs(
    outputs: &[PineOutput],
    bar_count: usize,
) -> BTreeMap<String, Vec<OutputValue>> {
    let mut keys = HashMap::new();
    let mut actual = BTreeMap::new();
    for output in outputs {
        if output.channel != PineOutputChannel::Plot || output.event != PineOutputEventKind::Emit {
            continue;
        }
        let key_base = output_title(output).unwrap_or(&output.name);
        let key = indicator_output_key(&mut keys, key_base, output.call_site);
        let values = actual
            .entry(key)
            .or_insert_with(|| vec![OutputValue::undefined(); bar_count]);
        if let Some(slot) = values.get_mut(output.bar_index) {
            *slot = output
                .args
                .first()
                .map_or_else(OutputValue::undefined, |value| {
                    output_value_from_text(value)
                });
        }
    }
    actual
}

fn indicator_output_key(
    keys: &mut HashMap<(String, Option<usize>), String>,
    name: &str,
    call_site: Option<usize>,
) -> String {
    let identity = (name.to_string(), call_site);
    if let Some(key) = keys.get(&identity) {
        return key.clone();
    }
    let mut ordinal = 0;
    let key = loop {
        let candidate = if ordinal == 0 {
            name.to_string()
        } else {
            format!("{name}#{ordinal}")
        };
        if !keys.values().any(|key| key == &candidate) {
            break candidate;
        }
        ordinal += 1;
    };
    keys.insert(identity, key.clone());
    key
}

fn output_title(output: &PineOutput) -> Option<&str> {
    if !matches!(
        output.name.as_str(),
        "plot" | "plotarrow" | "plotchar" | "plotshape"
    ) {
        return None;
    }
    // piners-runner stores the title at args[1] for both positional and
    // named-argument calls; the named-title fixture pins that contract.
    output.args.get(1).and_then(|title| {
        let title = title.trim();
        (!title.is_empty() && title != "na").then_some(title)
    })
}

fn output_value_from_text(text: &str) -> OutputValue {
    match text {
        "na" | "NaN" => OutputValue::special(OutputValueKind::Na),
        "inf" | "Infinity" => OutputValue::special(OutputValueKind::PosInfinity),
        "-inf" | "-Infinity" => OutputValue::special(OutputValueKind::NegInfinity),
        "true" => OutputValue::bool(true),
        "false" => OutputValue::bool(false),
        _ => text
            .parse::<f64>()
            .map_or_else(|_| OutputValue::undefined(), OutputValue::from_f64),
    }
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

    const RANGE_BARS: &str = r#"{
        "symbol": "NASDAQ:SPY",
        "timeframe": "1D",
        "source": "test",
        "bars": [
            {"timestamp": 1735689600, "open": 10.0, "high": 11.0, "low": 9.0, "close": 10.0, "volume": 100.0},
            {"timestamp": 1735776000, "open": 10.0, "high": 12.0, "low": 9.0, "close": 11.0, "volume": 110.0},
            {"timestamp": 1735862400, "open": 11.0, "high": 13.0, "low": 10.0, "close": 12.0, "volume": 120.0}
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
        assert_eq!(report.expected_output_keys, vec!["plot"]);
        assert_eq!(report.actual_output_keys, vec!["plot"]);
    }

    #[test]
    fn strict_fixture_matches_named_plot_titles() {
        let fixture = parse_fixture(
            "named-titles",
            "indicator(\"fixture\")\nplot(close, title=\"Close Line\")\nplotshape(close > open, title=\"Up Shape\")\n".to_string(),
            BARS,
            r#"{
                "schema_version": 1,
                "indicator_slug": "named-titles",
                "outputs": {
                    "Close Line": [10.0, 11.0],
                    "Up Shape": [false, true]
                }
            }"#,
            None,
        )
        .expect("fixture");
        let report = run_fixture(&fixture).expect("run");
        assert!(report.ok, "{:?}", report.mismatches);
        assert_eq!(report.expected_output_keys, vec!["Close Line", "Up Shape"]);
        assert_eq!(report.actual_output_keys, vec!["Close Line", "Up Shape"]);
    }

    #[test]
    fn output_keys_do_not_collide_with_literal_hash_titles() {
        let mut keys = HashMap::new();
        assert_eq!(
            indicator_output_key(&mut keys, "Signal#1", Some(10)),
            "Signal#1"
        );
        assert_eq!(
            indicator_output_key(&mut keys, "Signal", Some(20)),
            "Signal"
        );
        assert_eq!(
            indicator_output_key(&mut keys, "Signal", Some(30)),
            "Signal#2"
        );
    }

    #[test]
    fn duplicate_output_keys_reuse_call_site_identity() {
        let mut keys = HashMap::new();
        assert_eq!(
            indicator_output_key(&mut keys, "Signal", Some(10)),
            "Signal"
        );
        assert_eq!(
            indicator_output_key(&mut keys, "Signal", Some(20)),
            "Signal#1"
        );
        assert_eq!(
            indicator_output_key(&mut keys, "Signal", Some(10)),
            "Signal"
        );
    }

    #[test]
    fn baked_fixtures_validate_strictly() {
        let entries = INDICATORS
            .find("**/source.pine")
            .expect("walking baked fixtures");
        let mut count = 0;
        for entry in entries {
            let Some(file) = entry.as_file() else {
                continue;
            };
            let Some(parent) = file.path().parent() else {
                continue;
            };
            let Some(slug) = parent.to_str() else {
                continue;
            };
            if slug.is_empty() {
                continue;
            }
            load_fixture(slug)
                .unwrap_or_else(|err| panic!("fixture {slug} failed strict validation: {err}"));
            count += 1;
        }
        assert!(count >= 10, "expected baked smoke fixtures, found {count}");
    }

    #[test]
    fn baked_smoke_fixtures_run_cleanly() {
        for slug in [
            "smoke-close",
            "smoke-close-plus-one",
            "smoke-duplicate-titles",
            "smoke-na-output",
            "smoke-plotshape-bool",
            "smoke-request-security-current",
            "smoke-titled-outputs",
            "smoke-two-plots",
            "smoke-sma-warmup",
            "smoke-test-range",
        ] {
            let report = run_strict(slug).expect("strict run");
            assert!(report.ok, "{slug}: {:?}", report.mismatches);
            assert_eq!(report.baseline, BaselineKind::Smoke);
        }
    }

    #[test]
    fn baked_fixture_list_contains_smoke_fixtures() {
        let fixtures = list_fixtures().expect("fixtures");
        let slugs = fixtures
            .iter()
            .map(|fixture| fixture.slug.as_str())
            .collect::<Vec<_>>();
        assert!(slugs.contains(&"smoke-close"));
        assert!(slugs.contains(&"smoke-close-plus-one"));
        assert!(slugs.contains(&"smoke-duplicate-titles"));
        assert!(slugs.contains(&"smoke-na-output"));
        assert!(slugs.contains(&"smoke-plotshape-bool"));
        assert!(slugs.contains(&"smoke-request-security-current"));
        assert!(slugs.contains(&"smoke-titled-outputs"));
        assert!(slugs.contains(&"smoke-two-plots"));
        assert!(slugs.contains(&"smoke-sma-warmup"));
        assert!(slugs.contains(&"smoke-test-range"));
        for fixture in fixtures
            .iter()
            .filter(|fixture| fixture.slug.starts_with("smoke-"))
        {
            assert_eq!(fixture.baseline, BaselineKind::Smoke);
        }
        let counts = fixture_counts().expect("counts");
        assert_eq!(counts.total, fixtures.len());
        assert!(counts.smoke >= 10);
    }

    #[test]
    fn fixture_listings_include_shape_counts_and_range() {
        let fixtures = list_fixtures().expect("fixtures");
        let close = fixtures
            .iter()
            .find(|fixture| fixture.slug == "smoke-close")
            .expect("smoke-close listing");
        assert_eq!(close.symbol.as_deref(), Some("SMOKE:FIXTURE"));
        assert_eq!(close.timeframe.as_deref(), Some("1D"));
        assert_eq!(close.bar_count, Some(4));
        assert_eq!(close.output_count, Some(1));
        assert!(close.test_range.is_none());

        let ranged = fixtures
            .iter()
            .find(|fixture| fixture.slug == "smoke-test-range")
            .expect("smoke-test-range listing");
        assert!(ranged.test_range.is_some());
    }

    #[test]
    fn fixture_list_filters_by_grep_and_baseline() {
        let fixtures =
            list_fixtures_filtered(Some("request"), Some("SMOKE")).expect("filtered fixtures");
        assert!(
            fixtures
                .iter()
                .any(|fixture| fixture.slug == "smoke-request-security-current")
        );
        assert!(
            fixtures
                .iter()
                .all(|fixture| fixture.baseline == BaselineKind::Smoke)
        );
    }

    #[test]
    fn indicator_baseline_catalog_counts_fixtures() {
        let counts = fixture_counts().expect("counts");
        let catalog = baseline_catalog().expect("catalog");
        let catalog_total = catalog.iter().map(|entry| entry.count).sum::<usize>();
        assert_eq!(catalog_total, counts.total);
        assert!(
            catalog
                .iter()
                .any(|entry| entry.baseline == BaselineKind::Smoke && entry.count == counts.smoke)
        );
        assert!(
            catalog
                .iter()
                .any(|entry| entry.baseline == BaselineKind::Tv && entry.count == counts.tv)
        );
    }

    #[test]
    fn invalid_baseline_filter_errors() {
        let err = list_fixtures_filtered(None, Some("paper")).expect_err("must reject");
        assert!(err.to_string().contains("unknown indicator baseline"));
    }

    #[test]
    fn strict_batch_runs_matching_fixtures() {
        let report =
            run_strict_filtered(Some("request-security"), Some("smoke")).expect("batch run");
        assert!(report.ok, "{:?}", report.reports);
        assert_eq!(report.fixture_count, 1);
        assert_eq!(report.passed_count, 1);
        assert_eq!(report.failed_count, 0);
        assert_eq!(report.reports[0].slug, "smoke-request-security-current");
    }

    #[test]
    fn strict_batch_rejects_empty_filter_result() {
        let err = run_strict_filtered(Some("definitely-not-a-fixture"), Some("smoke"))
            .expect_err("must reject empty batch");
        assert!(err.to_string().contains("no indicator fixtures matched"));
        assert!(err.to_string().contains("grep=definitely-not-a-fixture"));
        assert!(err.to_string().contains("baseline=smoke"));
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
    fn test_range_compares_sliced_expected_values() {
        let fixture = parse_fixture(
            "range-close",
            "indicator(\"fixture\")\nplot(close)\n".to_string(),
            RANGE_BARS,
            r#"{
                "schema_version": 1,
                "indicator_slug": "range-close",
                "outputs": {"plot": [11.0, 12.0]},
                "test_range": {
                    "start": "2025-01-02T00:00:00Z",
                    "end": "2025-01-03T00:00:00Z"
                }
            }"#,
            None,
        )
        .expect("fixture");
        let report = run_fixture(&fixture).expect("run");
        assert!(report.ok, "{:?}", report.mismatches);
        assert_eq!(report.bar_count, 3);
        assert_eq!(report.compared_bar_count, 2);
    }

    #[test]
    fn test_range_accepts_full_series_expected() {
        let fixture = parse_fixture(
            "range-close-full",
            "indicator(\"fixture\")\nplot(close)\n".to_string(),
            RANGE_BARS,
            r#"{
                "schema_version": 1,
                "indicator_slug": "range-close-full",
                "outputs": {"plot": ["__NaN__", 11.0, "__NaN__"]},
                "test_range": {
                    "start": "2025-01-02T00:00:00Z",
                    "end": "2025-01-02T00:00:00Z"
                }
            }"#,
            None,
        )
        .expect("fixture");
        let report = run_fixture(&fixture).expect("run");
        assert!(report.ok, "{:?}", report.mismatches);
        assert_eq!(report.bar_count, 3);
        assert_eq!(report.compared_bar_count, 1);
    }

    #[test]
    fn test_range_reports_original_bar_index() {
        let fixture = parse_fixture(
            "range-close",
            "indicator(\"fixture\")\nplot(close)\n".to_string(),
            RANGE_BARS,
            r#"{
                "schema_version": 1,
                "indicator_slug": "range-close",
                "outputs": {"plot": [99.0]},
                "test_range": {
                    "start": "2025-01-02T00:00:00Z",
                    "end": "2025-01-02T00:00:00Z"
                }
            }"#,
            None,
        )
        .expect("fixture");
        let report = run_fixture(&fixture).expect("run");
        assert!(!report.ok);
        assert_eq!(report.mismatch_count, 1);
        assert_eq!(report.mismatches[0].bar_index, Some(1));
    }

    #[test]
    fn missing_output_is_reported_once() {
        let mut actual = BTreeMap::new();
        actual.insert("plot".to_string(), vec![OutputValue::number(1.0)]);
        let expected = BTreeMap::from([("plot#1".to_string(), vec![OutputValue::number(2.0)])]);
        let mismatches = diff_outputs(&expected, &actual, 0.0, &ComparisonPlan::full(1));
        assert_eq!(mismatches.len(), 2);
        assert_eq!(mismatches[0].reason, MismatchReason::MissingOutput);
        assert_eq!(mismatches[1].reason, MismatchReason::UnexpectedOutput);
    }

    #[test]
    fn parses_runner_output_text_values() {
        assert!(values_match(
            output_value_from_text("12.5"),
            OutputValue::number(12.5),
            0.0
        ));
        assert!(values_match(
            output_value_from_text("na"),
            OutputValue::special(OutputValueKind::Na),
            0.0
        ));
        assert!(values_match(
            output_value_from_text("true"),
            OutputValue::bool(true),
            0.0
        ));
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
