// SPDX-License-Identifier: Apache-2.0 OR MPL-2.0

use anyhow::{anyhow, bail, Context, Result};
use serde::Serialize;
use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};

const CORPUS_REPO_URL: &str = "https://github.com/fullpass-4pass/pineforge-corpus.git";

const ENGINE_HISTORY_SLUGS: &[(&str, &str)] = &[
    ("97", "97-tp-sl-gap-reversal-oca"),
    ("98", "98-inside-bar-engulf"),
    ("99", "99-keltner-channel-break"),
    ("100", "100-ma-dual-cross"),
    ("101", "101-parabolic-sar-flip"),
    ("102", "102-pivot-extension-break"),
    ("103", "103-stochastic-slow-cross"),
    ("104", "104-supertrend-flip"),
    ("105", "105-volty-expansion-close"),
];

const ENGINE_HISTORY_ONLY_NUMBERS: &[&str] = &[
    "52", "54", "62", "63", "72", "80", "83", "92", "93", "95", "96",
];

const SUMMARY_INTRADAY_CAP: &str = "`97`: when the cap-triggering fill is a STOP entry that fired intra-bar (stop > bar.open for long, stop < bar.open for short), TV's synthetic \"Close Position (Max number of filled orders in one day)\" exit emits at the bar's favorable extreme (bar.high for long, bar.low for short), not the entry's stop trigger price. `97a`: short -> long MA-cross flip leaves the pre-existing buy-stop bracket alive; its `created_position_side` is SHORT but the live position is LONG. The `pre_armed_opposite_priced` semantic in `add_to_pyramid_market` admits the add even when pyramiding would reject it. `97b`: when the entry filled AT bar.open (gap-fill or market, no intra-bar travel), TV's cap-close emits at `fill_price = bar.open`.";
const SUMMARY_MAGNIFIER_DIST: &str = "With magnifier ON, TV treats each lower-TF sub-bar's open as a fresh gap event and DOES fill wrong-side exits at the entry bar's open. 340 of 871 trades on probe-01 are wrong-side gap fills. Allow gap shortcut on the entry bar in magnifier mode only; without this, the legacy non-magnifier rule makes entry==exit for these trades.";
const SUMMARY_IES_PROBE_08: &str = "TV margin check: `required_margin = qty * fill_price * margin_pct / 100`. If `required_margin > available equity`, TV silently rejects the fill. Default margin = 100 (1x leverage) so \"position value <= equity\". Without the gate, dynamic-qty strategies (community/IES, community/VCP) over-leverage on low-ATR bars and produce about 5x more trades than TV. The check happens at SIGNAL time (with signal-bar close), NOT at fill time. Empirically, matched-trade qty ratio in probe 08 was equal to `engine_equity / TV_equity`.";
const SUMMARY_PARITY_PROBE_03_06: &str = "Validates that margin fires at signal time (with `current_bar_.close`) NOT at fill time (`next_bar.open`). Close-vs-open slippage routinely inflates overshoot from about zero to about $20; pre-fix engine rejected those at fill while TV accepted them at signal. 57/57 matched post-fix.";
const SUMMARY_OCA_THREE_WAY_02: &str = "TV cancels CANCEL-group siblings only after the originating order is FULLY filled, not after the first contract. qty=4 long + qty=2 sibling A_TP: A_TP fills qty=2, position=2 remaining, A_SL stays alive until the second sibling fires. Plus: `strategy.exit`'s `oca_name` plumbing; without it, all `strategy.exit`-issued orders shared an empty name and the first bracket's TP would silently leave the other bracket's legs intact. About 42% trade loss without the fix.";
const SUMMARY_TYPED_MATRIX_BOOL: &str = "Pre-fix the engine wrote chart TZ into `syminfo_.timezone`, which codegen reads as the default tz of the 1-arg `hour(time)`/`minute(time)`/`dayofweek(time)` form, conflating two distinct TV concepts and silently shifting results by the chart-vs-exchange offset (Asia/Taipei vs UTC = +8h for crypto). The shift cascaded into `hour`-bucketed accumulators: the 24x7 `matrix<bool>` regime mask filled in 8 hours earlier than TV. Pre-fix trade counts: TV 773, engine 714; post-fix about 778.";
const SUMMARY_ANOMALY_EQUITY_MIRROR: &str = "Full-equity sizing right at the 1x margin boundary, where TV's behavior is itself non-deterministic. Documented in `corpus/parity-anomalies/README.md`. The close-vs-open margin distinction is load-bearing here for `qty = strategy.equity / close` sizing patterns. piners should inherit the anomaly tier, not count this as a parity failure.";

