// One file per `pine <subcommand>` (everything except `version`, which
// stays in main.rs because it self-describes the binary it lives in).
// Each module exposes `pub fn run(...)` taking the parsed args + the
// resolved output format (+ Style when the command emits styled text).

pub(crate) mod behavior;
pub(crate) mod diff;
pub(crate) mod indicator;
pub(crate) mod lookup;
pub(crate) mod parse;
pub(crate) mod probe;
pub(crate) mod probes;
pub(crate) mod search;
pub(crate) mod tokens;
pub(crate) mod validate;
