---
title: Hilbert Transform - Dominant Cycle Phase
aliases: ht_dcphase, htdcphase, hilbert transform dominant cycle phase, dominant cycle phase
---

# Hilbert Transform - Dominant Cycle Phase

This shares Ehlers' Hilbert Transform machinery with HT_DCPERIOD (smoother,
detrender, in-phase/quadrature, homodyne discriminator) but outputs the phase
angle of the dominant cycle instead of its length. The phase is recovered by
summing the smoothed price against sine and cosine of the cycle period, then
correcting the quadrant. There is no `ta.ht_dcphase()` builtin. TradingView's
`ta` library exports the underlying Hilbert FIR primitive as `ht()` (not this
full indicator):
https://pine-facade.tradingview.com/pine-facade/lib_list/?lib_id_prefix=TradingView/ta/12

## Recipe

```pine
//@version=6
indicator("HT Dominant Cycle Phase", "HT_DCPHASE")

src = input.source(close, "Source")

htDcPhase(source) =>
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
    dcPeriodInt  = math.max(int(smoothPeriod + 0.5), 1)
    realPart = 0.0
    imagPart = 0.0
    for j = 0 to dcPeriodInt - 1
        angle = 2.0 * math.pi * j / dcPeriodInt
        spv   = nz(smooth[j])
        realPart += math.sin(angle) * spv
        imagPart += math.cos(angle) * spv
    dcPhase = 0.0
    if math.abs(imagPart) > 0.0
        dcPhase := math.todegrees(math.atan(realPart / imagPart))
    else
        dcPhase := nz(dcPhase[1])
        dcPhase := realPart < 0.0 ? dcPhase - 90.0 : realPart > 0.0 ? dcPhase + 90.0 : dcPhase
    dcPhase := dcPhase + 90.0
    if smoothPeriod > 0.0
        dcPhase := dcPhase + 360.0 / smoothPeriod
    if imagPart < 0.0
        dcPhase := dcPhase + 180.0
    if dcPhase > 315.0
        dcPhase := dcPhase - 360.0
    dcPhase

plot(htDcPhase(src), "HT_DCPHASE", color.purple)
```

## How to read it

The output is the cycle phase in degrees, sweeping repeatedly from low to high as
the market rotates through one cycle. A clean, steady ramp signals a clear cyclic
rhythm; a phase that stalls or jumps erratically signals a trend or noise where
cycle timing is unreliable. Note: this is a faithful bar-by-bar port of TA-Lib's
recursive loop, so the early bars are a warmup that has not yet converged.
