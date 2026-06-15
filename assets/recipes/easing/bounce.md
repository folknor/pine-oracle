---
title: Bounce Easing
aliases: bounce easing, ease bounce
---

# Bounce Easing

Bounce easing makes the value bounce off the boundary like a dropped ball, with
successively smaller rebounds. The ease-in is the time-reverse of the ease-out.
Pine has no easing builtins.

## Recipe

```pine
//@version=6
indicator("Bounce Easing", "EaseBounce")

t = (bar_index % 100) / 100.0   // demo ramp; in use, t is your progress in [0,1]

n1 = 7.5625
d1 = 2.75

easeOut(x) =>
    if x < 1.0 / d1
        n1 * x * x
    else if x < 2.0 / d1
        xx = x - 1.5 / d1
        n1 * xx * xx + 0.75
    else if x < 2.5 / d1
        xx = x - 2.25 / d1
        n1 * xx * xx + 0.9375
    else
        xx = x - 2.625 / d1
        n1 * xx * xx + 0.984375

easeIn(x)    => 1.0 - easeOut(1.0 - x)
easeInOut(x) => x < 0.5 ? (1.0 - easeOut(1.0 - 2.0 * x)) / 2.0 : (1.0 + easeOut(2.0 * x - 1.0)) / 2.0

plot(easeIn(t),    "In",    color.red)
plot(easeOut(t),   "Out",   color.green)
plot(easeInOut(t), "InOut", color.blue)
```

## How to read it

`easeOut` is the canonical "ball settling" with three diminishing bounces;
`easeIn` mirrors it (bounces that grow into the move); `easeInOut` bounces at both
ends. The piecewise segments are the successive rebounds. Great for a drawn object
that drops and settles. (`easeIn` is defined in terms of `easeOut`, so keep both
functions in the script.)
