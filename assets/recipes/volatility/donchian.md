---
title: Donchian Channels
aliases: donchian channels, donchian channel, dc
---

# Donchian Channels

Donchian Channels plot the highest high and lowest low over the window, with a
midline between them. They frame the recent price range and are the basis of
classic breakout systems (the Turtle traders). There is no `ta.donchian()`
builtin. It is also exported as `donchian()` by TradingView's `ta` library:
https://pine-facade.tradingview.com/pine-facade/lib_list/?lib_id_prefix=TradingView/ta/12

## Recipe

```pine
//@version=6
indicator("Donchian Channels", "DC", overlay = true)

length = input.int(20, "Length", minval = 1)

donchian(len) =>
    upper = ta.highest(high, len)
    lower = ta.lowest(low, len)
    [upper, (upper + lower) / 2, lower]

[dcU, dcM, dcL] = donchian(length)
plot(dcU, "Upper", color.blue)
plot(dcM, "Mid",   color.orange)
plot(dcL, "Lower", color.blue)
```

## How to read it

A close at the upper band is a bullish breakout (new N-bar high); a close at the
lower band is bearish. The channel widening signals rising volatility; the midline
acts as a mean-reversion target. Breakout systems buy the upper, sell the lower.
