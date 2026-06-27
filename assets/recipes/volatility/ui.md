---
title: Ulcer Index
aliases: ulcer index, ui
---

# Ulcer Index

Peter Martin's Ulcer Index measures downside volatility only: the root-mean-square
of percentage drawdowns from the running high. Squaring the drawdowns emphasises
deep, prolonged declines - the "ulcers" a portfolio causes. There is no `ta.ui()`
builtin. It is also exported as `ulcerIndex()` by TradingView's `ta` library:
https://pine-facade.tradingview.com/pine-facade/lib_list/?lib_id_prefix=TradingView/ta/12

## Recipe

```pine
//@version=6
indicator("Ulcer Index", "UI")

length = input.int(14, "Length", minval = 1)
src    = input.source(close, "Source")

ui(source, len) =>
    hc       = ta.highest(source, len)
    downside = 100 * (source - hc) / hc
    math.sqrt(math.sum(downside * downside, len) / len)

plot(ui(src, length), "UI")
```

## How to read it

A low UI means shallow, brief drawdowns (smooth uptrend); a high or rising UI
means deep or sustained declines. Unlike standard deviation it ignores upside
moves entirely, so it is a cleaner risk gauge for long positions.
