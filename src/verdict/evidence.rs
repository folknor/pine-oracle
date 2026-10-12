// Evidence: the artifacts (chart exports, probe JSON) an observation points
// at. Unlike fixtures they are not copied into the store: each entry names a
// file elsewhere in the caller's repository, and since those files are large
// and get compressed or relocated after the fact, an entry carries its
// identity apart from its location.
//
// Identity is the sha256 of the content. Establishing one (`observe`,
// `attest`, the baseline of an unhashed entry's move) has no digest to be
// guided by, so it reads by the leading bytes: a gzip or zstd stream is
// decoded (every member / frame, each checksum, nothing trailing), anything
// else hashed as it is, and `--raw` hashes a stream-looking artifact as it
// is. Verifying one is guided by the digest: the file's raw bytes are one
// candidate and, when they start like a stream, the decoded content another,
// and either matching is the same evidence. No interpretation is recorded:
// it would be how the bytes were once reached, never what they are, and
// following it rejected a file compressed or decompressed in place. So a
// record compressed to `record.json.zst` is the same evidence, while
// compacting its JSON is not.
// A digest computed when `observe` recorded the entry identifies the bytes
// supplied then; one established later (`attest`, or a `relocate` of an entry
// that had none) is marked `attested`: it identifies the bytes from then on,
// and says nothing about the capture before.
//
// Locations are paths relative to the verdicts directory's `evidence/`, in
// one lexical convention: a caller's path is made absolute against the
// working directory and normalized without touching the filesystem (`.`
// dropped, `..` folded), refused unless it then lies inside `evidence/`, and
// po opens exactly the spelling it stores. Within one observation a location
// (current, former, or that of an entry merged into it) belongs to one
// entry, so a keyed rerun naming any of them maps to one entry by lookup.
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
use super::validate::evidence_dir;
use super::write::{check_many, lock, now_utc, relative_path, write_question};
use super::{Question, Store};

/// A compressed stream an evidence file can be decoded from. Never stored:
/// only a legacy record's `encoding` key spells one, read and dropped.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Deserialize)]
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

/// A location an entry left: where it was, when, and why.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, from = "MoveRecord")]
pub struct Move {
    /// UTC timestamp of the move.
    pub at: Datetime,
    /// The former location, relative to the evidence directory.
    pub from: String,
    pub reason: String,
}

/// A `Move` as stored, accepting the `encoding` key records carried before
/// identity stopped recording how a location was read.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct MoveRecord {
    at: Datetime,
    from: String,
    #[serde(default, rename = "encoding")]
    _encoding: Option<Encoding>,
    reason: String,
}

impl From<MoveRecord> for Move {
    fn from(r: MoveRecord) -> Move {
        Move {
            at: r.at,
            from: r.from,
            reason: r.reason,
        }
    }
}

/// An entry merged into another of the same observation because both cite
/// the same content: kept whole, so its locations stay its survivor's and its
/// provenance is not lost.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Merge {
    /// UTC timestamp of the merge.
    pub at: Datetime,
    pub reason: String,
    pub entry: Evidence,
}

/// One evidence artifact of an observation. A legacy entry (written before
/// digests, or as a bare path string) has only `path`, and po never invents
/// a digest for it silently.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, from = "EvidenceRecord")]
pub struct Evidence {
    /// Current location, relative to the evidence directory.
    pub path: String,
    /// sha256 of the content (decoded, where it was established from a
    /// compressed stream).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sha256: Option<String>,
    /// Set when the digest was established after the observation was
    /// recorded.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub attested: Option<Attested>,
    /// Former locations, oldest first.
    #[serde(default, rename = "moved", skip_serializing_if = "Vec::is_empty")]
    pub moves: Vec<Move>,
    /// Entries merged into this one, oldest first.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub merged: Vec<Merge>,
}

/// An `Evidence` table as stored, accepting the legacy `encoding` key.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct EvidenceRecord {
    path: String,
    #[serde(default)]
    sha256: Option<String>,
    #[serde(default, rename = "encoding")]
    _encoding: Option<Encoding>,
    #[serde(default)]
    attested: Option<Attested>,
    #[serde(default, rename = "moved")]
    moves: Vec<Move>,
    #[serde(default)]
    merged: Vec<Merge>,
}

