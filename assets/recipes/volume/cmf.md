---
title: Chaikin Money Flow
aliases: chaikin money flow, cmf
---

# Chaikin Money Flow

CMF sums the money flow volume (volume weighted by close position in range) over
a window and divides by total volume, giving a bounded -1 to +1 read of buying vs
selling pressure. There is no `ta.cmf()` builtin.

## Recipe

```pine
//@version=6
indicator("Chaikin Money Flow", "CMF")

length = input.int(20, "Length", minval = 1)

mfv = (2 * close - high - low) / (high - low) * volume
plot(math.sum(mfv, length) / math.sum(volume, length), "CMF")
```

## How to read it

CMF above zero signals net accumulation (buying pressure), below zero
distribution. Readings beyond +/-0.25 are notable; the zero-line cross is the
basic signal, and divergence against price warns of a weakening move.
