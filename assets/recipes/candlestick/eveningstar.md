---
title: Evening Star
aliases: evening star
---

# Evening Star

The Evening Star is the bearish mirror of the Morning Star: a long up candle, a
small-bodied star, then a strong down candle that closes well into the first
candle's body. It marks an uptrend giving way to sellers. There is no candlestick
builtin in Pine.

## Recipe

```pine
//@version=6
indicator("Evening Star", "EveningStar", overlay = true)

body2 = math.abs(close[2] - open[2])    // first candle
body1 = math.abs(close[1] - open[1])    // star
mid2  = (open[2] + close[2]) / 2

isPat = body2 > 0 and close[2] > open[2] and body1 < body2 * 0.3 and close < open and close < mid2
plotshape(isPat, "Evening Star", shape.triangledown, location.abovebar, color.red)
```

## How to read it

It is a topping signal after an uptrend; the deeper the third candle sinks into
the first body, the stronger the reversal. A Doji as the middle candle makes it
an Evening Doji Star, considered more potent.
