---
title: Triple Exponential Moving Average
aliases: triple exponential moving average, triple ema
---

# Triple Exponential Moving Average

The TEMA pushes the DEMA idea one step further, combining three nested EMAs to
strip out even more lag while staying smoother than a raw price series. There is
no `ta.tema()` builtin, so it is built from three `ta.ema` calls. It is also
exported as `tema()` by TradingView's `ta` library:
https://pine-facade.tradingview.com/pine-facade/lib_list/?lib_id_prefix=TradingView/ta/12

## Recipe

```pine
//@version=6
indicator("Triple Exponential Moving Average", "TEMA", overlay = true)

length = input.int(20, "Length", minval = 1)
src    = input.source(close, "Source")

tema(source, len) =>
    e1 = ta.ema(source, len)
    e2 = ta.ema(e1, len)
    e3 = ta.ema(e2, len)
    3 * (e1 - e2) + e3

plot(tema(src, length), "TEMA", color.orange, 2)
```

## How to read it

The TEMA reacts faster than both the EMA and DEMA of the same length, making it
popular for short-term trend timing. Because it leans hard into recent price, it
can overshoot at turns; pair it with a slower average to confirm direction.
