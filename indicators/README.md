# Indicator strict fixtures

This directory is embedded into the `pine` binary and is the fixture root for
`pine indicator --strict <slug>`.

Each fixture lives under `indicators/<slug>/` with:

- `source.pine`
- `bars.json`
- `expect.json`
- optional `metadata.json`

No TradingView indicator baselines are baked yet. The runner and diffing
substrate is implemented so real baselines can be added as data.
