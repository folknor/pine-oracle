---
title: Coppock Curve
aliases: coppock curve, coppock
---

# Coppock Curve

The Coppock Curve is a long-term momentum indicator: a weighted moving average of
the sum of two rates of change. Originally a monthly buy-signal tool, it is also
used on daily charts. There is no `ta.coppock()` builtin.

## Recipe

```pine
//@version=6
indicator("Coppock Curve", "COPC")

length = input.int(10, "WMA Length", minval = 1)
fast   = input.int(11, "Fast ROC",   minval = 1)
slow   = input.int(14, "Slow ROC",   minval = 1)
src    = input.source(close, "Source")

coppock(source, len, f, s) =>
    ta.wma(ta.roc(source, f) + ta.roc(source, s), len)

plot(coppock(src, length, fast, slow), "COPC")
```

## How to read it

The classic signal is the curve turning up from below zero (a long entry) and
turning down from above zero. Because it is heavily smoothed it lags, so it is
used to confirm major trend changes rather than to time short-term moves.
