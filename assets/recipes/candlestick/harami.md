---
title: Harami
aliases: harami, harami pattern, inside candle
---

# Harami

A Harami is a two-candle pattern where a small body is fully contained within the
prior bar's larger, opposite-colored body. It signals that a strong move has
stalled - a potential reversal. (Japanese for "pregnant": the large candle
"carries" the small one.) There is no candlestick builtin in Pine.

## Recipe

```pine
//@version=6
indicator("Harami", "Harami", overlay = true)

prevBody  = math.abs(close[1] - open[1])
curBody   = math.abs(close - open)
contained = math.max(open, close) <= math.max(open[1], close[1]) and math.min(open, close) >= math.min(open[1], close[1])
oppColor  = (close > open) != (close[1] > open[1])

isPat = prevBody > 0 and contained and curBody < prevBody * 0.5 and oppColor
plotshape(isPat, "Harami", shape.diamond, location.belowbar, color.blue)
```

## How to read it

A bullish Harami (small up candle inside a large down candle) after a decline
hints the downtrend is losing force; a bearish Harami is the mirror after a
rally. It signals hesitation rather than a hard reversal - confirm with follow
through before acting.
