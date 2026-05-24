// SPDX-License-Identifier: Apache-2.0 OR MPL-2.0
//
// Baked PineForge validation corpus.
//
// The vendored tree under `vendor/pineforge-corpus/validation/` is embedded
// into the binary at compile time via include_dir. Probes are looked up by
// the published-corpus slug, which is the relative path from the validation
// root to the directory containing `strategy.pine`. Most slugs are flat
// (e.g. `oca-multi-bracket-isolation-01`); some live under nested roots
// (e.g. `symbol-specified/AAPL/session-ismarket-nyse-rth-01`).
//
// Per-probe summaries are currently absent: docs/probe-summaries.md was
// authored against engine-internal probe identifiers that do not match the
// published corpus slugs, so re-curation is open work. `Probe::summary` and
// `ProbeListing::summary` remain in the API as `Option<&'static str>` so
// callers stay stable when the re-curated map lands.

use anyhow::{anyhow, bail, Result};
use include_dir::{include_dir, Dir};
use serde::Serialize;

static CORPUS: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/vendor/pineforge-corpus/validation");

#[derive(Debug, Clone, Serialize)]
pub struct Probe {
    pub slug: String,
    pub strategy_pine: &'static str,
    pub tv_trades_csv: &'static str,
    pub inputs_json: Option<&'static str>,
    pub summary: Option<&'static str>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProbeListing {
    pub slug: String,
    pub summary: Option<&'static str>,
}

/// Load one probe by its published-corpus slug. Accepts an optional
/// `validation/` prefix for convenience.
pub fn load_probe(slug: &str) -> Result<Probe> {
    let slug = sanitise_slug(slug)?;

    let strategy_pine = require_utf8(slug, "strategy.pine")?;
    let tv_trades_csv = require_utf8(slug, "tv_trades.csv")?;
    let inputs_json = optional_utf8(slug, "inputs.json")?;

    Ok(Probe {
        slug: slug.to_string(),
        strategy_pine,
        tv_trades_csv,
        inputs_json,
        summary: summary_for(slug),
    })
}

/// List every baked probe. `grep`, when present, filters by case-insensitive
/// substring against the slug. Summary-text filtering will return once the
/// re-curated summaries land.
pub fn list_probes(grep: Option<&str>) -> Result<Vec<ProbeListing>> {
    let needle = grep.map(|s| s.to_ascii_lowercase());
    let entries = CORPUS
        .find("**/strategy.pine")
        .map_err(|e| anyhow!("walking corpus: {e}"))?;

    let mut out: Vec<ProbeListing> = entries
        .filter_map(|entry| {
            let file = entry.as_file()?;
            let parent = file.path().parent()?;
            let slug = parent.to_str()?.to_string();
            if let Some(n) = &needle {
                if !slug.to_ascii_lowercase().contains(n) {
                    return None;
                }
            }
            let summary = summary_for(&slug);
            Some(ProbeListing { slug, summary })
        })
        .collect();
    out.sort_by(|a, b| a.slug.cmp(&b.slug));
    Ok(out)
}

/// Always returns None until the slug-aligned summary map is re-curated. See
/// the module-level doc comment.
fn summary_for(_slug: &str) -> Option<&'static str> {
    None
}

fn sanitise_slug(slug: &str) -> Result<&str> {
    let slug = slug.trim();
    let slug = slug.strip_prefix("validation/").unwrap_or(slug);
    if slug.is_empty() {
        bail!("probe slug cannot be empty");
    }
    if slug.starts_with('/') || slug.starts_with('\\') {
        bail!("probe slug cannot be absolute: `{slug}`");
    }
    for segment in slug.split(['/', '\\']) {
        if segment.is_empty() || segment == "." || segment == ".." {
            bail!("probe slug has invalid segment `{segment}` in `{slug}`");
        }
    }
    Ok(slug)
}

fn require_utf8(slug: &str, filename: &str) -> Result<&'static str> {
    let path = format!("{slug}/{filename}");
    let file = CORPUS
        .get_file(&path)
        .ok_or_else(|| anyhow!("probe `{slug}` is missing {filename}"))?;
    file.contents_utf8()
        .ok_or_else(|| anyhow!("probe `{slug}/{filename}` is not valid UTF-8"))
}

fn optional_utf8(slug: &str, filename: &str) -> Result<Option<&'static str>> {
    let path = format!("{slug}/{filename}");
    let Some(file) = CORPUS.get_file(&path) else {
        return Ok(None);
    };
    match file.contents_utf8() {
        Some(s) => Ok(Some(s)),
        None => bail!("probe `{slug}/{filename}` is not valid UTF-8"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const FLAT_SLUG: &str = "anomaly-equity-mirror-strategy-equity-01";
    const NESTED_SLUG: &str = "symbol-specified/AAPL/session-ismarket-nyse-rth-01";

    #[test]
    fn loads_a_flat_probe() {
        let probe = load_probe(FLAT_SLUG).expect("flat slug must load");
        assert_eq!(probe.slug, FLAT_SLUG);
        assert!(!probe.strategy_pine.is_empty());
        assert!(!probe.tv_trades_csv.is_empty());
    }

    #[test]
    fn loads_a_nested_probe() {
        let probe = load_probe(NESTED_SLUG).expect("nested slug must load");
        assert_eq!(probe.slug, NESTED_SLUG);
        assert!(!probe.strategy_pine.is_empty());
        assert!(!probe.tv_trades_csv.is_empty());
    }

    #[test]
    fn unknown_slug_errors() {
        let err = load_probe("definitely-not-a-real-probe").expect_err("must error");
        assert!(err.to_string().contains("missing"));
    }

    #[test]
    fn accepts_validation_prefix() {
        let probe =
            load_probe(&format!("validation/{FLAT_SLUG}")).expect("validation/ prefix must work");
        assert_eq!(probe.slug, FLAT_SLUG);
    }

    #[test]
    fn rejects_dot_dot_segment() {
        let err = load_probe("foo/../bar").expect_err("`..` in slug must error");
        assert!(err.to_string().contains("invalid segment"));
    }

    #[test]
    fn rejects_empty_slug() {
        let err = load_probe("").expect_err("empty slug must error");
        assert!(err.to_string().contains("empty"));
    }

    #[test]
    fn lists_every_probe_including_nested() {
        let all = list_probes(None).expect("list must succeed");
        assert!(
            all.len() >= 200,
            "expected at least 200 baked probes, got {}",
            all.len()
        );
        assert!(all.iter().any(|p| p.slug == FLAT_SLUG));
        assert!(all.iter().any(|p| p.slug == NESTED_SLUG));
    }

    #[test]
    fn grep_filters_by_slug_substring() {
        let oca = list_probes(Some("oca")).expect("grep must succeed");
        assert!(!oca.is_empty(), "expected some oca matches");
        assert!(oca
            .iter()
            .all(|p| p.slug.to_ascii_lowercase().contains("oca")));
    }
}
