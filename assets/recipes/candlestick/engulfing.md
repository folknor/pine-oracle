---
title: Engulfing Pattern
aliases: engulfing, bullish engulfing, bearish engulfing
---

# Engulfing Pattern

An engulfing pattern is a two-candle reversal signal. A bullish engulfing has a
down candle followed by an up candle whose real body fully covers the prior
body; a bearish engulfing is the mirror. TradingView ships no pattern builtin,
so the condition is expressed directly over `open` / `close`.

## Recipe

```pine
//@version=6
indicator("Engulfing Pattern", overlay = true)

bodyHigh(idx) => math.max(open[idx], close[idx])
bodyLow(idx)  => math.min(open[idx], close[idx])

bullEngulf = close[1] < open[1] and close > open and
             bodyHigh(0) >= bodyHigh(1) and bodyLow(0) <= bodyLow(1)
bearEngulf = close[1] > open[1] and close < open and
             bodyHigh(0) >= bodyHigh(1) and bodyLow(0) <= bodyLow(1)

plotshape(bullEngulf, "Bullish", shape.triangleup,   location.belowbar, color.green)
plotshape(bearEngulf, "Bearish", shape.triangledown, location.abovebar, color.red)
```

## How to read it

The pattern is read as a momentum flip: the engulfing body shows the new side
overwhelmed the prior bar's range. It carries more weight after an extended move
in the opposite direction and is usually confirmed by the next bar holding the
new direction.
