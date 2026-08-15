#!/usr/bin/env python3
"""Refresh the vendored pine-tools snapshots (pine-data JSON + pine-manual markdown).

Usage:
    python3 scripts/vendor-refresh.py [--apply] [--upstream PATH]

Default is a dry run: it reports which files differ, entry-count deltas per
pine-data catalog, and added/removed manual pages. With --apply it copies the
upstream files over the vendored tree (LICENSE / NOTICE files are never
touched - update the NOTICE by hand afterwards).
"""

import argparse
import filecmp
import json
import shutil
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent
DATA_FILES = [
    "functions.json",
    "variables.json",
    "constants.json",
    "keywords.json",
    "types.json",
    "annotations.json",
    "operators.json",
]


def report_data(upstream: Path, vendor: Path, apply: bool) -> None:
    print("== pine-data ==")
    for name in DATA_FILES:
        src, dst = upstream / name, vendor / name
        if not src.exists():
            print(f"  {name}: MISSING upstream")
            continue
        same = dst.exists() and filecmp.cmp(src, dst, shallow=False)
        n_src = len(json.loads(src.read_text()))
        n_dst = len(json.loads(dst.read_text())) if dst.exists() else 0
        state = "identical" if same else f"CHANGED ({n_dst} -> {n_src} entries)"
        print(f"  {name}: {state}")
        if not same:
            names_src = entry_names(src)
            names_dst = entry_names(dst) if dst.exists() else set()
            added = sorted(names_src - names_dst)
            removed = sorted(names_dst - names_src)
            if added:
                print(f"    added:   {', '.join(added)}")
            if removed:
                print(f"    removed: {', '.join(removed)}")
            if apply:
                shutil.copyfile(src, dst)


def entry_names(path: Path) -> set:
    return {e.get("name", "") for e in json.loads(path.read_text()) if isinstance(e, dict)}


def report_manual(upstream: Path, vendor: Path, apply: bool) -> None:
    print("== pine-manual ==")
    src_pages = {p.relative_to(upstream) for p in upstream.rglob("*.md")}
    dst_pages = {p.relative_to(vendor) for p in vendor.rglob("*.md")}
    for rel in sorted(src_pages - dst_pages):
        print(f"  added:   {rel}")
        if apply:
            (vendor / rel).parent.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(upstream / rel, vendor / rel)
    for rel in sorted(dst_pages - src_pages):
        print(f"  removed: {rel}")
        if apply:
            (vendor / rel).unlink()
    changed = [
        rel
        for rel in sorted(src_pages & dst_pages)
        if not filecmp.cmp(upstream / rel, vendor / rel, shallow=False)
    ]
    for rel in changed:
        print(f"  changed: {rel}")
        if apply:
            shutil.copyfile(upstream / rel, vendor / rel)
    print(f"  {len(src_pages)} upstream pages, {len(changed)} changed")


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("--apply", action="store_true")
    ap.add_argument("--upstream", default=str(REPO.parent / "pine-tools"))
    args = ap.parse_args()
    up = Path(args.upstream)
    report_data(up / "pine-data" / "v6", REPO / "vendor" / "pine-data" / "v6", args.apply)
    report_manual(up / "pine-manual" / "v6", REPO / "vendor" / "pine-manual" / "v6", args.apply)


if __name__ == "__main__":
    main()
