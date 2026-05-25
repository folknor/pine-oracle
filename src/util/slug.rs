// SPDX-License-Identifier: MPL-2.0
//
// Shared slug sanitisation for the corpus and indicator-fixture loaders.
//
// The corpus version is the canonical reference: it rejects backslash-style
// slugs, validates each segment individually, and produces clear error
// messages. The indicator-fixture version (Path::components) only splits on
// `/` on Linux and would silently accept `\` in a segment name.

use anyhow::{Result, bail};

/// Validate and normalise a probe / indicator-fixture slug.
///
/// `prefix` is the leading directory component to strip when present
/// (e.g. `"validation"` for corpus slugs, `"indicators"` for fixture slugs).
/// The slash after the prefix is consumed automatically.
///
/// Rules:
/// - Leading and trailing whitespace is trimmed.
/// - An optional `<prefix>/` leader is stripped.
/// - The slug must be non-empty after stripping.
/// - The slug must not start with `/` or `\`.
/// - Every path segment (split on `/` or `\`) must be non-empty and must not
///   be `.` or `..`.
/// - No segment may contain a `\` character (defensive: ensures neither slash
///   style sneaks a traversal through on Linux where `\` is a valid filename
///   character).
///
/// Returns a borrow of the relevant slice of the input string (post-prefix,
/// pre-whitespace trim) so no allocation is needed for the happy path.
pub(crate) fn sanitise_slug<'a>(slug: &'a str, prefix: &str) -> Result<&'a str> {
    let slug = slug.trim();
    let prefix_leader = format!("{prefix}/");
    let slug = slug.strip_prefix(prefix_leader.as_str()).unwrap_or(slug);
    if slug.is_empty() {
        bail!("slug cannot be empty (prefix: `{prefix}`)");
    }
    if slug.contains('\\') {
        bail!("slug cannot contain backslash: `{slug}`");
    }
    if slug.starts_with('/') {
        bail!("slug cannot be absolute: `{slug}`");
    }
    for segment in slug.split('/') {
        if segment.is_empty() || segment == "." || segment == ".." {
            bail!("slug has invalid segment `{segment}` in `{slug}`");
        }
    }
    Ok(slug)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_known_prefix() {
        let result = sanitise_slug("validation/my-probe-01", "validation").unwrap();
        assert_eq!(result, "my-probe-01");
    }

    #[test]
    fn strips_indicator_prefix() {
        let result = sanitise_slug("indicators/smoke-basic", "indicators").unwrap();
        assert_eq!(result, "smoke-basic");
    }

    #[test]
    fn passthrough_without_prefix() {
        let result = sanitise_slug("my-probe-01", "validation").unwrap();
        assert_eq!(result, "my-probe-01");
    }

    #[test]
    fn rejects_empty_slug() {
        let err = sanitise_slug("", "validation").unwrap_err();
        assert!(err.to_string().contains("empty"), "got: {err}");
    }

    #[test]
    fn rejects_prefix_only() {
        // "validation/" after stripping prefix gives ""
        let err = sanitise_slug("validation/", "validation").unwrap_err();
        assert!(err.to_string().contains("empty"), "got: {err}");
    }

    #[test]
    fn rejects_dot_dot_segment() {
        let err = sanitise_slug("foo/../bar", "validation").unwrap_err();
        assert!(err.to_string().contains("invalid segment"), "got: {err}");
    }

    #[test]
    fn rejects_dot_segment() {
        let err = sanitise_slug("foo/./bar", "validation").unwrap_err();
        assert!(err.to_string().contains("invalid segment"), "got: {err}");
    }

    #[test]
    fn rejects_empty_segment() {
        let err = sanitise_slug("foo//bar", "validation").unwrap_err();
        assert!(err.to_string().contains("invalid segment"), "got: {err}");
    }

    #[test]
    fn rejects_backslash() {
        let err = sanitise_slug("foo\\bar", "validation").unwrap_err();
        // A backslash-separated slug has an invalid segment (empty after splitting
        // on both slash types) or triggers the backslash check directly.
        assert!(!err.to_string().is_empty(), "must produce an error: {err}");
    }

    #[test]
    fn accepts_nested_slug() {
        let result = sanitise_slug("symbol-specified/AAPL/foo-01", "validation").unwrap();
        assert_eq!(result, "symbol-specified/AAPL/foo-01");
    }

    #[test]
    fn trims_whitespace() {
        let result = sanitise_slug("  my-probe  ", "validation").unwrap();
        assert_eq!(result, "my-probe");
    }
}
