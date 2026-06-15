---
title: Sine Easing
aliases: sine easing, ease sine, ease in out sine
---

# Sine Easing

Sine easing smooths a normalized progress value `t` (in `[0,1]`) along a quarter
sine wave - the gentlest of the easing families. Easing functions shape *how* a
value moves over its range; they're used to animate drawn objects (lines, labels,
boxes) smoothly rather than linearly. Pine has no easing builtins.

## Recipe

```pine
//@version=6
indicator("Sine Easing", "EaseSine")

t = (bar_index % 100) / 100.0   // demo ramp; in use, t is your progress in [0,1]

easeIn(x)    => 1.0 - math.cos(x * math.pi / 2.0)
easeOut(x)   => math.sin(x * math.pi / 2.0)
easeInOut(x) => -(math.cos(math.pi * x) - 1.0) / 2.0

plot(easeIn(t),    "In",    color.red)
plot(easeOut(t),   "Out",   color.green)
plot(easeInOut(t), "InOut", color.blue)
```

## How to read it

`easeIn` starts slow and accelerates; `easeOut` starts fast and decelerates;
`easeInOut` does both (slow-fast-slow). Feed a `[0,1]` progress value (e.g. bars
elapsed / total) and use the eased output to drive a coordinate, width, or
transparency. Clamp `t` to `[0,1]` if your input can stray outside.
