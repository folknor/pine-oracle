---
title: Archer On-Balance Volume
aliases: archer on-balance volume, archer obv, aobv
---

# Archer On-Balance Volume

Archer OBV wraps plain On-Balance Volume with a fast and a slow moving average of
the OBV line, so you can read OBV momentum the way you read a price MA pair: the
fast MA crossing the slow one flags a turn in volume flow. It returns the OBV
line plus both averages. There is no `ta.aobv()` builtin (and no `ta.obv()`
either), so build OBV from `ta.cum` of signed volume and smooth it.

## Recipe

```pine
//@version=6
indicator("Archer On-Balance Volume", "AOBV")

fast = input.int(4,  "Fast", minval = 1)
slow = input.int(12, "Slow", minval = 1)

aobv(f, s) =>
    line = ta.cum(math.sign(ta.change(close)) * volume)
    [line, ta.ema(line, f), ta.ema(line, s)]

[obvLine, obvFast, obvSlow] = aobv(fast, slow)
plot(obvLine, "OBV",  color.blue)
plot(obvFast, "Fast", color.green)
plot(obvSlow, "Slow", color.red)
```

## How to read it

The fast MA crossing above the slow one signals volume flow turning bullish (a
long run), the opposite crossing turns it bearish (a short run). Pairing the
crossovers with OBV's own divergence against price filters out shallow,
unconfirmed swings.
