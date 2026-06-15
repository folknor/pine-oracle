// `include_dir!` bakes a directory tree into the binary at macro-expansion time,
// but - unlike `include_str!` / `include_bytes!` - it does NOT register the
// embedded files with Cargo's change tracker. So adding, removing, or editing a
// file inside an embedded tree does not, on its own, mark the crate dirty, and a
// stale snapshot ships until the file holding the macro is recompiled for some
// other reason. That bites two trees here: the authored recipe corpus
// (`assets/recipes`, embedded in `src/recipe.rs`) and the vendored manual
// (`vendor/pine-manual/v6`, embedded in `src/manual.rs`).
//
// Emitting `rerun-if-changed` for every file and directory under those trees
// fixes it: a directory entry triggers a rebuild when files are added or removed
// (the dir's mtime changes), and a per-file entry triggers one on edits. A
// re-run of this build script invalidates the crate, forcing the `include_dir!`
// macros to re-expand over the current tree.
//
// pine-data is embedded with `include_str!`, which already tracks its files, so
// it needs no entry here.

use std::path::Path;

fn main() {
    for dir in ["assets/recipes", "vendor/pine-manual/v6"] {
        register(Path::new(dir));
    }
}

fn register(path: &Path) {
    println!("cargo:rerun-if-changed={}", path.display());
    if path.is_dir() {
        let Ok(entries) = std::fs::read_dir(path) else {
            return;
        };
        for entry in entries.flatten() {
            register(&entry.path());
        }
    }
}
