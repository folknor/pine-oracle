#!/usr/bin/env python3
"""Summarize the semantic delta of a vendored pine-data catalog vs. its git HEAD copy.

Usage:
    python3 scripts/vendor-json-delta.py                       # every vendored catalog
    python3 scripts/vendor-json-delta.py functions.json        # one catalog (bare name ok)
    python3 scripts/vendor-json-delta.py functions.json --key flags
    python3 scripts/vendor-json-delta.py functions.json --entry math.sign

Reports added/removed entries and, for entries present in both, which top-level
keys changed - so a whitespace-only rescrape can be told apart from a real
schema or content change. `--key` and `--entry` drill into the actual before/after
values, which is what the vendor NOTICE entry needs to be written accurately.
"""

import argparse
import json
import subprocess
import sys
from collections import Counter
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent
CATALOG_DIR = "vendor/pine-data/v6"


def resolve(arg: str) -> str:
    """Accept a repo-relative path, an absolute path, or a bare catalog file name."""
    path = Path(arg)
    if path.is_absolute():
        path = path.relative_to(REPO)
    rel = path.as_posix()
    if "/" not in rel:
        rel = f"{CATALOG_DIR}/{rel}"
    if not rel.endswith(".json"):
        rel += ".json"
    if not (REPO / rel).is_file():
        sys.exit(f"no such vendored catalog: {rel}")
    return rel


def load_head(rel: str):
    result = subprocess.run(
        ["git", "show", f"HEAD:{rel}"], cwd=REPO, capture_output=True
    )
    if result.returncode != 0:
        return None
    return json.loads(result.stdout)


def norm(value):
    """Collapse invisible/odd whitespace so a whitespace-only rescrape reads as equal."""
    if isinstance(value, str):
        return " ".join(value.replace(" ", " ").replace("​", "").split())
    if isinstance(value, list):
        return [norm(v) for v in value]
    if isinstance(value, dict):
        return {k: norm(v) for k, v in value.items()}
    return value


def render(value) -> str:
    if value is None:
        return "<absent>"
    return json.dumps(value, ensure_ascii=False, sort_keys=True)


def changed_keys(new_entry, old_entry):
    return [
        key
        for key in sorted(set(new_entry) | set(old_entry))
        if norm(new_entry.get(key)) != norm(old_entry.get(key))
    ]


def report(rel: str, args) -> None:
    new_list = json.loads((REPO / rel).read_text())
    old_list = load_head(rel)
    print(f"== {rel} ==")
    if old_list is None:
        print(f"  new file ({len(new_list)} entries), no HEAD copy to diff against")
        return

    new = {e["name"]: e for e in new_list}
    old = {e["name"]: e for e in old_list}

    added = sorted(set(new) - set(old))
    removed = sorted(set(old) - set(new))
    print(f"  {len(old)} -> {len(new)} entries")
    if added:
        print(f"  added:   {', '.join(added)}")
    if removed:
        print(f"  removed: {', '.join(removed)}")

    key_changes = Counter()
    key_names = {}
    whitespace_only = 0
    changed_names = []
    for name in sorted(set(new) & set(old)):
        if new[name] == old[name]:
            continue
        if norm(new[name]) == norm(old[name]):
            whitespace_only += 1
            continue
        changed_names.append(name)
        for key in changed_keys(new[name], old[name]):
            key_changes[key] += 1
            key_names.setdefault(key, []).append(name)

    print(f"  whitespace-only changes: {whitespace_only}")
    print(f"  semantic changes: {len(changed_names)}")
    for key, count in key_changes.most_common():
        detail = ", ".join(key_names[key][:20]) if count <= 20 else ""
        print(f"    {key}: {count} {detail}")
    if changed_names:
        shown = changed_names if args.all else changed_names[:60]
        more = "" if len(shown) == len(changed_names) else f" (+{len(changed_names) - len(shown)} more, --all)"
        print(f"  names: {', '.join(shown)}{more}")

    for key in args.key:
        names = key_names.get(key, [])
        print(f"\n  --- key '{key}': {len(names)} entries ---")
        for name in names[: args.limit]:
            print(f"    {name}")
            print(f"      old: {render(old[name].get(key))}")
            print(f"      new: {render(new[name].get(key))}")
        if len(names) > args.limit:
            print(f"    ... {len(names) - args.limit} more (--limit)")

    for name in args.entry:
        if name not in new and name not in old:
            continue
        print(f"\n  --- entry '{name}' ---")
        if name not in old:
            print(f"    added: {render(new[name])}")
            continue
        if name not in new:
            print(f"    removed: {render(old[name])}")
            continue
        keys = changed_keys(new[name], old[name])
        if not keys:
            print("    no semantic change")
        for key in keys:
            print(f"    {key}")
            print(f"      old: {render(old[name].get(key))}")
            print(f"      new: {render(new[name].get(key))}")


def main() -> None:
    parser = argparse.ArgumentParser(
        description=__doc__,
        formatter_class=argparse.RawDescriptionHelpFormatter,
    )
    parser.add_argument(
        "catalog",
        nargs="*",
        help="vendored catalog(s); bare names resolve under vendor/pine-data/v6. Default: all.",
    )
    parser.add_argument(
        "--key",
        action="append",
        default=[],
        metavar="KEY",
        help="dump old/new values for every entry whose KEY changed (repeatable)",
    )
    parser.add_argument(
        "--entry",
        action="append",
        default=[],
        metavar="NAME",
        help="dump the per-key old/new values for one entry (repeatable)",
    )
    parser.add_argument(
        "--limit", type=int, default=10, help="max entries printed per --key (default 10)"
    )
    parser.add_argument("--all", action="store_true", help="print every changed name")
    args = parser.parse_args()

    catalogs = (
        [resolve(c) for c in args.catalog]
        if args.catalog
        else sorted(p.name for p in (REPO / CATALOG_DIR).glob("*.json"))
    )
    if not args.catalog:
        catalogs = [f"{CATALOG_DIR}/{name}" for name in catalogs]

    for rel in catalogs:
        report(rel, args)


if __name__ == "__main__":
    main()
