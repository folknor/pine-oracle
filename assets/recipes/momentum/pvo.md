---
title: Percentage Volume Oscillator
aliases: percentage volume oscillator, pvo
---

# Percentage Volume Oscillator

The PVO applies the PPO formula to volume instead of price: the percentage gap
between a fast and slow EMA of volume, with a signal line and histogram. It is a
momentum oscillator for volume. There is no `ta.pvo()` builtin.

## Recipe

```pine
//@version=6
indicator("Percentage Volume Oscillator", "PVO")

fast   = input.int(12, "Fast",   minval = 1)
slow   = input.int(26, "Slow",   minval = 1)
sigLen = input.int(9,  "Signal", minval = 1)

pvo(f, s, sig) =>
    fastMa = ta.ema(volume, f)
    slowMa = ta.ema(volume, s)
    line   = 100 * (fastMa - slowMa) / slowMa
    signal = ta.ema(line, sig)
    [line, signal, line - signal]

[pvoLine, pvoSignal, pvoHist] = pvo(fast, slow, sigLen)
plot(pvoHist,   "Histogram", color.gray, style = plot.style_histogram)
plot(pvoLine,   "PVO",       color.blue)
plot(pvoSignal, "Signal",    color.orange)
```

## How to read it

A PVO above zero means recent volume is running hotter than its longer average -
rising participation, which tends to accompany strong moves. Crossovers and
histogram flips mark surges and lulls in activity rather than price direction.
