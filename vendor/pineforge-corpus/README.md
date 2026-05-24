# PineForge validation corpus

The corpus is PineForge's reproducibility kit for the parity claim in the
project README. Every probe is a hand-written, clean-room PineScript v6
strategy paired with TradingView's exported trade list and PineForge's own
trade list, so a third party can diff the two CSVs and confirm engine
behaviour matches TradingView on the same bar feed.

## Headline parity

- **228** verified strategies, all under `corpus/validation/`.
- **~313,000 trades** total across the suite (TV: 312,829; engine: 312,920 - `+91` ≈ 0.03 % over TV).
- **227** excellent (bit-for-bit or within strict thresholds on every
  parity dimension).
- **1** documented anomaly - `anomaly-equity-mirror-strategy-equity-01` -
  where TradingView's broker emulator exhibits non-deterministic
  accept/reject behaviour at the exact 1× equity margin boundary. Engine
  is deterministic and correct; full write-up lives in
  `pineforge-utils/parity-anomalies/tv-margin-boundary.md`.
- **0** strong / moderate / weak.

The canonical, regenerated-each-sweep disposition table is
[`validation_report.md`](validation_report.md) (rendered as
`validation_report.html` and `validation_report.pdf`).

## Artifact tuple

Each probe directory ships four files in git:

| File                | Source                       | Role                                                       |
| ------------------- | ---------------------------- | ---------------------------------------------------------- |
| `strategy.pine`     | hand-written                 | PineScript v6 source                                       |
| `generated.cpp`     | pineforge-codegen transpiler | C++ output of the transpiler over `strategy.pine`          |
| `tv_trades.csv`     | TradingView export           | TV broker emulator's trade list for `strategy.pine`        |
| `engine_trades.csv` | PineForge                    | Engine's trade list for the same script (TV-format CSV)    |

`generated.cpp` is the transpiler output of our own clean-room
PineScript and ships under the same Apache-2.0 license as
`strategy.pine`. It is included in-tree so public users can rebuild
without needing access to the closed-source `pineforge-codegen`
transpiler - `cmake --build build --target corpus_strategies` compiles
each `generated.cpp` into a per-strategy shared library. The compiled
`strategy.dylib` / `.so` / `.dll` are platform-specific build artefacts
and remain ignored.

## Reference OHLCV

The comparison feed is **Binance ETH-USDT-USDT 15-minute bars**, with an
optional 1-minute companion for `bar_magnifier` and lower-timeframe probes:

- `data/ohlcv_ETH-USDT-USDT_15m.csv` - primary 15m bar feed.
- `data/ohlcv_ETH-USDT-USDT_15m_warmup6m.csv` - same window plus six
  months of leading warmup bars; used by default when present so TA, MTF,
  pivot, and equity-feedback state starts closer to TradingView's chart
  state.
- `data/ohlcv_ETH-USDT-USDT_1m.csv` and `..._1m_warmup6m.csv` - 1m feed
  for `magnifier-*` and `ltf-*` probes.

## Layout

```
corpus/
├── validation/                228 probes - surface-driven probe family
│   ├── ta-*                    50 probes - TA built-in math (rsi, macd, sma, ...)
│   ├── order-*                 40 probes - entry/exit/cancel placement
│   ├── bracket-*               13 probes - TP/SL via strategy.exit / strategy.order
│   ├── oca-*                    3 probes - OCA group cancel/reduce/none
│   ├── cap-*                    2 probes - intraday cap (max_intraday_filled_orders)
│   ├── pyramid-*                4 probes - pyramiding=N
│   ├── mtf-*                   15 probes - request.security regular HTF
│   ├── ltf-*                    2 probes - request.security_lower_tf arrays
│   ├── magnifier-*              3 probes - bar_magnifier sub-bar walks
│   ├── barstate-*               2 probes - barstate.* checks
│   ├── varip-*                  1 probe  - varip semantics
│   ├── recompute-*              2 probes - calc_on_every_tick / TA recompute
│   ├── na-*                     2 probes - na propagation
│   ├── udt-*                   22 probes - user-defined types + methods
│   ├── matrix-*                 6 probes - matrix<T> typed/generic
│   ├── session-*                2 probes - session() / TZ / DST
│   ├── composite-*             52 probes - multi-surface integration (community-style)
│   ├── analyzer-*               6 probes - engine analyzer / parity isolation
│   └── anomaly-*                1 probe  - documented TV non-determinism
├── data/                       reference OHLCV (Binance ETH-USDT-USDT 15m + 1m)
├── LICENSE                     Apache-2.0
├── NOTICE                      attribution
├── LEGAL.md                    provenance / trademarks
├── README.md                   this file
├── CMakeLists.txt              per-strategy .so build glob
├── .gitignore                  ignores compiled strategy.dylib/.so/.dll only
├── validation_report.md        canonical parity disposition, regenerated each sweep
└── validation_report.{html,pdf}   rendered from .md
```

