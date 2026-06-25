// One file per `pine <subcommand>` (everything except `version`, which
// stays in main.rs because it self-describes the binary it lives in).
// Each module exposes `pub(crate) fn run(...)` taking the parsed args + the
// `Style` used for terminal text output.

pub(crate) mod lookup;
pub(crate) mod recipe;
pub(crate) mod search;
pub(crate) mod show;
