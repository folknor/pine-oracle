---
title: Exponential Easing
aliases: exponential easing, expo easing, ease expo
---

# Exponential Easing

Exponential easing uses a power-of-2 curve for an extremely sharp acceleration,
with explicit endpoints at 0 and 1 (the raw formula doesn't quite reach them).
Pine has no easing builtins.

## Recipe

```pine
//@version=6
indicator("Exponential Easing", "EaseExpo")

t = (bar_index % 100) / 100.0   // demo ramp; in use, t is your progress in [0,1]

easeIn(x)  => x == 0.0 ? 0.0 : math.pow(2.0, 10.0 * x - 10.0)
easeOut(x) => x == 1.0 ? 1.0 : 1.0 - math.pow(2.0, -10.0 * x)
easeInOut(x) =>
    switch
        x == 0.0 => 0.0
        x == 1.0 => 1.0
        x < 0.5  => math.pow(2.0, 20.0 * x - 10.0) / 2.0
        =>          (2.0 - math.pow(2.0, -20.0 * x + 10.0)) / 2.0

plot(easeIn(t),    "In",    color.red)
plot(easeOut(t),   "Out",   color.green)
plot(easeInOut(t), "InOut", color.blue)
```

## How to read it

The sharpest of the standard "monotone" easings: motion is almost imperceptible
until late, then rushes to the end (ease-in). The explicit `x == 0`/`x == 1`
checks pin the endpoints exactly, since `2^(10x-10)` never truly hits 0.
