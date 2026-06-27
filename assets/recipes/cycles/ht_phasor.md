---
title: Hilbert Transform Phasor
aliases: ht_phasor, hilbert transform phasor, phasor components
---

# Hilbert Transform Phasor

Ehlers' Hilbert Transform decomposes price into a rotating phasor: an in-phase
component and a quadrature component 90 degrees out of phase with it. Together
they trace the dominant cycle as a vector spinning once per cycle. The full
machinery (WMA smoothing, the FIR detrender, and the period-feedback loop) feeds
the two outputs. There is no `ta.ht_phasor()` builtin.

## Recipe

```pine
//@version=6
indicator("Hilbert Transform Phasor", "HT_PHASOR")

src = input.source(close, "Source")

ht_phasor(source) =>
    sp = (4 * source + 3 * nz(source[1]) + 2 * nz(source[2]) + nz(source[3])) / 10.0
    float period = na
    adj     = 0.075 * nz(period[1]) + 0.54
    detrend = (0.0962 * sp + 0.5769 * nz(sp[2]) - 0.5769 * nz(sp[4]) - 0.0962 * nz(sp[6])) * adj
    q1      = (0.0962 * detrend + 0.5769 * nz(detrend[2]) - 0.5769 * nz(detrend[4]) - 0.0962 * nz(detrend[6])) * adj
    i1      = nz(detrend[3])
    ji      = (0.0962 * i1 + 0.5769 * nz(i1[2]) - 0.5769 * nz(i1[4]) - 0.0962 * nz(i1[6])) * adj
    jq      = (0.0962 * q1 + 0.5769 * nz(q1[2]) - 0.5769 * nz(q1[4]) - 0.0962 * nz(q1[6])) * adj
    i2      = i1 - jq
    q2      = q1 + ji
    i2     := 0.2 * i2 + 0.8 * nz(i2[1])
    q2     := 0.2 * q2 + 0.8 * nz(q2[1])
    re      = i2 * nz(i2[1]) + q2 * nz(q2[1])
    im      = i2 * nz(q2[1]) - q2 * nz(i2[1])
    re     := 0.2 * re + 0.8 * nz(re[1])
    im     := 0.2 * im + 0.8 * nz(im[1])
    period := im != 0 and re != 0 ? 360.0 / math.todegrees(math.atan(im / re)) : nz(period[1])
    period := math.min(period, 1.5 * nz(period[1]))
    period := math.max(period, 0.67 * nz(period[1]))
    period := math.max(period, 6.0)
    period := math.min(period, 50.0)
    period := 0.2 * period + 0.8 * nz(period[1])
    [i1, q1]

[inphase, quad] = ht_phasor(src)
plot(inphase, "InPhase", color.blue)
plot(quad, "Quadrature", color.orange)
```

## How to read it

The two lines lead and lag each other by a quarter cycle, so their crossings pace
the dominant cycle and the angle between them tracks its phase. When in-phase and
quadrature swing with similar, steady amplitude a clean cycle is present; when one
collapses toward zero the cycle has weakened and the market is trending. The
period loop self-adapts, so no length input is needed.
