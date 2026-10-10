// Evidence: the artifacts (chart exports, probe JSON) an observation points
// at. Unlike fixtures they are not copied into the store: each entry names a
// file elsewhere in the caller's repository, and since those files are large
// and get compressed or relocated after the fact, an entry carries its
// identity apart from its location.
//
// Identity is the sha256 of the DECODED content. A file is read raw, or as a
// gzip or zstd stream (every member / frame, each checksum, nothing trailing),
// and the interpretation is recorded beside the location and followed on
// every later read, never re-sniffed. So a record compressed to
// `record.json.zst` is the same evidence, while compacting its JSON is not.
// A digest computed when `observe` recorded the entry identifies the bytes
// supplied then; one established later (`attest`, or a `relocate` of an entry
// that had none) is marked `attested`: it identifies the bytes from then on,
// and says nothing about the capture before.
//
// Locations are records-relative paths in one lexical convention: a caller's
// path is made absolute against the working directory and normalized without
// touching the filesystem (`.` dropped, `..` folded), and po opens exactly the
// spelling it stores. Within one observation a location (current or past)
// belongs to one entry, so a keyed rerun naming an old location maps to one
// entry by lookup.
//
// Availability is never a load error: a missing or unfetched file does not
// stop the store from loading or from being repaired. `audit` (`po verdict
// evidence check`) is the content gate, and `show` reports each entry's state.
// Evidence never changes a question's status.

use anyhow::{Context as _, Result, bail};
use serde::{Deserialize, Serialize};
use sha2::{Digest as _, Sha256};
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::fmt;
use std::io::Read as _;
use std::path::{Component, Path, PathBuf};
use toml::value::Datetime;

use super::load::load_unlocked;
use super::write::{check_many, lock, now_utc, relative_path, write_question};
use super::{Question, Store};

/// How a compressed evidence file is decoded for its digest. Absent (on a
/// hashed entry) means the bytes are hashed as they are.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Encoding {
    Gzip,
    Zstd,
}

impl Encoding {
    pub fn as_str(self) -> &'static str {
        match self {
            Encoding::Gzip => "gzip",
            Encoding::Zstd => "zstd",
        }
    }
}

/// When and why a digest was established after the observation was recorded.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Attested {
    /// UTC timestamp.
    pub at: Datetime,
    pub reason: String,
}

/// A location an entry left: where it was, and how it was read there.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Move {
    /// UTC timestamp of the move.
    pub at: Datetime,
    /// The former location, records-relative.
    pub from: String,
    /// The interpretation the digest was read under there (absent: raw).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub encoding: Option<Encoding>,
    pub reason: String,
}

/// One evidence artifact of an observation. A legacy entry (written before
/// digests, or as a bare path string) has only `path`: its interpretation is
/// unknown, not raw, and po never invents a digest for it silently.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Evidence {
    /// Current location, relative to the records directory.
    pub path: String,
    /// sha256 of the decoded content.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sha256: Option<String>,
    /// The current location's interpretation (absent: raw). Only beside a
    /// digest.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub encoding: Option<Encoding>,
    /// Set when the digest was established after the observation was
    /// recorded.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub attested: Option<Attested>,
    /// Former locations, oldest first.
    #[serde(default, rename = "moved", skip_serializing_if = "Vec::is_empty")]
    pub moves: Vec<Move>,
}

/// The on-disk forms of an entry: the legacy bare path string, or the table.
pub(super) fn deserialize_entries<'de, D>(d: D) -> Result<Vec<Evidence>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    Vec::<toml::Value>::deserialize(d)?
        .into_iter()
        .map(|v| match v {
            toml::Value::String(path) => Ok(Evidence::legacy(path)),
            other => Evidence::deserialize(other)
                .map_err(|e| serde::de::Error::custom(format!("evidence: {e}"))),
        })
        .collect()
}

impl Evidence {
    fn legacy(path: String) -> Evidence {
        Evidence {
            path,
            sha256: None,
            encoding: None,
            attested: None,
            moves: Vec::new(),
        }
    }

    /// Every location this entry has had, oldest first, each with the
    /// interpretation it was read under (`None` when the entry is unhashed).
    fn locations(&self) -> impl Iterator<Item = (&str, Option<Encoding>)> {
        self.moves
            .iter()
            .map(|m| (m.from.as_str(), m.encoding))
            .chain(std::iter::once((self.path.as_str(), self.encoding)))
    }

    /// The interpretation recorded for `location`, if it is one of this
    /// entry's.
    fn interpretation_at(&self, location: &str) -> Option<Option<Encoding>> {
        self.locations()
            .find(|(path, _)| *path == location)
            .map(|(_, e)| e)
    }
}

/// An evidence file for `observe`, as the caller named it. `raw` hashes it
/// as it is, never decoding, for an artifact that merely starts like a
/// compressed stream.
#[derive(Debug, Clone)]
pub struct NewEvidence {
    pub path: PathBuf,
    pub raw: bool,
}

// --- locations --------------------------------------------------------------

