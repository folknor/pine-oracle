---
title: Dragonfly Doji
aliases: dragonfly doji
---

# Dragonfly Doji

A Dragonfly Doji is a Doji where open, high, and close cluster at the top: a tiny
body, almost no upper shadow, and a long lower shadow. It shows sellers drove
price down intrabar but buyers pushed it all the way back, a potential bullish
reversal. There is no candlestick builtin in Pine.

## Recipe

```pine
//@version=6
indicator("Dragonfly Doji", "DragonflyDoji", overlay = true)

body  = math.abs(close - open)
rng   = high - low
upper = high - math.max(open, close)
lower = math.min(open, close) - low

isPat = rng > 0 and body <= rng * 0.1 and upper <= rng * 0.1 and lower >= rng * 0.6
plotshape(isPat, "Dragonfly Doji", shape.diamond, location.belowbar, color.green)
```

## How to read it

It is most meaningful at the bottom of a downtrend or at support, where the long
lower wick shows rejected lows. Treat it as a reversal candidate to confirm with
the following bar, not a standalone buy.
