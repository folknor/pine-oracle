---
title: Ehlers Super Smoother Filter
aliases: super smoother filter, ehlers super smoother, ehlers super smoother filter
---

# Ehlers Super Smoother Filter

John Ehlers' Super Smoother is a two-pole recursive filter borrowed from analog
signal design. It removes high-frequency noise (aliasing) far more cleanly than
a moving average of comparable lag, by weighting the current price against the
two prior filter outputs with coefficients derived from the length. There is no
`ta.ssf()` builtin.

## Recipe

```pine
//@version=6
indicator("Ehlers Super Smoother Filter", "SSF", overlay = true)

length = input.int(10, "Length", minval = 1)
src    = input.source(close, "Source")

ssf(source, len) =>
    x  = math.pi * math.sqrt(2) / len
    a0 = math.exp(-x)
    a1 = -a0 * a0
    b1 = 2 * a0 * math.cos(x)
    c1 = 1 - a1 - b1
    var float result = na
    result := c1 * source + b1 * nz(result[1], source) + a1 * nz(result[2], source)
    result

plot(ssf(src, length), "SSF", color.orange, 2)
```

## How to read it

The Super Smoother gives a clean, low-lag line that follows the dominant move
while ignoring bar-to-bar jitter, so it is well suited to smoothing a noisy
source before further calculation (e.g. as the input to an oscillator). Read it
by slope; a two-pole filter trades a touch of lag for much less whipsaw.
