---
title: Holt-Winter Channel
aliases: holt-winter channel, hwc
---

# Holt-Winter Channel

The Holt-Winter Channel wraps the Holt-Winter Moving Average (HWMA) in volatility
bands derived from a recursively smoothed variance of the price-to-average error.
It is the channel counterpart of the HWMA. There is no `ta.hwc()` builtin.

## Recipe

```pine
//@version=6
indicator("Holt-Winter Channel", "HWC", overlay = true)

nA     = input.float(0.2, "na")
nB     = input.float(0.1, "nb")
nC     = input.float(0.1, "nc")
nD     = input.float(0.1, "nd")
scalar = input.float(1, "Scalar")

hwc(a, b, c, d, sc) =>
    var float F = na
    var float V = 0.0
    var float A = 0.0
    var float varr = 0.0
    var float lastPrice  = na
    var float lastResult = na
    prevF = na(F) ? close : F
    lp    = na(lastPrice)  ? close : lastPrice
    lr    = na(lastResult) ? close : lastResult
    curF   = (1 - a) * (prevF + V + 0.5 * A) + a * close
    curV   = (1 - b) * (V + A) + b * (curF - prevF)
    curA   = (1 - c) * A + c * (curV - V)
    result = curF + curV + 0.5 * curA
    stddev = math.sqrt(varr)                       // prior-bar variance
    curVar = (1 - d) * varr + d * (lp - lr) * (lp - lr)
    F := curF
    V := curV
    A := curA
    varr := curVar
    lastPrice  := close
    lastResult := result
    [result, result + sc * stddev, result - sc * stddev]

[mid, up, lo] = hwc(nA, nB, nC, nD, scalar)
plot(up,  "Upper", color.blue)
plot(mid, "Mid",   color.orange)
plot(lo,  "Lower", color.blue)
```

## How to read it

The midline is the HWMA forecast; the bands breathe with recent error variance.
Price tagging a band signals a stretch from the forecast (possible reversion);
the bands widening signals rising volatility. See the [[hwma]] recipe for the
midline alone.
