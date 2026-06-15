---
title: Elastic Easing
aliases: elastic easing, ease elastic, spring easing
---

# Elastic Easing

Elastic easing oscillates past the target like a released spring before settling,
combining a decaying exponential with a sine wave. Pine has no easing builtins.

## Recipe

```pine
//@version=6
indicator("Elastic Easing", "EaseElastic")

t = (bar_index % 100) / 100.0   // demo ramp; in use, t is your progress in [0,1]

c4 = 2.0 * math.pi / 3.0
c5 = 2.0 * math.pi / 4.5

easeIn(x)  => x == 0.0 ? 0.0 : x == 1.0 ? 1.0 : -math.pow(2.0, 10.0 * x - 10.0) * math.sin((x * 10.0 - 10.75) * c4)
easeOut(x) => x == 0.0 ? 0.0 : x == 1.0 ? 1.0 : math.pow(2.0, -10.0 * x) * math.sin((x * 10.0 - 0.75) * c4) + 1.0
easeInOut(x) =>
    switch
        x == 0.0 => 0.0
        x == 1.0 => 1.0
        x < 0.5  => -(math.pow(2.0, 20.0 * x - 10.0) * math.sin((20.0 * x - 11.125) * c5)) / 2.0
        =>          math.pow(2.0, -20.0 * x + 10.0) * math.sin((20.0 * x - 11.125) * c5) / 2.0 + 1.0

plot(easeIn(t),    "In",    color.red)
plot(easeOut(t),   "Out",   color.green)
plot(easeInOut(t), "InOut", color.blue)
```

## How to read it

The output overshoots and wobbles around the target before settling - a
spring/rubber-band feel. Use sparingly for emphasis (e.g. a label that "boings"
into place). The endpoints are pinned explicitly since the oscillation never
naturally lands exactly on 0 or 1.
