// Record validation, shared by the write path (`add` / `observe` refuse to
// write an invalid record) and the strict read path (`load` refuses to serve
// a store holding one). Errors block; warnings are surfaced but allowed.

use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use toml::value::{Datetime, Offset};

use super::load::Relation;
use super::{
    AmendField, Amendment, Citation, Diag, Kind, Observation, Outcome, Question, Source, Void,
};
use crate::behavior;

/// Problems found in one or more records.
#[derive(Debug, Default)]
pub(super) struct Report {
    pub(super) errors: Vec<String>,
    pub(super) warnings: Vec<String>,
}

/// What validation needs beyond the question itself.
pub(super) struct Context<'a> {
    /// Every question id in the question's own records directory, mapped to
    /// whether that question is retired (an active question may not rest on
    /// a retired premise).
    pub(super) known_ids: &'a BTreeMap<String, bool>,
    /// A fixture sha256 about to be written alongside this question: treated
    /// as present, since the write path validates before touching disk.
    pub(super) pending_fixture: Option<&'a str>,
}

/// The records file a question lives in.
pub(super) fn question_path(root: &Path, id: &str) -> PathBuf {
    root.join(format!("{id}.toml"))
}

/// The content-addressed fixture store under a records directory.
pub(super) fn fixture_path(root: &Path, sha256: &str) -> PathBuf {
    root.join("fixtures").join(format!("{sha256}.pine"))
}

/// Lowercase hex sha256 of `bytes`.
pub(super) fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    digest.iter().map(|b| format!("{b:02x}")).collect()
}

/// A question id: 8 lowercase hex digits.
pub(super) fn is_id(s: &str) -> bool {
    s.len() == 8 && s.bytes().all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f'))
}

fn is_sha256(s: &str) -> bool {
    s.len() == 64 && s.bytes().all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f'))
}

