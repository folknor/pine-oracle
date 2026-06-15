---
title: Quadratic Easing
aliases: quadratic easing, quad easing, ease quad
---

# Quadratic Easing

Quadratic easing shapes a normalized progress `t` with a power-of-2 curve. It is
the mildest of the polynomial easings (quad < cubic < quart < quint in
steepness). Pine has no easing builtins.

## Recipe

```pine
//@version=6
indicator("Quadratic Easing", "EaseQuad")

t = (bar_index % 100) / 100.0   // demo ramp; in use, t is your progress in [0,1]

easeIn(x)    => x * x
easeOut(x)   => 1.0 - (1.0 - x) * (1.0 - x)
easeInOut(x) => x < 0.5 ? 2.0 * x * x : 1.0 - math.pow(-2.0 * x + 2.0, 2) / 2.0

plot(easeIn(t),    "In",    color.red)
plot(easeOut(t),   "Out",   color.green)
plot(easeInOut(t), "InOut", color.blue)
```

## How to read it

`easeIn` accelerates from rest, `easeOut` decelerates to rest, `easeInOut`
combines them. Swap `x*x` for higher powers to get cubic/quartic/quintic - the
higher the power, the more dramatic the acceleration.
