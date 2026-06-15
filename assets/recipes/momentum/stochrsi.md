---
title: Stochastic RSI
aliases: stochastic rsi, stochrsi, stoch rsi
---

# Stochastic RSI

The Stochastic RSI applies the Stochastic formula to RSI rather than price: it
measures where the current RSI sits within its own recent high-low range, then
smooths into %K and %D lines. It is more sensitive than either RSI or Stochastic
alone. There is no `ta.stochrsi()` builtin (it composes `ta.rsi` and the stoch
formula).

## Recipe

```pine
//@version=6
indicator("Stochastic RSI", "StochRSI")

length = input.int(14, "Stoch Length", minval = 1)
rsiLen = input.int(14, "RSI Length",   minval = 1)
kLen   = input.int(3,  "K Smoothing",  minval = 1)
dLen   = input.int(3,  "D Smoothing",  minval = 1)
src    = input.source(close, "Source")

stochrsi(source, len, rl, k, d) =>
    r     = ta.rsi(source, rl)
    ll    = ta.lowest(r, len)
    hh    = ta.highest(r, len)
    rng   = hh - ll
    st    = rng != 0 ? 100 * (r - ll) / rng : 0.0
    kLine = ta.sma(st, k)
    [kLine, ta.sma(kLine, d)]

[kk, dd] = stochrsi(src, length, rsiLen, kLen, dLen)
plot(kk, "K", color.blue)
plot(dd, "D", color.orange)
```

## How to read it

Use 80/20 bands: %K crossing up through 20 is a bullish trigger, down through 80
bearish, with the %K/%D crossover as the entry. Because it is doubly derived it is
fast and noisy - best confirmed against the slower RSI.
