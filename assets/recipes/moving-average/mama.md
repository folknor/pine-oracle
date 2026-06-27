---
title: MESA Adaptive Moving Average
aliases: mama, fama, mesa adaptive moving average, following adaptive moving average
---

# MESA Adaptive Moving Average

John Ehlers' MAMA measures the dominant cycle with a Hilbert transform and sets
its smoothing factor from the rate of change of that cycle's phase: fast when
price turns, slow when it drifts. FAMA (Following Adaptive MA) is a second,
gentler pass over MAMA, and the pair behaves like an adaptive crossover system.
There is no `ta.mama()` builtin, and the Hilbert chain feeds back bar to bar, so
it is built with `var` state and `[1]` history. This is Ehlers' canonical
formulation, equivalent to the even/odd Hilbert variant pandas-ta ports from
TA-Lib.

## Recipe

```pine
//@version=6
indicator("MESA Adaptive Moving Average", "MAMA", overlay = true)

fastLimit = input.float(0.5,  "Fast Limit", minval = 0)
slowLimit = input.float(0.05, "Slow Limit", minval = 0)
src       = input.source(close, "Source")

deg = 180.0 / math.pi

var float detrender = 0.0
var float i1        = 0.0
var float q1        = 0.0
var float i2        = 0.0
var float q2        = 0.0
var float re        = 0.0
var float im        = 0.0
var float period    = 0.0
var float phase     = 0.0
var float mama      = na
var float fama      = na

adj    = 0.075 * nz(period[1]) + 0.54
smooth = (4 * src + 3 * nz(src[1], src) + 2 * nz(src[2], src) + nz(src[3], src)) / 10

detrender := (0.0962 * smooth + 0.5769 * nz(smooth[2]) - 0.5769 * nz(smooth[4]) - 0.0962 * nz(smooth[6])) * adj
q1        := (0.0962 * detrender + 0.5769 * nz(detrender[2]) - 0.5769 * nz(detrender[4]) - 0.0962 * nz(detrender[6])) * adj
i1        := nz(detrender[3])
ji         = (0.0962 * i1 + 0.5769 * nz(i1[2]) - 0.5769 * nz(i1[4]) - 0.0962 * nz(i1[6])) * adj
jq         = (0.0962 * q1 + 0.5769 * nz(q1[2]) - 0.5769 * nz(q1[4]) - 0.0962 * nz(q1[6])) * adj

i2 := 0.2 * (i1 - jq) + 0.8 * nz(i2[1])
q2 := 0.2 * (q1 + ji) + 0.8 * nz(q2[1])

re := 0.2 * (i2 * nz(i2[1]) + q2 * nz(q2[1])) + 0.8 * nz(re[1])
im := 0.2 * (i2 * nz(q2[1]) - q2 * nz(i2[1])) + 0.8 * nz(im[1])

periodPrev = nz(period[1])
period := im != 0 and re != 0 ? 360.0 / (math.atan(im / re) * deg) : periodPrev
period := period > 1.5 * periodPrev ? 1.5 * periodPrev : period < 0.67 * periodPrev ? 0.67 * periodPrev : period
period := period < 6.0 ? 6.0 : period > 50.0 ? 50.0 : period
period := 0.2 * period + 0.8 * periodPrev

phasePrev = phase
phase := i1 != 0 ? math.atan(q1 / i1) * deg : phase
deltaPhase = math.max(phasePrev - phase, 1.0)
alpha = math.max(fastLimit / deltaPhase, slowLimit)

mama := alpha * src + (1 - alpha) * nz(mama[1], src)
fama := 0.5 * alpha * mama + (1 - 0.5 * alpha) * nz(fama[1], src)

plot(mama, "MAMA", color.orange, 2)
plot(fama, "FAMA", color.aqua,   2)
```

## How to read it

Trade MAMA and FAMA as a crossover pair: MAMA above FAMA is an uptrend, below is
a downtrend, and the crossings are the signals. Because the smoothing adapts,
the lines hug price through trends and pull apart sharply at turns. A wider fast
limit makes MAMA more reactive; a lower slow limit lets it coast longer in quiet
markets.
