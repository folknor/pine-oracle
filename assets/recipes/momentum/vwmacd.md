---
title: Volume Weighted MACD
aliases: volume weighted macd, vwmacd
---

# Volume Weighted MACD

The Volume Weighted MACD replaces MACD's EMAs with volume-weighted moving
averages, so high-volume bars move the indicator more. It reads like a standard
MACD but weights conviction by participation. There is no `ta.vwmacd()` builtin
(it composes `ta.vwma`).

## Recipe

```pine
//@version=6
indicator("Volume Weighted MACD", "VWMACD")

fast   = input.int(12, "Fast",   minval = 1)
slow   = input.int(26, "Slow",   minval = 1)
sigLen = input.int(9,  "Signal", minval = 1)

vwmacd(f, s, sig) =>
    line   = ta.vwma(close, f) - ta.vwma(close, s)
    signal = ta.vwma(line, sig)
    [line, signal, line - signal]

[vwLine, vwSignal, vwHist] = vwmacd(fast, slow, sigLen)
plot(vwHist,   "Histogram", color.gray, style = plot.style_histogram)
plot(vwLine,   "VWMACD",    color.blue)
plot(vwSignal, "Signal",    color.orange)
```

## How to read it

Interpret crossovers, zero-line crosses, and histogram flips exactly as with
MACD - but because volume is in the weighting, signals carry more meaning when
they form on heavy volume and can be discounted on thin volume. (`ta.vwma` uses
the chart's volume implicitly, so no volume argument is needed.)