/// `path` made absolute against the working directory and normalized
/// lexically: `.` dropped, `..` folded into the preceding component. The
/// filesystem is never consulted, so the spelling hashed is the spelling
/// stored.
pub(super) fn absolute(path: &Path) -> Result<PathBuf> {
    let joined = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()
            .context("reading the working directory")?
            .join(path)
    };
    let mut out = PathBuf::new();
    for c in joined.components() {
        match c {
            Component::CurDir => {}
            Component::ParentDir => {
                if !matches!(out.components().next_back(), Some(Component::Normal(_))) {
                    bail!("{} climbs above the filesystem root", path.display());
                }
                out.pop();
            }
            other => out.push(other.as_os_str()),
        }
    }
    Ok(out)
}

/// Where a caller's `path` is stored: relative to the records directory,
/// `/`-separated, UTF-8.
pub(super) fn stored_location(root: &Path, path: &Path) -> Result<String> {
    let (base, target) = (absolute(root)?, absolute(path)?);
    for p in [&base, &target] {
        if p.to_str().is_none() {
            bail!(
                "{} is not UTF-8: evidence locations are stored as text",
                p.display()
            );
        }
    }
    Ok(relative_path(&base, &target))
}

/// The file a stored location names: opened by its lexical spelling, never
/// handing `..` to the kernel.
fn located(root: &Path, location: &str) -> Result<PathBuf> {
    absolute(&root.join(location))
}

/// Why a stored location is malformed, if it is: it must be the form
/// `stored_location` writes (relative, `/`-separated, leading `..` only, no
/// `.` or empty components).
fn location_shape(location: &str) -> Option<String> {
    if location.trim().is_empty() {
        return Some("an empty evidence path".to_string());
    }
    if location.starts_with('/') || Path::new(location).is_absolute() {
        return Some(format!(
            "evidence `{location}` is absolute: locations are relative to the records directory"
        ));
    }
    let mut climbing = true;
    for part in location.split('/') {
        match part {
            "" | "." => {
                return Some(format!(
                    "evidence `{location}` has an empty or `.` component"
                ));
            }
            ".." if !climbing => {
                return Some(format!(
                    "evidence `{location}` has `..` after a directory name"
                ));
            }
            ".." => {}
            _ => climbing = false,
        }
    }
    if climbing {
        return Some(format!("evidence `{location}` names no file"));
    }
    None
}

// --- reading ----------------------------------------------------------------

/// How to interpret a file: by its leading bytes, or as recorded.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Reading {
    Sniff,
    As(Option<Encoding>),
}

/// Why an evidence file could not be read for its digest.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Fault {
    Missing,
    Unreadable(String),
    /// A Git LFS pointer: the object was never fetched (or a pointer was
    /// captured in its place).
    Pointer,
    /// It does not decode under its interpretation.
    Undecodable {
        encoding: Encoding,
        why: String,
    },
    /// The file changed while it was read.
    Unstable,
}

impl fmt::Display for Fault {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Fault::Missing => f.write_str("missing"),
            Fault::Unreadable(why) => write!(f, "unreadable: {why}"),
            Fault::Pointer => f.write_str("a Git LFS pointer: the object is not fetched"),
            Fault::Undecodable { encoding, why } => {
                write!(f, "does not decode as {}: {why}", encoding.as_str())
            }
            Fault::Unstable => f.write_str("changed while being read"),
        }
    }
}

/// The digest of one file under one interpretation.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Hashed {
    sha256: String,
    encoding: Option<Encoding>,
}

/// What identifies a file's state for change detection: device, inode, size
/// and modification time. Detection, not a lock: an in-place rewrite that
/// restores all four goes unseen.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Fingerprint {
    dev: u64,
    ino: u64,
    len: u64,
    modified: Option<std::time::SystemTime>,
}

fn fingerprint(meta: &std::fs::Metadata) -> Fingerprint {
    #[cfg(unix)]
    let (dev, ino) = {
        use std::os::unix::fs::MetadataExt as _;
        (meta.dev(), meta.ino())
    };
    #[cfg(not(unix))]
    let (dev, ino) = (0, 0);
    Fingerprint {
        dev,
        ino,
        len: meta.len(),
        modified: meta.modified().ok(),
    }
}

/// gzip, zstd, or a zstd skippable frame (which may lead a zstd stream).
fn sniff(head: &[u8]) -> Option<Encoding> {
    match head {
        [0x1f, 0x8b, ..] => Some(Encoding::Gzip),
        [0x28, 0xb5, 0x2f, 0xfd, ..] => Some(Encoding::Zstd),
        [b0, 0x2a, 0x4d, 0x18, ..] if b0 & 0xf0 == 0x50 => Some(Encoding::Zstd),
        _ => None,
    }
}

/// A pointer file is tiny; anything longer is content.
const POINTER_MAX: usize = 1024;

