# ta-isolate-09-rsi14-gt60-lt45-no-matrix

## Purpose
`validation_typed_matrix/typed-matrix-probe-01-bool-regime-mask` minus the matrix layer. Same RSI > 55 entry, RSI < 45 exit, same `position_size == 0` / `> 0` guards. NO matrix.new<bool>, NO matrix.set/get/transpose, NO hotCount counter.

If this is excellent and typed-matrix-01 still drifts → bug isolated to the matrix bool dispatch (matrix.new<bool>, matrix.set/get/transpose for bool element type, or the hotCount accumulation pattern).

If this drifts too → bug in RSI value-comparison entry/exit pattern with explicit `position_size` guards (different from probes 01/02/06 which don't use those guards).

## Setup + export
See `../README.md`. Apply strategy.pine, export `tv_trades.csv`.

## Diagnostic interpretation
| 09 result | Conclusion |
|---|---|
| `excellent` | typed-matrix-01 drift isolated to matrix bool dispatch. Inspect `PineGenericMatrix<bool>` engine impl for set/get/transpose round-trip correctness, and the hotCount loop accumulator. |
| `strong/moderate` | RSI > 55 / RSI < 45 entry/exit pair with `position_size` guards drifts vs TV. Inspect engine `strategy.position_size` reporting timing within bar. |
