---
title: Kaufman Adaptive Moving Average
aliases: kaufman adaptive moving average, kaufman moving average
---

# Kaufman Adaptive Moving Average

The KAMA speeds up when price trends and slows down when price chops, by scaling
its smoothing constant with Kaufman's Efficiency Ratio (directional travel over
total travel). It hugs strong trends yet flattens through noise. There is no
`ta.kama()` builtin. It is also exported as `kama()` by TradingView's `ta`
library:
https://pine-facade.tradingview.com/pine-facade/lib_list/?lib_id_prefix=TradingView/ta/12

## Recipe

```pine
//@version=6
indicator("Kaufman Adaptive Moving Average", "KAMA", overlay = true)

length = input.int(10, "Efficiency Length", minval = 1)
src    = input.source(close, "Source")

kama(source, len) =>
    fastAlpha = 2.0 / (2 + 1)
    slowAlpha = 2.0 / (30 + 1)
    momentum   = math.abs(ta.change(source, len))
    volatility = math.sum(math.abs(ta.change(source)), len)
    efficiency = volatility != 0 ? momentum / volatility : 0.0
    alpha = math.pow(efficiency * (fastAlpha - slowAlpha) + slowAlpha, 2)
    var float result = na
    result := na(result[1]) ? source : result[1] + alpha * (source - result[1])
    result

plot(kama(src, length), "KAMA", color.orange, 2)
```

## How to read it

A flat KAMA marks a noisy, directionless market (treat its level as support or
resistance); a steep KAMA marks an efficient trend worth following. It is often
used as a trend filter precisely because it stops chasing price when efficiency
drops.
