# ta-isolate-08-rsi-and-macd-continuous-state

## Purpose
Mirror of `validation/37-regex-string-filter` with `str.match` REMOVED - `useRsi` and `useMacd` hard-coded true. Same branchy `longCond` formulation, same continuous-state entry/close pattern.

If 37-regex is moderate but this is excellent → bug in `str.match()` codegen or runtime. If this also drifts → bug in AND-combination of two value-cmp conds OR continuous-state pattern (compare with probe 06).

## Setup + export
See `../README.md`. Apply strategy.pine, export `tv_trades.csv`.

## Diagnostic interpretation
| 08 result | And given 06 + 07 results | Conclusion |
|---|---|---|
| `excellent` | 06+07 both `excellent` | 37-regex bug ISOLATED to engine `str.match()` - debug `pine_str_match` regex parse + return semantics |
| `strong/moderate` | 06+07 both `excellent` | AND of two value-cmp conds drifts (boundary cases). Engine evaluates `(a AND b)` differently from TV at boundary equality. |
| `strong/moderate` | 06 fails | Continuous-state pattern bug masks str.match question - fix probe 06 root cause first |
