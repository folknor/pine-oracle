---
title: Morning Star
aliases: morning star
---

# Morning Star

The Morning Star is a three-candle bullish reversal: a long down candle, then a
small-bodied "star" that stalls the move, then a strong up candle that closes
well into the first candle's body. It marks a downtrend giving way to buyers.
There is no candlestick builtin in Pine.

## Recipe

```pine
//@version=6
indicator("Morning Star", "MorningStar", overlay = true)

body2 = math.abs(close[2] - open[2])    // first candle
body1 = math.abs(close[1] - open[1])    // star
mid2  = (open[2] + close[2]) / 2

isPat = body2 > 0 and close[2] < open[2] and body1 < body2 * 0.3 and close > open and close > mid2
plotshape(isPat, "Morning Star", shape.triangleup, location.belowbar, color.green)
```

## How to read it

It is a bottoming signal after a downtrend; the deeper the third candle closes
into the first body, the stronger the reversal. The small middle candle is the
indecision pivot - a Doji there makes it a Morning Doji Star.
