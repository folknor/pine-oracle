---
title: TTM Trend
aliases: ttm trend, ttm_trend
---

# TTM Trend

John Carter's TTM Trend colours bars by whether the close sits above or below the
average of recent median prices (`hl2`). It is meant to keep you in a trade until
the colour flips. There is no `ta.ttm_trend()` builtin.

## Recipe

```pine
//@version=6
indicator("TTM Trend", "TTM_TREND", overlay = true)

length = input.int(6, "Length", minval = 1)

ttmTrend(len) =>
    trendAvg = ta.sma(hl2, len)
    close > trendAvg ? 1 : -1

dir = ttmTrend(length)
plot(dir, "TTM Trend", dir > 0 ? color.green : color.red, style = plot.style_histogram)
barcolor(dir > 0 ? color.green : color.red)
```

## How to read it

Green (+1) means price is above the recent average - stay long; red (-1) means
below - stay short or out. Carter's rule: two consecutive bars of the opposite
colour is the signal to flip. It is a simple trend-persistence filter.
