# Local vendoring notes

This tree is a snapshot of <https://github.com/fullpass-4pass/pineforge-corpus>
(Apache-2.0, see LICENSE / NOTICE alongside this file) pruned for embedding
into the `pine` binary.

## What was kept

- `LICENSE`, `NOTICE`, `LEGAL.md`, `README.md` (upstream attribution and docs).
- `validation/<slug>/strategy.pine` -- the canonical Pine v6 source for each probe.
- `validation/<slug>/tv_trades.csv` -- TradingView's exported trade list, the
  ground truth that probes are validated against.
- `validation/<slug>/inputs.json` (where present) -- per-probe configuration
  (`expected_tier`, `runtime_overrides`, etc.).

## What was dropped

- `.git/` -- pine-oracle vendors a snapshot, not a tracked submodule.
- `data/` (~75 MB of Binance ETH/USDT OHLCV across 4 feeds) -- not currently
  embedded. Required when `pine indicator --strict` and `pine diff` v1 land;
  re-add by re-cloning and re-running `scripts/prune-vendored-corpus.sh` with
  the data step skipped.
- `validation/<slug>/generated.cpp` -- PineForge transpiler output; not used
  by pine-oracle.
- `validation/<slug>/engine_trades.csv` -- PineForge's own engine output;
  pine-oracle only consumes the TV ground truth.
- `validation_report.{html,md,pdf}` -- output artefacts, not inputs.
- `CMakeLists.txt`, `.gitignore`, `.claude/` -- upstream tooling.

## Refreshing the vendor

1. `rm -rf vendor/pineforge-corpus`
2. `git clone --depth 1 https://github.com/fullpass-4pass/pineforge-corpus.git vendor/pineforge-corpus`
3. `rm -rf vendor/pineforge-corpus/.git`
4. `./scripts/prune-vendored-corpus.sh`
