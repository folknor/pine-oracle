#!/usr/bin/env bash
# Prune vendor/pineforge-corpus/ to the subset pine-oracle bakes into the
# binary. Idempotent. Re-run after re-cloning the upstream corpus.
#
# Kept:
#   - LICENSE, NOTICE, LEGAL.md, README.md (vendoring attribution)
#   - validation/<slug>/strategy.pine     (canonical source)
#   - validation/<slug>/tv_trades.csv     (TV ground truth)
#   - validation/<slug>/inputs.json       (optional, per-probe config)
#
# Dropped:
#   - data/*.csv                     (~75 MB of OHLCV feeds; we keep only the
#                                    derived ohlcv_spans.json baked by
#                                    scripts/bake-ohlcv-spans.py, which must
#                                    run BEFORE this prune)
#   - validation/<slug>/generated.cpp   (PineForge transpiler output, not used)
#   - validation/<slug>/engine_trades.csv (PineForge's own engine output, not used)
#   - validation_report.{html,md,pdf}   (output artefacts)
#   - CMakeLists.txt, .gitignore, .claude/  (upstream tooling)

set -euo pipefail

ROOT="vendor/pineforge-corpus"

if [[ ! -d "$ROOT" ]]; then
    echo "no $ROOT to prune" >&2
    exit 1
fi

shopt -s nullglob

if [[ -d "$ROOT/data" ]]; then
    if [[ ! -f "$ROOT/data/ohlcv_spans.json" ]]; then
        echo "ERROR: $ROOT/data exists but ohlcv_spans.json is missing." >&2
        echo "Run scripts/bake-ohlcv-spans.py before pruning, or the trim" >&2
        echo "metadata for pine diff will be lost." >&2
        exit 1
    fi
    rm -f "$ROOT/data/"*.csv
fi
rm -rf "$ROOT/.claude"
rm -f "$ROOT/.gitignore"
rm -f "$ROOT/CMakeLists.txt"
rm -f "$ROOT/validation_report.html"
rm -f "$ROOT/validation_report.md"
rm -f "$ROOT/validation_report.pdf"

for probe in "$ROOT"/validation/*/; do
    rm -f "$probe/generated.cpp"
    rm -f "$probe/engine_trades.csv"
done

echo "pruned $ROOT"
du -sh "$ROOT"
