#!/usr/bin/env python3
"""Bake per-feed OHLCV span metadata into vendor/pineforge-corpus/data/.

Reads the four upstream OHLCV CSVs (ETH-USDT-USDT 15m / 1m, plain and
warmup6m) and emits a single ohlcv_spans.json with first/last/bar
timestamps in milliseconds. That JSON gets baked into the binary and
consulted by `pine diff` for interior-trim bounds.

Run order during vendor refresh (see vendor/pineforge-corpus/
VENDORING_NOTES.md):

  1. git clone the upstream corpus into vendor/pineforge-corpus
  2. rm -rf vendor/pineforge-corpus/.git
  3. python3 scripts/bake-ohlcv-spans.py
  4. scripts/prune-vendored-corpus.sh  (preserves ohlcv_spans.json)

Idempotent: re-running overwrites with the same numbers if the CSVs
haven't changed.
"""

from __future__ import annotations

import csv
import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
DATA_DIR = ROOT / "vendor" / "pineforge-corpus" / "data"

FEEDS = [
    "ohlcv_ETH-USDT-USDT_15m",
    "ohlcv_ETH-USDT-USDT_15m_warmup6m",
    "ohlcv_ETH-USDT-USDT_1m",
    "ohlcv_ETH-USDT-USDT_1m_warmup6m",
]


def feed_span(csv_path: Path) -> dict[str, int]:
    """Read first + second + last timestamps of an OHLCV csv.

    bar_ms is inferred as (second_ts - first_ts). Returns
    {"first_ms", "last_ms", "bar_ms"}.
    """
    with csv_path.open(encoding="utf-8") as f:
        reader = csv.reader(f)
        header = next(reader, None)
        if not header or header[0].strip().lower() != "timestamp":
            raise SystemExit(
                f"{csv_path}: expected first column 'timestamp', got {header}"
            )
        first_row = next(reader, None)
        second_row = next(reader, None)
        if first_row is None or second_row is None:
            raise SystemExit(f"{csv_path}: fewer than two data rows")
        first_ms = int(first_row[0])
        second_ms = int(second_row[0])
        bar_ms = second_ms - first_ms
        last_ms = second_ms
        for row in reader:
            if row:
                last_ms = int(row[0])
    return {"first_ms": first_ms, "last_ms": last_ms, "bar_ms": bar_ms}


def main() -> None:
    if not DATA_DIR.is_dir():
        raise SystemExit(
            f"{DATA_DIR} not found; run after cloning upstream and before pruning"
        )

    spans: dict[str, dict[str, int]] = {}
    for feed in FEEDS:
        path = DATA_DIR / f"{feed}.csv"
        if not path.is_file():
            print(f"skipping missing {path}", file=sys.stderr)
            continue
        spans[feed] = feed_span(path)
        print(f"{feed}: {spans[feed]}")

    if not spans:
        raise SystemExit(f"no OHLCV csvs under {DATA_DIR}")

    out = DATA_DIR / "ohlcv_spans.json"
    with out.open("w", encoding="utf-8") as f:
        json.dump(spans, f, indent=2, sort_keys=True)
        f.write("\n")
    print(f"wrote {out}")


if __name__ == "__main__":
    main()
