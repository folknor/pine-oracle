# Evaluation: pinecone

Subagent evaluation of `research/pinecone/`. Date: 2026-05-24.

Pinecone is the only Rust prior art in the research corpus. README states: "modular PineScript interpreter written in Rust", "Full PineScript v5 language support" (v5, not v6), "Type-safe generic architecture", "Modular output system - extend with custom types and builtins".

~12k LOC across 7 crates, all hand-written, no codegen, no IR. Workspace structure is clean: `pine-lexer` -> `pine-parser` -> `pine-ast` -> `pine-interpreter` -> `pine-builtins` -> `pine` facade, plus a `pine-builtin-macro` proc-macro crate and a `pine-reference` doc-binary. The Cargo workspace is the most polished piece of the project.

## 1. What it does right

- **Crate split.** Lexer/parser/AST/interpreter/builtins as separate crates is the right shape for piners; keeps the AST a pure data crate that both your VM and any analyzer (lints, optimizer, AOT) can depend on without pulling the runtime. (`research/pinecone/Cargo.toml:5-17`)
- **Python-style INDENT/DEDENT lexer with a pending-tokens queue.** The right approach for Pine's indentation-significant blocks. (`crates/pine-lexer/src/lib.rs:98-119, 646-770`)
- **AST golden tests.** Pairs of `.pine` + `_ast.json` files in `crates/pine-parser/testdata/{basics,types,methods,enums,generics,control_flow,functions,import_export,syntax}/`. Excellent regression-testing pattern; lift it for piners verbatim.
- **`BuiltinFunction` derive macro.** Declare a struct with named fields and `#[builtin(name="ta.sma")]`; the macro generates positional/named arg dispatch. Ergonomically pleasant; reduces a Pine namespace to a list of small structs with an `execute` method. (`crates/pine-builtin-macro/src/lib.rs:33-100`, used at `crates/pine-builtins/src/ta/moving_averages.rs:5-10`)
- **Generic output type `O: PineOutput`** plus the `impl_output_traits_delegate!` macro lets hosts add custom output channels without forking. Sound design for the kind of bag-of-outputs piners will need (trade list, equity curve, plot stream). (`crates/pine-interpreter/src/output.rs`, example at `examples/custom-builtin-func/src/main.rs:14-45`)
- **Argument enum that preserves positional vs named** at AST and eval level (`crates/pine-ast/src/lib.rs:24-28`, `crates/pine-interpreter/src/lib.rs:202-206`) with a positional-before-named check.
- **v6-ish surface features already parsed.** `type` declarations, `method` declarations with type-dispatch, `enum`, `switch`, `for...in`, generics syntax `<int>`, library `import`/`export`, `varip`, type qualifiers (const/input/simple/series). (`crates/pine-ast/src/lib.rs:157-190`, `crates/pine-parser/testdata/{methods,enums,generics,import_export}/`)
- **`pine-reference/spec/v6.md` (918 KB)** appears to be a vendored copy of TV's reference docs. Useful corpus for piners regardless of whether you adopt the runtime.

## 2. What it does wrong

