---
title: Circular Easing
aliases: circular easing, circle easing, circ easing, ease circ
---

# Circular Easing

Circular easing traces a quarter circle, giving a curve that hugs the axis then
swings sharply - distinct from the polynomial shapes. Pine has no easing
builtins.

## Recipe

```pine
//@version=6
indicator("Circular Easing", "EaseCirc")

t = (bar_index % 100) / 100.0   // demo ramp; in use, t is your progress in [0,1]

easeIn(x)  => 1.0 - math.sqrt(1.0 - x * x)
easeOut(x) => math.sqrt(1.0 - math.pow(x - 1.0, 2))
easeInOut(x) =>
    x < 0.5 ? (1.0 - math.sqrt(1.0 - math.pow(2.0 * x, 2))) / 2.0 : (math.sqrt(1.0 - math.pow(-2.0 * x + 2.0, 2)) + 1.0) / 2.0

plot(easeIn(t),    "In",    color.red)
plot(easeOut(t),   "Out",   color.green)
plot(easeInOut(t), "InOut", color.blue)
```

## How to read it

The circular curve stays flat longer than a quadratic then turns hard near the
end (ease-in) - a snappier feel than the polynomials without the overshoot of
back/elastic. Keep `t` in `[0,1]` so the `sqrt` argument stays non-negative.
