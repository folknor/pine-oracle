---
title: Sine Weighted Moving Average
aliases: sine weighted moving average
---

# Sine Weighted Moving Average

The SINWMA weights the window with one half-cycle of a sine wave: the middle
bars get the highest weight and the ends taper to near zero. The weighting is
symmetric, so it behaves like a smooth, centred window. There is no
`ta.sinwma()` builtin.

## Recipe

```pine
//@version=6
indicator("Sine Weighted Moving Average", "SINWMA", overlay = true)

length = input.int(14, "Length", minval = 1)
src    = input.source(close, "Source")

sinwma(source, len) =>
    float num = 0.0
    float den = 0.0
    for i = 0 to len - 1
        float w = math.sin((i + 1) * math.pi / (len + 1))
        num += w * source[i]
        den += w
    num / den

plot(sinwma(src, length), "SINWMA", color.orange, 2)
```

## How to read it

The sine taper suppresses the noise at the edges of the window, giving a smoother
line than a plain WMA without much added lag. Read it like any moving average,
by slope and by price crossing it.