/// Validate one question and all its observations into `out`.
pub(super) fn question(q: &Question, cx: &Context<'_>, out: &mut Report) {
    let file = question_path(&q.root, &q.id).display().to_string();
    let mut errs = Vec::new();
    let mut warns = Vec::new();

    if q.question.trim().is_empty() {
        errs.push("`question` is empty".to_string());
    }
    if q.answer.trim().is_empty() {
        errs.push("`answer` is empty".to_string());
    }
    if q.key.as_deref().is_some_and(|k| k.trim().is_empty()) {
        errs.push("`key` is empty".to_string());
    }
    let mut observation_keys = BTreeSet::new();
    for key in q.observations.iter().filter_map(|o| o.key.as_deref()) {
        if !observation_keys.insert(key) {
            errs.push(format!("observation key `{key}` used twice"));
        }
    }
    let mut seen_idents = BTreeSet::new();
    for ident in &q.identifiers {
        if let Err(msg) = identifier(ident) {
            errs.push(msg);
        }
        if !seen_idents.insert(ident.to_ascii_lowercase()) {
            errs.push(format!("identifier `{ident}` listed twice"));
        }
    }
    // Every record the old writer produced carries `derived_from = []`; an
    // empty list claims nothing, so only a populated one needs a decision.
    let empty_list = |v: &toml::Value| v.as_array().is_some_and(Vec::is_empty);
    if q.legacy_derived_from
        .as_ref()
        .is_some_and(|v| !empty_list(v))
    {
        errs.push(
            "`derived_from` was split: use `follow_up_to` for lineage (this investigation grew out of that one), `basis` for premises an inferred answer is concluded from, or `[retired]` to withdraw the question"
                .to_string(),
        );
    }
    for relation in [Relation::FollowUpTo, Relation::Basis, Relation::ReplacedBy] {
        let field = relation.field();
        let mut seen = BTreeSet::new();
        for id in relation.of(q) {
            if id == &q.id {
                errs.push(format!("`{field}` names the question itself"));
            } else if !cx.known_ids.contains_key(id) {
                errs.push(format!("`{field}` names unknown question `{id}`"));
            }
            if !seen.insert(id) {
                errs.push(format!("`{field}` lists `{id}` twice"));
            }
        }
    }
    if q.is_inferred() {
        for id in &q.basis {
            if cx.known_ids.get(id) == Some(&true) {
                errs.push(format!(
                    "`basis` names retired question `{id}`: an active inference cannot rest on a withdrawn premise"
                ));
            }
        }
        if q.observations.iter().any(Observation::decides) {
            errs.push(
                "an inferred question (non-empty `basis`) carries a counting editor/chart observation: either answer it by measurement (drop `basis`), or mark that observation inconclusive or void it"
                    .to_string(),
            );
        }
    }
    if let Some(r) = &q.retired
        && r.reason.trim().is_empty()
    {
        errs.push("`retired.reason` is empty".to_string());
    }
    if q.is_inferred() && q.citations.iter().any(|c| c.void.is_none()) {
        errs.push(
            "an inferred question (non-empty `basis`) carries an active citation: a citation is an answer route of its own, so either document it (drop `basis`) or void the citation"
                .to_string(),
        );
    }
    for (i, c) in q.citations.iter().enumerate() {
        for msg in citation(c) {
            errs.push(format!("citation {}: {msg}", i + 1));
        }
    }
    errs.extend(amendments(&q.amendments, |f| {
        (f == AmendField::Answer).then(|| Some(q.answer.clone()))
    }));
    // Runtime status compares candidates across runs by name, so a name must
    // mean the same model everywhere in the question. A void observation is
    // out of the comparison: voiding a mis-recorded model is how it is fixed.
    let mut models: BTreeMap<&str, Option<&str>> = BTreeMap::new();
    for c in q
        .observations
        .iter()
        .filter(|o| o.void.is_none())
        .flat_map(|o| &o.candidates)
    {
        let model = c.model.as_deref();
        match models.get(c.name.as_str()) {
            Some(first) if *first != model => {
                errs.push(format!(
                    "candidate `{}` names different models across observations",
                    c.name
                ));
            }
            Some(_) => {}
            None => {
                models.insert(c.name.as_str(), model);
            }
        }
    }
    for (i, o) in q.observations.iter().enumerate() {
        let label = format!(
            "observation {} ({} {})",
            i + 1,
            o.source.as_str(),
            o.date_string()
        );
        for msg in observation(q.kind, o, &q.root, cx) {
            errs.push(format!("{label}: {msg}"));
        }
        if q.kind == Kind::Compile && o.outcome == Some(Outcome::Rejected) && o.errors.is_empty() {
            warns.push(format!("{label}: rejected with no error recorded"));
        }
    }
    if q.identifiers.is_empty() {
        warns.push("no identifiers".to_string());
    }
    out.errors
        .extend(errs.into_iter().map(|m| format!("{file}: {m}")));
    out.warnings
        .extend(warns.into_iter().map(|m| format!("{file}: {m}")));
}

/// Every question that reaches itself through one relation (`basis`,
/// `follow_up_to` or `retired.replaced_by`), checked per relation: a parent
/// replaced by a child that follows up to it is legitimate. Lineage and
/// replacement are histories, and history is acyclic; a basis cycle would
/// make resolution circular. `add` cannot create a cycle (its id is new),
/// but `retire` and hand edits can, so both the write and read paths run it.
pub(super) fn cycles(questions: &[Question]) -> Vec<String> {
    let mut out = Vec::new();
    for relation in [Relation::Basis, Relation::FollowUpTo, Relation::ReplacedBy] {
        let name = relation.field();
        let edges: BTreeMap<&str, &[String]> = questions
            .iter()
            .map(|q| (q.id.as_str(), relation.of(q)))
            .collect();
        for q in questions {
            let mut stack: Vec<&str> = relation.of(q).iter().map(String::as_str).collect();
            let mut seen = BTreeSet::new();
            while let Some(id) = stack.pop() {
                if id == q.id {
                    out.push(format!("question {} reaches itself through `{name}`", q.id));
                    break;
                }
                if seen.insert(id) {
                    stack.extend(
                        edges
                            .get(id)
                            .into_iter()
                            .flat_map(|d| d.iter().map(String::as_str)),
                    );
                }
            }
        }
    }
    out
}

