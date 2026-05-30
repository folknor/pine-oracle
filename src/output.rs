// Shared output infrastructure for every `pine <subcommand>`:
//   - ResolvedFormat: the post-`Auto`-resolution flavour each command receives.
//   - Style: ANSI colour wrapper, off when the user passed `--no-color`,
//     the `NO_COLOR` env var is present, or stdout isn't a TTY.
//   - SCHEMA_VERSION + versioned_json + print_json: every JSON payload
//     gets wrapped in the same `schema_version`-bearing envelope.
//   - CATALOG_MARKER + is_catalog_request: unified sentinel check for
//     `--kind ?` catalog-listing requests.
//   - print_catalog: unified JSON-vs-text + quiet dispatch for the
//     catalog surfaces (search kinds, behavior kinds).

use anyhow::Result;
use serde::Serialize;

/// JSON output schema version. Bumped on any breaking shape change to a
/// subcommand's JSON output. Bump rules are documented in `README.md`.
///
/// v2: `po lookup` JSON changed from a bare behavior object to
/// `{query, exact, matches: [...]}` so multi-catalog names (e.g. `na`) can
/// return every meaning.
pub(crate) const SCHEMA_VERSION: u32 = 2;

/// The sentinel value a user passes to request a catalog listing instead of
/// a real filter. Every `--kind ?`, `--baseline ?`, `--feature ?` gate checks
/// against this constant via `is_catalog_request`.
pub(crate) const CATALOG_MARKER: &str = "?";

/// Returns `true` when the optional filter string is the catalog sentinel `?`.
/// Replaces the per-module `is_kind_catalog_request` / `is_baseline_catalog_request`
/// helpers that previously duplicated this one-liner.
pub(crate) fn is_catalog_request(s: Option<&str>) -> bool {
    s == Some(CATALOG_MARKER)
}

/// Unified catalog printer. Handles JSON-vs-text dispatch and `--quiet` short
/// rows for all four catalog surfaces (`--kind`, `--baseline`, `--feature`).
///
/// Parameters:
/// - `json_key`: top-level key wrapping `items` in the JSON object
///   (`"kinds"`, `"baselines"`, `"features"`).
/// - `items`: the catalog slice; items must implement `Serialize`.
/// - `text_row`: full row formatter (name + count + description columns).
/// - `text_row_quiet`: quiet row formatter (name column only).
/// - `format`: resolved output format.
/// - `quiet`: when true, use `text_row_quiet` instead of `text_row`.
pub(crate) fn print_catalog<T: Serialize>(
    json_key: &str,
    items: &[T],
    text_row: impl Fn(&T) -> String,
    text_row_quiet: impl Fn(&T) -> String,
    format: ResolvedFormat,
    quiet: bool,
) -> Result<()> {
    match format {
        ResolvedFormat::Json => print_json(&serde_json::json!({ json_key: items })),
        ResolvedFormat::Text => {
            for item in items {
                if quiet {
                    println!("{}", text_row_quiet(item));
                } else {
                    println!("{}", text_row(item));
                }
            }
            Ok(())
        }
    }
}

#[must_use]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ResolvedFormat {
    Text,
    Json,
}

/// Wrap `payload` in the versioned envelope. Objects get the field
/// inserted at the top level; arrays and scalars are wrapped as
/// `{schema_version, items}` / `{schema_version, value}`. Exposed so
/// the binary's tests can pin the wire shape.
///
/// # schema_version collision
///
/// If the serialised payload is an object that already contains a
/// `"schema_version"` field, that field is **silently overwritten** with
/// `SCHEMA_VERSION`. This is intentional: the wrapper is the canonical
/// authority on envelope versioning. In debug / test builds a
/// `debug_assert` fires so the collision is caught early; in release
/// builds the overwrite is silent and the old value is discarded.
///
/// See `versioned_json_overwrites_payload_schema_version` in the test
/// suite for the pinned wire behaviour.
pub(crate) fn versioned_json<T: Serialize>(payload: &T) -> Result<serde_json::Value> {
    let mut v = serde_json::to_value(payload)?;
    let kind = match &v {
        serde_json::Value::Object(_) => 0,
        serde_json::Value::Array(_) => 1,
        _ => 2,
    };
    match kind {
        0 => {
            if let serde_json::Value::Object(ref mut obj) = v {
                debug_assert!(
                    !obj.contains_key("schema_version"),
                    "versioned_json: payload already has a \"schema_version\" field \
                     (value = {:?}); the wrapper will overwrite it with {}. \
                     Remove the field from the payload type or rename it.",
                    obj.get("schema_version"),
                    SCHEMA_VERSION
                );
                obj.insert("schema_version".into(), serde_json::json!(SCHEMA_VERSION));
            }
        }
        1 => {
            v = serde_json::json!({
                "schema_version": SCHEMA_VERSION,
                "items": v,
            });
        }
        _ => {
            v = serde_json::json!({
                "schema_version": SCHEMA_VERSION,
                "value": v,
            });
        }
    }
    Ok(v)
}

