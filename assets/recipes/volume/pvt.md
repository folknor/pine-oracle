---
title: Price Volume Trend
aliases: price volume trend, pvt
---

# Price Volume Trend

The Price Volume Trend accumulates volume weighted by the percentage price
change, so each bar adds more when both the move and the volume are large. It is a
volume-confirmation line like OBV but scaled by the size of the move. There is no
`ta.pvt()` builtin.

## Recipe

```pine
//@version=6
indicator("Price Volume Trend", "PVT")

plot(ta.cum(ta.change(close) / close[1] * volume), "PVT")
```

## How to read it

A rising PVT confirms an uptrend is backed by volume; falling confirms a
downtrend. Because it weights by percentage change (unlike OBV, which adds full
volume regardless of move size), it reacts more to big moves. Watch for divergence
against price.
