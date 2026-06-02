# TODO

Living list of pending work. Each item should be deletable in one commit
when done.

## lookup <-> manual cross-linking

The two prose surfaces now reference each other by exact, resolvable strings -
wire the links so a result in one jumps to the other:

- **Manual reference links -> `po lookup`.** Section bodies carry kind-prefixed
  reference links like `[ta.ema()](.../#fun_ta.ema)`, `#var_year`, `#type_float`,
  `#op_?:`. Strip the `fun_`/`var_`/`type_`/`op_` prefix and the symbol is an
  exact `po lookup` target. The `render` module already walks the pulldown_cmark
  event stream, so a `Tag::Link` whose dest matches `pine-script-reference/v6/#`
  could render as a `po lookup <name>` hint instead of (or beside) the raw URL.
- **`po lookup` "See the User Manual" pointers -> `po search`.** pine-data
  `remarks` cite manual pages in prose ("See the User Manual's Time page", "the
  Type System page"). These are currently dead text. The manual's `source` URLs
  are vendored, so a remark mentioning a manual page could resolve to a
  `po search <page>` ref - or `lookup`'s `seeAlso` could gain manual section
  refs alongside symbol names.

Both are presentation-layer polish (no new data); decide how aggressively to
surface them without cluttering the cards.

## Documentation

- Per-subcommand worked example in the README or a separate
  `docs/examples.md`. Show `po lookup math.max --format json`,
  `po lookup na` (multi-catalog dump), `po search "operator precedence"`,
  `po search language/operators#operator-precedence`.
