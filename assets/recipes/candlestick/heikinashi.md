---
title: Heikin-Ashi
aliases: heikin ashi, heikin-ashi, ha candles
---

# Heikin-Ashi

Heikin-Ashi ("average bar") redraws candles from averaged prices to filter noise
and make trends easier to read. Each HA close is the bar's OHLC average and each
HA open is the average of the prior HA open and close, producing smooth runs of
same-colored candles. There is no Heikin-Ashi builtin in Pine (though
`ticker.heikinashi()` can request HA data; this computes it inline).

## Recipe

```pine
//@version=6
indicator("Heikin-Ashi", "HA", overlay = true)

haClose = (open + high + low + close) / 4
var float haOpen = na
haOpen := na(haOpen[1]) ? (open + close) / 2 : (haOpen[1] + haClose[1]) / 2
haHigh = math.max(high, math.max(haOpen, haClose))
haLow  = math.min(low,  math.min(haOpen, haClose))

plotcandle(haOpen, haHigh, haLow, haClose, "Heikin-Ashi",
     color = haClose >= haOpen ? color.green : color.red)
```

## How to read it

Long unbroken runs of one color signal a strong trend; small bodies with wicks on
both sides signal indecision or a turn. Because HA averages prices, it lags real
price and hides gaps - use it to read trend, not for exact entry/exit levels.
