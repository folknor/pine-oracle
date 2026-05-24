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
// Per-probe summaries are extracted live from each strategy.pine's header
// comment block - the strategy author's own one-paragraph description. The
// extractor skips license / SPDX / copyright lines and the `//@version`
// directive, takes the first prose comment block, and stops at the first
// blank `//` or non-comment line after prose begins. Cached behind a
// OnceLock so repeat lookups are cheap. >=80% of the 235 baked probes
// have author-written summaries that this picks up. docs/probe-summaries.md
// still ships richer engine-internals prose for ~21 probes; that file is
// not currently keyed to published slugs and so is not loaded here.

use anyhow::{anyhow, bail, Result};
use include_dir::{include_dir, Dir};
use serde::Serialize;
use std::sync::OnceLock;

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
/// substring against the slug OR the extracted summary text.
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
            let summary = summary_for(&slug);
            if let Some(n) = &needle {
                let slug_hit = slug.to_ascii_lowercase().contains(n);
                let summary_hit = summary
                    .map(|s| s.to_ascii_lowercase().contains(n))
                    .unwrap_or(false);
                if !slug_hit && !summary_hit {
                    return None;
                }
            }
            Some(ProbeListing { slug, summary })
        })
        .collect();
    out.sort_by(|a, b| a.slug.cmp(&b.slug));
    Ok(out)
}

/// Pull the first prose comment block out of a `strategy.pine` source.
///
/// Each baked strategy.pine carries an author-written header comment block
/// (Apache-2.0 boilerplate first, then a blank `//` separator, then a
/// title + purpose paragraph). The extractor:
///   - Skips license / SPDX / copyright lines.
///   - Skips `//@version=` directives.
///   - Skips empty `//` separators until the first prose line.
///   - Collects contiguous prose comment lines into one space-joined string.
///   - Stops at the first blank `//`, blank line, or non-comment line after
///     prose begins.
///
/// The output is the strategy author's own one-paragraph description, which
/// gives 235 probes real summaries without an LLM curation pass and unblocks
/// `pine probes --grep <text>` matching against summary content.
fn extract_summary(strategy_pine: &str) -> Option<String> {
    let mut lines = Vec::new();
    let mut in_prose = false;
    for line in strategy_pine.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("//") {
            let content = rest.trim_start_matches('/').trim();
            if content.is_empty() {
                if in_prose {
                    break;
                }
                continue;
            }
            if is_header_noise(content) {
                continue;
            }
            in_prose = true;
            lines.push(content.to_string());
        } else if trimmed.is_empty() {
            if in_prose {
                break;
            }
        } else {
            break;
        }
    }
    if lines.is_empty() {
        None
    } else {
        Some(lines.join(" "))
    }
}

fn is_header_noise(content: &str) -> bool {
    let lower = content.to_ascii_lowercase();
    lower.contains("spdx-license-identifier")
        || lower.contains("licensed under")
        || lower.starts_with("(c)")
        || content.starts_with('\u{00A9}') // (c) symbol
        || content.starts_with("@version")
        || lower.contains("pine script\u{00ae} code is licensed")
}

/// Summary for the given baked probe, derived live from the strategy.pine
/// header comments. Cached behind a OnceLock so repeat lookups are cheap.
fn summary_for(slug: &str) -> Option<&'static str> {
    use std::collections::HashMap;
    static SUMMARIES: OnceLock<HashMap<String, &'static str>> = OnceLock::new();
    let map = SUMMARIES.get_or_init(build_summary_index);
    map.get(slug).copied()
}

fn build_summary_index() -> std::collections::HashMap<String, &'static str> {
    use std::collections::HashMap;
    let mut out: HashMap<String, &'static str> = HashMap::new();
    let Ok(entries) = CORPUS.find("**/strategy.pine") else {
        return out;
    };
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
        let Some(content) = file.contents_utf8() else {
            continue;
        };
        if let Some(summary) = extract_summary(content) {
            // Leak the summary string so the map can hand out `&'static str`
            // for the lifetime of the binary. ~235 entries; the leak is
            // bounded and amortised by the cache.
            let leaked: &'static str = Box::leak(summary.into_boxed_str());
            out.insert(slug.to_string(), leaked);
        }
    }
    out
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
    fn grep_filters_by_slug_or_summary_substring() {
        let oca = list_probes(Some("oca")).expect("grep must succeed");
        assert!(!oca.is_empty(), "expected some oca matches");
        assert!(oca.iter().all(|p| {
            let slug_hit = p.slug.to_ascii_lowercase().contains("oca");
            let summary_hit = p
                .summary
                .map(|s| s.to_ascii_lowercase().contains("oca"))
                .unwrap_or(false);
            slug_hit || summary_hit
        }));
    }

    #[test]
    fn extracts_summary_skipping_license_header() {
        let src = "// This Pine Script\u{00ae} code is licensed under Apache-2.0\n\
                   // SPDX-License-Identifier: Apache-2.0\n\
                   // \u{00a9} PineForge contributors 2026\n\
                   //\n\
                   // OCA probe 02 - multi-bracket\n\
                   // Purpose: two strategy.exit brackets attached to same long entry,\n\
                   // different ATR widths + different oca_name.\n\
                   //\n\
                   // strategy.exit is close-only.\n\
                   //@version=6\n\
                   strategy(\"x\")\n";
        let summary = extract_summary(src).expect("must extract");
        assert!(summary.contains("OCA probe 02"));
        assert!(summary.contains("Purpose"));
        assert!(!summary.contains("Apache-2.0"));
        assert!(!summary.contains("SPDX"));
        // Should stop at the first blank `//` after prose begins, so the
        // "strategy.exit is close-only" trailing fragment is excluded.
        assert!(!summary.contains("close-only"));
    }

    #[test]
    fn extracts_summary_returns_none_for_bare_source() {
        let src = "//@version=6\nstrategy(\"x\")\n";
        assert!(extract_summary(src).is_none());
    }

    #[test]
    fn summary_for_returns_real_text_for_a_known_probe() {
        let summary = summary_for("oca-multi-bracket-isolation-01");
        assert!(
            summary.is_some(),
            "oca-multi-bracket-isolation-01 should have an extractable summary"
        );
        let s = summary.unwrap();
        assert!(s.len() > 30, "summary should be non-trivial, got: {s:?}");
    }

    #[test]
    fn most_probes_have_extractable_summaries() {
        let listings = list_probes(None).expect("list");
        let covered = listings.iter().filter(|p| p.summary.is_some()).count();
        let total = listings.len();
        let coverage = covered as f64 / total as f64;
        assert!(
            coverage >= 0.80,
            "expected >=80% probe summary coverage, got {covered}/{total} ({:.1}%)",
            coverage * 100.0
        );
    }
}
