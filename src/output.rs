// Shared output infrastructure for every `pine <subcommand>`:
//   - Style: ANSI colour wrapper, off when the user passed `--no-color`,
//     the `NO_COLOR` env var is present, or stdout isn't a TTY.
//   - CATALOG_MARKER + is_catalog_request: unified sentinel check for
//     `--kind ?` catalog-listing requests.
//   - print_catalog: text rows for the `--kind` catalog (honours --quiet).
//
// Every command is text-only.

use anyhow::Result;

/// The sentinel value a user passes to request a catalog listing instead of
/// a real filter. Every `--kind ?` gate checks against this constant via
/// `is_catalog_request`.
pub(crate) const CATALOG_MARKER: &str = "?";

/// Returns `true` when the optional filter string is the catalog sentinel `?`.
pub(crate) fn is_catalog_request(s: Option<&str>) -> bool {
    s == Some(CATALOG_MARKER)
}

/// Catalog printer for the `--kind` listing: one text row per item, honouring
/// `--quiet` (name-only rows).
///
/// Parameters:
/// - `items`: the catalog slice.
/// - `text_row`: full row formatter (name + count + description columns).
/// - `text_row_quiet`: quiet row formatter (name column only).
/// - `quiet`: when true, use `text_row_quiet` instead of `text_row`.
pub(crate) fn print_catalog<T>(
    items: &[T],
    text_row: impl Fn(&T) -> String,
    text_row_quiet: impl Fn(&T) -> String,
    quiet: bool,
) -> Result<()> {
    for item in items {
        if quiet {
            println!("{}", text_row_quiet(item));
        } else {
            println!("{}", text_row(item));
        }
    }
    Ok(())
}

/// ANSI styling for terminal text output. Disabled when `--no-color` is
/// set, when the `NO_COLOR` env var is present (no-color.org convention),
/// or when stdout is not a TTY.
#[must_use]
#[derive(Clone, Copy)]
pub(crate) struct Style {
    enabled: bool,
}

/// Pure, fully-deterministic styling decision. All three inputs are passed
/// in explicitly so this function is trivially testable without touching
/// process env vars or file descriptors.
///
/// - `no_color_flag`: the caller-parsed `--no-color` flag value.
/// - `stdout_is_tty`: pre-probed TTY state for stdout.
/// - `no_color_env`: whether the `NO_COLOR` environment variable is set.
///   Pass `std::env::var_os("NO_COLOR").is_some()` from the call site.
pub(crate) fn style_compute(no_color_flag: bool, stdout_is_tty: bool, no_color_env: bool) -> Style {
    let enabled = !no_color_flag && !no_color_env && stdout_is_tty;
    Style { enabled }
}

impl Style {
    /// Resolve styling from the two determinants: whether the caller passed
    /// `--no-color`, and whether stdout is a TTY. `stdout_is_tty` must be
    /// resolved once at startup and threaded in so multiple calls do not
    /// independently probe the stream.
    ///
    /// Reads `NO_COLOR` from the process environment once per call and
    /// delegates to [`style_compute`] for the actual decision. Use
    /// `style_compute` directly in tests to avoid process-global env var
    /// races.
    pub(crate) fn resolve(no_color: bool, stdout_is_tty: bool) -> Self {
        style_compute(
            no_color,
            stdout_is_tty,
            std::env::var_os("NO_COLOR").is_some(),
        )
    }

    /// Whether ANSI styling is on. `po search` passes the inverse to the
    /// markdown renderer as its `no_color` flag.
    pub(crate) fn enabled(self) -> bool {
        self.enabled
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
    fn style_disabled_returns_unwrapped_text() {
        let s = Style::forced(false);
        assert_eq!(s.bold("name"), "name");
        assert_eq!(s.dim("12"), "12");
    }

    #[test]
    fn style_enabled_wraps_with_ansi_escape_codes() {
        let s = Style::forced(true);
        assert_eq!(s.bold("x"), "\x1b[1mx\x1b[0m");
        assert_eq!(s.dim("12"), "\x1b[2m12\x1b[0m");
    }

    #[test]
    fn style_resolve_disables_with_no_color_flag() {
        let s = Style::resolve(true, true);
        assert!(!s.enabled);
    }

    /// Exhaustive table test for `style_compute`. Covers all 8 combinations
    /// of (no_color_flag, stdout_is_tty, no_color_env). Style is enabled if
    /// and only if ALL of: no_color_flag=false, stdout_is_tty=true,
    /// no_color_env=false.
    #[test]
    fn style_compute_all_8_combinations() {
        // (no_color_flag, stdout_is_tty, no_color_env, expected_enabled)
        let cases: &[(bool, bool, bool, bool)] = &[
            // The only enabled case: all inhibitors off, is a tty.
            (false, true, false, true),
            // no_color_env set kills it.
            (false, true, true, false),
            // not a tty kills it.
            (false, false, false, false),
            (false, false, true, false),
            // no_color_flag kills it regardless of everything else.
            (true, true, false, false),
            (true, true, true, false),
            (true, false, false, false),
            (true, false, true, false),
        ];
        for (i, &(nc_flag, is_tty, nc_env, expected)) in cases.iter().enumerate() {
            let s = style_compute(nc_flag, is_tty, nc_env);
            assert_eq!(
                s.enabled, expected,
                "case {i}: style_compute(no_color_flag={nc_flag}, \
                 stdout_is_tty={is_tty}, no_color_env={nc_env}) \
                 -> expected enabled={expected}, got {}",
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
        // Smoke: empty catalog renders without panic.
        let items: Vec<&str> = vec![];
        print_catalog(&items, |_| unreachable!(), |_| unreachable!(), false)
            .expect("empty catalog must not error");
    }
}
