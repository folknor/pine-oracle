---
title: Relative Vigor Index
aliases: relative vigor index, rvgi
---

# Relative Vigor Index

The RVGI measures conviction by comparing the close-open move to the high-low
range, on the premise that strong markets close near their highs. Both the
numerator and denominator are symmetrically weighted (`ta.swma`, 4-period) and
summed, then a signal line is taken. There is no `ta.rvgi()` builtin.

## Recipe

```pine
//@version=6
indicator("Relative Vigor Index", "RVGI")

length = input.int(14, "Length", minval = 1)

rvgi(len) =>
    num  = math.sum(ta.swma(close - open), len)
    den  = math.sum(ta.swma(high - low),  len)
    line = num / den
    [line, ta.swma(line)]

[rvgiLine, rvgiSignal] = rvgi(length)
plot(rvgiLine,   "RVGI",   color.blue)
plot(rvgiSignal, "Signal", color.orange)
```

## How to read it

The RVGI line crossing above its signal line suggests buyers are gaining vigor;
crossing below suggests sellers are. Divergence between RVGI and price warns of a
tiring move. `ta.swma` supplies the 4-period symmetric weighting the formula uses.
