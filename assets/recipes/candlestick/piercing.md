---
title: Piercing Line
aliases: piercing line, piercing pattern
---

# Piercing Line

A Piercing Line is a two-candle bullish reversal: a down candle followed by an up
candle that opens below the prior close but rallies to close above the midpoint
of the prior body. Buyers reclaimed more than half the prior loss. There is no
candlestick builtin in Pine.

## Recipe

```pine
//@version=6
indicator("Piercing Line", "Piercing", overlay = true)

midPrior = (open[1] + close[1]) / 2
isPat = close[1] < open[1] and close > open and open < close[1] and
        close > midPrior and close < open[1]
plotshape(isPat, "Piercing", shape.triangleup, location.belowbar, color.green)
```

## How to read it

It is a bullish reversal signal at the end of a downtrend - the deeper the close
pierces into the prior body, the stronger the signal (a close all the way above
the prior open would be a bullish engulfing). Confirm with continuation on the
next bar.
