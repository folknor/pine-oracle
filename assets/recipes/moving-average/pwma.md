---
title: Pascal Weighted Moving Average
aliases: pascal weighted moving average, pascals weighted moving average
---

# Pascal Weighted Moving Average

The PWMA weights the window with a row of Pascal's triangle (the binomial
coefficients), which form a smooth bell shape centred on the middle of the
window. It is a symmetric weighting similar to a Gaussian taper. There is no
`ta.pwma()` builtin.

## Recipe

```pine
//@version=6
indicator("Pascals Weighted Moving Average", "PWMA", overlay = true)

length = input.int(10, "Length", minval = 1)
src    = input.source(close, "Source")

pwma(source, len) =>
    float num = 0.0
    float den = 0.0
    float c   = 1.0
    for i = 0 to len - 1
        num += c * source[i]
        den += c
        c := c * (len - 1 - i) / (i + 1)
    num / den

plot(pwma(src, length), "PWMA", color.orange, 2)
```

## How to read it

The binomial bell concentrates weight in the centre of the window, so the PWMA
is smooth and centred much like a sine- or Gaussian-weighted average. The
running `c` term builds each binomial coefficient from the previous one, avoiding
a factorial. Read it by slope.
