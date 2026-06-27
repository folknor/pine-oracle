---
title: Chaikin Volatility
aliases: chaikin volatility, cvi, chaikins volatility
---

# Chaikin Volatility

Marc Chaikin's volatility gauge measures how the high-low range is widening or
narrowing: it takes an EMA of the daily range, then reports that EMA's rate of
change over the same lookback as a percentage. Rising CVI means the range is
expanding (volatility building); falling CVI means it is contracting. There is
no `ta.cvi()` builtin.

## Recipe

```pine
//@version=6
indicator("Chaikin Volatility", "CVI")

length = input.int(10, "Length", minval = 1)

cvi(len) =>
    emaHL = ta.ema(high - low, len)
    100 * (emaHL - emaHL[len]) / emaHL[len]

plot(cvi(length), "CVI")
```

## How to read it

A reading of 25 means the smoothed range is 25% wider than it was `length` bars
ago. Sharp positive spikes often accompany tops (panic widens the range), while
sustained negative readings mark the quiet drift of a maturing trend.
