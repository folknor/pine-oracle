---
title: Shooting Star
aliases: shooting star
---

# Shooting Star

A Shooting Star has the Inverted Hammer's shape - small body at the bottom, long
upper shadow, little lower shadow - but appears after an uptrend, signalling
rejected highs and a potential bearish reversal. There is no candlestick builtin
in Pine.

## Recipe

```pine
//@version=6
indicator("Shooting Star", "ShootingStar", overlay = true)

body  = math.abs(close - open)
rng   = high - low
upper = high - math.max(open, close)
lower = math.min(open, close) - low

isShape = rng > 0 and body > 0 and upper >= 2 * body and lower <= rng * 0.1
priorUp = close[1] > close[5]              // simple prior-uptrend filter
isPat   = isShape and priorUp
plotshape(isPat, "Shooting Star", shape.triangledown, location.abovebar, color.red)
```

## How to read it

The long upper wick after a rally shows buyers pushed to new highs and failed to
hold them. It is a bearish reversal candidate; confirm with the next bar closing
below the star's body. As with the Hanging Man, `priorUp` is only a minimal trend
filter.
