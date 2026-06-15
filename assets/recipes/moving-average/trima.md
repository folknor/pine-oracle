---
title: Triangular Moving Average
aliases: triangular moving average, tma
---

# Triangular Moving Average

The TRIMA is a double-smoothed average whose weights form a triangle: the
middle bars of the window carry the most weight and the ends carry the least.
It is computed as an SMA of an SMA, which produces the triangular weighting for
free. There is no `ta.trima()` builtin.

## Recipe

```pine
//@version=6
indicator("Triangular Moving Average", "TRIMA", overlay = true)

length = input.int(10, "Length", minval = 1)
src    = input.source(close, "Source")

trima(source, len) =>
    ta.sma(ta.sma(source, math.ceil(len / 2)), math.floor(len / 2) + 1)

plot(trima(src, length), "TRIMA", color.orange, 2)
```

## How to read it

The double smoothing makes the TRIMA flatter and slower than a single SMA of
the same length, which suppresses noise at the cost of more lag. It is best read
as a trend backbone, not a timing trigger.