impl From<EvidenceRecord> for Evidence {
    fn from(r: EvidenceRecord) -> Evidence {
        Evidence {
            path: r.path,
            sha256: r.sha256,
            attested: r.attested,
            moves: r.moves,
            merged: r.merged,
        }
    }
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
            attested: None,
            moves: Vec::new(),
            merged: Vec::new(),
        }
    }

    /// The locations this entry itself has had, oldest first.
    fn own_locations(&self) -> impl Iterator<Item = &str> {
        self.moves
            .iter()
            .map(|m| m.from.as_str())
            .chain(std::iter::once(self.path.as_str()))
    }

    /// Every location this entry answers to: its own, then (recursively)
    /// those of the entries merged into it.
    pub fn locations(&self) -> Vec<&str> {
        let mut out: Vec<&str> = self.own_locations().collect();
        for m in &self.merged {
            out.extend(m.entry.locations());
        }
        out
    }

    fn has_location(&self, location: &str) -> bool {
        self.locations().contains(&location)
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

/// Where a caller's `path` is stored: relative to the evidence directory,
/// `/`-separated, UTF-8. A path outside the evidence directory is refused:
/// po cites only files inside the tree it owns.
pub(super) fn stored_location(root: &Path, path: &Path) -> Result<String> {
    let (base, target) = (absolute(&evidence_dir(root))?, absolute(path)?);
    for p in [&base, &target] {
        if p.to_str().is_none() {
            bail!(
                "{} is not UTF-8: evidence locations are stored as text",
                p.display()
            );
        }
    }
    if target == base || !target.starts_with(&base) {
        bail!(
            "evidence {} is outside {}: put capture evidence under the evidence directory",
            path.display(),
            base.display()
        );
    }
    Ok(relative_path(&base, &target))
}

/// The file a stored location names: opened by its lexical spelling. Every
/// read of evidence resolves its location here, so this is where links are
/// refused: neither `verdicts/`, nor `evidence/`, nor a directory below it,
/// nor the file may be a symbolic link, or a location inside the tree could
/// name bytes outside it. `hash_file` walks again after reading.
fn located(root: &Path, location: &str) -> Result<PathBuf> {
    let path = absolute(&evidence_dir(root).join(location))?;
    no_links(root, &path)?;
    Ok(path)
}

/// Why a stored location is malformed, if it is: it must be the form
/// `stored_location` writes (relative to the evidence directory,
/// `/`-separated, no `.`, `..` or empty components).
fn location_shape(location: &str) -> Option<String> {
    if location.trim().is_empty() {
        return Some("an empty evidence path".to_string());
    }
    if location.starts_with('/') || Path::new(location).is_absolute() {
        return Some(format!(
            "evidence `{location}` is absolute: locations are relative to the evidence directory"
        ));
    }
    for part in location.split('/') {
        match part {
            "" | "." => {
                return Some(format!(
                    "evidence `{location}` has an empty or `.` component"
                ));
            }
            ".." => {
                return Some(format!(
                    "evidence `{location}` climbs out of the evidence directory"
                ));
            }
            _ => {}
        }
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

/// The state of a path a read depended on: absent, or a file version.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Seen {
    Absent,
    File(Fingerprint),
}

/// What `path` is now, if that can be told. Never follows a link: a file
/// swapped for a link to an identical one is a change.
fn seen_now(path: &Path) -> Option<Seen> {
    match std::fs::symlink_metadata(path) {
        Ok(m) => Some(Seen::File(fingerprint(&m))),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Some(Seen::Absent),
        Err(_) => None,
    }
}

/// Read `path` under `reading` through one handle and hash its decoded
/// content, with the path state the result belongs to (`None`: no stable
/// state, the result must not be remembered). Refuses a pointer (raw or
/// decoded), a stream that does not decode completely, a compressed stream
/// that decodes to nothing, and a file whose fingerprint changed during the
/// read, which overrides any other outcome: a fault is only reported for a
/// file version that held still while it was found.
///
/// No link is followed: the file is examined with `symlink_metadata` and
/// opened without following a final link (Unix), and after the read the
/// path from `root` down is walked for links again, so a file or directory
/// swapped for a link during the read is caught. A link swapped in and back
/// out between those checks is not; "no links" is a store rule, not a
/// security boundary.
fn hash_file(root: &Path, path: &Path, reading: Reading) -> (Result<Hashed, Fault>, Option<Seen>) {
    use std::io::ErrorKind;
    let unreadable = |e: std::io::Error| match e.kind() {
        ErrorKind::NotFound => Fault::Missing,
        _ => Fault::Unreadable(e.to_string()),
    };
    let before = match std::fs::symlink_metadata(path) {
        Ok(m) => m,
        Err(e) if e.kind() == ErrorKind::NotFound => {
            return (Err(Fault::Missing), Some(Seen::Absent));
        }
        Err(e) => return (Err(unreadable(e)), None),
    };
    if before.file_type().is_symlink() {
        return (Err(Fault::Unreadable(link_refusal(path))), None);
    }
    if !before.is_file() {
        return (
            Err(Fault::Unreadable("not a regular file".to_string())),
            None,
        );
    }
    let file = match open_no_follow(path) {
        Ok(f) => f,
        // It was there a moment ago.
        Err(e) if e.kind() == ErrorKind::NotFound => return (Err(Fault::Unstable), None),
        Err(e) => return (Err(unreadable(e)), None),
    };
    let opened = match file.metadata() {
        Ok(m) => fingerprint(&m),
        Err(e) => return (Err(unreadable(e)), None),
    };
    let result = read_open(&file, reading);
    let read = file.metadata().map(|m| fingerprint(&m)).ok();
    let now = std::fs::symlink_metadata(path)
        .map(|m| fingerprint(&m))
        .ok();
    if opened != fingerprint(&before)
        || read.as_ref() != Some(&opened)
        || now.as_ref() != Some(&opened)
    {
        return (Err(Fault::Unstable), None);
    }
    if let Err(e) = no_links(root, path) {
        return (Err(Fault::Unreadable(e.to_string())), None);
    }
    (result, Some(Seen::File(opened)))
}

/// Open `path` for reading, refusing (Unix) to follow a link at its last
/// component.
fn open_no_follow(path: &Path) -> std::io::Result<std::fs::File> {
    let mut options = std::fs::OpenOptions::new();
    options.read(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt as _;
        options.custom_flags(libc::O_NOFOLLOW);
    }
    options.open(path)
}

fn link_refusal(at: &Path) -> String {
    format!(
        "{} is a symbolic link: evidence is never reached through a link",
        at.display()
    )
}

/// Fails when the verdicts directory `root`, or any component from it down
/// to `path`, is a symbolic link. A component that does not exist ends the
/// walk (the read then reports the file missing).
fn no_links(root: &Path, path: &Path) -> Result<()> {
    let base = absolute(root)?;
    let Ok(below) = path.strip_prefix(&base) else {
        bail!("{} is not inside {}", path.display(), base.display());
    };
    let mut at = base;
    for step in std::iter::once(None).chain(below.components().map(Some)) {
        if let Some(c) = step {
            at.push(c);
        }
        match std::fs::symlink_metadata(&at) {
            Ok(m) if m.file_type().is_symlink() => bail!(link_refusal(&at)),
            Ok(_) => {}
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => break,
            Err(e) => return Err(e).with_context(|| format!("reading {}", at.display())),
        }
    }
    Ok(())
}

/// The digest of an open file under `reading`, stability aside.
fn read_open(file: &std::fs::File, reading: Reading) -> Result<Hashed, Fault> {
    let unreadable = |e: std::io::Error| Fault::Unreadable(e.to_string());
    let mut head = Vec::with_capacity(4);
    file.take(4).read_to_end(&mut head).map_err(unreadable)?;
    let encoding = match reading {
        Reading::Sniff => sniff(&head),
        Reading::As(e) => e,
    };
    let reader = std::io::Cursor::new(head).chain(file);
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
    let sha256 = sink
        .hasher
        .finalize()
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect();
    Ok(Hashed { sha256, encoding })
}

/// Reads evidence for one operation, hashing each (file, reading) once. A
/// remembered result (digest or fault) is reused only while the path is in
/// the state it was found in (absent, or the same device, inode, size and
/// mtime), so the key's path stands for the file version it was read at.
/// Every remembered result's state is kept for the whole operation, and
/// `unchanged` re-checks all of them before a write is published: a file
/// that changed (or appeared, or vanished) between two reads of one
/// operation fails it even when the later read succeeded. A file found
/// changing while read is never remembered and fails the operation too.
#[derive(Debug, Default)]
pub struct Inspector {
    memo: HashMap<(PathBuf, Reading), (Seen, Result<Hashed, Fault>)>,
    /// Every tracked read: the verdicts root, the path, and its state then.
    reads: Vec<(PathBuf, PathBuf, Seen)>,
    unstable: BTreeSet<PathBuf>,
}

impl Inspector {
    pub fn new() -> Inspector {
        Inspector::default()
    }

    /// `path` (inside the verdicts directory `root`) read under `reading`.
    fn hash(&mut self, root: &Path, path: &Path, reading: Reading) -> Result<Hashed, Fault> {
        let key = (path.to_path_buf(), reading);
        // A directory on the way may have become a link since: the file's
        // own fingerprint would not show it.
        if let Some((seen, result)) = self.memo.get(&key)
            && seen_now(path).as_ref() == Some(seen)
            && no_links(root, path).is_ok()
        {
            return result.clone();
        }
        let (result, seen) = hash_file(root, path, reading);
        if result == Err(Fault::Unstable) {
            self.unstable.insert(path.to_path_buf());
        }
        if let Some(seen) = seen {
            self.reads
                .push((root.to_path_buf(), path.to_path_buf(), seen.clone()));
            self.memo.insert(key, (seen, result.clone()));
        }
        result
    }

    /// Compare the file at `path` with a recorded digest, guided by it: the
    /// raw bytes first, then, when they start like a stream, the decoded
    /// content. Either matching is the same evidence (a file compressed or
    /// decompressed in place, a raw artifact that merely starts like a
    /// stream), and the raw match short-circuits.
    /// A file that changed while read fails whatever matched; an absent or
    /// unreadable one fails before decoding. When nothing matches, an LFS
    /// pointer found by either reading is that fault, and otherwise every
    /// reading tried is reported.
    fn verify(&mut self, root: &Path, path: &Path, sha256: &str) -> Result<Verified, Fault> {
        let raw = self.hash(root, path, Reading::As(None));
        match &raw {
            Ok(h) if h.sha256 == sha256 => return Ok(Verified::Matches),
            Err(f @ (Fault::Unstable | Fault::Missing | Fault::Unreadable(_))) => {
                return Err(f.clone());
            }
            _ => {}
        }
        // The stream reading is chosen by a tracked read of its own (never a
        // separate peek), so the leading bytes that pick it belong to a
        // checked file version. A file that does not start like a stream
        // yields the raw reading again: nothing more to try.
        let mut pointer = raw == Err(Fault::Pointer);
        let decoded = match self.hash(root, path, Reading::Sniff) {
            Ok(h) if h.sha256 == sha256 => return Ok(Verified::Matches),
            Ok(h) => h.encoding.map(|enc| (enc, Ok(h.sha256))),
            Err(Fault::Undecodable { encoding, why }) => {
                Some((encoding, Err(Fault::Undecodable { encoding, why })))
            }
            Err(Fault::Pointer) => {
                pointer = true;
                None
            }
            Err(f) => return Err(f),
        };
        if pointer {
            return Err(Fault::Pointer);
        }
        let mut readings = vec![(None, raw.map(|h| h.sha256))];
        readings.extend(decoded.map(|(enc, result)| (Some(enc), result)));
        Ok(Verified::Changed(readings))
    }

    /// The files this operation found changing while read, or whose state
    /// differs from one a remembered result was found in.
    fn changed_since_read(&self) -> Vec<PathBuf> {
        let mut out: BTreeSet<PathBuf> = self.unstable.clone();
        for (root, path, seen) in &self.reads {
            if seen_now(path).as_ref() != Some(seen) || no_links(root, path).is_err() {
                out.insert(path.clone());
            }
        }
        out.into_iter().collect()
    }

    /// Fails when any file read by this operation changed since: what was
    /// shown is not one state of the files. For a read verb, which has
    /// already printed.
    pub fn stable(&self) -> Result<()> {
        let changed = self.changed_since_read();
        if !changed.is_empty() {
            bail!(
                "evidence changed while it was read; the states shown above are not one \
                 snapshot:\n  {}",
                changed
                    .iter()
                    .map(|p| p.display().to_string())
                    .collect::<Vec<_>>()
                    .join("\n  ")
            );
        }
        Ok(())
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
    /// is available, and how it compares with the recorded digest. Only the
    /// current location is read: former and merged locations are history.
    /// An unhashed entry is still read (by its leading bytes), so a missing
    /// or unfetched file is reported for it too.
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
            Some(recorded) => match self.verify(root, &path, recorded) {
                Ok(Verified::Matches) => Inspection {
                    fault: None,
                    comparison: Comparison::Matches,
                },
                Ok(Verified::Changed(readings)) => Inspection {
                    fault: None,
                    comparison: Comparison::Changed(readings),
                },
                Err(fault) => Inspection {
                    fault: Some(fault),
                    comparison: Comparison::Unknown,
                },
            },
            None => Inspection {
                fault: self.hash(root, &path, Reading::Sniff).err(),
                comparison: Comparison::Unhashed,
            },
        }
    }
}

/// Each reading of a file tried against a digest: raw (`None`) or decoded,
/// with the digest it yields or why it yields none.
pub type Readings = Vec<(Option<Encoding>, Result<String, Fault>)>;

/// What `Inspector::verify` found.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Verified {
    Matches,
    /// No reading yields the digest.
    Changed(Readings),
}