/// Every capture key held by more than one question of one records directory.
/// A key identifies one question per directory, so a rerun capture script can
/// find the question it created.
pub(super) fn duplicate_keys(questions: &[Question]) -> Vec<String> {
    let mut holders: BTreeMap<(&Path, &str), Vec<&str>> = BTreeMap::new();
    for q in questions {
        if let Some(key) = &q.key {
            holders
                .entry((q.root.as_path(), key.as_str()))
                .or_default()
                .push(q.id.as_str());
        }
    }
    holders
        .into_iter()
        .filter(|(_, ids)| ids.len() > 1)
        .map(|((root, key), ids)| {
            format!(
                "{}: key `{key}` is held by questions {}",
                root.display(),
                ids.join(", ")
            )
        })
        .collect()
}

/// Question id -> retired, for every question in `questions` that lives in
/// `root` (relations resolve within a question's own directory only).
pub(super) fn ids_in(questions: &[Question], root: &Path) -> BTreeMap<String, bool> {
    questions
        .iter()
        .filter(|q| q.root == root)
        .map(|q| (q.id.clone(), q.is_retired()))
        .collect()
}

/// An identifier is a pine-data name (function, variable, keyword, operator,
/// ...) or a qualified parameter `function(parameter)` whose parameter belongs
/// to that function in some overload. A bare parameter name is refused: the
/// same name is a parameter of many functions.
fn identifier(ident: &str) -> Result<(), String> {
    let qualified = ident
        .strip_suffix(')')
        .and_then(|s| s.split_once('('))
        .filter(|(func, param)| !func.is_empty() && !param.is_empty());
    let Some((func, param)) = qualified else {
        return if behavior::lookup_all(ident).is_empty() {
            Err(format!(
                "identifier `{ident}` is not in pine-data (qualify a parameter as `function(parameter)`)"
            ))
        } else {
            Ok(())
        };
    };
    let functions: Vec<behavior::FunctionBehavior> = behavior::lookup_all(func)
        .into_iter()
        .filter_map(|b| match b {
            behavior::Behavior::Function(f) => Some(f),
            _ => None,
        })
        .collect();
    if functions.is_empty() {
        return Err(format!(
            "identifier `{ident}`: `{func}` is not a pine-data function"
        ));
    }
    let has_param = functions.iter().any(|f| {
        f.parameters
            .iter()
            .chain(f.overloads.iter().flat_map(|o| &o.parameters))
            .any(|p| p.name == param)
    });
    if has_param {
        Ok(())
    } else {
        Err(format!(
            "identifier `{ident}`: `{param}` is not a parameter of `{func}`"
        ))
    }
}

/// Every error in one observation, as messages without location.
fn observation(kind: Kind, o: &Observation, root: &Path, cx: &Context<'_>) -> Vec<String> {
    let mut errs = Vec::new();

    match (&o.date, &o.date_before) {
        (Some(_), Some(_)) => errs.push("both `date` and `date_before` are set".to_string()),
        (None, None) => errs.push("needs a `date` or a `date_before` bound".to_string()),
        _ => {}
    }
    for (field, value) in [("date", &o.date), ("date_before", &o.date_before)] {
        if let Some(d) = value
            && (d.date.is_none() || d.time.is_some() || d.offset.is_some())
        {
            errs.push(format!("{field} `{d}` must be a plain YYYY-MM-DD date"));
        }
    }
    for (field, value) in [
        ("key", &o.key),
        ("crash", &o.crash),
        ("result", &o.result),
        ("settings", &o.settings),
        ("inconclusive", &o.inconclusive),
        ("environment", &o.environment),
        ("note", &o.note),
        ("fixture_name", &o.fixture_name),
    ] {
        if value.as_deref().is_some_and(|v| v.trim().is_empty()) {
            errs.push(format!("`{field}` is empty"));
        }
    }

    match kind {
        Kind::Compile => compile_fields(o, &mut errs),
        Kind::Runtime => runtime_fields(o, &mut errs),
    }
    for d in &o.errors {
        diag(kind, d, false, &mut errs);
    }
    for d in &o.warnings {
        diag(kind, d, true, &mut errs);
    }

    match &o.fixture {
        Some(sha) => fixture(root, sha, cx, &mut errs),
        None => {
            if o.fixture_name.is_some() {
                errs.push("`fixture_name` without `fixture`".to_string());
            }
        }
    }
    for e in &o.evidence {
        if e.trim().is_empty() {
            errs.push("empty evidence path".to_string());
        } else if !root.join(e).is_file() {
            errs.push(format!("evidence `{e}` is not an existing file"));
        }
    }
    if let Some(v) = &o.void {
        errs.extend(void(v));
    }
    errs.extend(amendments(&o.amendments, |f| o.field(f).cloned()));
    errs
}

