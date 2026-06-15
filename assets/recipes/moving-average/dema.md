---
title: Double Exponential Moving Average
aliases: double exponential moving average, double ema
---

# Double Exponential Moving Average

The DEMA reduces the lag of a regular EMA by subtracting the EMA of the EMA: it
adds back the smoothing error so the result tracks price more closely. There is
no `ta.dema()` builtin (`ta.ema` is the only EMA primitive), so it is composed
from two nested EMAs.

## Recipe

```pine
//@version=6
indicator("Double Exponential Moving Average", "DEMA", overlay = true)

length = input.int(20, "Length", minval = 1)
src    = input.source(close, "Source")

dema(source, len) =>
    e1 = ta.ema(source, len)
    e2 = ta.ema(e1, len)
    2 * e1 - e2

plot(dema(src, length), "DEMA", color.orange, 2)
```

## How to read it

The DEMA hugs price more tightly than an EMA of the same length, so crossovers
and slope changes arrive earlier. The trade-off is more whipsaw in ranging
markets; it is most useful for timing trend entries, not for filtering noise.
