---
title: Hammer
aliases: hammer, hammer candle
---

# Hammer

A Hammer is a small-bodied candle at the top of its range with a long lower
shadow (at least twice the body) and little or no upper shadow. Appearing after a
decline, it signals that sellers were overwhelmed intrabar - a potential bullish
reversal. There is no candlestick builtin in Pine.

## Recipe

```pine
//@version=6
indicator("Hammer", "Hammer", overlay = true)

body  = math.abs(close - open)
rng   = high - low
upper = high - math.max(open, close)
lower = math.min(open, close) - low

isPat = rng > 0 and body > 0 and lower >= 2 * body and upper <= rng * 0.1
plotshape(isPat, "Hammer", shape.triangleup, location.belowbar, color.green)
```

## How to read it

The Hammer is a reversal signal only in context - after a downtrend or at
support. The same shape appearing after an uptrend is a Hanging Man (bearish).
Confirm with a higher close on the next bar before acting.
