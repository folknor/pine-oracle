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
//
// Some probes override the default trade CSV filename via the
// `tv_trades_csv` key in their `inputs.json`; `load_probe` honours this
// override so probes with non-standard filenames (e.g. multi-mode probes
// that ship several CSVs) load the correct one.

use anyhow::{Result, anyhow};
use include_dir::{Dir, include_dir};
use serde::Serialize;
use std::sync::OnceLock;

use crate::util::include_dir_io;
use crate::util::pine_text;
use crate::util::slug;

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
///
/// The trade CSV filename defaults to `tv_trades.csv` but can be overridden
/// per-probe by setting `"tv_trades_csv": "<filename>"` in `inputs.json`.
/// If `inputs.json` is present but not valid JSON, the function fails with
/// a clear error pointing at the slug.
pub fn load_probe(slug_input: &str) -> Result<Probe> {
    let slug = slug::sanitise_slug(slug_input, "validation")?;

    let strategy_pine = include_dir_io::require_utf8(&CORPUS, slug, "strategy.pine", "probe")?;
    let inputs_json = include_dir_io::optional_utf8(&CORPUS, slug, "inputs.json", "probe")?;

    // Derive the trade CSV filename: honour `tv_trades_csv` in inputs.json
    // if present; fall back to the corpus-wide default `tv_trades.csv`.
    let csv_filename = if let Some(raw) = inputs_json {
        let parsed: serde_json::Value = serde_json::from_str(raw)
            .map_err(|e| anyhow!("probe `{slug}` has malformed inputs.json: {e}"))?;
        match parsed.get("tv_trades_csv").and_then(|v| v.as_str()) {
            Some(name) => name.to_string(),
            None => "tv_trades.csv".to_string(),
        }
    } else {
        "tv_trades.csv".to_string()
    };

    let tv_trades_csv = include_dir_io::require_utf8(&CORPUS, slug, &csv_filename, "probe")?;

    Ok(Probe {
        slug: slug.to_string(),
        strategy_pine,
        tv_trades_csv,
        inputs_json,
        summary: summary_for(slug),
    })
}

