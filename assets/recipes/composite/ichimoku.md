---
title: Ichimoku Cloud
aliases: ichimoku cloud, ichimoku, ichimoku kinko hyo
---

# Ichimoku Cloud

Ichimoku Kinko Hyo is a complete trend system in one overlay: a fast Conversion
Line and slower Base Line (each the midpoint of a Donchian range), a forward-
projected "cloud" (Span A/B) marking future support/resistance, and a Chikou
(lagging) line. There is no `ta.ichimoku()` builtin.

## Recipe

```pine
//@version=6
indicator("Ichimoku Cloud", "Ichimoku", overlay = true)

conLen  = input.int(9,  "Conversion",   minval = 1)
baseLen = input.int(26, "Base",         minval = 1)
spanLen = input.int(52, "Span B",       minval = 1)
disp    = input.int(26, "Displacement", minval = 1)

midpoint(len) =>
    (ta.highest(high, len) + ta.lowest(low, len)) / 2

tenkan  = midpoint(conLen)
kijun   = midpoint(baseLen)
senkouA = math.avg(tenkan, kijun)
senkouB = midpoint(spanLen)

plot(tenkan,  "Conversion", color.blue)
plot(kijun,   "Base",       color.red)
plot(senkouA, "Span A",     color.green,  offset = disp)
plot(senkouB, "Span B",     color.orange, offset = disp)
plot(close,   "Chikou",     color.purple, offset = -disp)
```

## How to read it

Price above the cloud is bullish, below is bearish, inside is no-trend. The
Conversion crossing the Base (the "TK cross") is the entry trigger, strongest when
it agrees with price's position relative to the cloud. A thick cloud is strong
support/resistance; the Chikou confirms when it is clear of past price. (For
a true cloud fill use `fill()` between the two Span plots.)