#[derive(Debug, Clone, Serialize)]
pub struct Probe {
    pub slug: String,
    pub strategy_pine: String,
    pub tv_trades_csv: PathBuf,
    pub summary: Option<String>,
    pub inputs_json: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProbeListing {
    pub slug: String,
    pub summary: Option<String>,
}

/// Resolve the corpus root from PINE_CORPUS, XDG_DATA_HOME, or HOME.
pub fn corpus_root() -> Result<PathBuf> {
    let root = corpus_root_for(|key| env::var(key).ok())?;
    ensure_corpus_root_exists(root)
}

/// Clone the public PineForge corpus into the resolved corpus root if absent.
pub fn install() -> Result<()> {
    let root = corpus_root_for(|key| env::var(key).ok())?;

    if root.is_dir() {
        eprintln!("pine corpus already installed at {}", root.display());
        return Ok(());
    }

    if root.exists() {
        bail!(
            "pine corpus path exists but is not a directory: {}",
            root.display()
        );
    }

    if let Some(parent) = root.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create corpus parent {}", parent.display()))?;
    }

    let output = Command::new("git")
        .arg("clone")
        .arg(CORPUS_REPO_URL)
        .arg(&root)
        .output()
        .context("failed to run git clone for pine corpus")?;

    if !output.status.success() {
        return Err(git_failure("git clone", &output));
    }

    Ok(())
}

/// Update the installed corpus with a fast-forward-only git pull.
pub fn update() -> Result<()> {
    let root = corpus_root()?;
    let output = Command::new("git")
        .arg("-C")
        .arg(&root)
        .arg("pull")
        .arg("--ff-only")
        .output()
        .with_context(|| format!("failed to run git pull in {}", root.display()))?;

    if !output.status.success() {
        return Err(git_failure("git pull --ff-only", &output));
    }

    Ok(())
}

/// Load a single validation probe by published slug, or by mapped engine-history number.
pub fn load_probe(slug: &str, engine_history: bool) -> Result<Probe> {
    let slug = resolve_probe_slug(slug, engine_history)?;
    let validation_dir = corpus_root()?.join("validation");
    let probe_dir = validation_dir.join(&slug);

    if !probe_dir.is_dir() {
        bail!(
            "probe `{}` not found under {}",
            slug,
            validation_dir.display()
        );
    }

    let strategy_path = probe_dir.join("strategy.pine");
    let strategy_pine = fs::read_to_string(&strategy_path)
        .with_context(|| format!("failed to read {}", strategy_path.display()))?;

    let tv_trades_csv = probe_dir.join("tv_trades.csv");
    if !tv_trades_csv.is_file() {
        bail!(
            "probe `{}` is missing tv_trades.csv at {}",
            slug,
            tv_trades_csv.display()
        );
    }

    let inputs_path = probe_dir.join("inputs.json");
    let inputs_json = match fs::read_to_string(&inputs_path) {
        Ok(contents) => Some(contents),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => None,
        Err(err) => {
            return Err(err).with_context(|| format!("failed to read {}", inputs_path.display()));
        }
    };

    Ok(Probe {
        summary: summary_for_slug(&slug).map(str::to_owned),
        slug,
        strategy_pine,
        tv_trades_csv,
        inputs_json,
    })
}

