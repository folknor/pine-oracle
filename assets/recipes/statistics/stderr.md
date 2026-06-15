---
title: Standard Error
aliases: standard error, stderr
---

# Standard Error

Standard Error is the standard deviation of the window divided by the square root
of its length - the precision of the mean as an estimate. It is the basis of
linear-regression error bands. There is no `ta.stderr()` builtin.

## Recipe

```pine
//@version=6
indicator("Standard Error", "STDERR")

length = input.int(14, "Length", minval = 1)
src    = input.source(close, "Source")

stderr(source, len) =>
    ta.stdev(source, len, false) / math.sqrt(len)

plot(stderr(src, length), "StdErr")
```

## How to read it

A small standard error means the sample mean is well-determined (low, stable
dispersion); a large one means it is uncertain (high volatility). It is most
useful as the half-width for error/confidence bands around a moving average or
regression line. `ddof = 1` (sample std) is selected via `ta.stdev(..., false)`.
