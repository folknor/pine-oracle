---
title: Inside Bar
aliases: inside bar, inside candle
---

# Inside Bar

An Inside Bar is a candle whose entire high-low range sits within the prior bar's
range: a lower high and a higher low. It marks a contraction in volatility and a
pause in the trend, often preceding a breakout. There is no candlestick builtin
in Pine.

## Recipe

```pine
//@version=6
indicator("Inside Bar", "Inside", overlay = true)

isPat = high < high[1] and low > low[1]
plotshape(isPat, "Inside Bar", shape.square, location.belowbar, close > open ? color.green : color.red)
```

## How to read it

The inside bar shows the market coiling inside the prior bar's range. Traders
watch for a break above the prior high (bullish) or below the prior low (bearish)
as the trigger; the pattern itself is a setup, not a direction.
