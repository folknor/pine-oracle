---
title: Slope
aliases: slope, momentum slope
---

# Slope

Slope is the rise-over-run of price: the change over `length` bars divided by
`length`. It is the simplest possible momentum measure and a building block for
many others. There is no `ta.slope()` builtin.

## Recipe

```pine
//@version=6
indicator("Slope", "SLOPE")

length = input.int(1, "Length", minval = 1)
src    = input.source(close, "Source")

slope(source, len) =>
    (source - source[len]) / len

plot(slope(src, length), "SLOPE")
```

## How to read it

Positive slope means price is rising over the window, negative means falling, and
the magnitude is the speed. Apply it to a smoothed series (e.g. an EMA) to read
the trend's direction and acceleration without the raw price noise.