/// Every reading tried, for a mismatch message: `raw 1a2b..., as zstd
/// 3c4d...` or `as gzip: does not decode ...`.
fn describe(readings: &Readings) -> String {
    readings
        .iter()
        .map(|(enc, r)| {
            let how = enc.map_or("raw".to_string(), |e| format!("as {}", e.as_str()));
            match r {
                Ok(sha) => format!("{how} {}", short(sha)),
                Err(fault) => format!("{how}: {fault}"),
            }
        })
        .collect::<Vec<_>>()
        .join(", ")
}

/// How an entry's file compares with its recorded digest.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Comparison {
    Matches,
    /// No reading yields it: slimmed, rewritten or replaced.
    Changed(Readings),
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
        self.fault.is_some() || matches!(self.comparison, Comparison::Changed(_))
    }
}

impl fmt::Display for Inspection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let comparison = match &self.comparison {
            Comparison::Matches => "matches its digest".to_string(),
            Comparison::Changed(readings) => format!(
                "CHANGED since its digest was recorded (reads {})",
                describe(readings)
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

/// The entries `observe` records for `given`: each file read now, decoded by
/// its leading bytes (or raw on request), and that digest kept.
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
            .hash(root, &located(root, &location)?, reading)
            .map_err(|fault| anyhow::anyhow!("evidence {}: {fault}", g.path.display()))?;
        out.push(Evidence {
            path: location,
            sha256: Some(h.sha256),
            attested: None,
            moves: Vec::new(),
            merged: Vec::new(),
        });
    }
    Ok(out)
}

