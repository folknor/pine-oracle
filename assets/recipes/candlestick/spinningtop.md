---
title: Spinning Top
aliases: spinning top
---

# Spinning Top

A Spinning Top is a small-bodied candle with upper and lower shadows both longer
than the body. Like a Doji it signals indecision, but with a slightly more
defined body. There is no candlestick builtin in Pine.

## Recipe

```pine
//@version=6
indicator("Spinning Top", "SpinningTop", overlay = true)

body  = math.abs(close - open)
rng   = high - low
upper = high - math.max(open, close)
lower = math.min(open, close) - low

isPat = rng > 0 and body <= rng * 0.3 and upper > body and lower > body
plotshape(isPat, "Spinning Top", shape.circle, location.belowbar, color.gray)
```

## How to read it

It marks a stalemate: neither side could hold control of the bar. After a strong
trend a cluster of spinning tops often precedes a reversal or consolidation;
alone it is weak and best used as a context clue rather than a trigger.
