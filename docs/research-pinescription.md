# Evaluation: pinescription

Subagent evaluation of `research/pinescription/`. Date: 2026-05-24.

Pinescription is a Go-language Pine Script v6 compiler + bytecode VM by Woodstock K.K., dual-licensed AGPL-3.0 / Commercial. It compiles Pine to optimized bytecode and runs it bar-by-bar against a Provider interface. Performance: claims 164x faster than full recomputation via "streaming execution model updates only the active bar state, avoiding full recalculation on each iteration." 30+ indicators, full `ta.*`/`math.*` namespaces, arrays, matrices, tuples. **Critical caveat:** strategy APIs (`strategy.entry`, etc.), request APIs (`request.security`, etc.), and plot APIs return runtime errors in OSS unless you register exact-name custom function hooks via `RegisterFunction`/`RegisterFunctionWithParamNames`. So OSS is indicator-strong, strategy-incomplete.

## TL;DR

**Don't port it. Lift one idea (per-call-site streaming indicator state with rolling windows) and write fresh.** Pinescription is mislabeled; it is *not* a bytecode VM. It is a tree-walking AST interpreter with gob-serialized AST as the "bytecode" wire format. Two subsystems are worth studying as design references: (1) the per-call-site streaming indicator state machines, (2) the `RegisterFunctionWithParamNames` hook seam. Everything else is either Go-isms that don't translate, or work you would redo in Rust anyway.

## 1. What it does right

**The "bytecode" is a misnomer; the streaming-indicator design is the real win.**

- `research/pinescription/bytecode.go` shows the entire "bytecode" pipeline is `gob.NewEncoder(...).Encode(program)` over the `Program` struct. The magic prefix is `"PG2\x00"`, fallback is JSON. There is no flat instruction stream, no PC, no operand bytes.
- `opcode_ids.go` defines `exprKind*`, `unaryOp*`, `binaryOp*`, `builtinFast*`; these are tag enums on AST nodes (`Expr.KOp`, `Expr.BOp`, `Expr.UOp`, `Expr.BID`), pre-computed by the parser to avoid string compares in the hot path. That's a switch-on-tag tree-walker, not a stack/register machine.
- The actual execution is `r.eval(expr)` in `runtime_exec.go:413`, recursively switching on `expr.KOp`, returning `interface{}`. Statements walk `Stmt.Kind` strings in `execStmt` (`runtime_exec.go:62`).

So the "164x speedup" claim is *not* a bytecode-vs-interpreter story; it is the **streaming evaluation model** on top of the tree-walker. That part is genuinely good:

- `builtins.go:1815-2044`: `smaIndicatorState`, `emaIndicatorState`, `rsiIndicatorState`, `rmaIndicatorState`, `bbIndicatorState`, `extremaIndicatorState`. Each is a small struct holding a `rollingWindowState` (circular buffer) plus running aggregates (`sum`, `sumSq`, `sumGain`, `sumLoss`). EMA decays via `value = v*k + value*(1-k)`. SMA does push-and-subtract-old in O(1). RSI maintains separate sum-of-gains/sum-of-losses with proper add-on-push, remove-on-evict accounting.
- The state machine is keyed per *call site*: `indicatorStateKey` (`builtins.go:2106`) hashes `name | activeSymbol | activeValueType | %p(raw AST node) | params...`. Pointer identity of the AST node is the call-site identifier. This is how `sma(close, 20)` and `sma(volume, 20)` in the same script keep separate windows.
- `lastBar` gate (`builtins.go:2149`) ensures `Update` runs at most once per bar per call site even when the same `sma(close, 20)` is referenced multiple times in the same bar.
- Series storage is OHLCV-only per symbol via the `Provider` interface (`SeriesExtended`); derived series (`hl2`, `hlc3`) lazily wrap base series with `.Add`/`.Mul`/`.Div` (`runtime_series.go:67-79`). User variable history lives in `r.numericHistory[name] []float64` for hot float-typed vars, `r.history[name] []interface{}` for everything else (`runtime_state.go:402`, `recordHistory` with `historyKind` promotion from numeric to generic on type change). The numeric fast path is the right Rust optimization too.
- Bar-step caching: `priceCacheOpen/High/Low/Close/Vol` plus a `priceCacheMask` bitmask in `ensureActivePriceValue` (`runtime_state.go:641`); avoids re-resolving series-by-key for repeated `close` references within a bar.

