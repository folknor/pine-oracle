---
title: Squeeze Momentum
aliases: squeeze momentum, ttm squeeze, squeeze, lazybear squeeze
---

# Squeeze Momentum

John Carter's TTM Squeeze (in its widely-used LazyBear form) detects volatility
compression by checking when Bollinger Bands sit *inside* Keltner Channels - a
"squeeze" that often precedes a strong move - and pairs it with a linear-regression
momentum histogram to hint at the breakout direction. There is no `ta.squeeze()`
builtin (it composes `ta.kc`, plus a manual Bollinger band so the basis stays in
use).

## Recipe

```pine
//@version=6
indicator("Squeeze Momentum", "SQZ")

length = input.int(20, "Length", minval = 1)
bbMult = input.float(2.0, "BB Mult")
kcMult = input.float(1.5, "KC Mult")

squeeze(len, bbm, kcm) =>
    basis   = ta.sma(close, len)
    dev     = bbm * ta.stdev(close, len)
    bbUpper = basis + dev
    bbLower = basis - dev
    [kcBasis, kcUpper, kcLower] = ta.kc(close, len, kcm, true)
    sqzOn  = bbLower > kcLower and bbUpper < kcUpper   // BB inside KC
    sqzOff = bbLower < kcLower and bbUpper > kcUpper   // BB outside KC
    hh  = ta.highest(high, len)
    ll  = ta.lowest(low, len)
    avg = 0.25 * (hh + ll) + 0.5 * kcBasis
    val = ta.linreg(close - avg, len, 0)
    [val, sqzOn, sqzOff]

[momo, on, off] = squeeze(length, bbMult, kcMult)
plot(momo, "Momentum", momo >= 0 ? color.green : color.red, style = plot.style_histogram)
bgcolor(on ? color.new(color.gray, 80) : na)
plotchar(off, "Squeeze Off", "x", location.bottom)
```

## How to read it

While the background shows a squeeze (bands compressed), energy is building -
wait. The trade is the squeeze *releasing*: when it fires, the momentum
histogram's sign and slope point the likely breakout direction (rising green =
bullish thrust, falling red = bearish). The histogram alone is a momentum read;
the squeeze state is the timing.
