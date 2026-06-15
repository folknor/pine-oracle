---
title: Williams Accumulation/Distribution
aliases: williams accumulation distribution, wad, williams ad
---

# Williams Accumulation/Distribution

Larry Williams' A/D line accumulates each bar's gain or loss measured from the
true range: on up closes it adds the move from the true low, on down closes it
adds the (negative) move from the true high. There is no `ta.wad()` builtin.

## Recipe

```pine
//@version=6
indicator("Williams Accumulation/Distribution", "WAD")

prevC = close[1]
trh   = math.max(high, prevC)
trl   = math.min(low, prevC)
adDay = close > prevC ? close - trl : close < prevC ? close - trh : 0.0

plot(ta.cum(adDay), "WAD")
```

## How to read it

A rising WAD signals accumulation, falling signals distribution. Its primary use
is divergence: price making a new high while WAD does not (or vice versa) warns
the move is unsupported. It differs from the Chaikin A/D Line by using true-range
references rather than close-in-range.
