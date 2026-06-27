---
title: Efficiency Ratio
aliases: er, efficiency ratio, kaufman efficiency ratio, ker
---

# Efficiency Ratio

Perry Kaufman's Efficiency Ratio divides the net price change over N bars by the
sum of the absolute bar-to-bar changes over the same window. It measures how much
of the total travel was directional, scaling from 0 (pure noise) to 1 (a clean
trend). There is no `ta.er()` builtin (it composes `ta.change` and `math.sum`).

## Recipe

```pine
//@version=6
indicator("Efficiency Ratio", "ER")

length = input.int(10, "Length", minval = 1)
src    = input.source(close, "Source")

er(source, len) =>
    direction = math.abs(ta.change(source, len))
    volatility = math.sum(math.abs(ta.change(source)), len)
    direction / volatility

plot(er(src, length), "ER", color.blue)
```

## How to read it

Readings near 1 mean the move was efficient (most of the path went one way),
favouring trend-following; readings near 0 mean the price churned and went
nowhere, favouring mean-reversion or staying flat. It is the speed control behind
Kaufman's adaptive moving average, where a high ratio speeds the average up.
