# Evaluation: dellingr

Subagent evaluation of `research/dellingr/`. Date: 2026-05-24.

dellingr is the author's own pure-Rust embeddable Lua VM. From its README: "embeddable, deterministic, pure-Rust Lua VM with precise per-opcode instruction-cost accounting." Main consumer is sandboxed game scripting where untrusted Lua needs a per-game-tick CPU budget. Public API: `Engine` (factory, Send+Sync), `Program` (compiled bytecode handle, Clone), `State` (VM instance, Send), `Anchor` (cross-callback value retention), `HostCallbacks` trait, `RustFunc` for exposing Rust to Lua, `analyze_cost` for static worst-case. Pre-1.0. Explicitly won't-implement: integer division `//`, bitwise ops, coroutines, IO/OS, debug, pcall/xpcall/assert, goto, `string.rep/byte/char`, arithmetic/comparison/concat metamethods, long strings.

This evaluation is not adversarial review. dellingr is shipped, working code by the same author who is writing piners. The question is: which design decisions transplant to a Pine v6 VM, which need modification, which do not apply at all, and is dellingr the right starting point for piners or not.

## 1. Design decisions that transplant directly

**Engine / Program / State factory pattern.** `src/lib.rs:336-422` splits the lifecycle into a stateless `Engine` (Send+Sync compile factory), a refcounted immutable `Program(Arc<Bytecode>)`, and per-execution `State`. This is exactly right for piners: parse once, then run the same program across thousands of backtest permutations (parameter sweeps, walkforward windows, Monte Carlo seeds) in parallel without re-parsing. `Bytecode` being `Send + Sync` while `State` is `Send` only is the correct concurrency posture for an optimization harness too.

**Fixed-width 32-bit instruction encoding** (`src/instr.rs:183-345`). The `[opcode:8][A:8][B:8][C:8]` / `[opcode:8][A:8][sBx:16]` layout with `Instr(u32)` and accessor methods is general-purpose VM hygiene. Pine bytecode will have a different opcode menu but the encoding shape, the `Instr::op_a` / `op_ab` / `op_sbx` constructors, and the `Debug` disassembler all transplant unchanged.

**Bytecode dispatch loop shape** (`src/vm/frame.rs:142-389`). One big `match inst.opcode()` over `u8` constants inside a `loop { let inst = self.get_instr(); ... }`. `Frame` owning `bytecode: Arc<Bytecode>` plus `ip: usize` plus `upvalues` plus `varargs` plus `stack_bottom`, with the value stack on `State`, is the textbook layout. The `line_info: Vec<u32>` mapping bytecode index to source line (`src/compiler.rs:206`) and the `CallInfo` / `StackFrame` machinery (`src/error.rs:8-26`, `src/vm.rs:82-88`) for stack traces are reusable verbatim.

**Lexer with two-token lookahead** (`src/compiler/lexer.rs:16-91`). `TokenStream` with `lookahead` + `lookahead2` plus `linebreaks: Vec<usize>` for `line_and_column` lookup is the right shape for Pine's hand-rolled parser, including the indentation-significant rewrite (you just add an INDENT/DEDENT pre-pass producing virtual tokens that the same `TokenStream` consumes).

**Error model.** `src/error.rs` separates `Error { kind, line_num, column, stack_trace }` from kind-specific enums (`TypeError`, `SyntaxError`, `ArgError`). The `with_stack_trace` builder pattern and 1-based line/column convention are reusable. Pine will add `TypeError::SeriesShape`, `TypeError::QualifierMismatch`, etc. but the chassis is fine.

**Single shared stack with `stack_bottom` for Lua/Rust frames** (`src/vm.rs:100-103`, `src/vm/eval.rs:46-86`). Same stack for bytecode locals, temporaries, and host-call arg/return marshaling. Pine's `request.security` callbacks and built-in TA functions exposed as `RustFunc`-style entries want this same trick.

**The per-call-site `RuntimeCaches` indirection** (`src/compiler.rs:264-306`). This is the underappreciated one. `Bytecode.assign_cache_slots` (`src/compiler.rs:214-261`) walks the instruction stream and assigns a unique cache slot index to every `OP_GET_FIELD` call site, baking it into the instruction. At runtime, each `Closure` allocates its own `RuntimeCaches` sized from `bc.field_cache_slots`. The structural pattern - compiler enumerates per-call-site slots, runtime allocates a parallel state vector keyed by those slots - is exactly what Pine v6 needs for `ta.sma(close, 20)` at line 12 versus line 47 having independent rolling windows. This transplants as a design pattern even though the contents change.

## 2. What needs modification

**`Value` enum (`src/vm/lua_val.rs:17-25`)** is the deepest cut. Today it is `Nil | Bool | Num(f64) | Str | RustFn | Obj`. Pine needs a tagged value that is either a scalar of that type or a *series handle* (a per-script-instance ring buffer). It also needs `na` as a distinct variant (or a sentinel) propagating through arithmetic and comparison, which Lua nil does not do. The `Hash` / `Eq` impls and the `NaN` assertion at line 162 do not survive contact with Pine: `na` is the very thing comparisons must return `na` for, not panic on. Plan on rewriting this file.

