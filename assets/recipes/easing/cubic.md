---
title: Cubic Easing
aliases: cubic easing, ease cubic
---

# Cubic Easing

Cubic easing shapes a normalized progress `t` with a power-of-3 curve - steeper
acceleration than quadratic. Pine has no easing builtins.

## Recipe

```pine
//@version=6
indicator("Cubic Easing", "EaseCubic")

t = (bar_index % 100) / 100.0   // demo ramp; in use, t is your progress in [0,1]

easeIn(x)    => x * x * x
easeOut(x)   => 1.0 - math.pow(1.0 - x, 3)
easeInOut(x) => x < 0.5 ? 4.0 * x * x * x : 1.0 - math.pow(-2.0 * x + 2.0, 3) / 2.0

plot(easeIn(t),    "In",    color.red)
plot(easeOut(t),   "Out",   color.green)
plot(easeInOut(t), "InOut", color.blue)
```

## How to read it

Same three shapes as the other polynomials, with a stronger curve than quadratic
and milder than quartic. A common default for "natural" motion of drawn objects.
