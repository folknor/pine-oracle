---
title: Detrended Price Oscillator
aliases: detrended price oscillator, dpo
---

# Detrended Price Oscillator

The DPO strips the trend out of price to expose cycles, by subtracting a
displaced moving average from price. This recipe uses the **non-centered**
(causal) form, which never looks ahead and so is safe in real time. There is no
`ta.dpo()` builtin.

## Recipe

```pine
//@version=6
indicator("Detrended Price Oscillator", "DPO")

length = input.int(20, "Length", minval = 1)
src    = input.source(close, "Source")

dpo(source, len) =>
    t = int(0.5 * len) + 1
    source - ta.sma(source, len)[t]

plot(dpo(src, length), "DPO", style = plot.style_histogram)
```

## How to read it

With the trend removed, the DPO oscillates around zero and its peaks and troughs
reveal the dominant cycle length - useful for timing entries to cycle lows.
Note the classic "centered" DPO shifts the line back in time (a lookahead); this
causal version trades that visual alignment for being usable live.