/// Whether `bytes` is a Git LFS pointer: the spec version line, then an
/// `oid sha256:<64 hex>` and a `size <digits>` line.
fn is_pointer(bytes: &[u8]) -> bool {
    let Ok(text) = std::str::from_utf8(bytes) else {
        return false;
    };
    let mut lines = text.lines();
    if lines.next() != Some("version https://git-lfs.github.com/spec/v1") {
        return false;
    }
    let rest: Vec<&str> = lines.collect();
    let oid = rest.iter().any(|l| {
        l.strip_prefix("oid sha256:")
            .is_some_and(|h| h.len() == 64 && h.bytes().all(|b| b.is_ascii_hexdigit()))
    });
    let size = rest.iter().any(|l| {
        l.strip_prefix("size ")
            .is_some_and(|n| !n.is_empty() && n.bytes().all(|b| b.is_ascii_digit()))
    });
    oid && size
}

/// Hashes everything written to it, keeping the first `POINTER_MAX + 1`
/// bytes for the pointer check.
struct Sink {
    hasher: Sha256,
    head: Vec<u8>,
    total: u64,
}

impl std::io::Write for Sink {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.hasher.update(buf);
        let room = (POINTER_MAX + 1).saturating_sub(self.head.len());
        self.head.extend_from_slice(&buf[..room.min(buf.len())]);
        self.total += buf.len() as u64;
        Ok(buf.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

/// Read `path` under `reading` through one handle and hash its decoded
/// content. Refuses a pointer (raw or decoded), a stream that does not decode
/// completely, a compressed stream that decodes to nothing, and a file whose
/// fingerprint changed during the read.
fn hash_file(path: &Path, reading: Reading) -> Result<(Hashed, Fingerprint), Fault> {
    use std::io::ErrorKind;
    let unreadable = |e: std::io::Error| match e.kind() {
        ErrorKind::NotFound => Fault::Missing,
        _ => Fault::Unreadable(e.to_string()),
    };
    let before = std::fs::metadata(path).map_err(unreadable)?;
    if !before.is_file() {
        return Err(Fault::Unreadable("not a regular file".to_string()));
    }
    let file = std::fs::File::open(path).map_err(unreadable)?;
    let opened = fingerprint(&file.metadata().map_err(unreadable)?);
    let mut head = Vec::with_capacity(4);
    (&file).take(4).read_to_end(&mut head).map_err(unreadable)?;
    let encoding = match reading {
        Reading::Sniff => sniff(&head),
        Reading::As(e) => e,
    };
    let reader = std::io::Cursor::new(head).chain(&file);
    let mut sink = Sink {
        hasher: Sha256::new(),
        head: Vec::new(),
        total: 0,
    };
    // A raw pointer is caught by the same check as a decoded one: raw, the
    // sink sees the file's own bytes.
    let copied = match encoding {
        None => std::io::copy(&mut { reader }, &mut sink).map_err(unreadable),
        Some(Encoding::Gzip) => {
            std::io::copy(&mut flate2::read::MultiGzDecoder::new(reader), &mut sink).map_err(|e| {
                Fault::Undecodable {
                    encoding: Encoding::Gzip,
                    why: e.to_string(),
                }
            })
        }
        Some(Encoding::Zstd) => zstd::stream::read::Decoder::new(reader)
            .and_then(|mut d| std::io::copy(&mut d, &mut sink))
            .map_err(|e| Fault::Undecodable {
                encoding: Encoding::Zstd,
                why: e.to_string(),
            }),
    };
    copied?;
    if let Some(encoding) = encoding
        && sink.total == 0
    {
        return Err(Fault::Undecodable {
            encoding,
            why: "the stream decodes to nothing".to_string(),
        });
    }
    if sink.head.len() <= POINTER_MAX && is_pointer(&sink.head) {
        return Err(Fault::Pointer);
    }
    let read = fingerprint(&file.metadata().map_err(unreadable)?);
    let now = std::fs::metadata(path).map(|m| fingerprint(&m));
    if opened != fingerprint(&before) || read != opened || now.ok().as_ref() != Some(&opened) {
        return Err(Fault::Unstable);
    }
    let sha256 = sink
        .hasher
        .finalize()
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect();
    Ok((Hashed { sha256, encoding }, opened))
}

/// Reads evidence for one operation, hashing each (file, interpretation)
/// once. A memoized digest is reused only while the file's fingerprint
/// (device, inode, size, mtime) is unchanged, so the key's path stands for
/// the file version it was read at. Every successful read's fingerprint is
/// kept for the whole operation, and `unchanged` re-checks all of them
/// before a write is published: a file that changed between two reads of
/// one operation fails it even when the later read succeeded.
#[derive(Debug, Default)]
pub struct Inspector {
    memo: HashMap<(PathBuf, Reading), (Fingerprint, Result<Hashed, Fault>)>,
    reads: Vec<(PathBuf, Fingerprint)>,
}

impl Inspector {
    pub fn new() -> Inspector {
        Inspector::default()
    }

    fn hash(&mut self, path: &Path, reading: Reading) -> Result<Hashed, Fault> {
        let key = (path.to_path_buf(), reading);
        if let Some((fp, result)) = self.memo.get(&key)
            && std::fs::metadata(path).is_ok_and(|m| fingerprint(&m) == *fp)
        {
            return result.clone();
        }
        let result = hash_file(path, reading);
        match &result {
            Ok((h, fp)) => {
                self.reads.push((path.to_path_buf(), fp.clone()));
                self.memo.insert(key, (fp.clone(), Ok(h.clone())));
            }
            // Only a fault on a stable file is worth remembering: a file
            // that changed while read may be complete by the next read.
            Err(Fault::Unstable) => {}
            Err(fault) => {
                if let Ok(m) = std::fs::metadata(path) {
                    self.memo.insert(key, (fingerprint(&m), Err(fault.clone())));
                }
            }
        }
        result.map(|(h, _)| h)
    }

    /// The files this operation read successfully that no longer carry a
    /// fingerprint they were read with.
    fn changed_since_read(&self) -> Vec<PathBuf> {
        let mut out: BTreeSet<PathBuf> = BTreeSet::new();
        for (path, fp) in &self.reads {
            if !std::fs::metadata(path).is_ok_and(|m| fingerprint(&m) == *fp) {
                out.insert(path.clone());
            }
        }
        out.into_iter().collect()
    }

    /// Fails when any file read by this operation changed since: what was
    /// verified is not what is about to be recorded.
    pub(super) fn unchanged(&self) -> Result<()> {
        let changed = self.changed_since_read();
        if !changed.is_empty() {
            bail!(
                "refusing to write: evidence changed after it was verified:\n  {}",
                changed
                    .iter()
                    .map(|p| p.display().to_string())
                    .collect::<Vec<_>>()
                    .join("\n  ")
            );
        }
        Ok(())
    }

    /// The state of entry `e` of a question under `root`: whether its file
    /// is available under its interpretation, and how it compares with the
    /// recorded digest. An unhashed entry is still read (by its leading
    /// bytes), so a missing or unfetched file is reported for it too.
    pub fn inspect(&mut self, root: &Path, e: &Evidence) -> Inspection {
        let path = match located(root, &e.path) {
            Ok(p) => p,
            Err(err) => {
                return Inspection {
                    fault: Some(Fault::Unreadable(err.to_string())),
                    comparison: Comparison::Unknown,
                };
            }
        };
        match &e.sha256 {
            Some(recorded) => match self.hash(&path, Reading::As(e.encoding)) {
                Ok(h) if h.sha256 == *recorded => Inspection {
                    fault: None,
                    comparison: Comparison::Matches,
                },
                Ok(h) => Inspection {
                    fault: None,
                    comparison: Comparison::Changed { now: h.sha256 },
                },
                Err(fault) => Inspection {
                    fault: Some(fault),
                    comparison: Comparison::Unknown,
                },
            },
            None => Inspection {
                fault: self.hash(&path, Reading::Sniff).err(),
                comparison: Comparison::Unhashed,
            },
        }
    }
}

/// How an entry's file compares with its recorded digest.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Comparison {
    Matches,
    /// It decodes to other content: slimmed, rewritten or replaced.
    Changed {
        now: String,
    },
    /// No digest was ever recorded.
    Unhashed,
    /// The file could not be read (see the fault).
    Unknown,
}

/// One entry's state, in two dimensions: availability, and the digest
/// comparison when it could be made.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Inspection {
    pub fault: Option<Fault>,
    pub comparison: Comparison,
}

