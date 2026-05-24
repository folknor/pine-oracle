# Evaluation: PineTS

Subagent evaluation of `research/PineTS/`. Date: 2026-05-24.

PineTS is a TypeScript Pine Script v5/v6 transpiler + runtime by LuxAlgo, dual-licensed AGPL-3.0 / Commercial. It runs Pine source in Node.js / browsers / any JS runtime. Two-stage transpilation: Pine -> JavaScript-IR (with PineTS API calls) -> executable low-level JS. Indicator coverage is strong; strategy backtester is roadmap (the README's status table shows strategy backtesting engine as in-progress). The README and namespace docs are the best-documented semantics catalog in the entire research corpus.

## 1. What it gets right

PineTS demonstrates a real, hard-won understanding of Pine v5/v6 semantics that a from-scratch implementation will absolutely miss in v1. The crown jewels:

**The semantics catalog itself** (`research/PineTS/src/namespaces/README.md`) is the most valuable artifact in the entire research corpus. It is not aspirational documentation; it is a forensic field guide to seven distinct Pine quirks (`param()`, namespaces-as-functions, dual-use identifiers, tuple returns, epsilon equality, dual-getter/function properties, per-call-site state ID injection) with explicit file pointers to every implementation. Read it three times.

**Per-call-site state isolation via `_taN` IDs.** `ExpressionTransformer.ts` injects a synthetic last argument `'_ta0'`, `'_ta1'`, ... into every `ta.*` call. The recipient (e.g. `src/namespaces/ta/methods/sma.ts:13` uses `stateKey = _callId || ...`) keys its rolling state on that id. This is the only correct way to handle `ta.sma(close,20)` appearing twice in the same script with separate windows. A naive runtime keyed on function-name+args silently merges them.

**The `param()` shim, three flavors.** `src/namespaces/ta/methods/param.ts:5-27` materializes a scalar argument into a single-cell `context.params[name]` series so that `ta.sma(close, 14)` can treat `14` as a Series of length 1 with the same `.get(i)` API as `close`. The state-creating (`ta`/`math`), simple-unwrap (`input`/`array`/`map`/`matrix`), and tuple-tracking (`request`) variants are documented in `namespaces/README.md` Pattern #3. Bonus: `ExpressionTransformer.ts:26-44` prefixes the `pN` slot name with the local call-path id `$$` when inside a function scope so two callers of the same Pine function don't clobber each other's params. This bug exists by default in any naive approach; PineTS has the fix in tree.

**Dual-use identifiers (`na`, `time`).** `Core.ts:75-99` NAHelper + the `__value` rewrite documented at `namespaces/README.md` lines 376-411 is the cleanest design I have seen: a bare `na` is `na.__value`, lookback `na[1]` is `$.get(na.__value, 1)`, call `na(x)` is `na.any(x)`. Seven transformer functions are listed; that's the actual surface area, and it's an enumerated list rather than a sprawling special-case.

**Epsilon equality.** `src/namespaces/math/methods/__eq.ts` is six lines and `MainTransformer.ts:174` rewrites every `==` post-parse via `ASTFactory.ts:137`. The implementation uses `1e-9` (not 1e-8 as the docs claim; minor doc drift). NaN==NaN returns `false`, matching IEEE 754, not the docs' claim of `true`; the README is wrong about its own implementation. Worth flagging; the docs are aspirational here.

**Two-stage transpilation.** Pine -> JS-IR -> low-level JS (`transpiler/pineToJS/README.md`). The IR is real, executable JavaScript using the PineTS API; the second stage rewires it for the runtime (variable scoping into `$.let.*`, `$.get(...)` series access, callsite ID injection, factory-method deferral via `FACTORY_METHODS` in `settings.ts:60`). This separation is correct and worth preserving conceptually in Rust; Stage 1 is a syntax shim, Stage 2 is the actual compiler.

**Compatibility test harness.** `tests/compatibility/README.md` describes a `.pine.ts` indicator -> `.expect.json` baseline workflow with custom NaN/Infinity/undefined serialization (`__NaN__`/`__Infinity__`/`__-Infinity__`/`__undefined__` tokens). The architecture is sound: indicators are checked in as source, baselines are regenerated, the serializer is the only nontrivial piece. **This format should be lifted wholesale** as the cross-validation harness for piners-vs-TradingView. The mock data range (BTCUSDC daily 2025-01-01 -> 2025-11-20, test range filtered to 2025-10-01 -> 2025-11-20) is pre-baked so tests are deterministic and offline.

## 2. What it does wrong / badly

**v6 is "experimental" because v6 is not really there.** The README claims v5/v6 support, but the roadmap lists Pine Script v6 full compatibility as not-started and strategy backtesting engine as in-progress. There is **no `strategy/` namespace directory** under `src/namespaces/`. The `strategy` token appears only as a string in collision-avoidance lists (`settings.ts:84`). For piners, whose killer feature is strategy parity with TradingView, **PineTS contributes essentially zero strategy code**. You are writing the entire broker simulator, order book, position tracker, equity curve, and trade list from scratch regardless of which option you choose.

**JS-isms that don't translate.** The whole `Series.from()` discipline (`src/Series.ts:27-34`), auto-wrapping scalars in `new Series([source])`, is a workaround for JS's lack of types. In Rust, `Series<f64>` vs `f64` are statically distinguished; the entire `param()` Type-B unwrap pattern collapses into normal generic function arguments. The `__value` property hack for dual-use identifiers becomes an `enum NaOrFn { Value, Call(...) }` or two separate symbols in the symbol table. Roughly **40% of the transpiler's complexity is shadow-typing JS**.

**`sma.ts` is ugly.** Look at lines 67-123: a fast-path / slow-path branch with a mutating `useFastPath` flag, hand-rolled NaN contamination tracking, three layers of nested `if (useFastPath)`. The comments include "Actually, if prevSum was Number..."; running interior monologue. This is incremental-rolling-sum logic that should be 15 lines of clean Rust with `Option<f64>` for NaN tracking. Don't port this code.

**Prototype-based dispatch + auto-generated index files** (`namespaces/README.md` Pattern #1) means every namespace has a generation script that has to be re-run to wire new methods in. In Rust, `mod.rs` + `pub use` is the same idea but compile-checked. Skip the generation scripts.

**`(context) => { ... }` requires NO COMMENTS** in compatibility indicators (`tests/compatibility/README.md` lines 191-203). That's a brittle transpiler limitation worth noting but trivially fixable.

**File-size red flag:** `ExpressionTransformer.ts` is 1806 lines, `StatementTransformer.ts` is 1494 lines, `codegen.ts` is 2154 lines, `parser.ts` is 1985 lines. There's a lot of accreted special-casing. Some is essential Pine quirk; some is JS workaround. Hard to tell from outside which is which without a deep audit.

## 3. What it could do better

Within its existing structure: (a) replace ad-hoc `sma.ts`-style rolling-window code with a single `IncrementalAccumulator` abstraction, (b) merge the three `param()` variants into one with a config tag, (c) collapse `NAMESPACES_LIKE`/`KNOWN_NAMESPACES`/`NAMESPACE_COLLISION_NAMES`/`CONTEXT_PINE_VARS` (`settings.ts`) into a single namespace registry with capability flags, (d) fix the README on `math.__eq` (NaN==NaN, 1e-8 vs 1e-9). None of these matter to piners.

## 4. Recommendation for piners: option (b), heavy toward (c)

**Do not port code.** AGPL-3.0 is the dealbreaker. If piners ships under any license other than AGPL, you cannot copy a single function from `src/namespaces/ta/methods/`. Even keeping the file structure or copying control flow risks derivative-work claims. The commercial license costs unknown money and benefits LuxAlgo, a competitor.

**Treat as semantics scripture, not code scripture:**

- **Scripture (read, internalize, re-derive):**
  - `src/namespaces/README.md`: the seven Pattern/Case sections. Print it. This is your spec.
  - `src/transpiler/settings.ts`: `KNOWN_NAMESPACES`, `NAMESPACES_LIKE`, `ASYNC_METHODS`, `FACTORY_METHODS`, `CONTEXT_DATA_VARS`, `CONTEXT_PINE_VARS`. This is the *enumeration* of Pine's namespace surface; invaluable.
  - `src/namespaces/ta/methods/` filename list: your TA function checklist (60+ functions, names visible).
  - `src/namespaces/math/methods/` filename list: your math function checklist.
  - The compatibility-test architecture (`.pine.ts` + `.expect.json` + custom NaN serializer): lift the **design pattern**, write your own serializer in Rust. The JSON format itself is copyrightable only weakly (data formats generally aren't), and a Rust impl from a description is clean-room.

- **Ignore entirely:**
  - All of `src/namespaces/ta/methods/*.ts` implementations. Rewrite from TradingView's published Pine docs + the function names you got from filename listing.
  - `pineToJS/lexer.ts`, `parser.ts`, `codegen.ts`. Write a Rust lexer/parser using `chumsky` or hand-rolled; you have shipped `dellingr` and can do this in a week. The two-stage approach (Pine -> IR -> executable) is **not necessary in Rust**; you have one language, one stage, AST -> bytecode.
  - `src/transpiler/transformers/*`: most of this complexity is JS-IR fixup that vanishes when you compile to your own VM bytecode.
  - `Series.ts` and `Series.from()`: the whole reverse-indexed array trick is a JS hack. In Rust, use a `ringbuf` or `VecDeque<f64>` keyed forward.

- **Lift conceptually (re-derive in Rust, cite as inspiration in NOTICE):**
  - Per-call-site state IDs (the `_taN` injection). Make them stable AST node IDs in your Rust AST, not synthetic string args.
  - The `param()` materialization concept (scalar -> 1-element series) so all built-ins have one signature. In Rust, do this with a `Source<T>` enum (`Series(SeriesRef) | Scalar(T)`) and let the function decide.
  - The `__value` / `any()` / lookback triple for dual-use identifiers. Encode as a `BuiltinSymbol` enum variant.
  - The compatibility-test workflow: `.pine` source + `.expect.json` baseline + custom serializer. **This is the cross-validation harness against TradingView**, exactly piners' killer feature. Build it on day one.

**Bottom line:** PineTS is a brilliant **specification artifact** wrapped around a **rewrite-not-port codebase**. The author should read the seven sections of `src/namespaces/README.md` start to finish, scan the `methods/` directory listings for the function checklist, lift the test-harness design pattern, and otherwise ignore the implementation. The AGPL license forces this conclusion, and it is also the right engineering call: Rust + types + your own VM eliminates ~40% of the JS-shim complexity that bloats PineTS. PineTS contributes ~0% to the strategy engine you must build anyway. Treat it as the best Pine semantics documentation that exists, and don't read another line of the .ts implementations.

## Key files cited

- `research/PineTS/src/namespaces/README.md`: scripture
- `research/PineTS/src/transpiler/settings.ts`: namespace enumeration
- `research/PineTS/src/namespaces/ta/methods/param.ts`: `param()` Type A reference
- `research/PineTS/src/namespaces/Core.ts:75-99`: NAHelper pattern
- `research/PineTS/src/namespaces/math/methods/__eq.ts`: epsilon equality
- `research/PineTS/tests/compatibility/README.md`: test harness design
