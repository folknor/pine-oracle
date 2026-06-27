---
title: Even Better SineWave
aliases: ebsw, even better sinewave, even better sine wave
---

# Even Better SineWave

Ehlers' Even Better SineWave high-pass filters price to remove the trend, smooths
the result with a two-pole super smoother, then normalizes the wave by the square
root of its average power so the output stays bound between -1 and +1. The length
caps the longest trend it will track. There is no `ta.ebsw()` builtin.

## Recipe

```pine
//@version=6
indicator("Even Better SineWave", "EBSW")

length = input.int(40, "Duration", minval = 39)
bars   = input.int(10, "Smoothing", minval = 1)
src    = input.source(close, "Source")

ebsw(source, len, smooth) =>
    deg    = 360.0 / len
    alpha1 = (1 - math.sin(math.toradians(deg))) / math.cos(math.toradians(deg))
    hp     = 0.0
    hp    := 0.5 * (1 + alpha1) * (source - nz(source[1])) + alpha1 * nz(hp[1])
    a1     = math.exp(-math.sqrt(2) * math.pi / smooth)
    b1     = 2 * a1 * math.cos(math.toradians(math.sqrt(2) * 180 / smooth))
    c2     = b1
    c3     = -a1 * a1
    c1     = 1 - c2 - c3
    filt   = 0.0
    filt  := c1 * (hp + nz(hp[1])) / 2 + c2 * nz(filt[1]) + c3 * nz(filt[2])
    wave   = (filt + nz(filt[1]) + nz(filt[2])) / 3
    pwr    = (filt * filt + nz(filt[1]) * nz(filt[1]) + nz(filt[2]) * nz(filt[2])) / 3
    pwr > 0 ? wave / math.sqrt(pwr) : 0.0

plot(ebsw(src, length, bars), "EBSW", color.blue)
```

## How to read it

The wave swings between -1 and +1; turns near the extremes call cycle tops and
bottoms, and a zero crossing confirms the swing. Because the power normalization
keeps the amplitude steady, the signal stays readable in quiet and volatile
markets alike. Note Ehlers' formula evaluates its trig in degrees, so this port
converts the angles before calling the radian-based `math.sin`/`math.cos`.
