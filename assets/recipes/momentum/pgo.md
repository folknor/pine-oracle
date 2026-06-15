---
title: Pretty Good Oscillator
aliases: pretty good oscillator, pgo
---

# Pretty Good Oscillator

Mark Johnson's Pretty Good Oscillator measures how far the close has travelled
from its N-bar SMA, expressed in units of average true range. Normalizing by ATR
makes the distance comparable across volatility regimes. There is no `ta.pgo()`
builtin.

## Recipe

```pine
//@version=6
indicator("Pretty Good Oscillator", "PGO")

length = input.int(14, "Length", minval = 1)

pgo(len) =>
    (close - ta.sma(close, len)) / ta.ema(ta.atr(len), len)

plot(pgo(length), "PGO")
```

## How to read it

Johnson used it as a breakout system: go long above +3 and short below -3, since
those readings mean price has stretched several ATRs from its mean - a move large
enough to signal a genuine breakout rather than noise.
