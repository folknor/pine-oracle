---
title: Hull Moving Average
aliases: hull moving average, hull ma
---

# Hull Moving Average

The Hull Moving Average (HMA) is a low-lag moving average that smooths price
while staying close to it. It is built from three weighted moving averages: a
half-length WMA, a full-length WMA, and a final WMA over their reweighted
difference. TradingView has no `ta.hma()` builtin, so it is computed inline.

## Recipe

```pine
//@version=6
indicator("Hull Moving Average", overlay = true)

length = input.int(20, "Length", minval = 1)
src    = input.source(close, "Source")

hma(source, len) =>
    half = math.floor(len / 2)
    sqrt = math.floor(math.sqrt(len))
    ta.wma(2 * ta.wma(source, half) - ta.wma(source, len), sqrt)

plot(hma(src, length), "HMA", color.orange, 2)
```

## How to read it

The HMA turns up or down faster than an SMA or EMA of the same length, so a
change in its slope is an earlier (but noisier) trend signal. Because it can
overshoot at sharp reversals, it is usually read by slope direction rather than
by price crossing it.