Total: **52 + 50 + 40 + 22 + 15 + 13 + 6 + 6 + 4 + 3 + 3 + 2 + 2 + 2 + 2 + 2 + 2 + 1 + 1 = 228** probes.

## Naming convention

Every probe directory follows:

```
<category>-<descriptive-slug>-NN[a-z]?
```

- **`<category>`** - one of the 19 surface categories below. The category
  is the engine surface or PineScript feature the probe is built to
  exercise.
- **`<descriptive-slug>`** - kebab-case description of the specific
  behaviour under test (e.g. `atr-trail-series-int-points`,
  `kalman-filter-1d`, `bb-kc-squeeze-release`).
- **`NN`** - two-digit sequence number, used to disambiguate when more
  than one probe lands on the same `(category, slug)` pair.
- **`[a-z]?`** - optional letter suffix, used **only** for documented
  A/B variant pairs that share the same numeric slot (e.g.
  `barstate-isconfirmed-magnifier-on-01a` vs
  `…-magnifier-off-01b`).

The 19 categories (with probe counts):

| Category    | Count | Surface exercised                                          |
| ----------- | ----: | ---------------------------------------------------------- |
| `composite` |    52 | Multi-surface integration probes (community-style scripts) |
| `ta`        |    50 | TA built-in math (rsi, macd, sma, hma, …)                  |
| `order`     |    40 | Entry/exit/cancel order placement                          |
| `udt`       |    22 | User-defined types + methods                               |
| `mtf`       |    15 | `request.security` regular HTF                             |
| `bracket`   |    13 | TP/SL via `strategy.exit` / `strategy.order`               |
| `matrix`    |     6 | `matrix<T>` typed and generic                              |
| `analyzer`  |     6 | Engine analyzer / parity isolation                         |
| `pyramid`   |     4 | `pyramiding=N`                                             |
| `oca`       |     3 | OCA group cancel / reduce / none                           |
| `magnifier` |     3 | `bar_magnifier` sub-bar walks                              |
| `session`   |     2 | `session()` / TZ / DST                                     |
| `recompute` |     2 | `calc_on_every_tick` / TA recompute                        |
| `na`        |     2 | `na` propagation                                           |
| `ltf`       |     2 | `request.security_lower_tf` arrays                         |
| `cap`       |     2 | Intraday cap (`max_intraday_filled_orders`)                |
| `barstate`  |     2 | `barstate.*` checks                                        |
| `varip`     |     1 | `varip` semantics                                          |
| `anomaly`   |     1 | Documented TV non-determinism                              |

## Where the numbers come from

The headline figure is produced by the verifier sweep that emits
[`validation_report.md`](validation_report.md). That report is the
authoritative disposition for every probe - tier, profile, per-dimension
deltas, anomaly verdicts.

The full pipeline (build + run + verify across the whole tree) is one
command:

```bash
JOBS=8 scripts/run_corpus.sh
```

That script:

1. Configures CMake with `-DPINEFORGE_BUILD_CORPUS_STRATEGIES=ON`.
2. Builds `libpineforge.a` plus one `strategy.so` per probe via
   `cmake --build build --target corpus_strategies`.
3. Loads each `strategy.so` through `scripts/run_strategy.py`, runs it
   against `corpus/data/ohlcv_ETH-USDT-USDT_15m.csv` (or the warmup
   variant when present), and writes `engine_trades.csv` next to the
   probe.
4. Runs `scripts/verify_corpus.py --all` to produce the report.

## Reproducing parity locally

No transpiler access required - `generated.cpp` ships in-tree.

```bash
# 1. Clone the engine and pull this corpus submodule
git clone https://github.com/fullpass-4pass/pineforge-engine.git
cd pineforge-engine
git submodule update --init corpus

# 2. Build all per-strategy .so files, run them, and verify
JOBS=8 scripts/run_corpus.sh
```

You need the engine repo, this corpus, and a C++17 compiler. The engine
is deterministic given a fixed bar feed, the shipped `generated.cpp`,
and a fixed runtime build. If a probe's rebuilt `engine_trades.csv`
disagrees with the committed copy, that is a bug - please open an issue.

## CSV format

Both `tv_trades.csv` and `engine_trades.csv` use TradingView's row layout:

