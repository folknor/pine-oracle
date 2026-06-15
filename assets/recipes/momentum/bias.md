---
title: Bias
aliases: bias
---

# Bias

Bias is the percentage gap between price and its moving average: how far the
current close has stretched above or below the mean. There is no `ta.bias()`
builtin.

## Recipe

```pine
//@version=6
indicator("Bias", "BIAS")

length = input.int(26, "Length", minval = 1)
src    = input.source(close, "Source")

bias(source, len) =>
    source / ta.sma(source, len) - 1

plot(bias(src, length), "BIAS")
```

## How to read it

A large positive bias means price is extended above its average (potentially
overbought); a large negative bias means it is stretched below (potentially
oversold). Mean reversion tends to pull bias back toward zero, so extremes are
read as stretch, and the zero line as the average itself.
