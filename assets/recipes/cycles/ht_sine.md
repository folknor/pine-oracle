---
title: Hilbert Transform - SineWave
aliases: ht_sine, htsine, hilbert transform sinewave, sine leadsine, ht sine
---

# Hilbert Transform - SineWave

This drives Ehlers' Hilbert Transform machinery (the same smoother, detrender,
in-phase/quadrature and homodyne discriminator as HT_DCPERIOD) all the way to the
dominant cycle phase, then plots two waves: Sine (the sine of that phase) and
LeadSine (the same phase advanced 45 degrees). Their crossovers lead price turns
in cycle mode. There is no `ta.ht_sine()` builtin. TradingView's `ta` library
exports the underlying Hilbert FIR primitive as `ht()` (not this full indicator):
https://pine-facade.tradingview.com/pine-facade/lib_list/?lib_id_prefix=TradingView/ta/12

## Recipe

```pine
//@version=6
indicator("HT SineWave", "HT_SINE")

src = input.source(close, "Source")

htSine(source) =>
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
    sine     = math.sin(math.toradians(dcPhase))
    leadSine = math.sin(math.toradians(dcPhase + 45.0))
    [sine, leadSine]

[sine, leadSine] = htSine(src)
plot(sine,     "HT_SINE",     color.blue)
plot(leadSine, "HT_LEADSINE", color.orange)
```

## How to read it

In a cycling market the two waves swing smoothly between -1 and +1; LeadSine
crossing above Sine is an early buy and crossing below is an early sell. When the
market trends, the waves flatten and wander apart, which is the signal to stop
trading the crossovers. Note: this is a faithful bar-by-bar port of TA-Lib's
recursive loop, so the early bars are a warmup that has not yet converged.
