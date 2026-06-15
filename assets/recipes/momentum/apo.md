---
title: Absolute Price Oscillator
aliases: absolute price oscillator, apo
---

# Absolute Price Oscillator

The APO is the difference between a fast and a slow exponential moving average of
price, expressed in absolute price units. It is the MACD line by another name.
There is no `ta.apo()` builtin.

## Recipe

```pine
//@version=6
indicator("Absolute Price Oscillator", "APO")

fast = input.int(12, "Fast", minval = 1)
slow = input.int(26, "Slow", minval = 1)
src  = input.source(close, "Source")

apo(source, f, s) =>
    ta.ema(source, f) - ta.ema(source, s)

plot(apo(src, fast, slow), "APO", style = plot.style_histogram)
```

## How to read it

Above zero the fast average leads (bullish momentum); below zero it lags
(bearish). Because it is in price units rather than percent, APO readings are not
comparable across instruments at different price levels - use PPO for that.
