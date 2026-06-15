---
title: Mass Index
aliases: mass index, massi
---

# Mass Index

Donald Dorsey's Mass Index detects reversals through range expansion rather than
direction. It sums the ratio of a single to a double EMA of the high-low range:
when the range bulges, the ratio rises and the sum builds toward a "reversal
bulge." There is no `ta.massi()` builtin.

## Recipe

```pine
//@version=6
indicator("Mass Index", "MASSI")

fast = input.int(9,  "Fast", minval = 1)
slow = input.int(25, "Slow", minval = 1)

massi(f, s) =>
    hl = high - low
    e1 = ta.ema(hl, f)
    e2 = ta.ema(e1, f)
    math.sum(e1 / e2, s)

plot(massi(fast, slow), "Mass Index")
```

## How to read it

The classic "reversal bulge": watch for the index to rise above 27 and then drop
back below 26.5 - a signal that a reversal is likely (direction taken from a
separate trend tool). It is non-directional; it only flags that a turn is brewing.
