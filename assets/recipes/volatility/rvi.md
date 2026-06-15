---
title: Relative Volatility Index
aliases: relative volatility index, rvi
---

# Relative Volatility Index

The RVI is built like RSI but sums standard deviation instead of price change:
on up bars it accumulates volatility upward, on down bars downward, then takes the
ratio. It measures the direction in which volatility is expanding. There is no
`ta.rvi()` builtin.

## Recipe

```pine
//@version=6
indicator("Relative Volatility Index", "RVI")

length = input.int(14, "Length", minval = 1)
src    = input.source(close, "Source")

rvi(source, len) =>
    std   = ta.stdev(source, len)
    up    = source > source[1] ? std : 0.0
    dn    = source < source[1] ? std : 0.0
    upAvg = ta.ema(up, len)
    dnAvg = ta.ema(dn, len)
    denom = upAvg + dnAvg
    denom != 0 ? 100 * upAvg / denom : 0.0

plot(rvi(src, length), "RVI")
```

## How to read it

It runs 0-100 like RSI: above 50 means volatility is expanding more on up moves
(bullish bias), below 50 the reverse. It is often used as a confirming filter
alongside a price oscillator rather than as a standalone signal.
