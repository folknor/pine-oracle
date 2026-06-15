---
title: Positive Volume Index
aliases: positive volume index, pvi
---

# Positive Volume Index

The PVI is the counterpart of the NVI: it updates only on bars where volume
*rose*, accumulating the percentage price change on those active days - tracking
what the crowd does when volume surges. There is no `ta.pvi()` builtin.

## Recipe

```pine
//@version=6
indicator("Positive Volume Index", "PVI")

initial = input.int(1000, "Initial")

pvi(init) =>
    r       = ta.roc(close, 1)
    contrib = volume > volume[1] ? r : 0.0
    init + ta.cum(contrib)

plot(pvi(initial), "PVI")
```

## How to read it

Rising PVI means high-volume days are pushing price up - the crowd is buying.
Read alongside the NVI: when both confirm, the trend has broad participation; when
they diverge, smart money and the crowd disagree.
