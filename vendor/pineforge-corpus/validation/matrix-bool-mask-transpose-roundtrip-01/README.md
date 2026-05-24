# ta-isolate-12 - sample only, no hotCount

## Purpose

Tier-3 isolation probe for the `typed-matrix-01` engine bug (TV=774, eng=785, +11 extra trades). The bug manifests as matrix gating being a no-op in the engine regardless of whether transpose is applied.

This probe retains the double-transpose round-trip from `typed-matrix-01` but removes the `hotCount >= 6` gate. Entry fires on `sample and rsiVal > 55.0` only. If the bug disappears here (engine matches TV), the fault lies in the hotCount accumulation loop. If the bug persists, hotCount is innocent and the culprit is in the transpose path or mask.get/set.

## What is changed vs typed-matrix-01

- `hotCount` loop and its `>= 6` gate are removed from `entryCond`
- `entryCond = sample and rsiVal > 55.0`
- `mT` / `mTT` double-transpose retained; `sample` still reads from `mTT`
- All other logic is identical to `typed-matrix-01`

## Diagnostic decision table

Use in conjunction with probe 11 (`ta-isolate-11-mask-only-no-transpose`).

| probe-11 result | probe-12 result | Conclusion                                      |
|-----------------|-----------------|-------------------------------------------------|
| pass (TV match) | pass (TV match) | Bug only when BOTH transpose + hotCount active  |
| pass (TV match) | fail (+delta)   | Bug isolated to hotCount loop                   |
| fail (+delta)   | pass (TV match) | Bug isolated to transpose round-trip            |
| fail (+delta)   | fail (+delta)   | Bug in mask.set or mask.get (shared path)       |

## Data

Default: `data/ohlcv_ETH-USDT-USDT_15m_warmup6m.csv`
