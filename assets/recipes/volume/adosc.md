---
title: Chaikin Oscillator
aliases: chaikin oscillator, adosc, accumulation distribution oscillator
---

# Chaikin Oscillator

The Chaikin Oscillator is MACD applied to the Accumulation/Distribution Line: the
difference between a fast and slow EMA of the A/D Line. It measures the momentum
of money flow. There is no `ta.adosc()` builtin.

## Recipe

```pine
//@version=6
indicator("Chaikin Oscillator", "ADOSC")

fast = input.int(3,  "Fast", minval = 1)
slow = input.int(10, "Slow", minval = 1)

mfm    = (2 * close - high - low) / (high - low)
adLine = ta.cum(mfm * volume)

plot(ta.ema(adLine, fast) - ta.ema(adLine, slow), "ADOSC", style = plot.style_histogram)
```

## How to read it

Crossing above zero signals money flow momentum turning positive (bullish);
below, negative. It is mainly used for divergence against price and to confirm
A/D Line breakouts before the line itself makes them obvious.
