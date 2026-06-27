---
title: Hilbert Transform - Trend vs Cycle Mode
aliases: ht_trendmode, httrendmode, hilbert transform trend mode, trend vs cycle mode
---

# Hilbert Transform - Trend vs Cycle Mode

This runs the full Ehlers Hilbert Transform machinery (smoother, detrender,
in-phase/quadrature, homodyne discriminator, dominant cycle phase, Sine/LeadSine,
and an instantaneous trendline) and collapses it to a single 0/1 flag: 1 means
the market is trending, 0 means it is cycling. The flag follows TA-Lib's four-step
rule (crossover reset, bars-in-trend, phase-change range, price-vs-trendline
divergence). There is no `ta.ht_trendmode()` builtin.

## Recipe

```pine
//@version=6
indicator("HT Trend vs Cycle Mode", "HT_TRENDMODE")

src = input.source(close, "Source")

htTrendMode(source) =>
    var float period = 0.0
    var int   daysInTrend = 0
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
    sumC = 0.0
    for j = 0 to dcPeriodInt - 1
        sumC += nz(source[j], source)
    itTrend   = sumC / dcPeriodInt
    trendline = (4.0 * itTrend + 3.0 * nz(itTrend[1], itTrend) + 2.0 * nz(itTrend[2], itTrend) + nz(itTrend[3], itTrend)) / 10.0
    trend = 1
    sineCross = (sine > leadSine and nz(sine[1]) <= nz(leadSine[1])) or (sine < leadSine and nz(sine[1]) >= nz(leadSine[1]))
    if sineCross
        daysInTrend := 0
        trend := 0
    daysInTrend := daysInTrend + 1
    if daysInTrend < 0.5 * smoothPeriod
        trend := 0
    phaseDiff = dcPhase - nz(dcPhase[1])
    if smoothPeriod != 0.0 and phaseDiff > 0.67 * 360.0 / smoothPeriod and phaseDiff < 1.5 * 360.0 / smoothPeriod
        trend := 0
    if trendline != 0.0 and math.abs((smooth - trendline) / trendline) >= 0.015
        trend := 1
    trend

plot(htTrendMode(src), "HT_TRENDMODE", color.teal, style = plot.style_stepline)
```

## How to read it

The plot steps between 0 and 1: hold trend-following tools (moving averages,
breakouts) while it reads 1, and switch to cycle tools (the Sine/LeadSine
crossovers, oscillators) while it reads 0. It is a regime switch, not an entry
signal. Note: this is a faithful bar-by-bar port of TA-Lib's recursive loop, so
the early bars are a warmup that has not yet converged.
