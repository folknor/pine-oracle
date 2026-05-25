// SPDX-License-Identifier: MPL-2.0
//
// Shared UTF-8 file accessors for include_dir-embedded asset trees.
//
// Both the corpus loader and the indicator-fixture loader do the same two
// operations against their respective embedded Dir: fetch a required file
// (error when absent or non-UTF-8) and fetch an optional file (None when
// absent, error when present but non-UTF-8). The only difference between
// the two call sites is the framing noun used in error messages ("probe"
// vs "indicator fixture").

use anyhow::{Result, anyhow, bail};
use include_dir::Dir;

/// Load a required UTF-8 file from an embedded `include_dir` tree.
///
/// `dir` is the embedded directory (e.g. `&CORPUS` or `&INDICATORS`).
/// `slug` identifies the probe or fixture (used in error messages).
/// `file_name` is the filename within the slug subdirectory (e.g. `"strategy.pine"`).
/// `framing_noun` is the human-readable name for the asset kind (e.g. `"probe"`
/// or `"indicator fixture"`), used verbatim in error messages.
///
/// Returns a str slice tied to the Dir's embedded data lifetime `'data`
/// (typically `'static` for compile-time embedded directories).
pub(crate) fn require_utf8<'data>(
    dir: &Dir<'data>,
    slug: &str,
    file_name: &str,
    framing_noun: &str,
) -> Result<&'data str> {
    let path = format!("{slug}/{file_name}");
    let file = dir
        .get_file(&path)
        .ok_or_else(|| anyhow!("{framing_noun} `{slug}` is missing {file_name}"))?;
    file.contents_utf8()
        .ok_or_else(|| anyhow!("{framing_noun} `{slug}/{file_name}` is not valid UTF-8"))
}

/// Load an optional UTF-8 file from an embedded `include_dir` tree.
///
/// Returns `Ok(None)` when the file is absent. Returns `Err` when the file
/// exists but is not valid UTF-8.
///
/// See [`require_utf8`] for parameter documentation.
pub(crate) fn optional_utf8<'data>(
    dir: &Dir<'data>,
    slug: &str,
    file_name: &str,
    framing_noun: &str,
) -> Result<Option<&'data str>> {
    let path = format!("{slug}/{file_name}");
    let Some(file) = dir.get_file(&path) else {
        return Ok(None);
    };
    match file.contents_utf8() {
        Some(s) => Ok(Some(s)),
        None => bail!("{framing_noun} `{slug}/{file_name}` is not valid UTF-8"),
    }
}

#[cfg(test)]
mod tests {
    // The include_dir crate does not expose a way to build an in-process test
    // Dir from strings. Integration coverage lives in corpus::tests and
    // indicator::tests (which exercise the real CORPUS / INDICATORS trees via
    // load_probe / load_fixture). The helpers themselves are trivially thin
    // wrappers around include_dir's get_file / contents_utf8; if either of
    // those panics or returns unexpected results the corpus / indicator tests
    // will catch it.
    //
    // What we CAN test here: the absent-file path returns None, and the framing
    // noun is embedded in error messages. We do this via the real CORPUS tree
    // so we don't need a mock Dir.

    use include_dir::{Dir, include_dir};
    static CORPUS: Dir<'static> =
        include_dir!("$CARGO_MANIFEST_DIR/vendor/pineforge-corpus/validation");

    use super::*;

    #[test]
    fn require_utf8_errors_on_missing_file() {
        let slug = "oca-multi-bracket-isolation-01";
        let err = require_utf8(&CORPUS, slug, "nonexistent.txt", "probe").unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("probe"), "framing noun missing: {msg}");
        assert!(msg.contains("missing"), "expected 'missing': {msg}");
        assert!(msg.contains("nonexistent.txt"), "filename missing: {msg}");
    }

    #[test]
    fn optional_utf8_returns_none_on_missing_file() {
        let slug = "oca-multi-bracket-isolation-01";
        let result = optional_utf8(&CORPUS, slug, "nonexistent.txt", "probe").unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn require_utf8_loads_known_file() {
        let slug = "oca-multi-bracket-isolation-01";
        let content = require_utf8(&CORPUS, slug, "strategy.pine", "probe").unwrap();
        assert!(!content.is_empty());
    }

    #[test]
    fn optional_utf8_loads_known_file() {
        // inputs.json exists for the multi-mode probe
        let slug = "analyzer-self-test-multi-mode-01";
        let result = optional_utf8(&CORPUS, slug, "inputs.json", "probe").unwrap();
        assert!(result.is_some());
    }
}