- **No series history. At all.** `Series<O>` has only `id` + `current` (`crates/pine-interpreter/src/lib.rs:86-89`). History is fetched through a `HistoricalDataProvider` trait the *host* must implement, keyed by string `id` lookup (`lib.rs:23-27`, `lib.rs:447-454`, `lib.rs:1072-1081`). This is wrong end-to-end: every `[N]` and every TA function round-trips through a host callback, and there is no way for user-written Pine code (e.g. `myVar = close + nz(myVar[1])`) to lookback on a non-builtin series because the host has no idea what `myVar` is. **The whole point of a Pine VM is owning the series buffers.**
- **TA functions recompute from scratch every bar.** `ta.sma` sums `length` historicals on every call; `ta.ema` re-runs the recurrence over `length*2` historicals each bar (`crates/pine-builtins/src/ta/moving_averages.rs:14-30, 41-100`). Pine semantics demand per-bar incremental state. This is O(bars x length) instead of O(bars) and also non-equivalent to TV for warm-up bars.
- **Division by zero raises `RuntimeError::DivisionByZero`** (`crates/pine-interpreter/src/lib.rs:1247-1261`). Pine returns `na`. Hard TV-parity violation that will fire on real scripts the first time `close` rolls a zero.
- **3-valued `na` logic missing.** `BinOp::And/Or` are eager Rust booleans (`lib.rs:1275-1277`); `na == na` returns `true` (`lib.rs:1293`) instead of `na`; `values_equal` uses an `EPSILON` float compare (`lib.rs:1290`). All wrong vs TV.
- **`Value::Number(f64)` collapses int and float.** Pine distinguishes; many functions are int-only (e.g. `array.new<int>(size)`). `int()` cast just calls `f64::trunc` (`crates/pine-builtins/src/lib.rs:67-78`); no type. This will silently desync from TV in indexing/length math.
- **Type system is stringly-typed.** `type_annotation: Option<String>` in every AST node; method dispatch is by `String` comparison of the first parameter's annotation (`crates/pine-interpreter/src/lib.rs:873-896`). No resolution pass, no type inference, no compile-time check that the script even makes sense. The `TypeQualifier` enum exists but is barely enforced.
- **No spans / source positions in the AST.** The lexer tracks line/column on tokens, but the AST drops them. You cannot produce useful runtime error messages, can't do source-mapped diagnostics, can't do incremental editing. Piners will want spans on every node.
- **No `strategy.*`, no `request.*` (no `request.security`, no MTF), no `input.*`.** (`crates/pine-builtins/src/lib.rs:13-25, 149-208`). These are mandatory for the backtester piners is shipping. ~60% of v6's killer namespace surface is absent.
- **Tree-walking interpreter with no bytecode / no IR.** Every bar re-walks the AST, re-evaluates string-keyed `HashMap<String, Variable>` lookups, re-clones `Value`s through `Rc<RefCell<...>>`. For piners' optimize/MC/walkforward workloads (millions of bar executions) this is a non-starter.
- **No bar-counter / `bar_index` / `barstate.*`** in the engine. The `execute(&Bar)` API takes one bar at a time but the runtime doesn't track an index.
- **Object fields are a `HashMap<String, Value>` per instance** (`crates/pine-interpreter/src/lib.rs:100-103`). User types should be a `Vec` indexed by field number resolved at parse/type-check time.
- **Compound assignment via AST sleight-of-hand.** `PlusAssign` lowers to `BinOp::Add` (`crates/pine-parser/src/lib.rs:58-61`) but the assignment statement is a generic `target = value`, so the parser must synthesize `x = x + ...`. Workable but no provenance for diagnostics.
- **Single-bar execution model.** `Script::execute(&Bar)` resets `close = Series{current}` each call (`crates/pine/src/lib.rs:117-151`). Combined with the missing history, there is no real notion of running a script across a bar stream; `execute_bars` (`pine/src/lib.rs:160-165`) is just a `for` loop that drops outputs except the last.

## 3. What it could do better (within the existing structure)

- Put the series buffer **inside** the interpreter: each `Series` becomes a handle into a per-interpreter `Vec<Value>` ring buffer keyed by stable ID assigned at parse time. Kill `HistoricalDataProvider`.
- Make TA functions stateful structs registered per script, not free functions. The existing `BuiltinFunction` derive can be extended with a `#[stateful]` flag that allocates per-call-site state in the interpreter.
- Add `Value::Int(i64)` and propagate `na` properly. Fix `/`, `%`, `==`, `and`, `or` to TV semantics. These are isolated changes (~50 LOC each) but every fix needs golden tests against TV.
- Add `Span { lo, hi, file }` on every AST node; the lexer already has the data.
- Introduce a resolution/typecheck pass between parser and interpreter. The `pine-ast` crate is small enough that a `pine-resolve` crate could sit on top without disturbing it.
- Replace `HashMap<String, Variable>` with a resolved `Vec<Slot>` indexed by ID; pre-resolve every `Variable(name)` to a slot ID during a name-resolution pass. Order-of-magnitude perf win for the backtester.

## 4. Use as starting point for piners?

**Write from scratch. Use Pinecone only as inspiration for layout and as a parser corpus.**

