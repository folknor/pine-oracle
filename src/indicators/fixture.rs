// SPDX-License-Identifier: MPL-2.0

use std::path::{Component, Path};

use anyhow::{Context, Result, anyhow, bail};
use include_dir::{Dir, include_dir};
use piners_runner::Bar;
use piners_runner::types::BarSeriesContext;
use serde::{Deserialize, Serialize};

use super::types::{
    BaselineKind, EXPECT_SCHEMA_VERSION, IndicatorBaselineInfo, IndicatorFixtureCounts,
    IndicatorListing, OutputValue, TestRange,
};

pub(super) static INDICATORS: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/indicators");
const MAX_EXPECT_TOLERANCE: f64 = 1e-3;
const DAILY_TIMEFRAME_TOLERANCE_SECONDS: i64 = 60 * 60;
const CALENDAR_MONTH_MIN_SPACING_SECONDS: i64 = 27 * 24 * 60 * 60;

#[derive(Debug, Clone, Deserialize)]
pub(super) struct BarsFile {
    #[serde(default)]
    pub(super) symbol: Option<String>,
    #[serde(default)]
    pub(super) timeframe: Option<String>,
    #[serde(default)]
    pub(super) source: Option<String>,
    #[serde(default)]
    pub(super) context: Option<BarSeriesContext>,
    pub(super) bars: Vec<Bar>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
enum BarsDocument {
    Full(Box<BarsFile>),
    Bare(Vec<Bar>),
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub(super) struct ExpectFile {
    pub(super) schema_version: u32,
    pub(super) indicator_slug: String,
    #[serde(default)]
    pub(super) pine_version: Option<String>,
    #[serde(default)]
    pub(super) tv_snapshot: Option<String>,
    #[serde(default = "default_tolerance")]
    pub(super) tolerance: f64,
    pub(super) outputs: std::collections::BTreeMap<String, Vec<OutputValue>>,
    #[serde(default)]
    pub(super) test_range: Option<TestRange>,
}

#[derive(Default, Debug, Clone, Deserialize, Serialize)]
pub(super) struct MetadataFile {
    #[serde(default)]
    pub(super) baseline: BaselineKind,
    #[serde(default)]
    pub(super) pine_version: Option<String>,
    #[serde(default)]
    pub(super) tv_snapshot: Option<String>,
    #[serde(default)]
    pub(super) notes: Option<String>,
}

#[derive(Debug, Clone)]
pub(super) struct IndicatorFixture {
    pub(super) slug: String,
    pub(super) source: String,
    pub(super) bars: BarsFile,
    pub(super) expect: ExpectFile,
    pub(super) metadata: MetadataFile,
}

impl IndicatorFixture {
    /// `expect.pine_version` takes precedence; falls back to `metadata.pine_version`.
    pub(super) fn effective_pine_version(&self) -> Option<String> {
        effective_pine_version_inner(
            self.expect.pine_version.as_ref(),
            self.metadata.pine_version.as_ref(),
        )
    }

