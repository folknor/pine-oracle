# Evaluation: pineforge-engine

Subagent evaluation of `research/pineforge-engine/`. Date: 2026-05-24.

PineForge is the leader on TV parity: 227/228 reference strategies excellent (~313,000 trades), 100/100 on the public three-way benchmark (~167,000 TV trades) vs PyneCore/PineTS. Median 56x faster than PyneCore. Built in C++17. Apache-2.0. Static C ABI (10 symbols, 6 POD types, one header). **Critical caveat:** the engine is open-source, but the PineScript-to-C++ transpiler is CLOSED; it is only available via a hosted API (`@pineforge/codegen-mcp`). The repository ships `generated.cpp` files for the corpus strategies, not the transpiler.

PineForge is ~12.7k lines of dense, well-organised C++17 (`wc -l` on `include/` + `src/`) sitting behind a 10-symbol C ABI. It is the most TV-faithful PineScript backtest engine in public source.

## 1. What it does right

**The killer asset is empirically-derived broker semantics, not algorithms.** TV's `strategy.*` is undocumented in countless edge cases; PineForge has *paid the price of finding them*. Examples cited from the code:

- **Deterministic intra-bar path resolution** (`src/engine_path_resolve.cpp:17-68`): TV's broker emulator picks `O->H->L->C` vs `O->L->H->C` based on the *open's distance from H vs L*, not candle color. `bar_path_uses_high_first` encodes this. All fill priorities walk these 4 waypoints with parametric interpolation, including tie-breaking via `t_stop < t_limit`.
- **The dual-pass opposing-stop fill loop** (`src/engine_fills.cpp:43-78`) with `dual_entry_stop_path_winner` arbitration (`engine_path_resolve.cpp:206-251`), inc. the "long wins ties" rule (verified against probe 83).
- **Margin-at-signal-time vs fill-time** (`src/engine_strategy_commands.cpp:99-127`): the comment is a domain-knowledge fossil; TV rejects entries by `qty * signal_bar.close * margin/100`, not by fill-bar open. Engine drift was traced to community/IES through that distinction.
- **Intraday-cap LATCH semantics** (`engine.hpp:337-365`, `engine_fills.cpp:298-475`): two bugs (3459 vs 1957 trades, then 0 vs 382 cap-closes) led to the current "synthesize close + latch till chart-day rollover, even block placements" model.
- **`tv_carry_qty` deferred-flip rule** (`engine.hpp:130-172`, `engine_strategy_commands.cpp:156-180`): captures position size at order *placement* for re-place-aware carry, with `pending_close_qty_in_bar_` accumulating same-bar `strategy.close` for source-order arithmetic.
- **Directional mintick rounding** (`engine.hpp:468-495`): long-stop snaps up (ceil), short-stop snaps down (floor), with a 1e-9 boundary nudge. v0.3 fix.
- **`process_orders_on_close`** alternative bar pump (`src/engine_run.cpp:52-66`): four-step "fill old -> update extremes -> on_bar -> fill new market" mirrors TV.
- **`process_orders_on_close` magnifier interplay** (`engine_run.cpp:114-175`): the `is_first_tick_/is_last_tick_` discipline and forcing `is_first_tick_=true` on the last tick so series history only advances once.
- **Real-bar vs synthetic-bar magnifier** (`engine_run.cpp:99-137`): when multiple input sub-bars per script bar are present, forces `ENDPOINTS+4` regardless of user choice. Synthetic ticks inside a real 1m bar are correctly identified as noise that cannot recover missing information.
- **ABI hygiene** (`src/c_abi.cpp:25-102`): `static_assert` pins every POD field offset and enum value against the internal C++ types; `-fvisibility=hidden` keeps `BacktestEngine` out of strategy `.so` symbol tables.

The split-by-concern source layout (engine_orders / engine_fills / engine_path_resolve / engine_strategy_commands / engine_security / engine_run) is genuinely good. After v0.1's "phase 6/7 split" the largest TU is `engine_fills.cpp` at 920 lines; navigable.

