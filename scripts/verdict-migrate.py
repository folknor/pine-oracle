#!/usr/bin/env python3
"""Migrate a po verdict store from the old --records layout to verdicts/.

Usage:
    python3 scripts/verdict-migrate.py --repo PATH [--old-records DIR]
        [--old-evidence DIR] [--apply | --verify]

The old layout: question files and `fixtures/` in a records directory
(default `reference/records`), evidence locations stored relative to it and
free to point anywhere in the repository. The new layout, which po now owns:

    verdicts/records/<id>.toml
    verdicts/fixtures/<sha256>.pine
    verdicts/evidence/...          every location relative to this, no `..`

Default is a dry run: every check runs and the plan is printed; nothing is
written. With --apply, in this order:

  1. .gitattributes and .gitignore lines naming the old directories are
     rewritten, and LFS rules are added for copied files that were LFS at
     their source. This comes first: a file added under a path no LFS rule
     matches is committed as a plain blob for good.
  2. the old evidence directory is renamed to verdicts/evidence (ignored
     files such as record.json move with it);
  3. every current evidence location outside it is copied in, its bytes
     checked against the entry's digest, under
     verdicts/evidence/external/<path in the repository>, so the folder names
     where it came from. The source is left in place;
  4. question files, notes and fixtures move to verdicts/; the old lock is
     deleted;
  5. every location in every question file is rewritten: inside the old
     evidence directory -> relative to verdicts/evidence; outside -> the
     external/ copy. History (moved `from`, merged entries) is rewritten the
     same way, lexically: it names where the file was, never read;
  6. the changes are staged (`git add -A`) so git sees the renames and the
     LFS filter cleans the new paths. Nothing is committed.

Afterwards, before committing: `po verdict list` (the strict load),
`po verdict evidence check` (every digest), and `--verify`, which checks the
index: every staged file under verdicts/ that an LFS rule covers must be an
LFS pointer, and plain blobs over 1 MB are listed.

The run refuses, before writing anything, on: uncommitted changes in any
path it touches, a verdicts/ that already exists, a cited outside file that
is missing, unfetched (an LFS pointer), outside the repository or whose
bytes do not match the entry's digest, a location string po could not
store, a symbolic link on a new location, two entries of one observation
landing on one location, and a nested .gitignore / .gitattributes that
spells out an old path.

There is no rollback. If --apply fails part way, undo it by hand: move
verdicts/evidence back to the old evidence directory FIRST (ignored files
such as record.json exist nowhere else), then `git restore --staged
--worktree` the touched paths and delete what is left of verdicts/.
Question files are written to verdicts/records before their old copy is
deleted, so an interrupted run duplicates records, never loses one.
"""

import argparse
import gzip
import hashlib
import os
import re
import shutil
import subprocess
import sys
import tomllib
from collections import Counter, defaultdict
from pathlib import Path

try:
    from compression import zstd as ZSTD
except ImportError:  # Python before 3.14: zstd evidence is refused, never misjudged
    ZSTD = None

LFS_ATTRS = "filter=lfs diff=lfs merge=lfs -text"
LFS_POINTER = b"version https://git-lfs.github.com/spec/v1"
EXTERNAL = "external"


class Refusal(Exception):
    pass


def norm(path: Path) -> Path:
    """Lexical normalization, as po does: never through the filesystem."""
    return Path(os.path.normpath(path))


def inside(path: Path, base: Path) -> bool:
    return path == base or base in path.parents


def git(repo: Path, *args: str, check: bool = True) -> str:
    out = subprocess.run(
        ["git", "-C", str(repo), *args], capture_output=True, text=True
    )
    if check and out.returncode != 0:
        raise Refusal(f"git {' '.join(args)} failed: {out.stderr.strip()}")
    return out.stdout


def readings(data: bytes) -> list[str]:
    """sha256 of the raw bytes and, for a gzip or zstd stream, the decoded."""
    out = [hashlib.sha256(data).hexdigest()]
    try:
        if data[:2] == b"\x1f\x8b":
            out.append(hashlib.sha256(gzip.decompress(data)).hexdigest())
        elif data[:4] == b"\x28\xb5\x2f\xfd" or (
            len(data) >= 4 and data[0] & 0xF0 == 0x50 and data[1:4] == b"\x2a\x4d\x18"
        ):
            if ZSTD:
                out.append(hashlib.sha256(ZSTD.decompress(data)).hexdigest())
    except Exception:  # noqa: BLE001 - a stream that does not decode has no second reading
        pass
    return out


