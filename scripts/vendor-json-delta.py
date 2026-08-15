#!/usr/bin/env python3
"""Summarize the semantic delta of a vendored pine-data catalog vs. its git HEAD copy.

Usage:
    python3 scripts/vendor-json-delta.py vendor/pine-data/v6/functions.json

Reports added/removed entries and, for entries present in both, which top-level
keys changed - so a whitespace-only rescrape can be told apart from a real
schema or content change.
"""

import json
import subprocess
import sys
from collections import Counter
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent


def load_head(rel: str):
    blob = subprocess.run(
        ["git", "show", f"HEAD:{rel}"], cwd=REPO, capture_output=True, check=True
    ).stdout
    return json.loads(blob)


def norm(value):
    """Collapse invisible/odd whitespace so a whitespace-only rescrape reads as equal."""
    if isinstance(value, str):
        return " ".join(value.replace(" ", " ").replace("​", "").split())
    if isinstance(value, list):
        return [norm(v) for v in value]
    if isinstance(value, dict):
        return {k: norm(v) for k, v in value.items()}
    return value


def main() -> None:
    rel = sys.argv[1]
    new = {e["name"]: e for e in json.loads((REPO / rel).read_text())}
    old = {e["name"]: e for e in load_head(rel)}

    added = sorted(set(new) - set(old))
    removed = sorted(set(old) - set(new))
    print(f"{rel}: {len(old)} -> {len(new)} entries")
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
        for key in sorted(set(new[name]) | set(old[name])):
            if norm(new[name].get(key)) != norm(old[name].get(key)):
                key_changes[key] += 1
                key_names.setdefault(key, []).append(name)
    print(f"  whitespace-only changes: {whitespace_only}")
    print(f"  semantic changes: {len(changed_names)}")
    for key, count in key_changes.most_common():
        detail = ", ".join(key_names[key][:20]) if count <= 20 else ""
        print(f"    {key}: {count} {detail}")
    if changed_names:
        print(f"  names: {', '.join(changed_names[:60])}")


if __name__ == "__main__":
    main()
