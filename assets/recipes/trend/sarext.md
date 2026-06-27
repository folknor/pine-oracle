---
title: Parabolic SAR Extended
aliases: sarext, parabolic sar extended, extended parabolic sar, extended sar
---

# Parabolic SAR Extended

SAREXT is the Parabolic SAR with separate acceleration factors for long and
short legs plus an optional fractional offset applied on each reversal. The
builtin `ta.sar()` exposes only a single shared acceleration, so the extended
parameters force a hand-rolled stepwise SAR with `var` state and `[1]` feedback.
The signed output is positive while long and negative while short.

## Recipe

```pine
//@version=6
indicator("Parabolic SAR Extended", "SAREXT", overlay = true)

startVal = input.float(0.0,  "Start Value")
offRev   = input.float(0.0,  "Offset On Reverse", step = 0.01)
afInitL  = input.float(0.02, "AF Init Long",  step = 0.01)
afStepL  = input.float(0.02, "AF Step Long",  step = 0.01)
afMaxL   = input.float(0.2,  "AF Max Long",   step = 0.01)
afInitS  = input.float(0.02, "AF Init Short", step = 0.01)
afStepS  = input.float(0.02, "AF Step Short", step = 0.01)
afMaxS   = input.float(0.2,  "AF Max Short",  step = 0.01)

sarext(start, off, af0L, afL, maxL, af0S, afS, maxS) =>
    var bool  falling = false
    var float sar     = na
    var float ep      = na
    var float af      = na
    float out = na
    if na(sar)
        falling := false
        sar     := start != 0.0 ? start : low
        ep      := high
        af      := af0L
    else
        bool  rev = false
        float s   = sar + af * (ep - sar)
        if falling
            rev := high > s
            if low < ep
                ep := low
                af := math.min(af + afS, maxS)
            s := math.max(high[1], nz(high[2], high[1]), s)
        else
            rev := low < s
            if high > ep
                ep := high
                af := math.min(af + afL, maxL)
            s := math.min(low[1], nz(low[2], low[1]), s)
        if rev
            if off != 0.0
                s := falling ? s + off * s : s - off * s
            falling := not falling
            if falling
                ep := low
                af := af0S
            else
                ep := high
                af := af0L
        sar := s
        out := falling ? -sar : sar
    out

sx = sarext(startVal, offRev, afInitL, afStepL, afMaxL, afInitS, afStepS, afMaxS)
plot(math.abs(sx), "SAREXT", sx >= 0 ? color.green : color.red, style = plot.style_cross)
```

## How to read it

The dots trail below price on long legs (green) and above on short legs (red);
a touch reverses the leg and the offset, if set, nudges the new stop further from
price to dampen whipsaw. Separate long/short acceleration lets a trend that runs
faster in one direction tighten its stop sooner on that side. The source seeds
the initial direction from the first two bars' directional movement; this recipe
seeds long on the first bar, after which the path converges to the same series.