impl Inspection {
    /// Whether the entry fails the content audit: unavailable, or changed.
    /// An unhashed entry that reads fails nothing: there is no baseline.
    pub fn fails(&self) -> bool {
        self.fault.is_some() || matches!(self.comparison, Comparison::Changed { .. })
    }
}

impl fmt::Display for Inspection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let comparison = match &self.comparison {
            Comparison::Matches => "matches its digest".to_string(),
            Comparison::Changed { now } => format!(
                "CHANGED since its digest was recorded (decodes to {})",
                short(now)
            ),
            Comparison::Unhashed => "no digest recorded".to_string(),
            Comparison::Unknown => String::new(),
        };
        match (&self.fault, comparison.is_empty()) {
            (Some(fault), true) => write!(f, "{fault}"),
            (Some(fault), false) => write!(f, "{fault}; {comparison}"),
            (None, _) => f.write_str(&comparison),
        }
    }
}

/// The first 12 hex digits of a digest, for display.
fn short(sha: &str) -> &str {
    &sha[..sha.len().min(12)]
}

// --- capture (observe) ------------------------------------------------------

/// The entries `observe` records for `given`: each file read now, its
/// interpretation sniffed (or raw on request) and its decoded digest kept.
pub(super) fn capture(
    root: &Path,
    given: &[NewEvidence],
    inspector: &mut Inspector,
) -> Result<Vec<Evidence>> {
    let mut out: Vec<Evidence> = Vec::new();
    for g in given {
        let location = stored_location(root, &g.path)?;
        if out.iter().any(|e| e.path == location) {
            bail!("evidence `{location}` is given twice");
        }
        let reading = if g.raw {
            Reading::As(None)
        } else {
            Reading::Sniff
        };
        let h = inspector
            .hash(&located(root, &location)?, reading)
            .map_err(|fault| anyhow::anyhow!("evidence {}: {fault}", g.path.display()))?;
        out.push(Evidence {
            path: location,
            sha256: Some(h.sha256),
            encoding: h.encoding,
            attested: None,
            moves: Vec::new(),
        });
    }
    Ok(out)
}

