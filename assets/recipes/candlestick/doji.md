---
title: Doji
aliases: doji, doji candle
---

# Doji

A Doji is a candle whose body is tiny relative to its range: open and close are
nearly equal, signalling indecision between buyers and sellers. Following the
pandas-ta/TA-Lib definition, the body must be under a fraction of the average
high-low range of the prior bars. There is no candlestick builtin in Pine.

## Recipe

```pine
//@version=6
indicator("Doji", "Doji", overlay = true)

length = input.int(10, "Avg Length",  minval = 1)
factor = input.float(0.1, "Body Factor", minval = 0)

body     = math.abs(close - open)
rng      = high - low
avgRange = ta.sma(rng, length)[1]   // prior bars' average range (shifted 1)
isDoji   = body <= factor * avgRange

plotshape(isDoji, "Doji", shape.diamond, location.belowbar, color.yellow)
```

## How to read it

A Doji marks a pause or balance, not a direction. Its meaning comes from context:
after a strong trend it warns of exhaustion and possible reversal; in a range it
is just noise. Confirm with the next bar's direction.
