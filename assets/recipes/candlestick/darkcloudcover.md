---
title: Dark Cloud Cover
aliases: dark cloud cover
---

# Dark Cloud Cover

Dark Cloud Cover is the bearish mirror of the Piercing Line: an up candle
followed by a down candle that opens above the prior close but falls to close
below the midpoint of the prior body. Sellers erased more than half the prior
gain. There is no candlestick builtin in Pine.

## Recipe

```pine
//@version=6
indicator("Dark Cloud Cover", "DarkCloud", overlay = true)

midPrior = (open[1] + close[1]) / 2
isPat = close[1] > open[1] and close < open and open > close[1] and
        close < midPrior and close > open[1]
plotshape(isPat, "Dark Cloud Cover", shape.triangledown, location.abovebar, color.red)
```

## How to read it

It is a bearish reversal signal after an uptrend - the further the close sinks
into the prior body, the stronger the warning (a close below the prior open would
be a bearish engulfing). Confirm with a lower next bar.
