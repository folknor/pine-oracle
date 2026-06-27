---
title: Ultimate Oscillator
aliases: ultimate oscillator, uo
---

# Ultimate Oscillator

Larry Williams' Ultimate Oscillator blends buying pressure over three time frames
(fast, medium, slow) into a single 0-100 oscillator, weighting the fast horizon
most. Using three periods is meant to reduce the false divergences that plague
single-period oscillators. There is no `ta.uo()` builtin. It is also exported as
`uo()` by TradingView's `ta` library:
https://pine-facade.tradingview.com/pine-facade/lib_list/?lib_id_prefix=TradingView/ta/12

## Recipe

```pine
//@version=6
indicator("Ultimate Oscillator", "UO")

fast   = input.int(7,  "Fast",   minval = 1)
medium = input.int(14, "Medium", minval = 1)
slow   = input.int(28, "Slow",   minval = 1)

uo(f, m, s) =>
    minLow  = math.min(low, close[1])
    maxHigh = math.max(high, close[1])
    bp = close - minLow         // buying pressure
    tr = maxHigh - minLow       // true range
    avgF = math.sum(bp, f) / math.sum(tr, f)
    avgM = math.sum(bp, m) / math.sum(tr, m)
    avgS = math.sum(bp, s) / math.sum(tr, s)
    100 * (4 * avgF + 2 * avgM + avgS) / 7

plot(uo(fast, medium, slow), "UO")
```

## How to read it

Conventional thresholds are 70 (overbought) and 30 (oversold). Williams' setup
looks for divergence between price and UO, then enters when UO breaks the high of
the divergence. Buying pressure is the close measured from the lower of this
low and the prior close, normalized by true range.
