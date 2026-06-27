---
title: Ease of Movement
aliases: ease of movement, eom, emv
---

# Ease of Movement

Richard Arms' Ease of Movement relates price change to volume: how far the
midpoint moved per unit of volume. It is high when price advances on light volume
(price moves "easily") and low when heavy volume is needed. There is no
`ta.eom()` builtin. It is also exported as `eom()` by TradingView's `ta` library:
https://pine-facade.tradingview.com/pine-facade/lib_list/?lib_id_prefix=TradingView/ta/12

## Recipe

```pine
//@version=6
indicator("Ease of Movement", "EOM")

length  = input.int(14, "Length", minval = 1)
divisor = input.float(100000000, "Divisor")

distance = hl2 - hl2[1]
boxRatio = volume / divisor / (high - low)
plot(ta.sma(distance / boxRatio, length), "EOM")
```

## How to read it

Above zero means price is rising with relative ease (bullish); below, falling
easily (bearish). High absolute values mean price moved a lot on little volume.
The zero-line cross is the signal; the divisor just scales volume to a readable
range.
