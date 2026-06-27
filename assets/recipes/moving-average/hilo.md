---
title: Gann HiLo Activator
aliases: hilo, gann hilo, gann high low activator, hilo activator
---

# Gann HiLo Activator

The Gann HiLo Activator is a trend-following stop line built from two simple
moving averages: one of the highs and one of the lows. When the close breaks
above the prior high-MA the line snaps to the low-MA (long regime); when it
breaks below the prior low-MA it snaps to the high-MA (short regime); otherwise
it holds its last value. There is no `ta.hilo()` builtin.

## Recipe

```pine
//@version=6
indicator("Gann HiLo Activator", "HiLo", overlay = true)

highLen = input.int(13, "High Length", minval = 1)
lowLen  = input.int(21, "Low Length",  minval = 1)

highMa = ta.sma(high, highLen)
lowMa  = ta.sma(low,  lowLen)

var float hilo = na
hilo := close > highMa[1] ? lowMa : close < lowMa[1] ? highMa : nz(hilo[1])

plot(hilo, "HiLo", close > hilo ? color.green : color.red, 2)
```

## How to read it

Treat the line as a trailing trend stop: price above it is a long bias, price
below it a short bias, and the colour flip marks the regime change. Raising the
high length and lowering the low length biases the line toward short setups;
the reverse biases it toward longs.