/// For a keyed `observe` rerun: whether `given` is the evidence of `stored`.
/// Each given path must be a location (current or former) of exactly one
/// stored entry and every entry must be named once. Where the named file
/// exists, a hashed entry's digest must match under the interpretation
/// recorded for that location; a missing file is accepted by its location
/// (the landing script may predate a move). Returns the differences (empty:
/// a match) and, separately, notes for former locations given.
pub(super) fn rerun_differences(
    root: &Path,
    stored: &[Evidence],
    given: &[NewEvidence],
    inspector: &mut Inspector,
) -> Result<(Vec<String>, Vec<String>)> {
    let mut diffs = Vec::new();
    let mut notes = Vec::new();
    let mut named: BTreeMap<usize, String> = BTreeMap::new();
    for g in given {
        let location = stored_location(root, &g.path)?;
        let Some((i, entry)) = stored
            .iter()
            .enumerate()
            .find(|(_, e)| e.interpretation_at(&location).is_some())
        else {
            diffs.push(format!(
                "`evidence`: `{location}` is not a location of any stored evidence"
            ));
            continue;
        };
        if let Some(first) = named.insert(i, location.clone()) {
            diffs.push(format!(
                "`evidence`: `{first}` and `{location}` both name stored `{}`",
                entry.path
            ));
            continue;
        }
        if location != entry.path {
            notes.push(format!(
                "the given evidence `{location}` has moved to `{}`; the stored location stands. \
                 Update the script that emits it.",
                entry.path
            ));
        }
        let path = located(root, &location)?;
        match (&entry.sha256, entry.interpretation_at(&location).flatten()) {
            (Some(_), Some(encoding)) if g.raw => diffs.push(format!(
                "`evidence`: `{location}` is given raw but was recorded as {}",
                encoding.as_str()
            )),
            (Some(recorded), encoding) => match inspector.hash(&path, Reading::As(encoding)) {
                Ok(h) if h.sha256 == *recorded => {}
                Ok(h) => diffs.push(format!(
                    "`evidence`: `{location}` decodes to {}, not the stored {}",
                    short(&h.sha256),
                    short(recorded)
                )),
                // An unfetched LFS object is absent content, as on a fresh
                // clone: the location alone names the entry.
                Err(Fault::Missing | Fault::Pointer) => {}
                Err(fault) => diffs.push(format!("`evidence`: `{location}`: {fault}")),
            },
            // No digest to compare: matched by location only, and never
            // stamped by a rerun. A present file must still open: only
            // absence is exempt.
            (None, _) => match std::fs::File::open(&path).and_then(|f| f.metadata()) {
                Ok(m) if m.is_file() => {}
                Ok(_) => diffs.push(format!("`evidence`: `{location}` is not a regular file")),
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
                Err(e) => diffs.push(format!("`evidence`: `{location}`: {e}")),
            },
        }
    }
    for (i, e) in stored.iter().enumerate() {
        if !named.contains_key(&i) {
            diffs.push(format!("`evidence`: stored `{}` is not given", e.path));
        }
    }
    Ok((diffs, notes))
}

// --- validation -------------------------------------------------------------

/// Errors in one observation's evidence: shape, provenance, and the
/// location invariant (a location belongs to one entry, and an entry
/// revisiting a location reads it the same way). Whether the files exist is
/// not checked: availability is `audit`'s.
pub(super) fn validate(entries: &[Evidence], utc: impl Fn(&Datetime) -> bool) -> Vec<String> {
    let mut errs = Vec::new();
    let mut owner: BTreeMap<&str, usize> = BTreeMap::new();
    for (i, e) in entries.iter().enumerate() {
        let label = format!("evidence `{}`", e.path);
        for (location, _) in e.locations() {
            if let Some(msg) = location_shape(location) {
                errs.push(msg);
            }
            match owner.get(location) {
                Some(j) if *j != i => errs.push(format!(
                    "evidence location `{location}` belongs to two entries"
                )),
                _ => {
                    owner.insert(location, i);
                }
            }
        }
        let mut read_as: BTreeMap<&str, Option<Encoding>> = BTreeMap::new();
        for (location, encoding) in e.locations() {
            if read_as
                .insert(location, encoding)
                .is_some_and(|x| x != encoding)
            {
                errs.push(format!(
                    "{label}: location `{location}` recorded under two interpretations"
                ));
            }
        }
        let locations: Vec<&str> = e.locations().map(|(l, _)| l).collect();
        if locations.windows(2).any(|w| w[0] == w[1]) {
            errs.push(format!("{label}: a move that stays in place"));
        }
        match &e.sha256 {
            Some(sha) if !super::validate::is_sha256(sha) => {
                errs.push(format!(
                    "{label}: `sha256` `{sha}` is not a lowercase sha256"
                ));
            }
            Some(_) => {}
            None => {
                if e.encoding.is_some() || e.attested.is_some() || !e.moves.is_empty() {
                    errs.push(format!(
                        "{label}: `encoding`, `attested` and `moved` need a `sha256`"
                    ));
                }
            }
        }
        if let Some(a) = &e.attested {
            if a.reason.trim().is_empty() {
                errs.push(format!("{label}: `attested.reason` is empty"));
            }
            if !utc(&a.at) {
                errs.push(format!("{label}: `attested.at` must be a UTC timestamp"));
            }
        }
        for m in &e.moves {
            if m.reason.trim().is_empty() {
                errs.push(format!(
                    "{label}: a move from `{}` has an empty reason",
                    m.from
                ));
            }
            if !utc(&m.at) {
                errs.push(format!(
                    "{label}: the move from `{}` needs a UTC timestamp",
                    m.from
                ));
            }
        }
    }
    errs
}

