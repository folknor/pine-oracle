---
title: Accumulation/Distribution Line
aliases: accumulation distribution, ad line, a/d line, ad
---

# Accumulation/Distribution Line

The A/D Line gauges money flow by weighting each bar's volume by where the close
fell within its range (the Money Flow Multiplier), then accumulating it. Closes
near the high add volume; closes near the low subtract it. There is no `ta.ad()`
function builtin (Pine has `ta.obv`, not A/D).

## Recipe

```pine
//@version=6
indicator("Accumulation/Distribution", "AD")

mfm    = (2 * close - high - low) / (high - low)   // money flow multiplier
adLine = ta.cum(mfm * volume)

plot(adLine, "AD")
```

## How to read it

A rising A/D Line confirms buying pressure behind an uptrend; falling confirms
selling. The classic signal is divergence: price making new highs while the A/D
Line does not warns the move lacks volume support.