def storable(loc: str) -> bool:
    """A location po writes and this script can rewrite in place: a TOML
    basic string with no escapes, `/`-separated, no empty / `.` / `..`."""
    if not loc or any(c in loc for c in '"\\\n\r\t') or loc.startswith("/"):
        return False
    return all(part not in ("", ".", "..") for part in loc.split("/"))


class Plan:
    def __init__(self, repo: Path, old_records: Path, old_evidence: Path):
        self.repo = repo
        self.old_records = old_records
        self.old_evidence = old_evidence
        self.verdicts = repo / "verdicts"
        self.problems: list[str] = []
        # source path -> destination location (relative to verdicts/evidence)
        self.copies: dict[Path, str] = {}
        self.copy_sizes: dict[Path, int] = {}
        self.planned: dict[Path, str] = {}
        # per question file: old location -> new location
        self.maps: dict[Path, dict[str, str]] = {}
        self.docs: dict[Path, dict] = {}
        self.kinds: Counter = Counter()
        self.sources: Counter = Counter()
        self.links_redirected: list[tuple[str, str]] = []

    # -- locations ----------------------------------------------------------

    def map_location(self, qfile: Path, loc: str, sha: str | None, current: bool) -> str | None:
        target = norm(self.old_records / loc)
        if inside(target, self.old_evidence) and target != self.old_evidence:
            self.kinds["inside"] += 1
            return target.relative_to(self.old_evidence).as_posix()
        if not inside(target, self.repo) or target == self.repo:
            self.problems.append(f"{qfile.name}: `{loc}` is outside the repository")
            return None
        rel = target.relative_to(self.repo)
        if not current:
            # History names where a file was; it is never read.
            self.kinds["history outside"] += 1
            return f"{EXTERNAL}/{rel.as_posix()}"
        # Reached through a link (at any component) into the old evidence
        # tree, as a probe's tv_record.json is: cite the file it reaches
        # rather than copying it, once its bytes are checked.
        real = Path(os.path.realpath(target))
        real_evidence = Path(os.path.realpath(self.old_evidence))
        if real != target and inside(real, real_evidence) and real != real_evidence:
            new = real.relative_to(real_evidence).as_posix()
            self.check_source(qfile, loc, real, sha)
            self.links_redirected.append((loc, new))
            self.kinds["link into evidence"] += 1
            return new
        self.kinds["copied in"] += 1
        self.sources["/".join(rel.parts[:3]) if len(rel.parts) > 3 else rel.as_posix()] += 1
        dest = f"{EXTERNAL}/{rel.as_posix()}"
        if target not in self.copies:
            self.check_source(qfile, loc, target, sha)
            self.copies[target] = dest
        elif sha is not None:
            self.check_source(qfile, loc, target, sha)
        return dest

    def check_source(self, qfile: Path, loc: str, src: Path, sha: str | None) -> None:
        if not src.is_file():
            self.problems.append(f"{qfile.name}: `{loc}` is not a file")
            return
        data = src.read_bytes()
        self.copy_sizes[src] = len(data)
        # The copy must be these bytes, the ones checked against the digest,
        # even if the source changes before it is made.
        planned = hashlib.sha256(data).hexdigest()
        if self.planned.setdefault(src, planned) != planned:
            self.problems.append(f"{qfile.name}: `{loc}` changed while it was checked")
        if data.startswith(LFS_POINTER) and len(data) < 1024:
            self.problems.append(f"{qfile.name}: `{loc}` is an unfetched LFS pointer")
            return
        if sha is not None and sha not in readings(data):
            self.problems.append(f"{qfile.name}: `{loc}` no longer holds its digest {sha[:12]}")
        if data[:4] == b"\x28\xb5\x2f\xfd" and not ZSTD:
            self.problems.append(
                f"{qfile.name}: `{loc}` is zstd and this Python cannot decode it (needs 3.14+)"
            )

    def map_entry(self, qfile: Path, entry, current: bool, mapping: dict, seen: list) -> None:
        if isinstance(entry, str):
            entry = {"path": entry}
        sha = entry.get("sha256")
        locs = [(entry["path"], current)]
        locs += [(m["from"], False) for m in entry.get("moved", [])]
        for loc, is_current in locs:
            new = self.map_location(qfile, loc, sha, is_current)
            if new is None:
                continue
            if loc in mapping and mapping[loc] != new:
                self.problems.append(
                    f"{qfile.name}: `{loc}` maps to both `{mapping[loc]}` and `{new}`"
                )
            mapping[loc] = new
            seen.append(new)
        for m in entry.get("merged", []):
            self.map_entry(qfile, m["entry"], False, mapping, seen)

    def scan_records(self) -> None:
        for path in sorted(self.old_records.iterdir()):
            name = path.name
            if path.is_dir():
                if name != "fixtures":
                    self.problems.append(f"{path}: unexpected directory in the records dir")
                continue
            if name == ".lock" or name.endswith(".md") or name.startswith("."):
                continue
            if not name.endswith(".toml"):
                self.problems.append(f"{path}: unexpected file in the records dir")
                continue
            doc = tomllib.loads(path.read_text())
            self.docs[path] = doc
            mapping: dict[str, str] = {}
            for n, obs in enumerate(doc.get("observation", []), start=1):
                seen: list[str] = []
                for entry in obs.get("evidence", []):
                    self.map_entry(path, entry, True, mapping, seen)
                for loc, count in Counter(seen).items():
                    if count > 1:
                        self.problems.append(
                            f"{path.name} #{n}: two entries would share `{loc}`; "
                            "merge them with po before migrating"
                        )
            for new in mapping.values():
                if not storable(new):
                    self.problems.append(f"{path.name}: `{new}` is not a storable location")
            self.maps[path] = mapping

    def check_destinations(self) -> None:
        if self.verdicts.exists() or self.verdicts.is_symlink():
            self.problems.append(f"{self.verdicts} already exists")
        copied = set(self.copies.values())
        for dest in copied:
            if (self.old_evidence / dest).exists():
                self.problems.append(f"{dest} already exists in the evidence directory")
            if any(c in dest for c in " \t[]*?#!\\"):
                self.problems.append(f"{dest}: a gitattributes pattern could not name it")
        # History rewritten under external/ names where a file was; it must
        # not name a file that exists in the tree.
        for mapping in self.maps.values():
            for new in mapping.values():
                if new.startswith(f"{EXTERNAL}/") and new not in copied:
                    if (self.old_evidence / new).exists():
                        self.problems.append(f"history location {new} names an existing file")
        # Ignore and attribute files below the moved trees keep relative
        # patterns valid, but one spelling out an old path would go stale.
        old_names = (self.old_records.relative_to(self.repo).as_posix(),
                     self.old_evidence.relative_to(self.repo).as_posix())
        for tree in (self.old_records, self.old_evidence):
            for name in (".gitignore", ".gitattributes"):
                for f in tree.rglob(name):
                    for line in f.read_text().splitlines():
                        if not line.lstrip().startswith("#") and any(o in line for o in old_names):
                            self.problems.append(f"{f}: `{line}` names an old path; fix it by hand")
        # Links on the way to any new location are refused by po.
        for mapping in self.maps.values():
            for new in mapping.values():
                at = self.old_evidence
                for part in new.split("/"):
                    at = at / part
                    if at.is_symlink():
                        self.problems.append(
                            f"{at} is a symbolic link: po never reaches evidence through one"
                        )
                        break
        if self.old_evidence.is_symlink():
            self.problems.append(f"{self.old_evidence} is a symbolic link")

    # -- git attributes ------------------------------------------------------

    def lfs_sources(self) -> set[Path]:
        """The copied sources that are LFS where they are now. A file inside
        a submodule is asked of the submodule, whose attributes the
        superproject does not see."""
        by_repo: dict[Path, list[Path]] = defaultdict(list)
        for src in self.copies:
            owner = next(
                (d for d in src.parents if inside(d, self.repo) and (d / ".git").exists()),
                self.repo,
            )
            by_repo[owner].append(src)
        lfs = set()
        for owner, srcs in by_repo.items():
            rels = [s.relative_to(owner).as_posix() for s in srcs]
            out = git(owner, "check-attr", "-z", "filter", "--", *rels).split("\0")
            for i in range(0, len(out) - 2, 3):
                if out[i + 2] == "lfs":
                    lfs.add(owner / out[i])
        return lfs

    def lfs_rules(self) -> list[str]:
        """One rule per (directory, extension) of copied LFS files, or per
        file where that directory also receives non-LFS files of the same
        extension."""
        lfs = self.lfs_sources()
        groups: dict[tuple[str, str], list[tuple[str, bool]]] = defaultdict(list)
        for src, dest in self.copies.items():
            d, _, base = dest.rpartition("/")
            ext = base.rpartition(".")[2] if "." in base else ""
            groups[(d, ext)].append((base, src in lfs))
        rules = []
        for (d, ext), files in sorted(groups.items()):
            if not any(is_lfs for _, is_lfs in files):
                continue
            if ext and all(is_lfs for _, is_lfs in files):
                rules.append(f"verdicts/evidence/{d}/*.{ext} {LFS_ATTRS}")
            else:
                rules.extend(
                    f"verdicts/evidence/{d}/{base} {LFS_ATTRS}" for base, is_lfs in files if is_lfs
                )
        return rules

    def rewrite_patterns(self, text: str) -> tuple[str, list[str]]:
        rec = self.old_records.relative_to(self.repo).as_posix()
        ev = self.old_evidence.relative_to(self.repo).as_posix()
        # Most specific first; each must end at a component boundary.
        table = [
            (f"{rec}/.lock", "verdicts/.lock"),
            (f"{rec}/fixtures", "verdicts/fixtures"),
            (rec, "verdicts/records"),
            (ev, "verdicts/evidence"),
        ]
        lead = re.compile(r"^(\s*!?(?:/|\*\*/)?)")
        changes, lines = [], []
        for line in text.splitlines(keepends=True):
            new = line
            if not line.lstrip().startswith("#"):
                head = lead.match(line).group(1)
                rest = line[len(head) :]
                for old_p, new_p in table:
                    after = rest[len(old_p) : len(old_p) + 1]
                    if rest.startswith(old_p) and (after == "" or after in "/ \t\r\n"):
                        new = head + new_p + rest[len(old_p) :]
                        break
            if new != line:
                changes.append(f"  - {line.rstrip()}\n  + {new.rstrip()}")
            lines.append(new)
        return "".join(lines), changes


