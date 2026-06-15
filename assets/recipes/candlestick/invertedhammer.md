---
title: Inverted Hammer
aliases: inverted hammer
---

# Inverted Hammer

An Inverted Hammer is a small-bodied candle at the bottom of its range with a
long upper shadow (at least twice the body) and little or no lower shadow.
Appearing after a decline, it hints that buyers tested higher prices - a
potential bullish reversal. There is no candlestick builtin in Pine.

## Recipe

```pine
//@version=6
indicator("Inverted Hammer", "InvHammer", overlay = true)

body  = math.abs(close - open)
rng   = high - low
upper = high - math.max(open, close)
lower = math.min(open, close) - low

isPat = rng > 0 and body > 0 and upper >= 2 * body and lower <= rng * 0.1
plotshape(isPat, "Inverted Hammer", shape.triangleup, location.belowbar, color.green)
```

## How to read it

Like the Hammer, it is a reversal signal only after a downtrend; the identical
shape after an uptrend is a Shooting Star (bearish). It is weaker than a Hammer
and should be confirmed by a strong up bar next.