**Dispatch loop arithmetic ops** (`src/vm/frame.rs:301-336`) directly call `f64::add` etc. These become 3-valued (`na` propagates) and series-shape-aware (`close + 1` lifts the scalar). Two paths: emit separate opcodes per shape (`OP_ADD_SS`, `OP_ADD_SERIES_SCALAR`, ...) the way Lua 5.3 split int/float, or keep one opcode and dispatch on operand tag. The shape probably matters enough at hot-path that specialization wins; the dellingr opcode-count budget (around 50 opcodes, `src/instr.rs:191-256`) has plenty of headroom.

**Bar-by-bar execution model.** dellingr runs one main chunk per `call`. Pine runs the main chunk once per bar, with every series advanced by one and per-call-site state (TA rolling buffers, persistent `var` locals, `barssince`) preserved across runs. The `RuntimeCaches` pattern (above) gives you the *slot-allocation* mechanism; you need a new owner type, call it `SeriesContext` or `BarState`, sitting on `State`, holding `Vec<TaFuncState>` and `Vec<SeriesBuffer>` indexed by call-site slot. The dispatch loop itself does not change shape; one extra `OP_TA_CALL slot, argcount` style opcode reads/writes that vector.

**Parser.** `src/compiler/parser.rs` is 2871 lines of recursive-descent that is also the codegen. Pine's grammar is different enough (indentation, `=>` function literal syntax, `if expr` blocks-as-expressions, type qualifiers in declarations) that you rewrite this file. You keep the *strategy* (recursive-descent, parser-emits-bytecode, no separate AST in flight) because it is fast and the line count is dominated by Lua quirks (table constructors, method syntax, varargs) that Pine does not have. A Pine parser in this style probably lands at 1500-2000 lines.

**Globals model.** `IndexMap<String, Val>` plus a fast-path `builtins: [Val; 19]` (`src/vm.rs:94-97`, `src/instr.rs:78-104`) maps cleanly onto Pine's `ta.*` / `math.*` / `array.*` namespaces, but you need namespace-aware resolution at compile time rather than runtime string lookup. The Pine spec is closed (no user-defined globals into namespaces); you can resolve every `ta.sma` to a builtin slot index at parse time and never carry the string at all.

## 3. What does not apply

**Cost accounting.** Confirmed by the user. Every `add_cost!` invocation in `src/vm/frame.rs`, `State.cost_remaining` / `cost_budget` / `cost_used` (`src/vm.rs:120-127`), `consume_cost`, `COST_CHECK_INTERVAL`, `ErrorKind::BudgetExceeded`, and the entire `ScopeCost` / `CostAnalysis` / `analyze_cost` apparatus (`src/lib.rs:60-318`) exist to bound untrusted user scripts inside one game tick. Pine source is trusted, single-author, and runs to completion on every bar. Backtests want *total wall time* as a result, not a per-bar budget that aborts. Delete this entire layer in piners. It is roughly 400 lines of `src/lib.rs` plus the per-opcode `add_cost!` accumulator in `frame.rs`.

**`Anchor` system.** `src/vm/anchor.rs` exists because game-engine host code (`FleetCallbacks`) needs to retain references to Lua tables and closures across many independent host calls without polluting `globals`. The process-wide `state_id` allocator (`anchor.rs:32-37`), generational `SlotMap` keys, and `InvalidAnchor` error variant solve "host A holds a handle to State A's table, must not deref into State B." A backtest engine never holds Pine values across runs; everything Pine produces in a bar (orders, alerts, plot points) is consumed by the broker simulator within the same tick. Delete.

**`HostCallbacks` trait.** `src/host.rs` exists to redirect `print()` and `on_error` to a per-game-script console. Pine has no `print`; it has `log.info`, `plot`, `label.new`, `strategy.entry`. Those are not callbacks; they are direct API calls into deterministic data structures that the harness drains after the bar. The trait shape is fine for the few things piners *will* want (a `runtime` callbacks trait emitting `OrderRequest`, `PlotPoint`, `LogEntry`), but the *existing* `HostCallbacks` is the wrong API for Pine. Throw away the trait, keep the pattern.

**`analyze_cost`.** Static worst-case instruction count. Useless for piners.

**GC, `Anchor`-as-root, `mark_gc_roots`** (`src/vm.rs:59-79`, `src/vm/object.rs`). Lua needs tracing GC because tables form cycles via metatables and closures. Pine does not. Series buffers are owned by `SeriesContext`, arrays/maps/matrices are non-cyclic typed containers, strings can be `Arc<str>`. A Pine VM does not need a mark-and-sweep heap. The `GcHeap`, `Markable`, `UpvaluePool`, `ObjectPtr` machinery, all roughly 1000 lines of `src/vm/object.rs` plus the GC integration scattered through `eval.rs`, is dead weight. Pine's upvalue/closure story is also simpler (no metatables, no `__index`/`__newindex` chains, the entire `src/vm/metamethod.rs` is irrelevant).