    /// `expect.tv_snapshot` takes precedence; falls back to `metadata.tv_snapshot`.
    /// No `BaselineKind::Smoke` suppression here -- callers that need that constraint
    /// apply it themselves (e.g. `load_listing_lenient`).
    pub(super) fn effective_tv_snapshot(&self) -> Option<String> {
        effective_tv_snapshot_inner(
            self.expect.tv_snapshot.as_ref(),
            self.metadata.tv_snapshot.as_ref(),
        )
    }
}

/// Inner helper so both `IndicatorFixture::effective_pine_version` and
/// `load_listing_lenient` (which operates before a fixture is constructed) share
/// identical precedence logic.
fn effective_pine_version_inner(
    expect_version: Option<&String>,
    metadata_version: Option<&String>,
) -> Option<String> {
    expect_version.or(metadata_version).cloned()
}

/// Inner helper so both `IndicatorFixture::effective_tv_snapshot` and
/// `load_listing_lenient` share identical precedence logic.
fn effective_tv_snapshot_inner(
    expect_snapshot: Option<&String>,
    metadata_snapshot: Option<&String>,
) -> Option<String> {
    expect_snapshot.or(metadata_snapshot).cloned()
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

/// Returns `true` when `baseline` is the catalog sentinel `"?"`.
/// Thin delegate kept for library consumers that import `pine_cli::indicator`
/// directly; the binary uses `output::is_catalog_request` instead.
pub fn is_baseline_catalog_request(baseline: &str) -> bool {
    baseline == "?"
}

// Lenient listing: malformed metadata, bars.json, or expect.json downgrades to
// defaults rather than failing the whole catalog. A warning is emitted to stderr
// so misconfigured fixtures are visible without breaking the catalog entirely.
// Strict validation lives in `run_strict` (and the
// `baked_fixtures_validate_strictly` test).
fn load_listing_lenient(slug: &str) -> IndicatorListing {
    let metadata: MetadataFile = optional_utf8(slug, "metadata.json")
        .ok()
        .flatten()
        .map(serde_json::from_str::<MetadataFile>)
        .transpose()
        .unwrap_or_else(|e| {
            eprintln!("warning: indicator fixture {slug}: metadata.json parse failed: {e}");
            None
        })
        .unwrap_or_default();
    let bars = optional_utf8(slug, "bars.json")
        .ok()
        .flatten()
        .map(parse_bars)
        .transpose()
        .unwrap_or_else(|e| {
            eprintln!("warning: indicator fixture {slug}: bars.json parse failed: {e}");
            None
        });
    let (expect_pine, expect_tv, output_count, test_range) = optional_utf8(slug, "expect.json")
        .ok()
        .flatten()
        .map(|json| parse_expect(slug, json))
        .transpose()
        .unwrap_or_else(|e| {
            eprintln!("warning: indicator fixture {slug}: expect.json parse failed: {e}");
            None
        })
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
        pine_version: effective_pine_version_inner(
            expect_pine.as_ref(),
            metadata.pine_version.as_ref(),
        ),
        tv_snapshot: if metadata.baseline == BaselineKind::Smoke {
            None
        } else {
            effective_tv_snapshot_inner(expect_tv.as_ref(), metadata.tv_snapshot.as_ref())
        },
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

pub(super) fn filter_description(grep: Option<&str>, baseline_filter: Option<&str>) -> String {
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

pub(super) fn load_fixture(slug: &str) -> Result<IndicatorFixture> {
    let slug = sanitise_slug(slug)?;
    let source = require_utf8(slug, "source.pine")?.to_string();
    let bars_json = require_utf8(slug, "bars.json")?;
    let expect_json = require_utf8(slug, "expect.json")?;
    let metadata_json = optional_utf8(slug, "metadata.json")?;
    parse_fixture(slug, source, bars_json, expect_json, metadata_json)
}

pub(super) fn parse_fixture(
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

pub(super) fn parse_expect(slug: &str, json: &str) -> Result<ExpectFile> {
    let expect: ExpectFile =
        serde_json::from_str(json).with_context(|| format!("parsing {slug}/expect.json"))?;
    if expect.schema_version != EXPECT_SCHEMA_VERSION {
        bail!(
            "{slug}/expect.json has unsupported schema_version {}; expected {}",
            expect.schema_version,
            EXPECT_SCHEMA_VERSION
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
    if expect.tolerance > MAX_EXPECT_TOLERANCE {
        bail!(
            "{slug}/expect.json tolerance {} is too loose; expected <= {}",
            expect.tolerance,
            MAX_EXPECT_TOLERANCE
        );
    }
    if expect.outputs.is_empty() {
        bail!("{slug}/expect.json must define at least one output");
    }
    for (name, values) in &expect.outputs {
        if name.trim().is_empty() {
            bail!("{slug}/expect.json output keys must not be empty");
        }
        if values.is_empty() {
            bail!("{slug}/expect.json output `{name}` must define at least one value");
        }
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
    match metadata.baseline {
        BaselineKind::Tv => {
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
        BaselineKind::Smoke => {
            if expect.tv_snapshot.is_some() || metadata.tv_snapshot.is_some() {
                bail!("{slug} smoke fixtures must not define tv_snapshot");
            }
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
    validate_timeframe_spacing(slug, bars)?;
    Ok(())
}

fn validate_timeframe_spacing(slug: &str, bars: &BarsFile) -> Result<()> {
    let Some(timeframe) = bars.timeframe.as_deref() else {
        return Ok(());
    };
    let Some(min_seconds) = timeframe_min_spacing_seconds(timeframe) else {
        return Ok(());
    };
    for (index, pair) in bars.bars.windows(2).enumerate() {
        let delta = pair[1].timestamp - pair[0].timestamp;
        if delta < min_seconds {
            bail!(
                "{slug}/bars.json timestamp delta between bars {index} and {} is {delta}s, shorter than timeframe `{timeframe}` minimum {min_seconds}s",
                index + 1
            );
        }
    }
    Ok(())
}

fn timeframe_min_spacing_seconds(timeframe: &str) -> Option<i64> {
    let timeframe = timeframe.trim().to_ascii_uppercase();
    if timeframe.is_empty() {
        return None;
    }
    if let Ok(minutes) = timeframe.parse::<i64>() {
        return positive_seconds(minutes, 60);
    }
    let unit_start = timeframe
        .char_indices()
        .find_map(|(index, ch)| (!ch.is_ascii_digit()).then_some(index))
        .unwrap_or(timeframe.len());
    let (count, unit) = timeframe.split_at(unit_start);
    let count = if count.is_empty() {
        1
    } else {
        count.parse::<i64>().ok()?
    };
    let seconds = match unit {
        "S" => positive_seconds(count, 1)?,
        "H" => positive_seconds(count, 60 * 60)?,
        "D" => positive_seconds(count, 24 * 60 * 60)?,
        "W" => positive_seconds(count, 7 * 24 * 60 * 60)?,
        "M" => positive_seconds(count, CALENDAR_MONTH_MIN_SPACING_SECONDS)?,
        _ => return None,
    };
    let tolerance = match unit {
        "D" | "W" | "M" => DAILY_TIMEFRAME_TOLERANCE_SECONDS,
        _ => 0,
    };
    Some(seconds.saturating_sub(tolerance).max(1))
}

fn positive_seconds(count: i64, unit_seconds: i64) -> Option<i64> {
    if count <= 0 {
        return None;
    }
    count.checked_mul(unit_seconds)
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

pub(super) fn sanitise_slug(slug: &str) -> Result<&str> {
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

    fn make_fixture(
        expect_pine: Option<&str>,
        expect_tv: Option<&str>,
        meta_pine: Option<&str>,
        meta_tv: Option<&str>,
    ) -> IndicatorFixture {
        IndicatorFixture {
            slug: "test-fixture".to_string(),
            source: String::new(),
            bars: BarsFile {
                symbol: None,
                timeframe: None,
                source: None,
                context: None,
                bars: vec![],
            },
            expect: ExpectFile {
                schema_version: EXPECT_SCHEMA_VERSION,
                indicator_slug: "test-fixture".to_string(),
                pine_version: expect_pine.map(String::from),
                tv_snapshot: expect_tv.map(String::from),
                tolerance: 1e-9,
                outputs: std::collections::BTreeMap::new(),
                test_range: None,
            },
            metadata: MetadataFile {
                baseline: BaselineKind::Smoke,
                pine_version: meta_pine.map(String::from),
                tv_snapshot: meta_tv.map(String::from),
                notes: None,
            },
        }
    }

    // --- effective_pine_version precedence ---

    #[test]
    fn effective_pine_version_expect_wins_over_metadata() {
        let fixture = make_fixture(Some("from-expect"), None, Some("from-meta"), None);
        assert_eq!(
            fixture.effective_pine_version().as_deref(),
            Some("from-expect")
        );
    }

    #[test]
    fn effective_pine_version_falls_back_to_metadata() {
        let fixture = make_fixture(None, None, Some("from-meta"), None);
        assert_eq!(
            fixture.effective_pine_version().as_deref(),
            Some("from-meta")
        );
    }

    #[test]
    fn effective_pine_version_both_none_returns_none() {
        let fixture = make_fixture(None, None, None, None);
        assert!(fixture.effective_pine_version().is_none());
    }

    // --- effective_tv_snapshot precedence ---

    #[test]
    fn effective_tv_snapshot_expect_wins_over_metadata() {
        let fixture = make_fixture(None, Some("snap-expect"), None, Some("snap-meta"));
        assert_eq!(
            fixture.effective_tv_snapshot().as_deref(),
            Some("snap-expect")
        );
    }

    #[test]
    fn effective_tv_snapshot_falls_back_to_metadata() {
        let fixture = make_fixture(None, None, None, Some("snap-meta"));
        assert_eq!(
            fixture.effective_tv_snapshot().as_deref(),
            Some("snap-meta")
        );
    }

    #[test]
    fn effective_tv_snapshot_both_none_returns_none() {
        let fixture = make_fixture(None, None, None, None);
        assert!(fixture.effective_tv_snapshot().is_none());
    }

    // --- inner helpers used by load_listing_lenient ---

    #[test]
    fn inner_pine_version_expect_over_meta() {
        let e = String::from("a");
        let m = String::from("b");
        assert_eq!(
            effective_pine_version_inner(Some(&e), Some(&m)).as_deref(),
            Some("a")
        );
    }

    #[test]
    fn inner_pine_version_falls_back_when_expect_none() {
        let m = String::from("b");
        assert_eq!(
            effective_pine_version_inner(None, Some(&m)).as_deref(),
            Some("b")
        );
    }

    #[test]
    fn inner_pine_version_both_none() {
        assert!(effective_pine_version_inner(None, None).is_none());
    }

    // --- load_listing_lenient with malformed metadata.json ---
    // The slug points to a fixture directory that exists in the embedded
    // INDICATORS dir; if it does not exist the listing still returns a
    // IndicatorListing with default values (no panic).
    #[test]
    fn load_listing_lenient_nonexistent_slug_returns_defaults() {
        // A slug that doesn't exist in INDICATORS returns a default listing
        // without panicking. This exercises the None path of all optional_utf8
        // branches.
        let listing = load_listing_lenient("nonexistent-fixture-xyz");
        assert_eq!(listing.slug, "nonexistent-fixture-xyz");
        assert_eq!(listing.baseline, BaselineKind::Smoke);
        assert!(listing.pine_version.is_none());
        assert!(listing.tv_snapshot.is_none());
        assert!(listing.bar_count.is_none());
        assert!(listing.output_count.is_none());
    }
}