/// List validation probes, optionally filtering summaries by substring.
pub fn list_probes(feature: Option<&str>, grep: Option<&str>) -> Result<Vec<ProbeListing>> {
    if feature.is_some() {
        bail!("feature index not yet implemented; falls out of probe-summaries v2");
    }

    let validation_dir = corpus_root()?.join("validation");
    let grep = grep.filter(|s| !s.is_empty());
    let mut probes = Vec::new();

    for entry in fs::read_dir(&validation_dir)
        .with_context(|| format!("failed to read {}", validation_dir.display()))?
    {
        let entry = entry
            .with_context(|| format!("failed to read entry in {}", validation_dir.display()))?;
        let file_type = entry
            .file_type()
            .with_context(|| format!("failed to stat {}", entry.path().display()))?;

        if !file_type.is_dir() {
            continue;
        }

        let slug = entry.file_name().into_string().map_err(|name| {
            anyhow!(
                "probe directory name under {} is not valid UTF-8: {:?}",
                validation_dir.display(),
                name
            )
        })?;
        let summary = summary_for_slug(&slug).map(str::to_owned);

        if let Some(needle) = grep {
            let Some(summary_text) = summary.as_deref() else {
                continue;
            };

            if !contains_ascii_case_insensitive(summary_text, needle) {
                continue;
            }
        }

        probes.push(ProbeListing { slug, summary });
    }

    probes.sort_by(|a, b| a.slug.cmp(&b.slug));
    Ok(probes)
}

fn corpus_root_for<F>(env: F) -> Result<PathBuf>
where
    F: Fn(&str) -> Option<String>,
{
    if let Some(path) = nonempty_env(&env, "PINE_CORPUS") {
        return Ok(PathBuf::from(path));
    }

    if let Some(path) = nonempty_env(&env, "XDG_DATA_HOME") {
        return Ok(PathBuf::from(path).join("pine").join("corpus"));
    }

    if let Some(path) = nonempty_env(&env, "HOME") {
        return Ok(PathBuf::from(path)
            .join(".local")
            .join("share")
            .join("pine")
            .join("corpus"));
    }

    bail!("could not resolve pine corpus root: set PINE_CORPUS or HOME")
}

fn nonempty_env<F>(env: &F, key: &str) -> Option<String>
where
    F: Fn(&str) -> Option<String>,
{
    env(key).filter(|value| !value.trim().is_empty())
}

fn ensure_corpus_root_exists(root: PathBuf) -> Result<PathBuf> {
    if root.is_dir() {
        Ok(root)
    } else {
        bail!(
            "pine corpus directory does not exist at {}; run `pine corpus install` or set PINE_CORPUS",
            root.display()
        )
    }
}

fn resolve_probe_slug(slug: &str, engine_history: bool) -> Result<String> {
    let slug = slug.trim();
    if slug.is_empty() {
        bail!("probe slug cannot be empty");
    }

    if engine_history && slug.chars().all(|ch| ch.is_ascii_digit()) {
        if let Some((_, mapped_slug)) = ENGINE_HISTORY_SLUGS
            .iter()
            .find(|(engine_number, _)| *engine_number == slug)
        {
            return Ok((*mapped_slug).to_owned());
        }

        if ENGINE_HISTORY_ONLY_NUMBERS.contains(&slug) {
            bail!(
                "probe number `{}` is engine-history-only and is not in the published corpus",
                slug
            );
        }

        bail!(
            "probe number `{}` has no published corpus mapping in probe-summaries.md",
            slug
        );
    }

    clean_probe_slug(slug).map(str::to_owned)
}

fn clean_probe_slug(slug: &str) -> Result<&str> {
    let slug = slug.strip_prefix("validation/").unwrap_or(slug);

    if slug.is_empty() {
        bail!("probe slug cannot be empty");
    }

    if slug == "." || slug == ".." || slug.contains('/') || slug.contains('\\') {
        bail!(
            "probe slug must be a single validation directory name, got `{}`",
            slug
        );
    }

    Ok(slug)
}