pub(crate) fn print_json<T: Serialize>(payload: &T) -> Result<()> {
    println!("{}", serde_json::to_string(&versioned_json(payload)?)?);
    Ok(())
}

/// ANSI styling for terminal text output. Disabled when `--no-color` is
/// set, when the `NO_COLOR` env var is present (no-color.org convention),
/// when the resolved output format is JSON, or when stdout is not a TTY
/// (same probe `OutputFormat::Auto` uses to pick text vs json).
#[must_use]
#[derive(Clone, Copy)]
pub(crate) struct Style {
    enabled: bool,
}

/// Pure, fully-deterministic styling decision. All four inputs are passed
/// in explicitly so this function is trivially testable without touching
/// process env vars or file descriptors.
///
/// - `no_color_flag`: the caller-parsed `--no-color` flag value.
/// - `format`: resolved output format (`Text` or `Json`).
/// - `stdout_is_tty`: pre-probed TTY state for stdout.
/// - `no_color_env`: whether the `NO_COLOR` environment variable is set.
///   Pass `std::env::var_os("NO_COLOR").is_some()` from the call site.
pub(crate) fn style_compute(
    no_color_flag: bool,
    format: ResolvedFormat,
    stdout_is_tty: bool,
    no_color_env: bool,
) -> Style {
    let enabled =
        !no_color_flag && !no_color_env && format == ResolvedFormat::Text && stdout_is_tty;
    Style { enabled }
}

impl Style {
    /// Resolve styling from the three determinants: whether the caller passed
    /// `--no-color`, the resolved output format, and whether stdout is a TTY.
    /// `stdout_is_tty` must be resolved once at startup and threaded in so
    /// multiple calls do not independently probe the stream.
    ///
    /// Reads `NO_COLOR` from the process environment once per call and
    /// delegates to [`style_compute`] for the actual decision. Use
    /// `style_compute` directly in tests to avoid process-global env var
    /// races.
    pub(crate) fn resolve(no_color: bool, format: ResolvedFormat, stdout_is_tty: bool) -> Self {
        style_compute(
            no_color,
            format,
            stdout_is_tty,
            std::env::var_os("NO_COLOR").is_some(),
        )
    }

    pub(crate) fn red(self, s: &str) -> String {
        self.wrap(s, "31")
    }
    pub(crate) fn yellow(self, s: &str) -> String {
        self.wrap(s, "33")
    }
    pub(crate) fn cyan(self, s: &str) -> String {
        self.wrap(s, "36")
    }
    pub(crate) fn dim(self, s: &str) -> String {
        self.wrap(s, "2")
    }
    pub(crate) fn bold(self, s: &str) -> String {
        self.wrap(s, "1")
    }

    fn wrap(self, s: &str, code: &str) -> String {
        if self.enabled {
            format!("\x1b[{code}m{s}\x1b[0m")
        } else {
            s.to_string()
        }
    }
}

