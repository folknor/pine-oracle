---
title: Vertical Horizontal Filter
aliases: vertical horizontal filter, vhf
---

# Vertical Horizontal Filter

Adam White's VHF distinguishes trending from ranging markets. It divides the total
price range over the window (highest close minus lowest close) by the sum of
bar-to-bar changes: a directed move covers more net distance per unit of churn.
There is no `ta.vhf()` builtin. It is also exported as `vhf()` by TradingView's
`ta` library:
https://pine-facade.tradingview.com/pine-facade/lib_list/?lib_id_prefix=TradingView/ta/12

## Recipe

```pine
//@version=6
indicator("Vertical Horizontal Filter", "VHF")

length = input.int(28, "Length", minval = 1)
src    = input.source(close, "Source")

vhf(source, len) =>
    hcp = ta.highest(source, len)
    lcp = ta.lowest(source, len)
    math.abs(hcp - lcp) / math.sum(math.abs(ta.change(source)), len)

plot(vhf(src, length), "VHF")
```

## How to read it

High VHF (and rising) means a strong trend - net travel dominates the churn; low
VHF means a choppy, range-bound market. It is a regime filter: use trend-following
tools when VHF is high, mean-reversion tools when it is low.