// --- store-wide maintenance -------------------------------------------------

/// Where an entry sits in a store: question index, observation index, entry
/// index.
type Slot = (usize, usize, usize);

fn slots(questions: &[Question], keep: impl Fn(&Evidence) -> bool) -> Vec<Slot> {
    let mut out = Vec::new();
    for (qi, q) in questions.iter().enumerate() {
        for (oi, o) in q.observations.iter().enumerate() {
            for (ei, e) in o.evidence.iter().enumerate() {
                if keep(e) {
                    out.push((qi, oi, ei));
                }
            }
        }
    }
    out
}

fn label(q: &Question, oi: usize) -> String {
    format!("question {} observation #{}", q.id, oi + 1)
}

/// Validate every changed question against the store as it will be, check
/// that nothing read has changed since, then write them in id order. Each
/// file is replaced atomically; a crash between files leaves a valid store
/// with only some entries changed, which a rerun completes.
fn publish(
    store: &Store,
    questions: &[Question],
    changed: &BTreeSet<usize>,
    inspector: &Inspector,
) -> Result<()> {
    let changed: Vec<Question> = changed.iter().map(|i| questions[*i].clone()).collect();
    check_many(store, &changed)?;
    inspector.unchanged()?;
    for q in &changed {
        write_question(q)?;
    }
    Ok(())
}

/// What `relocate` did.
#[derive(Debug, Clone)]
pub struct Relocated {
    /// Entries whose current location was OLD and now is NEW.
    pub entries: usize,
    pub questions: usize,
    /// Of those, entries that had no digest and gained an attested one.
    pub attested: usize,
}

/// Move every evidence entry in `root` whose current location is `old` to
/// `new`, void observations and retired questions included (dispositions
/// never change). `new` must decode to each entry's digest; an entry without
/// one is compared with `old` as it reads now (`old_raw`: read it raw) and
/// gains that digest, marked attested by this move. All entries are checked
/// before any record is written. Evidence files are never deleted: once this
/// succeeds the caller may remove `old`, if nothing outside this store needs
/// it. Rerunning after an interruption moves the remaining entries.
pub fn relocate(
    root: &Path,
    old: &Path,
    new: &Path,
    old_raw: bool,
    new_raw: bool,
    reason: &str,
) -> Result<Relocated> {
    let root = &absolute(root)?;
    let _lock = lock(root)?;
    let store = load_unlocked(root)?;
    let reason = reason.trim().to_string();
    if reason.is_empty() {
        bail!("a move needs a --reason");
    }
    let (old_at, new_at) = (stored_location(root, old)?, stored_location(root, new)?);
    if old_at == new_at {
        bail!("`{old_at}` and `{new_at}` are the same location");
    }
    let mut questions: Vec<Question> = store.questions().to_vec();
    let targets = slots(&questions, |e| e.path == old_at);
    if targets.is_empty() {
        // Nothing left at OLD is a finished move only if something left it;
        // otherwise OLD is most likely mistyped.
        if slots(&questions, |e| e.moves.iter().any(|m| m.from == old_at)).is_empty() {
            bail!("no evidence entry is or was at `{old_at}`");
        }
        return Ok(Relocated {
            entries: 0,
            questions: 0,
            attested: 0,
        });
    }
    let entry = |s: &Slot| &questions[s.0].observations[s.1].evidence[s.2];
    let legacy = targets.iter().any(|s| entry(s).sha256.is_none());
    if old_raw && !legacy {
        bail!(
            "--old-raw applies to entries without a digest, and every entry at `{old_at}` has \
             one, read as recorded"
        );
    }
    let mut inspector = Inspector::new();
    let new_reading = if new_raw {
        Reading::As(None)
    } else {
        Reading::Sniff
    };
    let moved_to = inspector
        .hash(&located(root, &new_at)?, new_reading)
        .map_err(|fault| anyhow::anyhow!("{}: {fault}", new.display()))?;
    let baseline = if legacy {
        let reading = if old_raw {
            Reading::As(None)
        } else {
            Reading::Sniff
        };
        Some(
            inspector
                .hash(&located(root, &old_at)?, reading)
                .map_err(|fault| {
                    anyhow::anyhow!(
                        "{}: {fault} (an entry without a digest is compared with it)",
                        old.display()
                    )
                })?,
        )
    } else {
        None
    };
    let mut problems = Vec::new();
    for s in &targets {
        let e = entry(s);
        let o = &questions[s.0].observations[s.1];
        let at = label(&questions[s.0], s.1);
        let expected = e
            .sha256
            .as_deref()
            .or(baseline.as_ref().map(|b| b.sha256.as_str()));
        if expected != Some(moved_to.sha256.as_str()) {
            problems.push(format!(
                "{at}: `{new_at}` decodes to {}, but the entry's content is {}",
                short(&moved_to.sha256),
                expected.map_or("unknown", short)
            ));
        }
        for (j, other) in o.evidence.iter().enumerate() {
            if j != s.2 && other.interpretation_at(&new_at).is_some() {
                problems.push(format!(
                    "{at}: `{new_at}` is a location of its other evidence `{}`",
                    other.path
                ));
            }
        }
        if let Some(was) = e.interpretation_at(&new_at)
            && was != moved_to.encoding
        {
            problems.push(format!(
                "{at}: the entry was at `{new_at}` before, read {}; it now reads {}",
                interpretation(was),
                interpretation(moved_to.encoding)
            ));
        }
    }
    if !problems.is_empty() {
        bail!("refusing to move:\n  {}", problems.join("\n  "));
    }
    let at = now_utc()?;
    let mut changed = BTreeSet::new();
    let mut attested = 0;
    for s in &targets {
        let e = &mut questions[s.0].observations[s.1].evidence[s.2];
        let from_encoding = match &baseline {
            Some(b) if e.sha256.is_none() => {
                e.sha256 = Some(b.sha256.clone());
                e.attested = Some(Attested {
                    at,
                    reason: reason.clone(),
                });
                attested += 1;
                b.encoding
            }
            _ => e.encoding,
        };
        e.moves.push(Move {
            at,
            from: old_at.clone(),
            encoding: from_encoding,
            reason: reason.clone(),
        });
        e.path.clone_from(&new_at);
        e.encoding = moved_to.encoding;
        changed.insert(s.0);
    }
    publish(&store, &questions, &changed, &inspector)?;
    Ok(Relocated {
        entries: targets.len(),
        questions: changed.len(),
        attested,
    })
}