def rewrite_toml(text: str, mapping: dict[str, str]) -> str:
    def swap(m: re.Match) -> str:
        return '"' + mapping.get(m.group(1), m.group(1)) + '"'

    return re.sub(r'"([^"\\\n]*)"', swap, text)


def expected(doc, mapping: dict[str, str]):
    """`doc` with every location replaced, for checking the text rewrite."""

    def entry(e):
        if isinstance(e, str):
            return mapping.get(e, e)
        e = dict(e)
        e["path"] = mapping.get(e["path"], e["path"])
        if "moved" in e:
            e["moved"] = [{**m, "from": mapping.get(m["from"], m["from"])} for m in e["moved"]]
        if "merged" in e:
            e["merged"] = [{**m, "entry": entry(m["entry"])} for m in e["merged"]]
        return e

    doc = dict(doc)
    if "observation" in doc:
        doc["observation"] = [
            {**o, "evidence": [entry(e) for e in o["evidence"]]} if "evidence" in o else o
            for o in doc["observation"]
        ]
    return doc


def verify_staged(repo: Path) -> int:
    """After --apply: every staged file under verdicts/ that an LFS rule
    covers must be staged as an LFS pointer, and plain blobs over 1 MB are
    listed. Reads the index, so run it before committing."""
    entries = []
    for rec in git(repo, "ls-files", "-s", "-z", "--", "verdicts").split("\0"):
        if rec:
            meta, _, path = rec.partition("\t")
            entries.append((meta.split()[1], path))
    if not entries:
        print("nothing staged under verdicts/", file=sys.stderr)
        return 1
    attrs = subprocess.run(
        ["git", "-C", str(repo), "check-attr", "--cached", "filter", "--stdin", "-z"],
        input="\0".join(p for _, p in entries) + "\0", capture_output=True, text=True, check=True,
    ).stdout.split("\0")
    lfs_paths = {attrs[i] for i in range(0, len(attrs) - 2, 3) if attrs[i + 2] == "lfs"}
    batch = subprocess.run(
        ["git", "-C", str(repo), "cat-file", "--batch"],
        input="".join(f"{sha}\n" for sha, _ in entries).encode(), capture_output=True, check=True,
    ).stdout
    sizes, pointers, at = {}, set(), 0
    for sha, path in entries:
        header_end = batch.index(b"\n", at)
        header = batch[at:header_end].split()
        if len(header) != 3:
            print(f"{path}: staged object {sha} is missing", file=sys.stderr)
            return 1
        size = int(header[2])
        body = batch[header_end + 1 : header_end + 1 + size]
        at = header_end + 1 + size + 1
        sizes[path] = size
        if size < 1024 and body.startswith(LFS_POINTER):
            pointers.add(path)
    plain_lfs = sorted(lfs_paths - pointers)
    big = sorted((p for p in sizes if p not in pointers and sizes[p] > 1_000_000))
    print(f"{len(entries)} staged files under verdicts/: {len(pointers)} LFS pointers, "
          f"{len(entries) - len(pointers)} plain blobs")
    for p in big:
        print(f"  plain blob over 1 MB: {p} ({sizes[p] // 1024} KiB)")
    if plain_lfs:
        print(f"{len(plain_lfs)} file(s) an LFS rule covers are staged as plain blobs:", file=sys.stderr)
        for p in plain_lfs:
            print(f"  {p}", file=sys.stderr)
        return 1
    print("every file an LFS rule covers is staged as a pointer")
    return 0


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--repo", required=True, type=Path)
    ap.add_argument("--old-records", default="reference/records")
    ap.add_argument("--old-evidence", default="reference/evidence")
    ap.add_argument("--apply", action="store_true")
    ap.add_argument("--verify", action="store_true",
                    help="after --apply: check how verdicts/ is staged (LFS pointers, large blobs)")
    args = ap.parse_args()

    repo = args.repo.resolve()
    if not (repo / ".git").exists():
        print(f"{repo} is not a git repository root", file=sys.stderr)
        return 1
    if args.verify:
        return verify_staged(repo)
    plan = Plan(repo, norm(repo / args.old_records), norm(repo / args.old_evidence))
    for d in (plan.old_records, plan.old_evidence):
        if not d.is_dir():
            print(f"{d} does not exist", file=sys.stderr)
            return 1

    # Hold the old store's write lock, as po does, from before the scan to
    # the end of the run, so no po writer changes a record between the
    # snapshot planned here and its rewrite. Exclusive for --apply, shared
    # for a dry run. The lock file is removed only after staging, and the
    # lock is released when the process exits.
    lock = open(plan.old_records / ".lock", "a")  # noqa: SIM115 - held to exit
    try:
        import fcntl

        fcntl.flock(lock, (fcntl.LOCK_EX if args.apply else fcntl.LOCK_SH) | fcntl.LOCK_NB)
    except ImportError:
        pass
    except OSError:
        print("the old store is locked by a po writer: retry when it is done", file=sys.stderr)
        return 1

    plan.scan_records()
    plan.check_destinations()
    # `git add -A` would sweep unrelated edits in these paths into the
    # migration, and a dirty tree makes a failed run harder to undo.
    touched = [plan.old_records.relative_to(repo).as_posix(),
               plan.old_evidence.relative_to(repo).as_posix(), "verdicts",
               ".gitattributes", ".gitignore"]
    dirty = git(repo, "status", "--porcelain", "--", *touched).splitlines()
    for line in dirty:
        plan.problems.append(f"uncommitted change: {line}")
    try:
        rules = plan.lfs_rules()
    except Refusal as e:
        plan.problems.append(str(e))
        rules = []

    attr_path, ignore_path = repo / ".gitattributes", repo / ".gitignore"
    attr_text = attr_path.read_text() if attr_path.exists() else ""
    ignore_text = ignore_path.read_text() if ignore_path.exists() else ""
    new_attr, attr_changes = plan.rewrite_patterns(attr_text)
    new_ignore, ignore_changes = plan.rewrite_patterns(ignore_text)
    if rules:
        block = "\n# Evidence copied into the store from elsewhere in the repository.\n"
        new_attr = new_attr.rstrip("\n") + "\n" + block + "\n".join(rules) + "\n"

    print(f"== {len(plan.docs)} question files ==")
    for kind, n in sorted(plan.kinds.items()):
        print(f"  {kind}: {n} locations")
    print(f"== copied in: {len(plan.copies)} files ==")
    for src, n in plan.sources.most_common():
        print(f"  {n:5}  {src}")
    big = sorted((s for s in plan.copies if plan.copy_sizes.get(s, 0) > 1_000_000), key=str)
    for src in big:
        print(f"  large: {src.relative_to(repo)} ({plan.copy_sizes[src] // 1024} KiB)")
    for old, new in plan.links_redirected:
        print(f"  link into evidence: {old} -> {new}")
    print("== .gitattributes ==")
    print("\n".join(attr_changes) if attr_changes else "  (no path rules to rewrite)")
    for r in rules:
        print(f"  + {r}")
    print("== .gitignore ==")
    print("\n".join(ignore_changes) if ignore_changes else "  (no path rules to rewrite)")

    if plan.problems:
        print(f"== {len(plan.problems)} problem(s): refusing ==", file=sys.stderr)
        for p in plan.problems:
            print(f"  {p}", file=sys.stderr)
        return 1
    if not args.apply:
        print("dry run: nothing written (pass --apply)")
        return 0

    # Rewrite every question file in memory first, and check it, so a bad
    # rewrite stops the run before anything moves.
    rewritten: dict[Path, str] = {}
    for path, mapping in plan.maps.items():
        text = rewrite_toml(path.read_text(), mapping)
        if tomllib.loads(text) != expected(plan.docs[path], mapping):
            print(f"{path.name}: the rewrite changed more than its locations", file=sys.stderr)
            return 1
        rewritten[path] = text

    # 1. attributes before anything is added under a new path.
    if attr_changes or rules:
        attr_path.write_text(new_attr)
    if ignore_changes:
        ignore_path.write_text(new_ignore)
    # 2. evidence.
    plan.verdicts.mkdir()
    new_evidence = plan.verdicts / "evidence"
    os.rename(plan.old_evidence, new_evidence)
    # 3. outside files, copied byte for byte.
    for src, dest in plan.copies.items():
        target = new_evidence / dest
        target.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy(src, target)
        if hashlib.sha256(target.read_bytes()).hexdigest() != plan.planned[src]:
            print(f"{target}: {src} changed after it was checked; see the rollback note",
                  file=sys.stderr)
            return 1
    # 4. records, notes, fixtures.
    records = plan.verdicts / "records"
    records.mkdir()
    for path in sorted(plan.old_records.iterdir()):
        if path.name == ".lock":
            continue
        elif path.name == "fixtures":
            os.rename(path, plan.verdicts / "fixtures")
        elif path in rewritten:
            (records / path.name).write_text(rewritten[path])
            path.unlink()
        else:
            os.rename(path, records / path.name)
    # 6. stage, so the LFS filter sees the new paths.
    # The old directories are gone from the working tree; naming them stages
    # their removal. The old .lock is still there (held) and the rewritten
    # .gitignore no longer covers it, so it is excluded by name.
    old_lock = (plan.old_records / ".lock").relative_to(repo).as_posix()
    paths = ["verdicts", plan.old_records.relative_to(repo).as_posix(),
             plan.old_evidence.relative_to(repo).as_posix(), f":(exclude){old_lock}"]
    paths += [p for p in (".gitattributes", ".gitignore") if (repo / p).exists()]
    git(repo, "add", "-A", "--", *paths)
    # Only now does the old lock file go: until here it kept po writers out.
    (plan.old_records / ".lock").unlink()
    if not any(plan.old_records.iterdir()):
        plan.old_records.rmdir()
    print("applied and staged; nothing committed. Next: po verdict list, "
          "po verdict evidence check, then this script with --verify")
    return 0


if __name__ == "__main__":
    sys.exit(main())
