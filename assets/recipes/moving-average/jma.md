---
title: Jurik Moving Average
aliases: jma, jurik moving average, jurik ma
---

# Jurik Moving Average

Mark Jurik's JMA chases the "true" underlying price with very low lag and very
little overshoot. It adapts its smoothing to recent volatility (a Jurik
volatility band), then runs three stages: an adaptive EMA, a Kalman-style
correction weighted by the `phase` parameter, and a final Jurik filter. There
is no `ta.jma()` builtin, and because each stage feeds back on the prior bar it
is built with `var` state and `[1]` history rather than recursion.

## Recipe

```pine
//@version=6
indicator("Jurik Moving Average", "JMA", overlay = true)

length = input.int(7, "Length", minval = 1)
phase  = input.float(0, "Phase", minval = -100, maxval = 100)
src    = input.source(close, "Source")

pr   = phase < -100 ? 0.5 : phase > 100 ? 2.5 : 1.5 + phase * 0.01
half = 0.5 * (length - 1)
len1 = math.max(math.log(math.sqrt(half)) / math.log(2.0) + 2.0, 0)
pow1 = math.max(len1 - 2.0, 0.5)
len2 = len1 * math.sqrt(half)
bet  = len2 / (len2 + 1)
beta = 0.45 * (length - 1) / (0.45 * (length - 1) + 2.0)

var float uBand = src
var float lBand = src
var float ma1   = src
var float det0  = 0.0
var float det1  = 0.0
var float vSum  = 0.0
var float jma   = src

del1  = src - uBand
del2  = src - lBand
volty = math.abs(del1) != math.abs(del2) ? math.max(math.abs(del1), math.abs(del2)) : 0.0

vSum := vSum + (volty - nz(volty[10])) / 10
avgVolty = ta.sma(vSum, 66)
dVolty   = avgVolty == 0 ? 0 : volty / avgVolty
rVolty   = math.max(1.0, math.min(math.pow(len1, 1 / pow1), dVolty))

pow2  = math.pow(rVolty, pow1)
kv    = math.pow(bet, math.sqrt(pow2))
uBand := del1 > 0 ? src : src - kv * del1
lBand := del2 < 0 ? src : src - kv * del2

alpha = math.pow(beta, pow2)
ma1  := (1 - alpha) * src + alpha * ma1
det0 := (src - ma1) * (1 - beta) + beta * det0
ma2   = ma1 + pr * det0
det1 := (ma2 - jma) * (1 - alpha) * (1 - alpha) + alpha * alpha * det1
jma  := jma + det1

plot(jma, "JMA", color.orange, 2)
```

## How to read it

The JMA tracks price closely while staying smooth through noise, so it works as
both a fast signal line and a low-lag trend filter. Lower `phase` values lean
toward smoothness, higher values toward responsiveness (more overshoot on
turns). Read it by slope and by price crossing the line.
