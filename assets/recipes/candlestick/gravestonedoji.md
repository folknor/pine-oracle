---
title: Gravestone Doji
aliases: gravestone doji
---

# Gravestone Doji

A Gravestone Doji is the mirror of the Dragonfly: open, low, and close cluster at
the bottom, with a long upper shadow and almost no lower one. Buyers pushed price
up intrabar but sellers drove it all the way back, a potential bearish reversal.
There is no candlestick builtin in Pine.

## Recipe

```pine
//@version=6
indicator("Gravestone Doji", "GravestoneDoji", overlay = true)

body  = math.abs(close - open)
rng   = high - low
upper = high - math.max(open, close)
lower = math.min(open, close) - low

isPat = rng > 0 and body <= rng * 0.1 and lower <= rng * 0.1 and upper >= rng * 0.6
plotshape(isPat, "Gravestone Doji", shape.diamond, location.abovebar, color.red)
```

## How to read it

It carries the most weight at the top of an uptrend or at resistance, where the
long upper wick shows rejected highs. Treat it as a reversal candidate, confirmed
by the next bar closing lower.