The codebase is well-organized and the author made several good decisions (workspace split, INDENT/DEDENT, the builtin derive macro, the output trait, golden AST tests). But the **runtime is structurally wrong for what piners needs**: no internal series history, no incremental TA state, no IR, no type resolution, single-bar execution that delegates lookback to the host, plus TV-divergent semantics on every arithmetic operator. The first three on that list are not local fixes; they are the runtime's spine, and Pinecone's spine is built around "host owns history". Bending it into a TV-parity VM with strategy simulation is more work than starting clean.

Concrete recommendation:

- **Lift / port:**
  - The **crate layout** in `Cargo.toml` (`pine-lexer/-ast/-parser/-interpreter/-builtins/-builtin-macro/pine`). Same shape but add `pine-resolve` (typecheck/name-res) and `pine-ir` (or `pine-vm`) between parser and interpreter, and `pine-strategy` alongside `pine-builtins`.
  - The **lexer's INDENT/DEDENT logic** at `crates/pine-lexer/src/lib.rs:646-770`. Around ~150 lines, well-tested; port directly and add spans.
  - The **`BuiltinFunction` proc-macro** design at `crates/pine-builtin-macro/src/lib.rs`. Extend with stateful slots.
  - The **`PineOutput` generic trait pattern** at `crates/pine-interpreter/src/output.rs` and the `impl_output_traits_delegate!` macro. Reuse the idea for piners' broker/strategy/plot output channels.
  - The **AST golden-test corpus** in `crates/pine-parser/testdata/`. ~40 Pine snippets paired with JSON ASTs; drop in, retarget at your AST, and you have parser regression tests on day one.
  - The **`pine-reference/spec/v6.md`** vendored docs as a v6 surface checklist.

- **Discard:**
  - The **interpreter** (`crates/pine-interpreter/src/lib.rs`); wrong abstractions; tree-walking AST with string-keyed scopes and host-delegated history. The `Value` enum is salvageable as a sketch of the value taxonomy but needs `Int` split off and `na`-propagating arithmetic.
  - The **AST as-is**; re-do with spans, resolved type IDs instead of `Option<String>`, separated `IntLit`/`FloatLit`. Keep the *shape* (the `Stmt`/`Expr` variants are well-chosen).
  - **All TA implementations** in `crates/pine-builtins/src/ta/`. They are stateless re-summations against a host callback; piners needs incremental, per-call-site stateful operators. Re-implement against a real series buffer. (You can keep the namespace organization, one file per category, that's good.)
  - The **parser**, mostly. 2114 lines of hand-rolled recursive descent without spans is a rewrite either way. Keep it as a reference for how each Pine construct lowers.
  - The **`HistoricalDataProvider` trait** in its entirety.

The good news: Pinecone proves that ~12k LOC of Rust can parse a substantial slice of v6 surface and run trivial indicators. That's a useful sanity check on scope. But the runtime delta to "TV-parity backtester with strategy simulation" is larger than the delta from a blank `cargo new`. Given pbfhogg and dellingr in the author's track record, the parser+resolver+VM is not the hard part; TV-parity testing is, and Pinecone's runtime would cost you days of unlearning before you could start matching TV bar-by-bar.

## Relevant files for reference

- `research/pinecone/Cargo.toml`
- `research/pinecone/crates/pine-ast/src/lib.rs`
- `research/pinecone/crates/pine-lexer/src/lib.rs` (lines 646-770 = INDENT/DEDENT)
- `research/pinecone/crates/pine-interpreter/src/lib.rs` (lines 86-89, 425-468, 1072-1081, 1223-1278 are the load-bearing problem spots)
- `research/pinecone/crates/pine-builtin-macro/src/lib.rs`
- `research/pinecone/crates/pine-builtins/src/ta/moving_averages.rs`
- `research/pinecone/crates/pine-builtins/src/lib.rs` (lines 149-208 = namespace registration; note absence of strategy/request/input)
- `research/pinecone/crates/pine-parser/testdata/` (golden test corpus to lift)
- `research/pinecone/crates/pine-reference/spec/v6.md`
