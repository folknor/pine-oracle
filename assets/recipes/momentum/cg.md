---
title: Center of Gravity
aliases: center of gravity, ehlers center of gravity, cg
---

# Center of Gravity

John Ehlers' Center of Gravity oscillator treats the price window as a physical
mass and finds its weighted centroid, aiming to identify turning points with
minimal lag. There is no `ta.cg()` builtin.

## Recipe

```pine
//@version=6
indicator("Center of Gravity", "CG")

length = input.int(10, "Length", minval = 1)
src    = input.source(close, "Source")

cg(source, len) =>
    float num = 0.0
    float den = 0.0
    for i = 0 to len - 1
        num += (i + 1) * source[i]
        den += source[i]
    -num / den

plot(cg(src, length), "CG")
```

## How to read it

The CG line oscillates around a midpoint set by the window length; its turns tend
to lead price turns, and crossings of the line by its own one-bar lag are used as
entry/exit triggers. It is smooth but, like all centroid measures, can whip in
flat markets.
