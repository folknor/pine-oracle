---
title: Three Black Crows
aliases: three black crows, 3 black crows
---

# Three Black Crows

Three Black Crows is the bearish mirror of Three White Soldiers: three
consecutive down candles, each closing lower than the last and each opening within
the prior candle's body. It shows steady, sustained selling. There is no
candlestick builtin in Pine.

## Recipe

```pine
//@version=6
indicator("Three Black Crows", "3BC", overlay = true)

bear3   = close < open and close[1] < open[1] and close[2] < open[2]
lower   = close < close[1] and close[1] < close[2]
inBody1 = open >= close[1] and open <= open[1]
inBody2 = open[1] >= close[2] and open[1] <= open[2]

isPat = bear3 and lower and inBody1 and inBody2
plotshape(isPat, "Three Black Crows", shape.triangledown, location.abovebar, color.red)
```

## How to read it

Appearing after an uptrend, it is a high-conviction bearish reversal: three
orderly declines with each open inside the prior body. Long lower wicks would
weaken it, hinting buyers are starting to defend.
