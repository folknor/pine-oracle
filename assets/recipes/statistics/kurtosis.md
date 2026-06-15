---
title: Kurtosis
aliases: kurtosis, rolling kurtosis, kurt
---

# Kurtosis

Rolling excess kurtosis (Fisher) measures the "tailedness" of the return
distribution over the window: positive means fatter tails and a sharper peak than
a normal distribution (more extreme moves), negative means thinner tails. There
is no `ta.kurtosis()` builtin, so it is computed from the windowed moments.

## Recipe

```pine
//@version=6
indicator("Kurtosis", "KURT")

length = input.int(30, "Length", minval = 4)
src    = input.source(close, "Source")

kurtosis(source, len) =>
    mean = ta.sma(source, len)
    float m2 = 0.0
    float m4 = 0.0
    for i = 0 to len - 1
        d   = source[i] - mean
        d2  = d * d
        m2 += d2
        m4 += d2 * d2
    m2 /= len
    m4 /= len
    nf = float(len)
    numer = nf * (nf + 1) * (nf - 1) * m4
    denom = (nf - 2) * (nf - 3) * m2 * m2
    adj   = 3 * (nf - 1) * (nf - 1) / ((nf - 2) * (nf - 3))
    numer / denom - adj

plot(kurtosis(src, length), "Kurtosis")
```

## How to read it

High positive kurtosis warns of fat tails - rare but large moves are more likely
than a normal model assumes, so expect occasional shocks. Near zero means roughly
normal tails. Rising kurtosis often precedes volatile, gap-prone conditions.
