// Shared output infrastructure for every `pine <subcommand>`:
//   - ResolvedFormat: the post-`Auto`-resolution flavour each command receives.
//   - Style: ANSI colour wrapper, off when the user passed `--no-color`,
//     the `NO_COLOR` env var is present, or stdout isn't a TTY.
//   - SCHEMA_VERSION + versioned_json + print_json: every JSON payload
//     gets wrapped in the same `schema_version`-bearing envelope.

use anyhow::Result;
use serde::Serialize;
use std::io::IsTerminal;

/// JSON output schema version. Bumped on any breaking shape change to a
/// subcommand's JSON output. Documented in `docs/pine-oracle.md`.
pub(crate) const SCHEMA_VERSION: u32 = 1;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum ResolvedFormat {
    Text,
    Json,
}

/// Wrap `payload` in the versioned envelope. Objects get the field
/// inserted at the top level; arrays and scalars are wrapped as
/// `{schema_version, items}` / `{schema_version, value}`. Exposed so
/// the binary's tests can pin the wire shape.
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
#[derive(Clone, Copy)]
pub(crate) struct Style {
    enabled: bool,
}

impl Style {
    pub(crate) fn resolve(no_color: bool, format: ResolvedFormat) -> Self {
        let enabled = !no_color
            && std::env::var_os("NO_COLOR").is_none()
            && format == ResolvedFormat::Text
            && std::io::stdout().is_terminal();
        Style { enabled }
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
    fn schema_version_is_one() {
        // Hard-pin: bumping SCHEMA_VERSION requires updating this test AND
        // the docs in docs/pine-oracle.md "Schema versioning" section.
        assert_eq!(SCHEMA_VERSION, 1);
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
        let s = Style::resolve(false, ResolvedFormat::Json);
        assert!(!s.enabled);
    }

    #[test]
    fn style_resolve_disables_with_no_color_flag() {
        let s = Style::resolve(true, ResolvedFormat::Text);
        assert!(!s.enabled);
    }
}