fn interpretation(e: Option<Encoding>) -> &'static str {
    e.map_or("raw", Encoding::as_str)
}

/// What `attest` did.
#[derive(Debug, Clone)]
pub struct Attestation {
    pub entries: usize,
    pub files: usize,
    pub questions: usize,
}

/// Establish digests for evidence entries that have none: those at the
/// given locations, or every one in the store when none is given. Each file
/// is read now (`raw`: as it is, never decoded) and the digest is marked
/// attested with `reason`: it identifies the content from now on, not the
/// capture. A given location must hold some entry; entries there that
/// already have a digest are verified against it and skipped, so a rerun
/// after an interruption attests the remainder. Every file is read before
/// any record is written; one that cannot be read fails the whole run.
pub fn attest(root: &Path, paths: &[PathBuf], raw: bool, reason: &str) -> Result<Attestation> {
    let root = &absolute(root)?;
    let _lock = lock(root)?;
    let store = load_unlocked(root)?;
    let reason = reason.trim().to_string();
    if reason.is_empty() {
        bail!("an attestation needs a --reason");
    }
    let mut questions: Vec<Question> = store.questions().to_vec();
    let mut inspector = Inspector::new();
    let mut problems = Vec::new();
    // The locations attested: the given ones, or every one holding an entry
    // without a digest.
    let mut locations = BTreeSet::new();
    if paths.is_empty() {
        for s in slots(&questions, |e| e.sha256.is_none()) {
            locations.insert(questions[s.0].observations[s.1].evidence[s.2].path.clone());
        }
    } else {
        for p in paths {
            locations.insert(stored_location(root, p)?);
        }
    }
    // Entries already carrying a digest at those locations vouch for what
    // the file must still be: a file that changed since is not attested,
    // however the locations were selected.
    for location in &locations {
        let here = slots(&questions, |e| e.path == *location);
        if here.is_empty() {
            problems.push(format!("no evidence entry is at `{location}`"));
        }
        for s in &here {
            let e = &questions[s.0].observations[s.1].evidence[s.2];
            let Some(recorded) = &e.sha256 else { continue };
            match inspector.hash(&located(root, location)?, Reading::As(e.encoding)) {
                Ok(h) if h.sha256 == *recorded => {}
                Ok(h) => problems.push(format!(
                    "{}: `{location}` decodes to {}, not its digest {}",
                    label(&questions[s.0], s.1),
                    short(&h.sha256),
                    short(recorded)
                )),
                Err(fault) => problems.push(format!(
                    "{}: `{location}`: {fault}",
                    label(&questions[s.0], s.1)
                )),
            }
        }
    }
    let targets = slots(&questions, |e| {
        e.sha256.is_none() && locations.contains(&e.path)
    });
    let reading = if raw {
        Reading::As(None)
    } else {
        Reading::Sniff
    };
    let mut digests: BTreeMap<String, Hashed> = BTreeMap::new();
    let mut faulted: BTreeSet<String> = BTreeSet::new();
    for s in &targets {
        let location = questions[s.0].observations[s.1].evidence[s.2].path.clone();
        if digests.contains_key(&location) || faulted.contains(&location) {
            continue;
        }
        match inspector.hash(&located(root, &location)?, reading) {
            Ok(h) => {
                digests.insert(location, h);
            }
            Err(fault) => {
                // Named once per file, by the first entry holding it.
                problems.push(format!(
                    "{}: `{location}`: {fault}",
                    label(&questions[s.0], s.1)
                ));
                faulted.insert(location);
            }
        }
    }
    if !problems.is_empty() {
        bail!("refusing to attest:\n  {}", problems.join("\n  "));
    }
    if targets.is_empty() {
        inspector.unchanged()?;
        return Ok(Attestation {
            entries: 0,
            files: 0,
            questions: 0,
        });
    }
    let at = now_utc()?;
    let mut changed = BTreeSet::new();
    for s in &targets {
        let e = &mut questions[s.0].observations[s.1].evidence[s.2];
        let h = &digests[&e.path];
        e.sha256 = Some(h.sha256.clone());
        e.encoding = h.encoding;
        e.attested = Some(Attested {
            at,
            reason: reason.clone(),
        });
        changed.insert(s.0);
    }
    publish(&store, &questions, &changed, &inspector)?;
    Ok(Attestation {
        entries: targets.len(),
        files: digests.len(),
        questions: changed.len(),
    })
}

