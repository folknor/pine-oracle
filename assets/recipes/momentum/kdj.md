---
title: KDJ
aliases: kdj, kdj indicator, random index
---

# KDJ

KDJ is a Stochastic variant popular in Asian markets. %K and %D are RMA-smoothed
Stochastic lines, and the extra %J line (`3K - 2D`) measures their divergence,
amplifying turns and free to travel outside 0-100. There is no `ta.kdj()`
builtin.

## Recipe

```pine
//@version=6
indicator("KDJ", "KDJ")

length = input.int(9, "Length", minval = 1)
sigLen = input.int(3, "Signal", minval = 1)

kdj(len, sig) =>
    ll    = ta.lowest(low, len)
    hh    = ta.highest(high, len)
    rng   = hh - ll
    fastk = rng != 0 ? 100 * (close - ll) / rng : 0.0
    k = ta.rma(fastk, sig)
    d = ta.rma(k, sig)
    [k, d, 3 * k - 2 * d]

[k, d, j] = kdj(length, sigLen)
plot(k, "K", color.blue)
plot(d, "D", color.orange)
plot(j, "J", color.fuchsia)
```

## How to read it

Read K/D like a Stochastic (crossovers, 80/20 bands). The J line is the early
warning: J spiking above 100 or below 0 flags an overstretched move that often
snaps back, making it the most sensitive of the three lines.
