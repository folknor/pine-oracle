---
title: Hilbert Transform - Dominant Cycle Period
aliases: ht_dcperiod, htdcperiod, hilbert transform dominant cycle period, dominant cycle period
---

# Hilbert Transform - Dominant Cycle Period

Ehlers' Hilbert Transform machinery measures the dominant cycle hiding in price.
A WMA-smoothed price is detrended into in-phase and quadrature components, and a
homodyne discriminator turns those into the cycle period in bars, which is then
clamped (6 to 50, and within +/-50% of the prior bar) and smoothed. There is no
`ta.ht_dcperiod()` builtin.

## Recipe

```pine
//@version=6
indicator("HT Dominant Cycle Period", "HT_DCPERIOD")

src = input.source(close, "Source")

htDcPeriod(source) =>
    var float period = 0.0
    smooth   = (4.0 * source + 3.0 * nz(source[1], source) + 2.0 * nz(source[2], source) + nz(source[3], source)) / 10.0
    adj      = 0.075 * nz(period[1]) + 0.54
    detrend  = (0.0962 * smooth  + 0.5769 * nz(smooth[2])  - 0.5769 * nz(smooth[4])  - 0.0962 * nz(smooth[6]))  * adj
    q1       = (0.0962 * detrend + 0.5769 * nz(detrend[2]) - 0.5769 * nz(detrend[4]) - 0.0962 * nz(detrend[6])) * adj
    i1       = nz(detrend[3])
    ji       = (0.0962 * i1 + 0.5769 * nz(i1[2]) - 0.5769 * nz(i1[4]) - 0.0962 * nz(i1[6])) * adj
    jq       = (0.0962 * q1 + 0.5769 * nz(q1[2]) - 0.5769 * nz(q1[4]) - 0.0962 * nz(q1[6])) * adj
    i2       = 0.2 * (i1 - jq) + 0.8 * nz(i2[1])
    q2       = 0.2 * (q1 + ji) + 0.8 * nz(q2[1])
    re       = 0.2 * (i2 * nz(i2[1]) + q2 * nz(q2[1])) + 0.8 * nz(re[1])
    im       = 0.2 * (i2 * nz(q2[1]) - q2 * nz(i2[1])) + 0.8 * nz(im[1])
    period  := im != 0.0 and re != 0.0 ? 360.0 / math.todegrees(math.atan(im / re)) : nz(period[1])
    if period > 1.5 * nz(period[1])
        period := 1.5 * nz(period[1])
    if period < 0.67 * nz(period[1])
        period := 0.67 * nz(period[1])
    period  := math.max(6.0, math.min(50.0, period))
    period  := 0.2 * period + 0.8 * nz(period[1])
    smoothPeriod = 0.33 * period + 0.67 * nz(smoothPeriod[1])
    smoothPeriod

plot(htDcPeriod(src), "HT_DCPERIOD", color.blue)
```

## How to read it

The plot is the estimated length, in bars, of the dominant cycle. Feed it as the
adaptive length of another indicator (an RSI or stochastic that breathes with the
market) instead of a fixed window. Note: this is a faithful bar-by-bar port of
TA-Lib's recursive loop, so the first ~30 bars are a warmup that has not yet
converged.
