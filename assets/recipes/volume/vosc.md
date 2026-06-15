---
title: Volume Oscillator
aliases: volume oscillator, vosc
---

# Volume Oscillator

The Volume Oscillator shows the percentage difference between a fast and a slow
moving average of volume, revealing whether volume momentum is rising or falling
regardless of price direction. There is no `ta.vosc()` builtin.

## Recipe

```pine
//@version=6
indicator("Volume Oscillator", "VOSC")

fast = input.int(14, "Fast", minval = 1)
slow = input.int(28, "Slow", minval = 1)

vosc(f, s) =>
    fastSma = ta.sma(volume, f)
    slowSma = ta.sma(volume, s)
    100 * (fastSma - slowSma) / slowSma

plot(vosc(fast, slow), "VOSC", style = plot.style_histogram)
```

## How to read it

Above zero means short-term volume is running above its longer average - rising
participation, which lends weight to a price move; below zero means fading volume.
Breakouts confirmed by a positive VOSC are more trustworthy than those on
shrinking volume.