## 2. What it does wrong or badly

- **The closed transpiler is a structural defect for any FFI consumer.** The C ABI declares `strategy_create` / `run_backtest` as exports *of each compiled strategy `.so`*, not the runtime (see `pineforge.h:197-208` and `c_abi.cpp:110-115`). You cannot run anything without `generated.cpp`, and the only way to get one is the hosted API. Corpus ships pre-generated `.cpp` files; the public surface ends there. For piners this means: the engine alone is not a usable backend for arbitrary Pine source.
- **C++ history-buffer choice:** `Series<T>` uses `std::deque` (`include/pineforge/series.hpp:20`). For per-bar `[k]` access this is fine but cache-hostile vs a fixed ring buffer with circular indexing. TA classes mix `std::deque` (SMA), `RMA`-style scalar state, and ad-hoc saved/restore for `recompute`. Inconsistent.
- **The "save/restore for recompute" pattern** (e.g. `ta.hpp:18-29, 38-39, 87-103`) is a hand-rolled, type-specific snapshot for the rerun-same-bar idiom (magnifier intra-bar). Every stateful indicator needs bespoke save/restore. In Rust, this is naturally expressed by `Clone` on a typed state struct; PineForge re-invents that per class.
- **TZ handling reaches for a process-global mutex** (`pineforge.h:328-339`, `engine.hpp:289-293`). The cost is real: any non-UTC chart timezone serialises bar decomposition across threads. A parallel parameter sweep on the same TZ chart loses parallelism. The lazy day-rollover read of intraday cap state goes through the same mutex (`_intraday_cap_currently_latched`, `engine.hpp:355-365`).
- **Eigen** as a hard dep (`README:172`, `include/pineforge/generic_matrix.hpp`) for *matrix-typed Pine*; almost no strategies use this. Heavy build dep for a feature most users won't touch. Rust would naturally make this optional behind a feature flag.
- **Pyramid book is `std::vector<PyramidEntry>` with O(n) `remove_if` partial closes** (`engine_orders.cpp:166-178, 200-228`). Fine at typical pyramid depths but the per-bar reconstruction of `remaining` vectors is allocation-heavy in tight loops.
- **No public way to plug in a custom strategy class** without going through their closed transpiler; `BacktestEngine` is the runtime-internal base class that `generated.cpp` extends. The "internal C++ headers" disclaimer in `pineforge.h:38-40` and `README:233-234` means even using the engine in-process from C++ is officially unsupported.
- **Numeric reproducibility caveats:** `-ffp-contract=off` is added in v0.2 but bit reproducibility across compilers/CPUs is NOT promised. The README's "bit-reproducible" claim only holds within a single build.
- **C++ exception -> empty string at C boundary** (`c_abi.cpp:135-138`). Callers must poll `strategy_get_last_error` after every run. Forgettable; a result type would be safer.

## 3. What it could do better (within structure)

- Replace the chart-TZ mutex with a thread-local `std::tm`-equivalent decomposition (or a precomputed per-bar `BarTime` slab) so multi-threaded sweeps don't serialise.
- Move `Series<T>` to a fixed-capacity circular buffer; eliminate `std::deque` from every TA class. ~15% TA cost win at minimum.
- Make Eigen optional behind a CMake flag (`PINEFORGE_WITH_MATRIX=OFF`); only ~5% of corpus probes touch matrix.
- Replace per-class `save/restore` with a `Snapshot` POD per indicator; current scheme is footgun-prone if a maintainer adds field 12 to a class with 11 saved fields.
- Provide a documented C++ extension point (a `IStrategy` virtual interface) so non-transpiler consumers could write strategies in C++ directly. The runtime is good enough to deserve that, and it would remove the soft "internal only" disclaimer on `engine.hpp`.

## 4. Decision for piners: option (a), port to Rust from scratch, using PineForge as the design reference

