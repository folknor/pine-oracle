---
title: Acceleration Bands
aliases: acceleration bands, accbands, abands
---

# Acceleration Bands

Price Headley's Acceleration Bands envelope a moving average with bands whose
width scales by the bar's high-low range relative to its price - so the bands
widen as price "accelerates." There is no `ta.accbands()` builtin.

## Recipe

```pine
//@version=6
indicator("Acceleration Bands", "ACCBANDS", overlay = true)

length = input.int(20, "Length", minval = 1)
c      = input.float(4, "Factor")

accbands(len, factor) =>
    hlRatio = factor * (high - low) / (high + low)
    lower = ta.sma(low  * (1 - hlRatio), len)
    mid   = ta.sma(close, len)
    upper = ta.sma(high * (1 + hlRatio), len)
    [lower, mid, upper]

[lo, mid, up] = accbands(length, c)
plot(up,  "Upper", color.blue)
plot(mid, "Mid",   color.orange)
plot(lo,  "Lower", color.blue)
```

## How to read it

Headley's signal: two consecutive closes above the upper band suggest a strong
breakout (go long), while a close back inside signals the move is over. The bands
expanding marks acceleration; contracting marks consolidation.
