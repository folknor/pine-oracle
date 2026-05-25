// SPDX-License-Identifier: MPL-2.0
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
// comment block - every prose comment line up to the first real code
// (non-comment, non-blank) line, with license / SPDX / copyright / version
// directive noise filtered out and blank `//` paragraph separators
// collapsed. Cached behind a OnceLock so repeat lookups are cheap. 100%
// of the 239 baked probes get a usable summary out of this; carrying the
// full multi-paragraph header (typically a slug-title line plus a
// `Purpose:` block plus a `Trade shape:` block) gives BM25 something
// substantive to rank against rather than just a slug echo.

use anyhow::{Result, anyhow, bail};
use include_dir::{Dir, include_dir};
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
    let needle = grep.map(str::to_ascii_lowercase);
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
                let summary_hit = summary.is_some_and(|s| s.to_ascii_lowercase().contains(n));
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

/// Pull the entire author-written header comment block out of a
/// `strategy.pine` source, joined into one space-separated string.
///
/// PineForge headers follow a consistent layout:
///   ```text
///   // <license boilerplate>
///   // SPDX-License-Identifier: Apache-2.0
///   // (c) PineForge contributors 2026
///   //
///   // <title line, often the slug>
///   //
///   // Purpose: <multi-line description>
///   //
///   // Trade shape: <multi-line description>
///   //
///   //@version=6
///   strategy(...)
///   ```
///
/// The extractor:
///   - Skips `//` comments matching `is_header_noise` (license / SPDX /
///     copyright / `@version` directive).
///   - Skips empty `//` separators - they don't break the collection,
///     so multi-paragraph headers are joined into one string.
///   - Stops at the first non-comment, non-blank line (real Pine code).
///   - Returns `None` only when no prose comment lines were found at all.
///
/// Joining every prose paragraph (title + Purpose + Trade shape + TV
/// setup notes) gives BM25 substantive substrate to rank against
/// rather than a single slug-echo title.
fn extract_summary(strategy_pine: &str) -> Option<String> {
    let mut lines = Vec::new();
    let mut in_prose = false;
    for line in strategy_pine.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("//") {
            let content = rest.trim_start_matches('/').trim();
            if content.is_empty() {
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
                .is_some_and(|s| s.to_ascii_lowercase().contains("oca"));
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
        // Blank `//` separators no longer terminate collection - the
        // "strategy.exit is close-only" trailing paragraph is now
        // captured into the summary along with everything else.
        assert!(summary.contains("close-only"));
    }

    #[test]
    fn extracts_summary_joins_multiple_paragraphs_across_blank_separators() {
        // Header pattern seen in 19 short-summary probes pre-fix: title
        // line, blank `//`, then a `Purpose:` paragraph that the old
        // first-blank-stop heuristic dropped entirely.
        let src = "// SPDX-License-Identifier: Apache-2.0\n\
                   //\n\
                   // PF probe 82 - dual stop far only\n\
                   //\n\
                   // Purpose: isolate farther long/short stop competition from probe 80.\n\
                   // TV setup: 15m chart, same symbol/window as data/ohlcv_ETH-USDT-USDT_15m.csv.\n\
                   //@version=6\n\
                   strategy(\"x\")\n";
        let summary = extract_summary(src).expect("must extract");
        assert!(summary.contains("PF probe 82"));
        assert!(
            summary.contains("Purpose: isolate"),
            "Purpose paragraph must survive the blank // separator, got: {summary:?}"
        );
        assert!(summary.contains("TV setup"));
    }

    #[test]
    fn extracts_summary_stops_at_real_code() {
        // A non-comment, non-blank line ends collection. Comments AFTER
        // the first code line are inline docs, not header prose.
        let src = "// title line\n\
                   // body line\n\
                   strategy(\"x\")\n\
                   // inline comment that should NOT be in summary\n";
        let summary = extract_summary(src).expect("must extract");
        assert!(summary.contains("title line"));
        assert!(summary.contains("body line"));
        assert!(!summary.contains("inline comment"));
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
    fn every_baked_probe_has_an_extractable_summary() {
        // The wider header-collection heuristic (no first-blank stop, only
        // real-code stops) means every PineForge-format probe yields a
        // non-empty summary. If a new probe is vendored without a header,
        // this test will catch it and the heuristic likely needs another
        // widening pass.
        let listings = list_probes(None).expect("list");
        let missing: Vec<&str> = listings
            .iter()
            .filter(|p| p.summary.is_none())
            .map(|p| p.slug.as_str())
            .collect();
        assert!(
            missing.is_empty(),
            "{} probe(s) without summaries: {:?}",
            missing.len(),
            missing
        );
    }

    #[test]
    fn captured_summaries_are_substantive() {
        // Pre-fix the "PF probe N - slug-words" title-only summaries were
        // <40 chars and gave BM25 nothing to rank against. Post-fix the
        // multi-paragraph collection should put almost every summary
        // comfortably above that.
        let listings = list_probes(None).expect("list");
        let thin: Vec<(&str, &str)> = listings
            .iter()
            .filter_map(|p| p.summary.map(|s| (p.slug.as_str(), s)))
            .filter(|(_, s)| s.len() < 60)
            .collect();
        // A handful of stub probes may legitimately have title-only headers;
        // require <5% rather than 0 so adding a new minimal probe doesn't
        // tank CI.
        let cap = (listings.len() / 20).max(5);
        assert!(
            thin.len() <= cap,
            "{} probe(s) with summaries shorter than 60 chars (cap {cap}): {:?}",
            thin.len(),
            thin.iter().take(10).collect::<Vec<_>>()
        );
    }
}