- **Two rows per trade**, sharing the same `Trade #`. The exit row is
  emitted before the entry row (TV convention; PineForge mirrors it for
  direct diff).
- **Reverse-chronological by trade number** (newest first).
- **Time format**: `YYYY-MM-DD HH:MM`. Engine CSVs are UTC. TradingView
  exports use the chart's wall-clock timezone; this corpus defaults to
  UTC+8 unless a probe `inputs.json` overrides `tv_trades_csv_tz`.

`tv_trades.csv` (TradingView's actual export):

```
Trade #,Type,Date and time,Signal,Price USDT,Position size (qty),...
14,Exit long,2026-04-27 20:30,TPSL,2291.52,1,2291.52,...
14,Entry long,2026-04-27 20:15,…,2289.7,1,2289.7,…
```

`engine_trades.csv` (PineForge's mirrored format, fewer columns -
PineForge does not currently emit TV's "Signal" tag or percent-of-position
MFE/MAE):

```
Trade #,Type,Date and time,Price,Qty,Net PnL,Net PnL %,MFE,MAE,Cumulative PnL
14,Exit long,2026-04-27 20:30,2291.520000,1,1.820000,0.0795,2.250000,0.160000,-38.120000
14,Entry long,2026-04-27 20:15,2289.700000,1,1.820000,0.0795,2.250000,0.160000,-38.120000
```

`Net PnL` and `Net PnL %` are per-trade. `Cumulative PnL` is the
engine-side running total. `MFE`/`MAE` are dollars per unit qty,
mirroring TV's "Favorable / Adverse excursion USD".

## Parity thresholds

The verifier (`scripts/verify_corpus.py`) applies one of two threshold
profiles per probe and emits a tier label:

### Profiles

| Dimension                  | STRICT  | PRODUCTION |
| -------------------------- | ------: | ---------: |
| Trade-count delta          |   1.0%  |       1.0% |
| Entry-price p90 delta      |   0.01% |      0.01% |
| Exit-price p90 delta       |   0.01% |      0.05% |
| Per-trade P&L p90 delta    |   1.0%  |       1.0% |

PRODUCTION relaxes only the exit-price tolerance (5×) to absorb sub-bar
broker-side fill drift on probes that use `strategy.exit`. The verifier
auto-detects `strategy.exit` in `strategy.pine` and selects PRODUCTION
for those probes; everything else stays on STRICT.

A trade is "matched" when engine and TV agree on direction and entry/
exit times fall within a 1-hour gating window (plus a $3 entry-price
gate to defend against same-bar duplicates). The PnL p90 calc applies a
near-zero filter (`|tv_pnl| > $0.01`) to avoid div-by-near-zero blow-up
on TV's magnifier zero-PnL trades.

### Tier labels

| Tier          | Meaning |
| ------------- | ------- |
| `excellent`   | All four dimensions pass the resolved profile. Bit-for-bit or within strict-profile thresholds. |
| `strong`      | Dimensions pass a relaxed envelope (5× thresholds) - close but not excellent. Used as a pass-with-caveat tier. |
| `moderate`    | Some dimensions exceed the strong envelope but trades still align meaningfully. Investigate. |
| `weak`        | Significant divergence. Real bug or probe-design issue. |
| `minimal`     | Probe produces zero engine trades or zero TV trades - nothing to compare. |
| `anomaly`     | Engine output is correct per Pine spec; TV is non-deterministic on this probe. Documented per-probe via `inputs.json::expected_tier: "anomaly"` plus a `notes` field with the deep-analysis write-up. Excluded from headline excellent count. Currently 1 probe (`anomaly-equity-mirror-strategy-equity-01`). |
| `engine_only` | Engine produces correct trades that intentionally diverge from TV (e.g., engine fires a bar TV's broker emulator silently drops). Documented per-probe via `inputs.json::validation_overrides::expect_tv_match: false` plus an `expect_tv_match_reason` write-up. Currently 0 probes. |
| `missing`     | Required artefact (TV CSV or engine CSV) absent. Should never appear in committed state. |

The `anomaly` and `engine_only` overrides only fire when the computed
tier would be below `excellent` - a future engine fix that lifts a
documented divergence to bit-for-bit match still reports as `excellent`,
not silently masked.

## Publishing posture

The corpus is published under **Apache-2.0**, matching the engine. Every
`strategy.pine` is a clean-room PineForge original - no third-party
PineScript is redistributed. TradingView trade-list CSVs are factual
records of running each script on TV's broker emulator, included only for
parity verification. OHLCV is public market data from Binance USDT-M
futures. See [`LEGAL.md`](LEGAL.md) for the full provenance and trademark
notes.
