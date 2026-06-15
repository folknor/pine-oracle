---
title: Laguerre RSI
aliases: laguerre rsi, lrsi, ehlers laguerre rsi
---

# Laguerre RSI

John Ehlers' Laguerre RSI runs price through a four-stage Laguerre filter (a
low-lag smoother controlled by a single `gamma`), then forms an RSI-style ratio
from the differences between the filter stages. The result is a fast, smooth
0-100 oscillator. There is no `ta.lrsi()` builtin.

## Recipe

```pine
//@version=6
indicator("Laguerre RSI", "LRSI")

gamma = input.float(0.5, "Gamma", minval = 0, maxval = 1)
src   = input.source(close, "Source")

lrsi(source, g) =>
    var float l0 = na
    var float l1 = na
    var float l2 = na
    var float l3 = na
    p0 = nz(l0[1], source)
    p1 = nz(l1[1], source)
    p2 = nz(l2[1], source)
    p3 = nz(l3[1], source)
    l0 := (1 - g) * source + g * p0
    l1 := -g * l0 + p0 + g * p1
    l2 := -g * l1 + p1 + g * p2
    l3 := -g * l2 + p2 + g * p3
    cu = math.max(l0 - l1, 0) + math.max(l1 - l2, 0) + math.max(l2 - l3, 0)
    cd = math.max(l1 - l0, 0) + math.max(l2 - l1, 0) + math.max(l3 - l2, 0)
    denom = cu + cd
    denom != 0 ? 100 * cu / denom : 0.0

plot(lrsi(src, gamma), "LRSI")
```

## How to read it

Like RSI it runs 0-100, but it reaches extremes faster, so the common bands are
tighter (e.g. 80/20). A higher `gamma` smooths more (more lag, fewer signals); a
lower `gamma` is faster and noisier. Treat sustained readings near the extremes
as strong trend rather than imminent reversal.