/// Errors in one citation's shape. Whether it is still current against the
/// baked manual is NOT an error: a stale citation stops counting instead, so
/// a manual re-vendor never fails the load or blocks the writes that repair
/// it.
fn citation(c: &Citation) -> Vec<String> {
    let mut errs = Vec::new();
    let well_formed = c
        .section
        .split_once('#')
        .is_some_and(|(page, anchor)| !page.is_empty() && !anchor.is_empty());
    if !well_formed {
        errs.push(format!("section `{}` is not `page#anchor`", c.section));
    }
    if c.quotes.is_empty() {
        errs.push("no quotes".to_string());
    }
    if c.quotes.iter().any(|q| q.trim().is_empty()) {
        errs.push("an empty quote".to_string());
    }
    if !is_sha256(&c.digest) {
        errs.push(format!("digest `{}` is not a lowercase sha256", c.digest));
    }
    if !is_utc_timestamp(&c.at) {
        errs.push(format!("`at` `{}` must be a UTC timestamp", c.at));
    }
    if c.note.as_deref().is_some_and(|n| n.trim().is_empty()) {
        errs.push("`note` is empty".to_string());
    }
    if let Some(v) = &c.void {
        errs.extend(void(v));
    }
    errs
}

/// Errors in a `void` table.
pub(super) fn void(v: &Void) -> Vec<String> {
    let mut errs = Vec::new();
    if v.reason.trim().is_empty() {
        errs.push("`void.reason` is empty".to_string());
    }
    if !is_utc_timestamp(&v.at) {
        errs.push(format!("`void.at` `{}` must be a UTC timestamp", v.at));
    }
    errs
}

/// `YYYY-MM-DDTHH:MM:SSZ`, as po writes amendment and void times.
fn is_utc_timestamp(d: &Datetime) -> bool {
    d.date.is_some() && d.time.is_some() && d.offset == Some(Offset::Z)
}

/// Errors in an amendment history. `current(field)` is the field's current
/// value, or `None` when the field is not amendable at this level. Each
/// field's amendments must chain (`now` of one is `was` of the next) and end
/// at the current value, so an amended field is not edited by hand without
/// its history.
fn amendments(
    list: &[Amendment],
    current: impl Fn(AmendField) -> Option<Option<String>>,
) -> Vec<String> {
    let mut errs = Vec::new();
    let mut last: BTreeMap<AmendField, &Amendment> = BTreeMap::new();
    for (i, a) in list.iter().enumerate() {
        let label = format!("amendment {} ({})", i + 1, a.field.as_str());
        if current(a.field).is_none() {
            errs.push(format!(
                "{label}: `{}` is not amendable here",
                a.field.as_str()
            ));
            continue;
        }
        if a.reason.trim().is_empty() {
            errs.push(format!("{label}: `reason` is empty"));
        }
        if !is_utc_timestamp(&a.at) {
            errs.push(format!("{label}: `at` `{}` must be a UTC timestamp", a.at));
        }
        if a.was == a.now {
            errs.push(format!("{label}: changes nothing"));
        }
        if [&a.was, &a.now]
            .into_iter()
            .flatten()
            .any(|v| v.trim().is_empty())
        {
            errs.push(format!(
                "{label}: empty `was` or `now` (leave the field out instead)"
            ));
        }
        if let Some(prev) = last.get(&a.field)
            && prev.now != a.was
        {
            errs.push(format!(
                "{label}: `was` is not the previous amendment's `now`"
            ));
        }
        last.insert(a.field, a);
    }
    for (field, a) in last {
        if current(field).flatten() != a.now {
            errs.push(format!(
                "`{}` differs from its last amendment's `now`: amend it rather than editing it by hand",
                field.as_str()
            ));
        }
    }
    errs
}

