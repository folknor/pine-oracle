---
title: Quintic Easing
aliases: quintic easing, quint easing, ease quint
---

# Quintic Easing

Quintic easing shapes a normalized progress `t` with a power-of-5 curve - the
steepest of the standard polynomial easings. Pine has no easing builtins.

## Recipe

```pine
//@version=6
indicator("Quintic Easing", "EaseQuint")

t = (bar_index % 100) / 100.0   // demo ramp; in use, t is your progress in [0,1]

easeIn(x)    => math.pow(x, 5)
easeOut(x)   => 1.0 - math.pow(1.0 - x, 5)
easeInOut(x) => x < 0.5 ? 16.0 * math.pow(x, 5) : 1.0 - math.pow(-2.0 * x + 2.0, 5) / 2.0

plot(easeIn(t),    "In",    color.red)
plot(easeOut(t),   "Out",   color.green)
plot(easeInOut(t), "InOut", color.blue)
```

## How to read it

The most pronounced polynomial curve: very flat at the start, very steep at the
finish. Use it when you want a dramatic, sudden completion. Beyond quintic,
exponential easing is even sharper.
