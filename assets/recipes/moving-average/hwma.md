---
title: Holt-Winter Moving Average
aliases: holt-winter moving average, holt winter moving average
---

# Holt-Winter Moving Average

The HWMA applies the Holt-Winter forecasting method as a moving average: it
tracks a level (`F`), a trend/velocity (`V`), and an acceleration (`A`), each
with its own smoothing parameter, and sums them into a forward-leaning estimate
of price. There is no `ta.hwma()` builtin.

## Recipe

```pine
//@version=6
indicator("Holt-Winter Moving Average", "HWMA", overlay = true)

nA  = input.float(0.2, "Smoothing (na)",   minval = 0, maxval = 1)
nB  = input.float(0.1, "Trend (nb)",       minval = 0, maxval = 1)
nC  = input.float(0.1, "Seasonality (nc)", minval = 0, maxval = 1)
src = input.source(close, "Source")

hwma(source, smooth, trend, season) =>
    var float F = na
    var float V = 0.0
    var float A = 0.0
    float prevF = na(F) ? source : F
    float curF  = (1.0 - smooth) * (prevF + V + 0.5 * A) + smooth * source
    float curV  = (1.0 - trend)  * (V + A) + trend * (curF - prevF)
    float curA  = (1.0 - season) * A + season * (curV - V)
    F := curF
    V := curV
    A := curA
    curF + curV + 0.5 * curA

plot(hwma(src, nA, nB, nC), "HWMA", color.orange, 2)
```

## How to read it

Higher `na` makes the level track price faster; `nb` and `nc` control how much
the trend and acceleration terms lead price into turns. Tuned conservatively it
is a smooth trend line; tuned hot it leans ahead of price and can overshoot.
