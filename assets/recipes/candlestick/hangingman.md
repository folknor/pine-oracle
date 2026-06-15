---
title: Hanging Man
aliases: hanging man
---

# Hanging Man

A Hanging Man has the same shape as a Hammer - small body at the top, long lower
shadow, little upper shadow - but appears after an uptrend, where it warns that
sellers are starting to press. The shape is identical to the Hammer; only the
prior trend distinguishes them. There is no candlestick builtin in Pine.

## Recipe

```pine
//@version=6
indicator("Hanging Man", "HangingMan", overlay = true)

body  = math.abs(close - open)
rng   = high - low
upper = high - math.max(open, close)
lower = math.min(open, close) - low

isShape = rng > 0 and body > 0 and lower >= 2 * body and upper <= rng * 0.1
priorUp = close[1] > close[5]              // simple prior-uptrend filter
isPat   = isShape and priorUp
plotshape(isPat, "Hanging Man", shape.triangledown, location.abovebar, color.red)
```

## How to read it

It is a bearish reversal warning only after a rally. The long lower wick shows
intrabar selling that buyers undid - but the appearance of supply at highs is the
caution. Confirm with a lower close on the next bar. The `priorUp` check is a
minimal trend filter; a real strategy would use a stronger trend definition.
