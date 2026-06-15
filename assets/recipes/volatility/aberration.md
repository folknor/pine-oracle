---
title: Aberration
aliases: aberration, aber
---

# Aberration

Aberration is a Keltner-style volatility channel: a typical-price (`hlc3`) SMA
midline with ATR-width bands above and below. It frames where price sits relative
to recent volatility. There is no `ta.aberration()` builtin.

## Recipe

```pine
//@version=6
indicator("Aberration", "ABER", overlay = true)

length = input.int(5,  "Length",     minval = 1)
atrLen = input.int(15, "ATR Length", minval = 1)

aberration(len, al) =>
    atr_ = ta.atr(al)
    zg   = ta.sma(hlc3, len)
    [zg, zg + atr_, zg - atr_]

[zg, sg, xg] = aberration(length, atrLen)
plot(sg, "Upper", color.blue)
plot(zg, "Mid",   color.orange)
plot(xg, "Lower", color.blue)
```

## How to read it

A close pushing above the upper band signals a volatility breakout to the upside
(and vice versa); price oscillating inside the bands signals a quiet range. It is
read much like Keltner Channels, with the `hlc3` SMA as the mean-reversion anchor.