fn summary_for_slug(slug: &str) -> Option<&'static str> {
    match slug {
        "97-tp-sl-gap-reversal-oca" => Some(SUMMARY_INTRADAY_CAP),
        "magnifier-dist-probe-01"
        | "magnifier-dist-probe-02"
        | "magnifier-dist-probe-03"
        | "magnifier-dist-probe-04"
        | "magnifier-dist-probe-05"
        | "magnifier-dist-probe-06"
        | "magnifier-dist-probe-07"
        | "magnifier-dist-probe-08"
        | "magnifier-dist-probe-08b" => Some(SUMMARY_MAGNIFIER_DIST),
        "ies-probe-08" => Some(SUMMARY_IES_PROBE_08),
        "parity-probe-03" | "parity-probe-04" | "parity-probe-05" | "parity-probe-06" => {
            Some(SUMMARY_PARITY_PROBE_03_06)
        }
        "oca-three-way-probe-02" => Some(SUMMARY_OCA_THREE_WAY_02),
        "typed-matrix-probe-01-bool-regime-mask" => Some(SUMMARY_TYPED_MATRIX_BOOL),
        "anomaly-equity-mirror" => Some(SUMMARY_ANOMALY_EQUITY_MIRROR),
        _ => None,
    }
}

fn contains_ascii_case_insensitive(haystack: &str, needle: &str) -> bool {
    haystack
        .to_ascii_lowercase()
        .contains(&needle.to_ascii_lowercase())
}

fn git_failure(command: &str, output: &Output) -> anyhow::Error {
    let stderr = String::from_utf8_lossy(&output.stderr);
    let stderr = stderr.trim();

    if stderr.is_empty() {
        anyhow!("{} failed with status {}", command, output.status)
    } else {
        anyhow!(
            "{} failed with status {}: {}",
            command,
            output.status,
            stderr
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn corpus_root_prefers_pine_corpus() {
        let root = corpus_root_for(env_from(&[
            ("PINE_CORPUS", "/custom/corpus"),
            ("XDG_DATA_HOME", "/xdg"),
            ("HOME", "/home/example"),
        ]))
        .expect("root should resolve");

        assert_eq!(root, PathBuf::from("/custom/corpus"));
    }

    #[test]
    fn corpus_root_uses_xdg_data_home_before_home() {
        let root = corpus_root_for(env_from(&[
            ("XDG_DATA_HOME", "/xdg-data"),
            ("HOME", "/home/example"),
        ]))
        .expect("root should resolve");

        assert_eq!(root, PathBuf::from("/xdg-data/pine/corpus"));
    }

    #[test]
    fn corpus_root_falls_back_to_home_local_share() {
        let root =
            corpus_root_for(env_from(&[("HOME", "/home/example")])).expect("root should resolve");

        assert_eq!(
            root,
            PathBuf::from("/home/example/.local/share/pine/corpus")
        );
    }

    #[test]
    fn engine_history_number_resolves_published_slug() {
        let slug = resolve_probe_slug("97", true).expect("97 should resolve");

        assert_eq!(slug, "97-tp-sl-gap-reversal-oca");
    }

    #[test]
    fn engine_history_only_number_errors_clearly() {
        let error = resolve_probe_slug("52", true).expect_err("52 should not resolve");
        let message = error.to_string();

        assert!(message.contains("engine-history-only"));
        assert!(message.contains("not in the published corpus"));
    }

    #[test]
    fn summary_lookup_returns_known_summary() {
        let summary = summary_for_slug("97-tp-sl-gap-reversal-oca");

        assert!(summary.is_some());
    }

    #[test]
    fn summary_lookup_returns_none_for_uncovered_slug() {
        let summary = summary_for_slug("not-covered-by-probe-summaries");

        assert!(summary.is_none());
    }

    fn env_from<'a>(vars: &'a [(&'a str, &'a str)]) -> impl Fn(&str) -> Option<String> + 'a {
        move |key| {
            vars.iter()
                .find(|(env_key, _)| *env_key == key)
                .map(|(_, value)| (*value).to_owned())
        }
    }
}
