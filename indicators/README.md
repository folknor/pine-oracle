# Indicator strict fixtures

This directory is embedded into the `pine` binary and is the fixture root for
`pine indicator --strict <slug>`.

Each fixture lives under `indicators/<slug>/` with:

- `source.pine`
- `bars.json`
- `expect.json`
- optional `metadata.json`

`metadata.json`'s `baseline` field must be `"smoke"` (deterministic
substrate checks) or `"tv"` (TradingView-captured baselines). There is no
third tier; missing `metadata.json` defaults to `"smoke"`. TV baselines
additionally require `pine_version` + `tv_snapshot`; smoke fixtures must not
define `tv_snapshot`. `pine indicator --list` and `pine version` report smoke
vs TV counts separately so smoke fixtures are not confused with oracle-grade TV
captures. List mode accepts `--grep TEXT` and `--baseline smoke|tv`; pass
`--baseline ?` to list the baseline catalog with counts.

`expect.json` may include `test_range.start` and `test_range.end` as RFC3339
timestamps. The strict comparer then checks only matching bars from
`bars.json`; expected arrays can be full-series length or already sliced to
that range. Output keys can be either the runner's generated keys (`plot`,
`plot#1`, `plotshape`, etc.) or a plot title such as `"Close Line"` when the
Pine call supplies one. Duplicate titles are disambiguated with the same
suffix style as generated keys (`Signal`, `Signal#1`, `Signal#2`, etc.).
Strict reports include `expected_output_keys` and `actual_output_keys` to make
key mismatches visible while authoring fixtures. Every expected output key must
be non-empty, and every expected series must contain at least one value; strict
fixture validation rejects empty keys and empty series before running parity.
Tolerance must be finite, non-negative, and no greater than `0.001`. For
recognized fixed timeframes, adjacent bar timestamps must not be shorter than
the timeframe. Fixed `M` month timeframes use a conservative 27-day minimum
spacing check because calendar months vary.

`pine indicator --strict --all` runs every matching fixture and exits non-zero
if any report fails. It accepts the same `--grep` and `--baseline` filters as
list mode, which keeps smoke-only CI checks separate from future TV-captured
fixture batches.

`pine indicator <slug>` is the fixture authoring view. It does not run value
parity, but it does run the fixture once to report actual runner output keys
beside expected keys. Add `--metadata-only` to skip that runner key check for a
cheap source/bars/expect metadata read. It prints the embedded `source.pine`,
bar count/window, expected output keys, expected value counts, first/last
expected values, tolerance, notes, and baseline metadata. Use JSON output when
another tool needs the same detail in structured form.

`pine indicator <slug> --actual` runs the fixture once through piners-runner
and prints actual output keys + series without comparing against `expect.json`.
Use it when authoring deterministic smoke fixtures or checking whether a
runner/output-key change altered what a fixture would capture. JSON output also
contains `runner_expect`, an `expect.json`-shaped object with the actual runner
outputs. `runner_expect` intentionally omits fixture metadata such as
`pine_version` and `test_range`, and uses zero tolerance.

The `smoke-*` fixtures are deterministic substrate fixtures. They are not
TradingView captures; they pin fixture loading, piners-runner replay, output
slot naming, title-based output matching, duplicate-title disambiguation,
warmup token handling, ranged comparisons, bool `plotshape` output, and
same-symbol `request.security` replay. Real TV-captured baselines can be added
alongside them as data.
