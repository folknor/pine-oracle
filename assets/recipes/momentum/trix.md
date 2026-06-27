---
title: TRIX
aliases: trix, triple exponential average
---

# TRIX

TRIX is the one-bar rate of change of a triple-smoothed EMA. The triple
smoothing filters out price moves shorter than the length, leaving a clean
momentum oscillator that is well suited to divergence spotting. There is no
`ta.trix()` builtin. It is also exported as `trix()` by TradingView's `ta`
library:
https://pine-facade.tradingview.com/pine-facade/lib_list/?lib_id_prefix=TradingView/ta/12

## Recipe

```pine
//@version=6
indicator("TRIX", "TRIX")

length = input.int(18, "Length", minval = 1)
sigLen = input.int(9,  "Signal", minval = 1)
src    = input.source(close, "Source")

trix(source, len, sig) =>
    e1   = ta.ema(source, len)
    e2   = ta.ema(e1, len)
    e3   = ta.ema(e2, len)
    line = ta.roc(e3, 1)
    [line, ta.sma(line, sig)]

[trixLine, trixSignal] = trix(src, length, sigLen)
plot(trixLine,   "TRIX",   color.blue)
plot(trixSignal, "Signal", color.orange)
```

## How to read it

Zero-line crossings mark momentum turning positive or negative; line/signal
crossovers give earlier triggers. Its main use is divergence: price making a new
extreme that TRIX does not confirm warns of a fading trend.
