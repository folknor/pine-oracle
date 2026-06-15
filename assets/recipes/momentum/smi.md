---
title: Stochastic Momentum Index
aliases: stochastic momentum index, smi, smi ergodic
---

# Stochastic Momentum Index

The SMI Ergodic Indicator is William Blau's True Strength Index with a signal
line added. TSI double-smooths price momentum into a bounded ratio; the SMI plots
it with an EMA signal and a histogram of their difference. There is no `ta.smi()`
builtin (it builds on `ta.tsi`).

## Recipe

```pine
//@version=6
indicator("Stochastic Momentum Index", "SMI")

fast   = input.int(5,  "Fast",   minval = 1)
slow   = input.int(20, "Slow",   minval = 1)
sigLen = input.int(5,  "Signal", minval = 1)
src    = input.source(close, "Source")

smi(source, f, s, sig) =>
    erg    = ta.tsi(source, f, s)
    signal = ta.ema(erg, sig)
    [erg, signal, erg - signal]

[smiLine, smiSignal, smiOsc] = smi(src, fast, slow, sigLen)
plot(smiOsc,    "Oscillator", color.gray, style = plot.style_histogram)
plot(smiLine,   "SMI",        color.blue)
plot(smiSignal, "Signal",     color.orange)
```

## How to read it

The trend is bullish above zero and bearish below; the line/signal crossover (and
the oscillator histogram flipping sign) is the trigger. Because it rests on TSI's
double smoothing, it is steadier than a raw momentum reading.
