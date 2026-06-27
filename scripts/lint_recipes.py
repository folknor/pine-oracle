#!/usr/bin/env python3
"""Extract the ```pine fence from each recipe card and run pine-lint on it.

Usage:
    python3 scripts/lint_recipes.py [name ...]

With no args, lints every recipe under assets/recipes. With names, lints only
recipes whose file stem matches one of the given names. Prints one line per
recipe (clean / FAIL) and a summary; exits 1 if any recipe has errors.
"""
import os
import re
import subprocess
import sys

ROOT = os.path.join(os.path.dirname(__file__), "..", "assets", "recipes")
FENCE = re.compile(r"```pine\n(.*?)```", re.DOTALL)


def extract(path):
    with open(path, encoding="utf-8") as f:
        m = FENCE.search(f.read())
    return m.group(1) if m else None


def main(argv):
    wanted = set(argv)
    rows = []
    for cat in sorted(os.listdir(ROOT)):
        catdir = os.path.join(ROOT, cat)
        if not os.path.isdir(catdir):
            continue
        for fn in sorted(os.listdir(catdir)):
            if not fn.endswith(".md"):
                continue
            stem = fn[:-3]
            if wanted and stem not in wanted:
                continue
            code = extract(os.path.join(catdir, fn))
            label = f"{cat}/{stem}"
            if code is None:
                rows.append((label, False, "no pine fence found"))
                continue
            res = subprocess.run(
                ["pine-lint", "-H", "-c", code],
                capture_output=True, text=True,
            )
            out = (res.stdout + res.stderr).strip()
            clean = res.returncode == 0
            rows.append((label, clean, out))

    fails = [r for r in rows if not r[1]]
    for label, clean, out in rows:
        if clean:
            print(f"  ok   {label}")
        else:
            print(f"  FAIL {label}")
            for line in out.splitlines():
                print(f"         {line}")
    print(f"\n{len(rows)} recipes, {len(fails)} with errors")
    return 1 if fails else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
