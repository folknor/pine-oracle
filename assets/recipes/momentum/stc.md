---
title: Schaff Trend Cycle
aliases: schaff trend cycle, stc
---

# Schaff Trend Cycle

Doug Schaff's Trend Cycle treats MACD as a cyclical signal and runs it through two
cascaded stochastic passes, each smoothed by a factor, to produce a fast,
0-100 trend oscillator that turns earlier than MACD. There is no `ta.stc()`
builtin. It is also exported as `stc()` by TradingView's `ta` library:
https://pine-facade.tradingview.com/pine-facade/lib_list/?lib_id_prefix=TradingView/ta/12

## Recipe

```pine
//@version=6
indicator("Schaff Trend Cycle", "STC")

tclen  = input.int(10, "Cycle Length", minval = 1)
fast   = input.int(12, "Fast",         minval = 1)
slow   = input.int(26, "Slow",         minval = 1)
factor = input.float(0.5, "Factor", minval = 0, maxval = 1)
src    = input.source(close, "Source")

stc(source, tcl, f, s, fac) =>
    macd = ta.ema(source, f) - ta.ema(source, s)
    // Pass 1: stochastic of the MACD, smoothed by factor.
    ll1  = ta.lowest(macd, tcl)
    rng1 = ta.highest(macd, tcl) - ll1
    var float st1 = 0.0
    st1 := rng1 > 0 ? 100 * (macd - ll1) / rng1 : nz(st1[1])
    var float pf = 0.0
    pf := nz(pf[1]) + fac * (st1 - nz(pf[1]))
    // Pass 2: stochastic of pass 1, smoothed by factor.
    ll2  = ta.lowest(pf, tcl)
    rng2 = ta.highest(pf, tcl) - ll2
    var float st2 = 0.0
    st2 := rng2 > 0 ? 100 * (pf - ll2) / rng2 : nz(st2[1])
    var float pff = 0.0
    pff := nz(pff[1]) + fac * (st2 - nz(pff[1]))
    pff

plot(stc(src, tclen, fast, slow, factor), "STC")
```

## How to read it

STC swings between 0 and 100. The common reading is a buy when it turns up from
below 25 and a sell when it turns down from above 75 - earlier than the equivalent
MACD cross, at the cost of more false turns in choppy markets.
