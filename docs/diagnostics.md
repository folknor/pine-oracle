# `pine validate` diagnostic codes

Stable codes emitted by `pine validate` (local tier). Every diagnostic
carries a `code: Option<DiagnosticCode>` field in JSON output; the
source of truth is `piners-syntax/src/diagnostic.rs` and the per-stage
sites in `error.rs`, `typecheck/`, and `semantic.rs`.

Codes are grouped by stage:

- **PINE01xx** - lex stage (`Stage::Lex`)
- **PINE02xx** - parse stage (`Stage::Parse`)
- **PINE03xx** - typecheck stage (`Stage::Type`)
- **PINE04xx** - semantic stage (`Stage::Semantic`)

Severity is per-site, not per-code: most lex / parse / type codes are
errors; semantic codes are a mix of warnings (unused, shadowing, na
comparison) and hints (qualifier escalation). Check `severity` on the
diagnostic, not the code prefix.

## PINE01xx - lex

| Code | Meaning | Typical fix |
|---|---|---|
| `PINE0101` | Unterminated string literal | Close the string before end-of-line. Pine strings do not span lines. |
| `PINE0102` | Invalid color literal | Color literals must be `#RRGGBB` or `#RRGGBBAA` hex. |
| `PINE0103` | Mixed tabs and spaces in indentation | Pick one. Pine is indentation-sensitive; mixing is rejected outright. |
| `PINE0104` | Unexpected dedent | The line dedented to a level that doesn't match any enclosing block. Re-check the indentation of the surrounding `if` / `for` / function body. |
| `PINE0105` | Invalid number | Malformed numeric literal (e.g. trailing `.`, two decimal points, bad suffix). |
| `PINE0106` | Invalid escape | Unknown `\x` sequence inside a string. Pine supports `\\`, `\"`, `\'`, `\n`, `\r`, `\t`. |
| `PINE0107` | Unexpected character | The lexer hit a character it doesn't know how to start a token with. Message embeds the offending char. |

## PINE02xx - parse

| Code | Meaning | Typical fix |
|---|---|---|
| `PINE0201` | Unexpected token | Generic "I wasn't expecting this here." The `expected` / `got` fields on the diagnostic tell you what would have been accepted. |
| `PINE0202` | Expected expression | A position that demands a value (RHS of `=`, function argument, etc.) had no expression. |
| `PINE0203` | Expected type | A type annotation slot got a non-type token. |
| `PINE0204` | Expected identifier | A binding name slot got a non-identifier (e.g. a literal where a variable name belongs). |
| `PINE0205` | Unterminated call | A `(` was opened but the `)` never came before the next significant boundary. |
| `PINE0206` | Invalid assignment target | LHS of `=` or `:=` isn't assignable (e.g. assigning to a literal). |
| `PINE0207` | Invalid header | The `//@version=...` / `indicator(...)` / `strategy(...)` / `library(...)` header is malformed. |
| `PINE0208` | Invalid declaration | Top-level declaration didn't parse. Includes `var` / `varip` / `import` / `type` / `enum` etc. |
| `PINE0209` | Invalid parameter | Function or method parameter list contained an unparsable item. |
| `PINE0210` | Invalid item | Block contained something that isn't a valid statement or sub-block. |
| `PINE0211` | Invalid statement | Statement-level construct that couldn't be classified. |
| `PINE0212` | Lex error surfaced during parse | A `PINE01xx` lex failure showed up while the parser was scanning. The original lex diagnostic is the actionable one. |

## PINE03xx - typecheck

Every PINE03xx diagnostic carries a free-form message describing the
specific violation. The code groups related violations; the message
narrows them down.

| Code | Meaning | Common messages |
|---|---|---|
| `PINE0301` | Value type mismatch | `expected <T>, got <U>` for assignments, returns, function arguments, conditional branches. |
| `PINE0302` | Invalid declared type | A type annotation refers to a type that can't appear in that position (e.g. an invalid generic instantiation). |
| `PINE0303` | Duplicate parameter | Two parameters in the same function / method signature share a name. |
| `PINE0304` | Call signature mismatch | `unknown argument`, `duplicate argument`, `too many arguments for X`, `missing argument Y for X`, `no matching signature for X`. |
| `PINE0305` | Binding mutation rule violation | `cannot reassign const binding`, `cannot reassign read-only binding`, `use := to reassign declared binding`, `cannot reassign undeclared binding`, `cannot mutate const binding`, `cannot mutate read-only binding`, `cannot mutate undeclared binding`, `cannot mutate const field`. |
| `PINE0306` | Name conflict | `name X already declared`, `method X already declared`, `field X.Y already declared`, `enum variant X.Y already declared`. |
| `PINE0307` | Tuple destructuring arity mismatch | `tuple destructuring expected N values, got M`. |
| `PINE0308` | Import resolution failure | `import requires an alias or named path`, `unresolved import <path>`. |
| `PINE0309` | Private import access | `cannot access private imported function / method / value / enum / type X`. The target exists but isn't exported. |

## PINE04xx - semantic

Semantic diagnostics catch patterns that are syntactically and
type-wise valid but suspect at runtime. Severities vary: most are
warnings, `PINE0409` is a hint.

| Code | Severity | Meaning |
|---|---|---|
| `PINE0401` | Warning | Direct `na` comparison (e.g. `x == na`). Use `na(x)` instead - `==`/`!=` against `na` is always `na`. |
| `PINE0402` | Warning | Variable shadows an outer binding. The related-span list points at the shadowed declaration. |
| `PINE0403` | Warning | Unused binding (variable, parameter, import). Prefix with `_` to silence. |
| `PINE0404` | Warning | Function called from local scope when it must be called at script top level (e.g. `request.*`, `input.*`, `barstate.*`). Message format: `` `X` cannot be called from local scope ``. |
| `PINE0405` | Warning | Function must be called every bar at the top level for consistent series semantics. Message format: `` `X` should be called every bar at top level ``. Closely related to `PINE0404`; the two often fire together. |
| `PINE0406` | Warning | Deprecated or removed call. Examples: `deprecated function request.quandl`, `removed function iff; use a conditional expression`. |
| `PINE0407` | Warning | Unused `varip` binding. Same shape as `PINE0403` but specific to `varip`, since unused `varip` is more likely a real bug (no per-bar reset). |
| `PINE0408` | Warning | `max_bars_back` misuse. Either called from non-top-level / unreachable scope, or its target has no history reference (so the hint does nothing). |
| `PINE0409` | Hint | Qualifier escalation: an expression was used in a context that forced its qualifier up (e.g. `const` to `series`). Informational - tells you a higher qualifier is in play than the source might suggest. |

## Output format

Text mode prints `[severity stage CODE] message` per diagnostic with a
source-line caret frame underneath. JSON mode emits the `Diagnostic`
struct verbatim:

```json
{
  "severity": "Warning",
  "stage": "Semantic",
  "code": "PINE0401",
  "message": "direct na comparison; use na(value)",
  "span": { "start": 42, "end": 49 },
  "related": []
}
```

Codes are stable: once assigned, the meaning of a code does not
change. New diagnostics get new codes. Removed diagnostics leave gaps
rather than being reused.
