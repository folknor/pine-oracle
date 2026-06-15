---
title: Three White Soldiers
aliases: three white soldiers, 3 white soldiers
---

# Three White Soldiers

Three White Soldiers is a strong three-candle bullish reversal: three consecutive
up candles, each closing higher than the last and each opening within the prior
candle's body. It shows steady, sustained buying. There is no candlestick builtin
in Pine.

## Recipe

```pine
//@version=6
indicator("Three White Soldiers", "3WS", overlay = true)

bull3   = close > open and close[1] > open[1] and close[2] > open[2]
higher  = close > close[1] and close[1] > close[2]
inBody1 = open <= close[1] and open >= open[1]
inBody2 = open[1] <= close[2] and open[1] >= open[2]

isPat = bull3 and higher and inBody1 and inBody2
plotshape(isPat, "Three White Soldiers", shape.triangleup, location.belowbar, color.green)
```

## How to read it

Appearing after a downtrend or consolidation, it is a high-conviction reversal:
three orderly advances with each open inside the prior body (not gapping). Very
long upper wicks would weaken it, hinting buyers are meeting resistance.