Not (b), not (c), not a hybrid.

**Why not (b), FFI to libpineforge.a:** The C ABI is well-designed, but the runtime-side exports are only `strategy_set_trace_enabled`, `pf_version_get`, `strategy_set_chart_timezone`, `strategy_set_trade_start_time`, and `strategy_get_last_error` (see `c_abi.cpp:113-167`). Every other C symbol, including the lifecycle ones (`strategy_create`, `run_backtest`), is emitted by codegen into the strategy `.so`. The "stable C ABI" markets a contract whose other half lives behind a hosted closed-source service. To use the engine via FFI you would have to either (i) call the hosted API for every Pine source (network dep, rate limits, vendor lock-in, "OHLCV never leaves your machine but your IP source does"), or (ii) write your own Pine-to-C++ transpiler emitting against the *internal* C++ headers (`include/pineforge/engine.hpp`'s 1200-line `BacktestEngine` class with protected members and undocumented contract). That's most of the work of writing a Rust transpiler against a Rust engine, plus you also own a C++ FFI surface, plus you lose Rust-native determinism guarantees. Bad trade.

**Why not (c), ignore PineForge:** the parity work is irreplaceable. The comment density in `engine_fills.cpp`, `engine_strategy_commands.cpp`, and `engine_path_resolve.cpp` cites probe-specific empirical findings (probes 52, 62, 63, 72, 83, 92, 93, 95, 96, 97, 97b...). That knowledge took ~228 strategies and ~313k trades to extract. Re-deriving it from black-box TV would cost you the same. Use PineForge as the *executable specification*.

**Why (a):** the codebase reads like a TV parity textbook. Port the algorithms (path resolution, dual-pass fill loop, deferred-flip carry, intraday-cap latch, directional mintick rounding, magnifier sampling, `process_orders_on_close` semantics) into idiomatic Rust where:

- `Series<T>` becomes a `Vec`-backed fixed ring with O(1) `[k]`.
- save/restore becomes `#[derive(Clone)]` on typed state.
- Chart-TZ becomes a thread-local `chrono_tz::Tz`-decomposed `BarTime`.
- Strategy authoring becomes a Rust trait, with the Pine-to-Rust transpiler emitting `impl Strategy for FooStrategy`. No second language at the seam.
- The corpus's `generated.cpp` + `tv_trades.csv` + `engine_trades.csv` is your *test oracle*: every probe you can read is a trade-list you can diff against in CI.

Concretely:

1. Vendor `corpus/` as your validation suite from day 1; trade-list CSVs are language-agnostic.
2. Implement the engine first (no transpiler), with hand-written Rust strategies translated from each probe's `.pine`. Hit parity against `tv_trades.csv` strategy-by-strategy.
3. Once you have 50+ hand-translated probes passing trade-for-trade, write the Pine-to-Rust transpiler. The hand-translations *are* the test corpus for the transpiler.
4. Closed-transpiler problem solved by being open from the start; your transpiler emits Rust source you can read, diff, and unit-test, not closed-cloud opaque output.

**Cost estimate** (eye-balling 12.7k C++ lines, much of it comments + repetitive TA): ~6-8k Rust LOC for the engine, ~3-5k for transpiler/AST. The author has shipped pbfhogg and dellingr; language-runtime fluency is not in question. The bottleneck will be probe-by-probe parity work, not Rust ergonomics. Budget for that: weeks, not days. But the *direction* is unambiguous: port (a), use PineForge as the spec, keep the corpus as ground truth.

## Key files to mirror first

- `engine_path_resolve.cpp`
- `engine_fills.cpp` (process_pending_orders + apply_filled_order_to_state)
- `engine_strategy_commands.cpp` (strategy_entry's carry handling)
- `engine_run.cpp` (run + run_magnified_bar)
- `magnifier.cpp` (sample_price_path)
- `engine_security.cpp` and the `ta_*.cpp` files port last; they are more mechanical.
