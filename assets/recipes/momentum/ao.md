---
title: Awesome Oscillator
aliases: awesome oscillator, ao
---

# Awesome Oscillator

Bill Williams' Awesome Oscillator measures momentum as the gap between a fast and
a slow simple moving average of the median price (`hl2`). It is plotted as a
histogram around zero. There is no `ta.ao()` builtin.

## Recipe

```pine
//@version=6
indicator("Awesome Oscillator", "AO")

fast = input.int(5,  "Fast", minval = 1)
slow = input.int(34, "Slow", minval = 1)

ao(f, s) =>
    ta.sma(hl2, f) - ta.sma(hl2, s)

value = ao(fast, slow)
plot(value, "AO", value >= value[1] ? color.green : color.red, style = plot.style_histogram)
```

## How to read it

Crossing zero signals a shift between bullish and bearish momentum; the bar
colour (rising vs falling) is the faster cue traders use for the "saucer" and
"twin peaks" setups. It confirms trend strength rather than absolute direction.