**The hook system (`RegisterFunctionWithParamNames`, `engine.go:175`) is worth lifting conceptually.** The seam is: any call that fails name lookup falls through to `r.userFns[name]` (`runtime_calls.go:65`), and `isUnsupportedFeatureCallName` (`runtime_calls.go:99`) deliberately routes `strategy.*`, `request.*`, `plot*`, `alert*` to the hook path first. Param-names metadata enables binding Pine's named arguments. For piners this maps cleanly to a `trait HostFn` or `Box<dyn Fn(&[Value]) -> Result<Value>>` keyed on `&'static str`, with strategy/request/plot bound *internally* by piners' broker simulator rather than externally by users.

## 2. What it does wrong

- **It is not a bytecode VM.** The README marketing is misleading. Don't port a non-VM thinking you're porting a VM.
- **`interface{}` everywhere:** `eval` returns `(interface{}, error)`; values are boxed `float64`/`bool`/`string`/`*pineArray`/`*pineMap`/`*customTypeInstance`. `toFloat` (`runtime_exec.go:768`) is a type switch on every numeric coercion. In Go this is GC-managed but allocation-heavy on tight inner loops. The hottest path, `evalBinary`, does two `toFloat` calls plus an opcode switch per arithmetic op. This pattern does **not** translate to Rust. You want `enum Value { Float(f64), Bool(bool), Int(i64), Str(Rc<str>), Na, Array(Rc<RefCell<Vec<Value>>>), ... }` with NaN-tagging optional. Or, since dellingr already has a Value enum, port that shape.
- **GC pressure on bar-by-bar allocations:** `execStmt` on `tuple_assign` does `make([]interface{}, ...)`, arrays in expressions allocate via `make(...)` per `eval` (`runtime_exec.go:429-447`). There's a `floatSlicePool` and arg-pool (`callArgPooling`) bolted on, but the architecture is allocation-heavy by default.
- **Map-based env stack:** `r.envStack []map[string]interface{}` with `envStack[len-1][name] = v` on every assign. String-keyed lookup at every scope read. Pine has lexically resolvable names; you should resolve to slot indices at compile time. Pinescription doesn't.
- **String-keyed `indicatorState` cache:** `indicatorStateKey` builds a fresh string via `strings.Builder` on *every* indicator call (`builtins.go:2106`), including a `%p` pointer format. That's a heap allocation per `sma()` call per bar. In Rust this should be a `HashMap<(usize, ...), Box<dyn IndicatorState>>` keyed on `*const Expr as usize` or an interned `CallSiteId: u32` assigned at compile time.
- **Strategy/request/plot are stubs.** This is the hard part of any Pine runtime and Pinescription provides zero help: no broker simulation, no `strategy.position_size`/`strategy.equity`/`strategy.closedtrades[]`, no fill model, no commission/slippage, no `request.security` lookahead/repaint semantics, no MTF resampling. For piners, replacing nordquant means implementing all of this from scratch. The hook seam is a registration mechanism, not a design.
- **No TradingView parity machinery.** No golden-file harness, no bar-by-bar value comparison, no documented semantics on the subtleties that bite (`barstate.isconfirmed`, `lookahead_on/off`, `calc_on_every_tick`, security repaint, `var`/`varip` interactions with `na`). This is *the* piners differentiator and Pinescription contributes nothing.
- **Gob-as-bytecode is hostile to cross-language tooling.** If anyone ever wants to inspect or persist compiled programs from Rust/JS/Python, gob is a Go-only format. JSON-fallback works but the format is the AST verbatim, which is verbose and not stable.

## 3. What it could do better (within its existing structure)

- Replace pointer-identity `%p(raw)` keys with parse-time-assigned `CallSiteId u32`s threaded through `Expr`.
- Resolve identifiers to env-slot indices in `lowerProgram` (`lowering.go`) so `resolve(name)` becomes an indexed array read.
- Move from `interface{}` to a tagged-union `Value` type (Go can do `struct{tag uint8; f float64; ref unsafe.Pointer}` style).
- Type-specialize binary ops: `evalBinary_float_float`, `evalBinary_str_str` dispatched at lowering time. Currently `evalBinaryByOpcode` does runtime `toFloat`.
- Actually compile to bytecode. Once you have slot-indexed identifiers and call-site IDs, a flat opcode stream is straightforward and unlocks per-opcode cost accounting.
- Pre-bind indicator state at compile time: each `sma(...)` call site gets a pre-allocated `IndicatorState` slot. Eliminates the per-bar map lookup entirely.

## 4. As the starting point for piners

**Position: (c), use it as a design reference, not a port.**

Specifically:

- **Do not port the compiler+VM architecture.** It is a tree-walker with map-based scopes and `interface{}` values. A flat-opcode bytecode VM with slot-indexed identifiers and call-site IDs resolved at compile time would be both significantly faster and structurally simpler.
- **Lift the streaming-indicator design as a pattern, not as code.** The win is "per-call-site `IndicatorState` trait object that exposes `update(f64)` and `value() -> f64`, keyed by compile-time-assigned `CallSiteId`, gated by `last_bar` to prevent double-update within a bar." That fits naturally as a new opcode class in a bytecode VM: `OP_INDICATOR_UPDATE(site_id)` + `OP_INDICATOR_VALUE(site_id)`. Each `IndicatorState` impl (`SmaState`, `EmaState`, `RsiState`, `BbState`, `MacdState`) is a ~30-line Rust struct. Look at `builtins.go:1815-2044` for the math; do not look at the dispatcher.
- **Lift the host-function hook seam, but treat strategy/request/plot as first-class internal subsystems, not hooks.** The registration pattern (name + param-name list + callback) is useful for genuine *user* extensions and for letting the nordquant CLI binaries inject Python-style data sources. But `strategy.entry`, `strategy.exit`, `request.security` should be implemented inside piners against piners' own broker simulator, not deferred to userland. The AGPL Pinescription leaves these stubbed precisely because they are the hard part, and they are the part you must own to get TradingView trade-list parity.
- **Do not adopt the gob-AST "bytecode" format.** Pick a real flat opcode stream. The AGPL on Pinescription is a third reason not to port code (PineTS being the first, code-quality being the second), but legal risk is not the deciding factor here; the architectural regression from a real bytecode VM to a tag-coloured tree-walker is.
- **Use the series/host abstraction (`Provider` interface, `engine.go:60-80`) as a sketch for piners' data-source trait.** Methods like `GetSymbols`, `GetSeries(key)`, `SetTimeframe`, `SetSession` are the right shape. In Rust this becomes `trait MarketDataProvider`. Symbol|valueType key format ("AAPL|close") is reasonable; consider `(SymbolId, ValueTypeId)` interned pair instead of a string.
- **Read `runtime_state.go:402` (`recordHistory`) and `runtime_state.go:640` (`valueAt`) for the bar-history storage model.** Numeric-fast-path (`numericHistory []float64`) with promotion-to-generic (`history []interface{}`) on type change is the right shape. In Rust this is `enum HistoryStore { Float(Vec<f64>), Generic(Vec<Value>) }` with a one-time promotion.

## Pinescription's good parts, in a piners-shaped sketch

1. Lexer/parser -> AST (fresh; lexer.go and parser.go are reasonable structural references but Rust nom/chumsky/handwritten will be cleaner).
2. Lowering pass that:
   - Resolves identifiers to env-slot indices.
   - Assigns each call site a `CallSiteId u32`.
   - Pre-allocates indicator state slots per call site for `ta.*` calls with constant `length`.
   - Type-specializes ops where both operands are known float.
3. `IndicatorState` trait (`update(f64)`, `value() -> f64`), one impl per ta function, owned by the VM and indexed by `CallSiteId`.
4. Broker simulation as a built-in subsystem responding to `strategy.*` calls (not host hooks).
5. Host-fn hook for genuine user extensions and for Pythonland data sources called from nordquant CLIs.

(VM architecture itself - stack vs register, opcode set, dispatch loop - is an open piners design question; this sketch covers only what Pinescription gets right that you would lift in any architecture.)

## Two files worth keeping open while writing piners

- `research/pinescription/builtins.go` lines 1815-2280: copy the math of each indicator state machine.
- `research/pinescription/runtime_state.go` lines 402-464: copy the history-storage promotion shape.

Everything else: read once, close, write fresh in Rust.