/// For a keyed `observe` rerun: whether `given` is the evidence of `stored`.
/// Each given path must be a location (current, former or merged) of exactly
/// one stored entry, given once, and every entry must be named. Distinct
/// locations of one entry may all be given: a landing script that cited the
/// same content twice still names an entry merged since. Where the named
/// file exists, a hashed entry's digest must match it (raw or decoded,
/// whatever `--evidence-raw` says: that flag chooses an identity only when
/// one is established); a missing file is accepted by its location (the
/// landing script may predate a move). Returns the differences (empty: a
/// match) and, separately, notes for former locations given.
pub(super) fn rerun_differences(
    root: &Path,
    stored: &[Evidence],
    given: &[NewEvidence],
    inspector: &mut Inspector,
) -> Result<(Vec<String>, Vec<String>)> {
    let mut diffs = Vec::new();
    let mut notes = Vec::new();
    let mut named: BTreeSet<usize> = BTreeSet::new();
    let mut seen: BTreeSet<String> = BTreeSet::new();
    for g in given {
        let location = stored_location(root, &g.path)?;
        if !seen.insert(location.clone()) {
            diffs.push(format!("`evidence`: `{location}` is given twice"));
            continue;
        }
        let Some((i, entry)) = stored
            .iter()
            .enumerate()
            .find(|(_, e)| e.has_location(&location))
        else {
            diffs.push(format!(
                "`evidence`: `{location}` is not a location of any stored evidence"
            ));
            continue;
        };
        named.insert(i);
        if location != entry.path {
            let how = if entry.own_locations().any(|l| l == location) {
                "has moved to"
            } else {
                "was merged into the entry at"
            };
            notes.push(format!(
                "the given evidence `{location}` {how} `{}`; the stored location stands. \
                 Update the script that emits it.",
                entry.path
            ));
        }
        let path = located(root, &location)?;
        match &entry.sha256 {
            Some(recorded) => match inspector.verify(root, &path, recorded) {
                // An unfetched LFS object is absent content, as on a fresh
                // clone: the location alone names the entry.
                Ok(Verified::Matches) | Err(Fault::Missing | Fault::Pointer) => {}
                Ok(Verified::Changed(readings)) => diffs.push(format!(
                    "`evidence`: `{location}` is not the stored {} (reads {})",
                    short(recorded),
                    describe(&readings)
                )),
                Err(fault) => diffs.push(format!("`evidence`: `{location}`: {fault}")),
            },
            // No digest to compare: matched by location only, and never
            // stamped by a rerun. A present file must still be readable and
            // hold still: only absence is exempt, and how it decodes is not
            // asked (there is no identity to decode it for). Read through
            // the inspector, so its state is checked again before the
            // write like every other read's.
            None => match inspector.hash(root, &path, Reading::Sniff) {
                Ok(_) | Err(Fault::Missing | Fault::Pointer | Fault::Undecodable { .. }) => {}
                Err(fault) => diffs.push(format!("`evidence`: `{location}`: {fault}")),
            },
        }
    }
    for (i, e) in stored.iter().enumerate() {
        if !named.contains(&i) {
            diffs.push(format!("`evidence`: stored `{}` is not given", e.path));
        }
    }
    Ok((diffs, notes))
}