/// List every baked probe. `grep`, when present, filters by case-insensitive
/// substring against the slug OR the extracted summary text. `feature`,
/// when present, restricts to probes whose `strategy.pine` source uses
/// the named Pine feature (see `FEATURE_CATALOG`); unknown feature names
/// return an error with the catalog included.
pub fn list_probes(grep: Option<&str>, feature: Option<&str>) -> Result<Vec<ProbeListing>> {
    let needle = grep.map(str::to_ascii_lowercase);
    let feature_set = match feature {
        Some(name) => Some(probes_with_feature(name)?),
        None => None,
    };
    let entries = CORPUS
        .find("**/strategy.pine")
        .map_err(|e| anyhow!("walking corpus: {e}"))?;

    let mut out: Vec<ProbeListing> = entries
        .filter_map(|entry| {
            let file = entry.as_file()?;
            let parent = file.path().parent()?;
            let slug = parent.to_str()?.to_string();
            if let Some(set) = &feature_set
                && !set.contains(&slug)
            {
                return None;
            }
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

/// One Pine feature `--feature <name>` can filter on. Each spec is
/// resolved at first invocation against every baked `strategy.pine`
/// (with line + block comments stripped, so false positives from
/// commented-out code are avoided), and the slug -> features mapping
/// is cached behind a OnceLock.
struct FeatureSpec {
    name: &'static str,
    description: &'static str,
    detector: fn(&str) -> bool,
}

const FEATURE_CATALOG: &[FeatureSpec] = &[
    FeatureSpec {
        name: "oca",
        description: "Order Cancels Order brackets (oca_name= parameter)",
        detector: |s| s.contains("oca_name"),
    },
    FeatureSpec {
        name: "trail",
        description: "Trailing-stop exits (trail_points / trail_offset / trail_price)",
        detector: |s| pine_text::uses_trail_exits(s),
    },
    FeatureSpec {
        name: "pyramiding",
        description: "Real pyramiding (strategy(..., pyramiding=N) with N >= 2)",
        detector: detect_real_pyramiding,
    },
    FeatureSpec {
        name: "varip",
        description: "Intra-bar persistent state (varip keyword)",
        detector: |s| has_word(s, "varip"),
    },
    FeatureSpec {
        name: "mtf",
        description: "Multi-timeframe data sourcing (request.security)",
        detector: |s| s.contains("request.security"),
    },
    FeatureSpec {
        name: "magnifier",
        description: "Chart magnifier mode (magnifier=true / use_magnifier=)",
        detector: |s| s.contains("magnifier"),
    },
    FeatureSpec {
        name: "matrix",
        description: "Matrix data structure (matrix.new / matrix<...> typing)",
        detector: |s| s.contains("matrix.new") || s.contains("matrix<"),
    },
    FeatureSpec {
        name: "map",
        description: "Map data structure (map.new / map<...> typing)",
        detector: |s| s.contains("map.new") || s.contains("map<"),
    },
    FeatureSpec {
        name: "udt",
        description: "User-defined types (top-level `type Name` declarations)",
        detector: |s| has_line_start(s, "type "),
    },
    FeatureSpec {
        name: "method",
        description: "User-defined-type methods (top-level `method Name` declarations)",
        detector: |s| has_line_start(s, "method "),
    },
    FeatureSpec {
        name: "process_orders_on_close",
        description: "Bar-close order processing (process_orders_on_close=true)",
        detector: |s| {
            s.contains("process_orders_on_close=true")
                || s.contains("process_orders_on_close = true")
        },
    },
    FeatureSpec {
        name: "barstate_isfirst",
        description: "First-bar initialization gate (barstate.isfirst)",
        detector: |s| s.contains("barstate.isfirst"),
    },
];

/// `(name, description)` for every feature `--feature` accepts.
pub fn feature_catalog() -> Vec<(&'static str, &'static str)> {
    FEATURE_CATALOG
        .iter()
        .map(|f| (f.name, f.description))
        .collect()
}

/// Slugs of probes whose `strategy.pine` source uses the named feature.
/// Errors if `name` isn't in the catalog (the error message lists every
/// known feature).
fn probes_with_feature(name: &str) -> Result<std::collections::HashSet<String>> {
    let map = feature_index();
    map.get(name).cloned().ok_or_else(|| {
        let known: Vec<&str> = FEATURE_CATALOG.iter().map(|f| f.name).collect();
        anyhow!(
            "unknown feature `{name}`. Known features: {}",
            known.join(", ")
        )
    })
}

fn feature_index() -> &'static std::collections::HashMap<String, std::collections::HashSet<String>>
{
    use std::collections::{HashMap, HashSet};
    static IDX: OnceLock<HashMap<String, HashSet<String>>> = OnceLock::new();
    IDX.get_or_init(|| {
        let mut out: HashMap<String, HashSet<String>> = HashMap::new();
        for spec in FEATURE_CATALOG {
            out.insert(spec.name.to_string(), HashSet::new());
        }
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
            let stripped = pine_text::strip_pine_comments(content);
            for spec in FEATURE_CATALOG {
                if (spec.detector)(&stripped) {
                    out.get_mut(spec.name)
                        .expect("catalog entries pre-seeded above")
                        .insert(slug.to_string());
                }
            }
        }
        out
    })
}

fn detect_real_pyramiding(src: &str) -> bool {
    let mut start = 0;
    while let Some(pos) = src[start..].find("pyramiding") {
        let abs = start + pos;
        start = abs + "pyramiding".len();
        let after = src[start..].trim_start();
        let Some(after) = after.strip_prefix('=') else {
            continue;
        };
        let digits: String = after
            .trim_start()
            .chars()
            .take_while(char::is_ascii_digit)
            .collect();
        if let Ok(n) = digits.parse::<u32>()
            && n >= 2
        {
            return true;
        }
    }
    false
}

fn has_word(src: &str, word: &str) -> bool {
    let mut start = 0;
    while let Some(pos) = src[start..].find(word) {
        let abs = start + pos;
        start = abs + word.len();
        let before_ok = abs == 0 || !is_ident_char(src.as_bytes()[abs - 1]);
        let after_ok = start >= src.len() || !is_ident_char(src.as_bytes()[start]);
        if before_ok && after_ok {
            return true;
        }
    }
    false
}

fn has_line_start(src: &str, prefix: &str) -> bool {
    src.lines().any(|l| l.starts_with(prefix))
}

fn is_ident_char(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_'
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

/// Infallible count of baked corpus probes. Walks the embedded include_dir
/// once; does not allocate a Vec. Use in `pine version` instead of the
/// fallible `list_probes` round-trip.
pub fn probe_count() -> usize {
    CORPUS.find("**/strategy.pine").map_or(0, Iterator::count)
}

/// Infallible count of baked probes that have an extractable author summary.
/// Relies on `summary_for`, which is cached behind a OnceLock.
pub fn probe_summary_count() -> usize {
    let Ok(entries) = CORPUS.find("**/strategy.pine") else {
        return 0;
    };
    entries
        .filter_map(|e| e.as_file())
        .filter_map(|f| f.path().parent()?.to_str().map(str::to_string))
        .filter(|slug| summary_for(slug).is_some())
        .count()
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
    fn loads_probe_with_inputs_json_csv_override() {
        // analyzer-self-test-multi-mode-01 ships no tv_trades.csv; its
        // inputs.json declares `"tv_trades_csv": "trades-htf_d_high1.csv"`.
        // load_probe must honour that override instead of erroring.
        const SLUG: &str = "analyzer-self-test-multi-mode-01";
        let probe = load_probe(SLUG).expect("probe with csv override must load");
        assert_eq!(probe.slug, SLUG);
        assert!(!probe.strategy_pine.is_empty());
        // The resolved CSV must be non-empty and begin with a CSV header.
        assert!(!probe.tv_trades_csv.is_empty());
        let first_line = probe.tv_trades_csv.lines().next().unwrap_or("");
        assert!(
            first_line.contains("Trade #"),
            "expected a CSV header with 'Trade #', got: {first_line:?}"
        );
        // inputs_json must be populated.
        assert!(probe.inputs_json.is_some());
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
        let all = list_probes(None, None).expect("list must succeed");
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
        let oca = list_probes(Some("oca"), None).expect("grep must succeed");
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
        let listings = list_probes(None, None).expect("list");
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
        let listings = list_probes(None, None).expect("list");
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

    // ---------- feature index ----------

    #[test]
    fn feature_catalog_is_nonempty() {
        let cat = feature_catalog();
        assert!(cat.len() >= 10, "expected a substantive catalog");
        assert!(cat.iter().any(|(n, _)| *n == "oca"));
        assert!(cat.iter().any(|(n, _)| *n == "mtf"));
        assert!(cat.iter().any(|(n, _)| *n == "trail"));
    }

    #[test]
    fn unknown_feature_errors_with_catalog() {
        let err = list_probes(None, Some("totallymadeup")).expect_err("unknown feature must error");
        let msg = err.to_string();
        assert!(msg.contains("unknown feature"));
        assert!(
            msg.contains("oca"),
            "error must list known features, got: {msg}"
        );
    }

    #[test]
    fn feature_filter_oca_returns_real_subset() {
        let oca = list_probes(None, Some("oca")).expect("oca filter must succeed");
        assert!(
            !oca.is_empty(),
            "expected at least one oca probe in the corpus"
        );
        // OCA is rare - typical corpus has ~5 probes. If it ever balloons,
        // the catalog detector is probably false-positive.
        assert!(
            oca.len() < 50,
            "oca filter returned {} probes - detector is probably too broad",
            oca.len()
        );
    }

    #[test]
    fn feature_filter_combines_with_grep() {
        // Every mtf-prefixed probe should also be in the `mtf` feature
        // set (request.security is the defining signal for that family).
        let mtf_by_feature = list_probes(None, Some("mtf")).expect("mtf");
        let mtf_by_slug: Vec<&str> = mtf_by_feature
            .iter()
            .map(|p| p.slug.as_str())
            .filter(|s| s.starts_with("mtf-"))
            .collect();
        assert!(
            !mtf_by_slug.is_empty(),
            "expected mtf-prefixed slugs in the mtf feature set"
        );
        // Now combine with grep: `mtf` feature AND `60` in slug should
        // yield a strict subset of `mtf` feature.
        let mtf_60 = list_probes(Some("60"), Some("mtf")).expect("mtf+grep");
        assert!(mtf_60.len() <= mtf_by_feature.len());
        assert!(mtf_60.iter().all(|p| {
            p.slug.contains("60")
                || p.summary
                    .is_some_and(|s| s.to_ascii_lowercase().contains("60"))
        }));
    }

    #[test]
    fn strip_pine_comments_drops_line_and_block_comments() {
        let src = "real // commented out\n/* block */more real\n// only comment\n";
        let out = pine_text::strip_pine_comments(src);
        assert!(out.contains("real "));
        assert!(out.contains("more real"));
        assert!(!out.contains("commented out"));
        assert!(!out.contains("block "));
        assert!(!out.contains("only comment"));
    }

    #[test]
    fn detect_real_pyramiding_requires_n_ge_two() {
        assert!(!detect_real_pyramiding("pyramiding=1"));
        assert!(!detect_real_pyramiding("pyramiding = 0"));
        assert!(detect_real_pyramiding("pyramiding=2"));
        assert!(detect_real_pyramiding("pyramiding = 10"));
        assert!(detect_real_pyramiding(
            "strategy(\"x\", pyramiding=5, slippage=0)"
        ));
        // No `=` after pyramiding: skip.
        assert!(!detect_real_pyramiding("// pyramiding 2"));
    }

    #[test]
    fn has_word_respects_identifier_boundaries() {
        assert!(has_word("varip int x = 0", "varip"));
        assert!(has_word("foo\nvarip int x", "varip"));
        // No false positive on substring matches.
        assert!(!has_word("myvarip x", "varip"));
        assert!(!has_word("varipx x", "varip"));
    }

    #[test]
    fn feature_filter_pyramiding_excludes_pyramiding_eq_one() {
        // pyramiding=1 is the corpus-wide default (~95% of probes).
        // The `pyramiding` feature is reserved for real (N>=2) usage.
        let pyr = list_probes(None, Some("pyramiding")).expect("pyramiding");
        assert!(
            pyr.len() < 50,
            "pyramiding feature returned {} probes - detector is matching pyramiding=1 noise",
            pyr.len()
        );
    }
}
