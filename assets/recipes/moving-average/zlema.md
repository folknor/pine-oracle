---
title: Zero-Lag Exponential Moving Average
aliases: zero lag ema, zero-lag exponential moving average, zlma
---

# Zero-Lag Exponential Moving Average

The ZLEMA removes lag by feeding the EMA a "de-lagged" series: it adds the
difference between price now and price `lag` bars ago, where `lag` is half the
length. The result responds almost immediately to price changes. There is no
`ta.zlema()` builtin.

## Recipe

```pine
//@version=6
indicator("Zero-Lag Exponential Moving Average", "ZLEMA", overlay = true)

length = input.int(20, "Length", minval = 1)
src    = input.source(close, "Source")

zlema(source, len) =>
    lag = math.floor((len - 1) / 2)
    ta.ema(source + (source - source[lag]), len)

plot(zlema(src, length), "ZLEMA", color.orange, 2)
```

## How to read it

The ZLEMA is one of the most responsive averages, so it gives the earliest
crossover signals but also the most false ones in choppy conditions. It works
best as a fast line in a dual-MA system rather than as a standalone filter.