// --- validation -------------------------------------------------------------

/// Errors in one observation's evidence: shape, provenance, and the
/// location invariant (a location, current, former or merged, belongs to
/// one entry). Whether the files exist is not checked: availability is
/// `audit`'s.
pub(super) fn validate(entries: &[Evidence], utc: impl Fn(&Datetime) -> bool) -> Vec<String> {
    let mut errs = Vec::new();
    let mut owner: BTreeMap<&str, usize> = BTreeMap::new();
    for (i, e) in entries.iter().enumerate() {
        let label = format!("evidence `{}`", e.path);
        for location in e.locations() {
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
        entry_errors(e, &label, &utc, &mut errs);
    }
    errs
}

/// One entry's own shape and provenance, then (recursively) that of each
/// entry merged into it, which must carry its digest.
fn entry_errors(
    e: &Evidence,
    label: &str,
    utc: &impl Fn(&Datetime) -> bool,
    errs: &mut Vec<String>,
) {
    let locations: Vec<&str> = e.own_locations().collect();
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
            if e.attested.is_some() || !e.moves.is_empty() || !e.merged.is_empty() {
                errs.push(format!(
                    "{label}: `attested`, `moved` and `merged` need a `sha256`"
                ));
            }
        }
    }
    for m in &e.merged {
        let merged_label = format!("{label}: merged `{}`", m.entry.path);
        if m.entry.sha256 != e.sha256 {
            errs.push(format!(
                "{merged_label} has another digest: only the same content merges"
            ));
        }
        if m.reason.trim().is_empty() {
            errs.push(format!("{merged_label} has an empty reason"));
        }
        if !utc(&m.at) {
            errs.push(format!("{merged_label} needs a UTC timestamp"));
        }
        entry_errors(&m.entry, &merged_label, utc, errs);
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
/// never change). `new` must read (raw or decoded) as each entry's digest;
/// an entry without one is compared with `old` as it reads now (`old_raw`:
/// read it raw, never decoded) and gains that digest, marked attested by
/// this move. All entries are checked before any record is written.
/// Evidence files are never deleted: once this succeeds the caller may
/// remove `old`, if nothing outside this store needs it. Rerunning after an
/// interruption moves the remaining entries.
pub fn relocate(
    root: &Path,
    old: &Path,
    new: &Path,
    old_raw: bool,
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
        // Nothing left at OLD is a finished move only if something left it
        // (itself, or an entry since merged into another); otherwise OLD is
        // most likely mistyped.
        if slots(&questions, |e| e.has_location(&old_at)).is_empty() {
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
             one"
        );
    }
    let mut inspector = Inspector::new();
    let new_path = located(root, &new_at)?;
    let baseline = if legacy {
        let reading = if old_raw {
            Reading::As(None)
        } else {
            Reading::Sniff
        };
        Some(
            inspector
                .hash(root, &located(root, &old_at)?, reading)
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
    // NEW is verified against each distinct identity it must carry: entries
    // at one location may identify different readings of it (compressed
    // bytes, their decoded content), and each must hold at NEW.
    let mut verdicts: BTreeMap<String, Result<Verified, Fault>> = BTreeMap::new();
    let mut problems = Vec::new();
    for s in &targets {
        let e = entry(s);
        let o = &questions[s.0].observations[s.1];
        let at = label(&questions[s.0], s.1);
        let expected = e
            .sha256
            .clone()
            .or(baseline.as_ref().map(|b| b.sha256.clone()))
            .expect("an entry without a digest has the baseline");
        let verdict = verdicts
            .entry(expected.clone())
            .or_insert_with(|| inspector.verify(root, &new_path, &expected));
        match verdict {
            Ok(Verified::Matches) => {}
            Ok(Verified::Changed(readings)) => problems.push(format!(
                "{at}: `{new_at}` reads {}, but the entry's content is {}",
                describe(readings),
                short(&expected)
            )),
            Err(fault) => problems.push(format!("{at}: {}: {fault}", new.display())),
        }
        for (j, other) in o.evidence.iter().enumerate() {
            if j != s.2 && other.has_location(&new_at) {
                problems.push(format!(
                    "{at}: `{new_at}` is a location of its other evidence `{}`",
                    other.path
                ));
            }
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
        if let Some(b) = &baseline
            && e.sha256.is_none()
        {
            e.sha256 = Some(b.sha256.clone());
            e.attested = Some(Attested {
                at,
                reason: reason.clone(),
            });
            attested += 1;
        }
        e.moves.push(Move {
            at,
            from: old_at.clone(),
            reason: reason.clone(),
        });
        e.path.clone_from(&new_at);
        changed.insert(s.0);
    }
    publish(&store, &questions, &changed, &inspector)?;
    Ok(Relocated {
        entries: targets.len(),
        questions: changed.len(),
        attested,
    })
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
    // however the locations were selected. Their digests are also the only
    // identity a new one there may take (below).
    let mut existing: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for location in &locations {
        let here = slots(&questions, |e| e.path == *location);
        if here.is_empty() {
            problems.push(format!("no evidence entry is at `{location}`"));
        }
        for s in &here {
            let e = &questions[s.0].observations[s.1].evidence[s.2];
            let Some(recorded) = &e.sha256 else { continue };
            existing
                .entry(location.clone())
                .or_default()
                .insert(recorded.clone());
            match inspector.verify(root, &located(root, location)?, recorded) {
                Ok(Verified::Matches) => {}
                Ok(Verified::Changed(readings)) => problems.push(format!(
                    "{}: `{location}` reads {}, not its digest {}",
                    label(&questions[s.0], s.1),
                    describe(&readings),
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
        match inspector.hash(root, &located(root, &location)?, reading) {
            Ok(h) => {
                // A location whose hashed entries identify other content (or
                // disagree among themselves) supplies no one identity: the
                // reading asked for would make two records of one file
                // name different evidence.
                if let Some(theirs) = existing.get(&location)
                    && (theirs.len() > 1 || !theirs.contains(&h.sha256))
                {
                    problems.push(format!(
                        "{}: `{location}` reads {} {}, but entries already there identify {}; \
                         attest with the reading that yields theirs",
                        label(&questions[s.0], s.1),
                        h.encoding.map_or("raw", Encoding::as_str),
                        short(&h.sha256),
                        theirs
                            .iter()
                            .map(|d| short(d))
                            .collect::<Vec<_>>()
                            .join(" and ")
                    ));
                }
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

/// What `merge` did.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Merged {
    Merged,
    /// REDUNDANT was already merged into SURVIVOR: a rerun.
    Already,
}

/// Merge the entry at `redundant` into the entry at `into`, both evidence of
/// question `id`'s observation `observation` (1-based): two citations of the
/// same content, as when one capture is cited both directly and through a
/// probe's copy of it. Both must carry digests and they must be equal; no file is read,
/// so it works where the evidence is not checked out. The redundant entry is
/// kept whole under the survivor's `[[merged]]` (its locations stay
/// reserved to the survivor and still name it in keyed reruns, its
/// provenance stays readable), and only the survivor is read by `check` and
/// `show`. Both locations must be current ones; rerunning a finished merge
/// is a no-op.
pub fn merge(
    root: &Path,
    id: &str,
    observation: usize,
    redundant: &Path,
    into: &Path,
    reason: &str,
) -> Result<Merged> {
    let root = &absolute(root)?;
    let _lock = lock(root)?;
    let store = load_unlocked(root)?;
    let reason = reason.trim().to_string();
    if reason.is_empty() {
        bail!("a merge needs a --reason");
    }
    let (redundant_at, into_at) = (
        stored_location(root, redundant)?,
        stored_location(root, into)?,
    );
    if redundant_at == into_at {
        bail!("`{redundant_at}` and `{into_at}` are the same location");
    }
    let mut questions: Vec<Question> = store.questions().to_vec();
    let Some(qi) = questions.iter().position(|q| q.id == id) else {
        bail!("no question `{id}` under {}", root.display());
    };
    let Some(oi) = observation
        .checked_sub(1)
        .filter(|i| *i < questions[qi].observations.len())
    else {
        bail!(
            "question {id} has no observation #{observation} (it has {})",
            questions[qi].observations.len()
        );
    };
    let evidence = &questions[qi].observations[oi].evidence;
    let current = |at: &str| evidence.iter().position(|e| e.path == at);
    let owner = |at: &str| evidence.iter().position(|e| e.has_location(at));
    // A finished merge is recognized through ownership, wherever the
    // survivor has moved (or been merged) since: REDUNDANT sits in the
    // merged history of the entry INTO also names.
    if let (Some(j), Some(k)) = (owner(&redundant_at), owner(&into_at))
        && j == k
        && in_merged(&evidence[j], &redundant_at)
    {
        return Ok(Merged::Already);
    }
    let Some(survivor) = current(&into_at) else {
        bail!(match owner(&into_at) {
            Some(j) => format!(
                "`{into_at}` is a former or merged location of the entry now at `{}`; name that",
                evidence[j].path
            ),
            None => format!("`{into_at}` is not evidence of {id} #{observation}"),
        });
    };
    let Some(gone) = current(&redundant_at) else {
        bail!(match owner(&redundant_at) {
            Some(j) => format!(
                "`{redundant_at}` is a former or merged location of the entry now at `{}`; \
                 name that",
                evidence[j].path
            ),
            None => format!("`{redundant_at}` is not evidence of {id} #{observation}"),
        });
    };
    let (theirs, ours) = (&evidence[gone].sha256, &evidence[survivor].sha256);
    match (theirs, ours) {
        (Some(a), Some(b)) if a == b => {}
        (Some(a), Some(b)) => bail!(
            "`{redundant_at}` identifies {} and `{into_at}` {}: only the same content merges",
            short(a),
            short(b)
        ),
        _ => bail!(
            "both entries need a digest to merge (`po verdict evidence attest` first): \
             without one there is nothing to show they are the same content"
        ),
    }
    let at = now_utc()?;
    let evidence = &mut questions[qi].observations[oi].evidence;
    let entry = evidence.remove(gone);
    let survivor = if gone < survivor {
        survivor - 1
    } else {
        survivor
    };
    evidence[survivor].merged.push(Merge { at, reason, entry });
    publish(&store, &questions, &BTreeSet::from([qi]), &Inspector::new())?;
    Ok(Merged::Merged)
}

/// Whether `location` belongs to an entry merged (at any depth) into `e`.
fn in_merged(e: &Evidence, location: &str) -> bool {
    e.merged
        .iter()
        .any(|m| m.entry.own_locations().any(|l| l == location) || in_merged(&m.entry, location))
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
            None => bail!("no question `{id}`"),
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
    fn stored_locations_stay_inside_the_evidence_directory() {
        let root = Path::new("/repo/verdicts");
        let at = |p: &str| stored_location(root, Path::new(p));
        assert_eq!(
            at("/repo/verdicts/evidence/a/b.json").expect("inside"),
            "a/b.json"
        );
        assert_eq!(
            at("/repo/verdicts/evidence/./x/../b.json").expect("folded"),
            "b.json"
        );
        // Containment is by whole components: a sibling sharing the name as
        // a string prefix is outside.
        for outside in [
            "/repo/verdicts/evidence2/b.json",
            "/repo/verdicts/evidence",
            "/repo/verdicts/evidence/../records/x.toml",
            "/repo/other.json",
        ] {
            let err = at(outside).expect_err(outside).to_string();
            assert!(err.contains("is outside"), "{outside}: {err}");
        }
    }

    #[test]
    fn location_shapes() {
        for good in ["x.json", "run26/export.json", "a/b.json.zst"] {
            assert_eq!(location_shape(good), None, "{good}");
        }
        for bad in [
            "",
            "/abs.json",
            "a/../b",
            "./a",
            "a//b",
            "..",
            "a/",
            "../fieldwork/run26.json",
        ] {
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
