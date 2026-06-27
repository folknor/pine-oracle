---
title: Detrended Synthetic Price
aliases: dsp, detrended synthetic price
---

# Detrended Synthetic Price

DSP strips the trend out of price by subtracting an EMA, leaving the cyclical
component oscillating around zero. It is the simplest of Ehlers' cycle tools and
a clean base for spotting periodic swings the trend would otherwise hide. There
is no `ta.dsp()` builtin (it composes `close` and `ta.ema`).

## Recipe

```pine
//@version=6
indicator("Detrended Synthetic Price", "DSP")

length = input.int(14, "Length", minval = 1)
src    = input.source(close, "Source")

dsp(source, len) =>
    source - ta.ema(source, len)

plot(dsp(src, length), "DSP", color.blue)
```

## How to read it

DSP crossing zero marks the cycle turning from below trend to above (and back).
Peaks and troughs flag overextended swings; the distance between successive zero
crossings estimates the dominant cycle's half-period. A shorter length isolates
faster cycles, a longer one slower ones.