fn compile_fields(o: &Observation, errs: &mut Vec<String>) {
    match o.outcome {
        None => errs.push("compile observation needs an outcome".to_string()),
        Some(Outcome::Accepted) if !o.errors.is_empty() => {
            errs.push("accepted but carries errors".to_string());
        }
        Some(Outcome::Crashed) if !o.errors.is_empty() || !o.warnings.is_empty() => {
            errs.push("crashed but carries diagnostics".to_string());
        }
        _ => {}
    }
    match (o.outcome == Some(Outcome::Crashed), o.crash.is_some()) {
        (true, false) => errs.push("crashed without a `crash` message".to_string()),
        (false, true) => {
            errs.push("`crash` message on an observation that did not crash".to_string());
        }
        _ => {}
    }
    for (field, present) in [
        ("result", o.result.is_some()),
        ("settings", o.settings.is_some()),
        ("candidate", !o.candidates.is_empty()),
    ] {
        if present {
            errs.push(format!("`{field}` belongs to runtime observations"));
        }
    }
}

fn runtime_fields(o: &Observation, errs: &mut Vec<String>) {
    if o.source != Source::Chart {
        errs.push(format!(
            "runtime observations come from a chart run, not `{}`",
            o.source.as_str()
        ));
    }
    if o.outcome.is_some() || o.crash.is_some() {
        errs.push("accept / reject / crash belong to compile observations".to_string());
    }
    if o.result.is_none() {
        errs.push("runtime observation needs a `result`".to_string());
    }
    if o.settings.is_none() {
        errs.push("runtime observation needs `settings`".to_string());
    }
    if !o.warnings.is_empty() {
        errs.push("runtime observations carry errors only, not warnings".to_string());
    }
    let mut names = BTreeSet::new();
    for c in &o.candidates {
        if c.name.trim().is_empty() {
            errs.push("candidate with an empty name".to_string());
        } else if !names.insert(c.name.as_str()) {
            errs.push(format!("candidate `{}` listed twice", c.name));
        }
        if c.model.as_deref().is_some_and(|m| m.trim().is_empty()) {
            errs.push(format!("candidate `{}` has an empty model", c.name));
        }
    }
}

/// Code prefix by question kind and severity: compile errors are CE, compile
/// warnings CW, runtime errors RE; always followed by five digits.
fn diag(kind: Kind, d: &Diag, warning: bool, errs: &mut Vec<String>) {
    let prefix = match (kind, warning) {
        (Kind::Compile, false) => "CE",
        (Kind::Compile, true) => "CW",
        (Kind::Runtime, _) => "RE",
    };
    let digits = d.code.strip_prefix(prefix);
    if !digits.is_some_and(|n| n.len() == 5 && n.bytes().all(|b| b.is_ascii_digit())) {
        errs.push(format!(
            "code `{}` must be {prefix} followed by five digits on a {} {}",
            d.code,
            kind.as_str(),
            if warning { "warning" } else { "error" }
        ));
    }
    // An absent message is honest (the source never recorded it); an empty
    // one is a mistake.
    if d.message.as_deref().is_some_and(|m| m.trim().is_empty()) {
        errs.push(format!("{} has an empty message", d.code));
    }
    if let Some(span) = &d.span
        && !super::spec::is_span(span)
    {
        errs.push(format!(
            "{} span `{span}` is not an ordered 1-based line:col-line:col range",
            d.code
        ));
    }
    for (k, v) in &d.ctx {
        if k.trim().is_empty() || v.trim().is_empty() {
            errs.push(format!("{} has an empty ctx key or value", d.code));
        }
    }
}

fn fixture(root: &Path, sha: &str, cx: &Context<'_>, errs: &mut Vec<String>) {
    if !is_sha256(sha) {
        errs.push(format!("fixture `{sha}` is not a lowercase sha256"));
        return;
    }
    if cx.pending_fixture == Some(sha) {
        return;
    }
    let path = fixture_path(root, sha);
    match std::fs::read(&path) {
        Ok(bytes) => {
            let actual = sha256_hex(&bytes);
            if actual != sha {
                errs.push(format!(
                    "fixture {} hashes to {actual}, not its name",
                    path.display()
                ));
            }
        }
        Err(e) => errs.push(format!("fixture {}: {e}", path.display())),
    }
}
