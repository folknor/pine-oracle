---
title: Quartic Easing
aliases: quartic easing, quart easing, ease quart
---

# Quartic Easing

Quartic easing shapes a normalized progress `t` with a power-of-4 curve - sharper
than cubic. Pine has no easing builtins.

## Recipe

```pine
//@version=6
indicator("Quartic Easing", "EaseQuart")

t = (bar_index % 100) / 100.0   // demo ramp; in use, t is your progress in [0,1]

easeIn(x)    => math.pow(x, 4)
easeOut(x)   => 1.0 - math.pow(1.0 - x, 4)
easeInOut(x) => x < 0.5 ? 8.0 * math.pow(x, 4) : 1.0 - math.pow(-2.0 * x + 2.0, 4) / 2.0

plot(easeIn(t),    "In",    color.red)
plot(easeOut(t),   "Out",   color.green)
plot(easeInOut(t), "InOut", color.blue)
```

## How to read it

The strong power-of-4 curve keeps the value near its start for most of the range,
then snaps to the end (ease-in) - useful when you want motion that lingers then
finishes quickly. Quintic is the next step up in steepness.