/// One audited entry that fails or has no digest.
#[derive(Debug, Clone)]
pub struct AuditRow {
    pub id: String,
    /// The observation's `#N`.
    pub observation: usize,
    pub path: String,
    pub inspection: Inspection,
}

/// The content audit over a store.
#[derive(Debug, Clone, Default)]
pub struct Audit {
    pub entries: usize,
    pub matching: usize,
    /// Entries that fail: unavailable or changed.
    pub failing: Vec<AuditRow>,
    /// Entries with no digest (they fail only when unavailable).
    pub unhashed: Vec<AuditRow>,
}

/// Inspect every evidence entry of the questions `ids` (all when empty), void
/// observations and retired questions included: their files are still the
/// record of what was observed.
pub fn audit(store: &Store, ids: &[String]) -> Result<Audit> {
    let mut chosen: Vec<&Question> = Vec::new();
    for id in ids {
        match store.get(id) {
            Some(q) => chosen.push(q),
            None => bail!("no question `{id}` under the given --records"),
        }
    }
    if ids.is_empty() {
        chosen = store.questions().iter().collect();
    }
    let mut inspector = Inspector::new();
    let mut out = Audit::default();
    for q in chosen {
        for (oi, o) in q.observations.iter().enumerate() {
            for e in &o.evidence {
                out.entries += 1;
                let inspection = inspector.inspect(&q.root, e);
                let row = AuditRow {
                    id: q.id.clone(),
                    observation: oi + 1,
                    path: e.path.clone(),
                    inspection,
                };
                if row.inspection.fails() {
                    out.failing.push(row);
                } else if row.inspection.comparison == Comparison::Unhashed {
                    out.unhashed.push(row);
                } else {
                    out.matching += 1;
                }
            }
        }
    }
    // A file that changed after it was found matching must not be reported
    // as verified.
    let changed = inspector.changed_since_read();
    if !changed.is_empty() {
        bail!(
            "evidence changed during the audit:\n  {}",
            changed
                .iter()
                .map(|p| p.display().to_string())
                .collect::<Vec<_>>()
                .join("\n  ")
        );
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lexical_absolute_folds_without_the_filesystem() {
        assert_eq!(
            absolute(Path::new("/a/b/./c/../d")).expect("ok"),
            PathBuf::from("/a/b/d")
        );
        assert!(absolute(Path::new("/../x")).is_err());
    }

    #[test]
    fn location_shapes() {
        for good in ["x.json", "../fieldwork/run26.json", "../../a/b.json.zst"] {
            assert_eq!(location_shape(good), None, "{good}");
        }
        for bad in ["", "/abs.json", "a/../b", "./a", "a//b", "..", "a/"] {
            assert!(location_shape(bad).is_some(), "{bad}");
        }
    }

    #[test]
    fn pointers_are_recognized_by_structure() {
        let pointer = "version https://git-lfs.github.com/spec/v1\noid sha256:\
            4d7a214614ab2935c943f9e0ff69d22eadbb8f32b1258daaa5e2ca24d17e2393\nsize 12345\n";
        assert!(is_pointer(pointer.as_bytes()));
        assert!(!is_pointer(
            b"version https://git-lfs.github.com/spec/v1\nhello\n"
        ));
        assert!(!is_pointer(b"{\"json\": true}"));
    }

    #[test]
    fn sniffing() {
        assert_eq!(sniff(&[0x1f, 0x8b, 8, 0]), Some(Encoding::Gzip));
        assert_eq!(sniff(&[0x28, 0xb5, 0x2f, 0xfd]), Some(Encoding::Zstd));
        assert_eq!(sniff(&[0x5e, 0x2a, 0x4d, 0x18]), Some(Encoding::Zstd));
        assert_eq!(sniff(b"{\"a\""), None);
    }
}
