---
title: Back Easing
aliases: back easing, ease back, overshoot easing
---

# Back Easing

Back easing overshoots the target slightly and settles back - the value dips
below 0 before easing in, or past 1 before easing out. It gives an
anticipation/overshoot feel. Pine has no easing builtins.

## Recipe

```pine
//@version=6
indicator("Back Easing", "EaseBack")

t = (bar_index % 100) / 100.0   // demo ramp; in use, t is your progress in [0,1]

c1 = 1.70158
c2 = c1 * 1.525
c3 = c1 + 1.0

easeIn(x)  => c3 * math.pow(x, 3) - c1 * math.pow(x, 2)
easeOut(x) => 1.0 + c3 * math.pow(x - 1.0, 3) + c1 * math.pow(x - 1.0, 2)
easeInOut(x) =>
    x < 0.5 ? math.pow(2.0 * x, 2) * ((c2 + 1.0) * 2.0 * x - c2) / 2.0 : (math.pow(2.0 * x - 2.0, 2) * ((c2 + 1.0) * (x * 2.0 - 2.0) + c2) + 2.0) / 2.0

plot(easeIn(t),    "In",    color.red)
plot(easeOut(t),   "Out",   color.green)
plot(easeInOut(t), "InOut", color.blue)
```

## How to read it

Note the output leaves `[0,1]`: ease-in goes slightly negative first
(anticipation), ease-out passes 1 then returns (overshoot). The `c1` constant
sets the overshoot magnitude. Good for a playful "snap into place" on a drawn
object.
