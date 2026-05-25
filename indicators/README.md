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
additionally require `pine_version` + `tv_snapshot`. `pine indicator --list`
and `pine version` report smoke vs TV counts separately so smoke fixtures
are not confused with oracle-grade TV captures. List mode accepts
`--grep TEXT` and `--baseline smoke|tv`; pass `--baseline ?` to list the
baseline catalog with counts.

`expect.json` may include `test_range.start` and `test_range.end` as RFC3339
timestamps. The strict comparer then checks only matching bars from
`bars.json`; expected arrays can be full-series length or already sliced to
that range. Output keys can be either the runner's generated keys (`plot`,
`plot#1`, `plotshape`, etc.) or a plot title such as `"Close Line"` when the
Pine call supplies one. Duplicate titles are disambiguated with the same
suffix style as generated keys (`Signal`, `Signal#1`, `Signal#2`, etc.).
Strict reports include `expected_output_keys` and `actual_output_keys` to make
key mismatches visible while authoring fixtures.

`pine indicator --strict --all` runs every matching fixture and exits non-zero
if any report fails. It accepts the same `--grep` and `--baseline` filters as
list mode, which keeps smoke-only CI checks separate from future TV-captured
fixture batches.

The `smoke-*` fixtures are deterministic substrate fixtures. They are not
TradingView captures; they pin fixture loading, piners-runner replay, output
slot naming, title-based output matching, duplicate-title disambiguation,
warmup token handling, ranged comparisons, bool `plotshape` output, and
same-symbol `request.security` replay. Real TV-captured baselines can be added
alongside them as data.