#[cfg(test)]
impl Style {
    /// Test-only constructor that bypasses TTY / env detection.
    pub(crate) fn forced(enabled: bool) -> Self {
        Style { enabled }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn object_payload_gets_schema_version_inline() {
        let payload = serde_json::json!({ "name": "x", "n": 1 });
        let v = versioned_json(&payload).expect("must wrap");
        assert_eq!(v["schema_version"], serde_json::json!(SCHEMA_VERSION));
        assert_eq!(v["name"], "x");
        assert_eq!(v["n"], 1);
    }

    #[test]
    fn array_payload_gets_wrapped_under_items() {
        let payload = serde_json::json!([1, 2, 3]);
        let v = versioned_json(&payload).expect("must wrap");
        assert_eq!(v["schema_version"], serde_json::json!(SCHEMA_VERSION));
        assert_eq!(v["items"], serde_json::json!([1, 2, 3]));
        assert!(v.get("name").is_none());
    }

    #[test]
    fn scalar_payload_gets_wrapped_under_value() {
        let v = versioned_json(&42).expect("must wrap");
        assert_eq!(v["schema_version"], serde_json::json!(SCHEMA_VERSION));
        assert_eq!(v["value"], 42);
    }

    #[test]
    fn schema_version_is_two() {
        // Hard-pin: bumping SCHEMA_VERSION requires updating this test AND
        // the schema-versioning note in `README.md`.
        assert_eq!(SCHEMA_VERSION, 2);
    }

    /// Pin the overwrite-wins behaviour: if the payload object already has a
    /// `"schema_version"` field, `versioned_json` replaces it with the
    /// wrapper's own `SCHEMA_VERSION`. The payload-supplied value (99 here)
    /// is discarded. This is the documented canonical behaviour; changing it
    /// is a breaking wire-format change. Note: in debug/test builds the
    /// `debug_assert` inside `versioned_json` would normally fire for a real
    /// conflict -- this test constructs the conflict intentionally and uses
    /// `cfg(not(debug_assertions))` skipping is NOT needed because the assert
    /// documents developer intent, not a correctness invariant we need to
    /// suppress in tests. The test runs in release mode via `brokkr test`.
    ///
    /// If this test is run in debug mode the debug_assert will panic -- that
    /// is working as intended: a payload type should not carry its own
    /// schema_version. Fix the payload type rather than suppressing the assert.
    #[cfg(not(debug_assertions))]
    #[test]
    fn versioned_json_overwrites_payload_schema_version() {
        // Simulate a payload that already embeds schema_version (e.g. a
        // serialised IndicatorGeneratedExpect or a re-serialised envelope).
        let payload = serde_json::json!({ "schema_version": 99, "data": "hello" });
        let v = versioned_json(&payload).expect("must wrap");
        // The wrapper wins: SCHEMA_VERSION (1), NOT the payload value (99).
        assert_eq!(
            v["schema_version"],
            serde_json::json!(SCHEMA_VERSION),
            "wrapper schema_version must overwrite payload-supplied value"
        );
        assert_eq!(v["data"], "hello", "other fields must be preserved");
        assert_ne!(
            v["schema_version"],
            serde_json::json!(99),
            "payload-supplied schema_version must not survive wrapping"
        );
    }

    #[test]
    fn style_disabled_returns_unwrapped_text() {
        let s = Style::forced(false);
        assert_eq!(s.red("err"), "err");
        assert_eq!(s.yellow("warn"), "warn");
        assert_eq!(s.bold("name"), "name");
        assert_eq!(s.dim("12"), "12");
        assert_eq!(s.cyan("[reference]"), "[reference]");
    }

    #[test]
    fn style_enabled_wraps_with_ansi_escape_codes() {
        let s = Style::forced(true);
        assert_eq!(s.red("err"), "\x1b[31merr\x1b[0m");
        assert_eq!(s.yellow("warn"), "\x1b[33mwarn\x1b[0m");
        assert_eq!(s.bold("x"), "\x1b[1mx\x1b[0m");
        assert_eq!(s.dim("12"), "\x1b[2m12\x1b[0m");
    }

    #[test]
    fn style_resolve_disables_for_json_output() {
        // Even if --no-color is unset and stdout were a tty, JSON output
        // must never carry escape codes.
        let s = Style::resolve(false, ResolvedFormat::Json, true);
        assert!(!s.enabled);
    }

    #[test]
    fn style_resolve_disables_with_no_color_flag() {
        let s = Style::resolve(true, ResolvedFormat::Text, true);
        assert!(!s.enabled);
    }

    #[test]
    fn style_resolve_four_combinations() {
        // (stdout_is_tty=true, no_color=false) + Text -> enabled
        assert!(Style::resolve(false, ResolvedFormat::Text, true).enabled);
        // (stdout_is_tty=false, no_color=false) + Text -> disabled (not a tty)
        assert!(!Style::resolve(false, ResolvedFormat::Text, false).enabled);
        // (stdout_is_tty=true, no_color=true) + Text -> disabled (user opted out)
        assert!(!Style::resolve(true, ResolvedFormat::Text, true).enabled);
        // (stdout_is_tty=true, no_color=false) + Json -> disabled (machine output)
        assert!(!Style::resolve(false, ResolvedFormat::Json, true).enabled);
    }

    /// Exhaustive table test for `style_compute`. Covers all 16 combinations
    /// of (no_color_flag, format, stdout_is_tty, no_color_env). Style is
    /// enabled if and only if ALL of: no_color_flag=false, format=Text,
    /// stdout_is_tty=true, no_color_env=false.
    #[test]
    fn style_compute_all_16_combinations() {
        // (no_color_flag, format, stdout_is_tty, no_color_env, expected_enabled)
        let cases: &[(bool, ResolvedFormat, bool, bool, bool)] = &[
            // The only enabled case: all inhibitors off, Text, is a tty.
            (false, ResolvedFormat::Text, true, false, true),
            // no_color_env set kills it.
            (false, ResolvedFormat::Text, true, true, false),
            // not a tty kills it.
            (false, ResolvedFormat::Text, false, false, false),
            (false, ResolvedFormat::Text, false, true, false),
            // JSON format kills it regardless of tty / env.
            (false, ResolvedFormat::Json, true, false, false),
            (false, ResolvedFormat::Json, true, true, false),
            (false, ResolvedFormat::Json, false, false, false),
            (false, ResolvedFormat::Json, false, true, false),
            // no_color_flag kills it regardless of everything else.
            (true, ResolvedFormat::Text, true, false, false),
            (true, ResolvedFormat::Text, true, true, false),
            (true, ResolvedFormat::Text, false, false, false),
            (true, ResolvedFormat::Text, false, true, false),
            (true, ResolvedFormat::Json, true, false, false),
            (true, ResolvedFormat::Json, true, true, false),
            (true, ResolvedFormat::Json, false, false, false),
            (true, ResolvedFormat::Json, false, true, false),
        ];
        for (i, &(nc_flag, fmt, is_tty, nc_env, expected)) in cases.iter().enumerate() {
            let s = style_compute(nc_flag, fmt, is_tty, nc_env);
            assert_eq!(
                s.enabled,
                expected,
                "case {i}: style_compute(no_color_flag={nc_flag}, \
                 format={:?}, stdout_is_tty={is_tty}, no_color_env={nc_env}) \
                 -> expected enabled={expected}, got {}",
                if fmt == ResolvedFormat::Text {
                    "Text"
                } else {
                    "Json"
                },
                s.enabled
            );
        }
    }

    #[test]
    fn is_catalog_request_matches_only_question_mark() {
        assert!(is_catalog_request(Some("?")));
        assert!(!is_catalog_request(Some("function")));
        assert!(!is_catalog_request(Some("reference")));
        assert!(!is_catalog_request(None));
    }

    #[test]
    fn print_catalog_does_not_panic_on_empty_slice() {
        // Smoke: empty catalog renders without panic in both formats.
        let items: Vec<&str> = vec![];
        print_catalog(
            "kinds",
            &items,
            |_| unreachable!(),
            |_| unreachable!(),
            ResolvedFormat::Text,
            false,
        )
        .expect("empty text catalog must not error");
        print_catalog(
            "kinds",
            &items,
            |_| unreachable!(),
            |_| unreachable!(),
            ResolvedFormat::Json,
            false,
        )
        .expect("empty json catalog must not error");
    }
}