**Lua standard library** (`src/lua_std/*`, `src/patterns/`). Pine has no `pairs`, `ipairs`, `string.gsub`, Lua patterns, `table.insert`. The 32K of `src/lua_std/string.rs` and the patterns crate are gone. Pine builtins are a different set entirely (`ta`, `math`, `array`, `map`, `matrix`, `input`, `request`, `strategy`, `plot`, `color`, `line`, `label`, `table`, `box`, `chart`).

**Standalone CLI** (`src/main.rs`). Fine reference for argv parsing but piners will have many CLIs (backtest, optimize, walkforward, montecarlo, robust).

## 4. Verdict

**(b) Use dellingr as design reference, write piners fresh in a new workspace.**

Not a fork. The transplant ratio is too low and the things you would delete are load-bearing in dellingr. By rough line count: roughly 5200 lines of parser+lexer+dispatch (rewrite the parser, keep dispatch shape), roughly 2300 lines of GC/object/metamethod (delete), roughly 400 lines of cost accounting (delete), roughly 5000 lines of Lua stdlib (delete), roughly 400 lines of Anchor/HostCallbacks (delete or replace with different traits). A fork starts you at 30 percent reuse and 70 percent deletion, with the deletions touching deeply-integrated subsystems (GC roots threaded through every allocation). Worse, the architectural mistakes you would make as a Pine implementer are exactly the ones a "I already have a Lua VM, let me edit it" mindset would hide: keeping `Value` Lua-shaped, keeping a tracing GC because the scaffolding is already there, modeling `na` as `Nil` and inheriting Lua's nil-arithmetic-errors semantics instead of Pine's na-propagation, conflating Pine's compile-time-resolved namespaces with Lua's runtime global table.

A clean piners workspace, structurally:

- `piners-core`: lexer (port the `TokenStream` + `linebreaks` shape from `src/compiler/lexer.rs`), parser (recursive-descent like `src/compiler/parser.rs` but Pine grammar, with INDENT/DEDENT pre-pass), AST (Pine needs a real AST because of type-qualifier inference; dellingr skipping the AST is a Lua-specific simplification), type checker (qualifier `const/simple/series/input`), bytecode emitter, `Bytecode` shape borrowed wholesale from `src/compiler.rs:178-207` including the `line_info` and the per-call-site slot allocation pattern from `assign_cache_slots`.
- `piners-vm`: `Value` enum redesigned for Pine, `State` with `SeriesContext` and `TaState` vectors keyed by compiler-assigned call-site slot, dispatch loop in dellingr's shape (`src/vm/frame.rs:142-389` minus cost accounting minus GC), 1-based stack-indexing convention for builtin Rust calls.
- `piners-runtime`: namespace builtins, `request.security` driver.
- `piners-strategy`: order book, position tracker, fill model, equity curve, trade list (no dellingr analog).
- Backtest/optimize/walkforward/montecarlo/robust CLIs.

The author's Rust-VM fluency from dellingr is the real asset, not the code. Designing Pine's `Value`, the `SeriesContext`, the qualifier system, and the strategy simulator from scratch with the dellingr playbook in mind ("compile-time slot allocation, `Arc<Bytecode>` shared across states, fixed-width opcodes, line_info for traces, IndexMap not HashMap for determinism") is faster than negotiating with an existing codebase whose biggest subsystems are wrong for the target. The clean-room note in `LLM.md` is also relevant: piners cross-validates against TradingView, and the cleanest provenance story is "wrote from spec, never looked at TV's source," which is easier to maintain in a fresh tree than in a fork that already has a public crates.io lineage.

## Key files cited

- `research/dellingr/src/lib.rs` (Engine/Program/State, cost analysis)
- `research/dellingr/src/instr.rs` (opcode encoding)
- `research/dellingr/src/compiler.rs` (Bytecode shape, RuntimeCaches, per-call-site slot allocation)
- `research/dellingr/src/compiler/parser.rs` (parser-as-codegen, 2871 lines)
- `research/dellingr/src/compiler/lexer.rs` (TokenStream with 2-token lookahead)
- `research/dellingr/src/vm.rs` (State layout, MAX_CALL_DEPTH, GC roots)
- `research/dellingr/src/vm/frame.rs` (dispatch loop, cost accumulation)
- `research/dellingr/src/vm/eval.rs` (call protocol, eval_closure)
- `research/dellingr/src/vm/lua_val.rs` (Value enum)
- `research/dellingr/src/vm/anchor.rs` (host retention)
- `research/dellingr/src/host.rs` (HostCallbacks trait)
- `research/dellingr/src/error.rs` (error model)
- `research/dellingr/LLM.md` (clean-room development posture)
