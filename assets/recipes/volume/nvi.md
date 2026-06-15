---
title: Negative Volume Index
aliases: negative volume index, nvi
---

# Negative Volume Index

The NVI updates only on bars where volume *fell*, accumulating the percentage
price change on those quiet days - on the theory that "smart money" trades when
volume is low. There is no `ta.nvi()` builtin.

## Recipe

```pine
//@version=6
indicator("Negative Volume Index", "NVI")

initial = input.int(1000, "Initial")

nvi(init) =>
    r       = ta.roc(close, 1)
    contrib = volume < volume[1] ? r : 0.0
    init + ta.cum(contrib)

plot(nvi(initial), "NVI")
```

## How to read it

The classic signal compares the NVI to its own one-year EMA: NVI above that
average suggests a bull market driven by informed money. Rising NVI means quiet
days are advancing price - bullish accumulation. Often paired with the PVI.
