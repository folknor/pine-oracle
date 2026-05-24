# ta-isolate-11 - mask only, no transpose

## Purpose

Tier-3 isolation probe for the `typed-matrix-01` engine bug (TV=774, eng=785, +11 extra trades). The bug manifests as matrix gating being a no-op in the engine regardless of whether transpose is applied.

This probe reads from `mask` directly, skipping the double-transpose round-trip present in `typed-matrix-01`. If the bug disappears here (engine matches TV), the fault lies in the transpose path. If the bug persists, transpose is innocent and the culprit is elsewhere (hotCount loop or mask.get/set).

## What is changed vs typed-matrix-01

- `sample` is read from `mask` directly (no `mT`/`mTT` intermediaries)
- `hotCount` loop is retained and gates `entryCond` (`hotCount >= 6`)
- All other logic is identical to `typed-matrix-01`

## Diagnostic decision table

Use in conjunction with probe 12 (`ta-isolate-12-sample-only-no-hotcount`).

| probe-11 result | probe-12 result | Conclusion                                      |
|-----------------|-----------------|-------------------------------------------------|
| pass (TV match) | pass (TV match) | Bug only when BOTH transpose + hotCount active  |
| pass (TV match) | fail (+delta)   | Bug isolated to hotCount loop                   |
| fail (+delta)   | pass (TV match) | Bug isolated to transpose round-trip            |
| fail (+delta)   | fail (+delta)   | Bug in mask.set or mask.get (shared path)       |

## Data

Default: `data/ohlcv_ETH-USDT-USDT_15m_warmup6m.csv`
