---
title: Marubozu
aliases: marubozu
---

# Marubozu

A Marubozu is a candle with effectively no shadows: open and close sit at the
extremes of the range, so one side controlled the entire bar. A bullish Marubozu
opens at the low and closes at the high; a bearish one is the reverse. There is
no candlestick builtin in Pine.

## Recipe

```pine
//@version=6
indicator("Marubozu", "Marubozu", overlay = true)

rng   = high - low
upper = high - math.max(open, close)
lower = math.min(open, close) - low

isPat = rng > 0 and upper <= rng * 0.05 and lower <= rng * 0.05
plotshape(isPat, "Marubozu", shape.square, location.belowbar, close > open ? color.green : color.red)
```

## How to read it

A Marubozu is a strong continuation/conviction signal: the absence of wicks means
no rejection of the move. A bullish Marubozu in an uptrend confirms momentum; one
appearing against the trend can mark a powerful reversal thrust.
