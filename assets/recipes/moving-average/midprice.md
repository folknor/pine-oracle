---
title: Midprice
aliases: midprice, midpoint price
---

# Midprice

Midprice is the midpoint between the highest high and lowest low over the window -
the centre of the recent price range. It is a simple equilibrium/centre-line
overlay. There is no `ta.midprice()` builtin.

## Recipe

```pine
//@version=6
indicator("Midprice", "MIDPRICE", overlay = true)

length = input.int(2, "Length", minval = 1)

midprice(len) =>
    (ta.lowest(low, len) + ta.highest(high, len)) / 2

plot(midprice(length), "Midprice")
```

## How to read it

It marks the centre of the range, so price above it is in the upper half (relative
strength) and below it the lower half. Longer lengths make it a smooth
mean-reversion anchor; it is the midline of a Donchian channel.
