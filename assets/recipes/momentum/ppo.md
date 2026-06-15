---
title: Percentage Price Oscillator
aliases: percentage price oscillator, ppo
---

# Percentage Price Oscillator

The PPO is MACD expressed in percent: the gap between a fast and slow EMA divided
by the slow EMA, times 100, with an EMA signal line and a histogram. The percent
scaling makes it comparable across instruments. There is no `ta.ppo()` builtin.

## Recipe

```pine
//@version=6
indicator("Percentage Price Oscillator", "PPO")

fast   = input.int(12, "Fast",   minval = 1)
slow   = input.int(26, "Slow",   minval = 1)
sigLen = input.int(9,  "Signal", minval = 1)
src    = input.source(close, "Source")

ppo(source, f, s, sig) =>
    fastMa = ta.ema(source, f)
    slowMa = ta.ema(source, s)
    line   = 100 * (fastMa - slowMa) / slowMa
    signal = ta.ema(line, sig)
    [line, signal, line - signal]

[ppoLine, ppoSignal, ppoHist] = ppo(src, fast, slow, sigLen)
plot(ppoHist,   "Histogram", color.gray, style = plot.style_histogram)
plot(ppoLine,   "PPO",       color.blue)
plot(ppoSignal, "Signal",    color.orange)
```

## How to read it

Read it like MACD: line/signal crossovers and the histogram flipping sign signal
momentum shifts, and the zero line separates bullish from bearish. The percent
units let you compare PPO levels between a $5 stock and a $5000 one.
