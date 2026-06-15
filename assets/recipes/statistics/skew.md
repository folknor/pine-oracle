---
title: Skew
aliases: skew, skewness, rolling skew
---

# Skew

Rolling skewness measures the asymmetry of the return distribution over the
window (adjusted Fisher-Pearson): positive skew means a longer right tail (more
big up moves), negative means a longer left tail. There is no `ta.skew()`
builtin, so it is computed from the windowed moments.

## Recipe

```pine
//@version=6
indicator("Skew", "SKEW")

length = input.int(30, "Length", minval = 3)
src    = input.source(close, "Source")

skew(source, len) =>
    mean = ta.sma(source, len)
    float m2 = 0.0
    float m3 = 0.0
    for i = 0 to len - 1
        d   = source[i] - mean
        m2 += d * d
        m3 += d * d * d
    m2 /= len
    m3 /= len
    nf = float(len)
    nf * math.sqrt(nf - 1) / (nf - 2) * m3 / math.pow(m2, 1.5)

plot(skew(src, length), "Skew")
```

## How to read it

Positive skew warns that big upside surprises dominate the window (and the
downside is the "expected" side); negative skew the reverse. Traders use a shift
in skew as an early hint that the character of moves is changing.
