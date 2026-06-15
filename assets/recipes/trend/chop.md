---
title: Choppiness Index
aliases: choppiness index, chop
---

# Choppiness Index

E.W. Dreiss's Choppiness Index measures whether the market is trending or
chopping sideways, by comparing the summed true range to the overall high-low
range on a log scale. It runs roughly 0-100 and is direction-agnostic. There is
no `ta.chop()` builtin.

## Recipe

```pine
//@version=6
indicator("Choppiness Index", "CHOP")

length = input.int(14, "Length", minval = 1)

chop(len) =>
    diff   = ta.highest(high, len) - ta.lowest(low, len)
    atrSum = math.sum(ta.atr(1), len)
    100 * (math.log10(atrSum) - math.log10(diff)) / math.log10(len)

plot(chop(length), "CHOP")
```

## How to read it

High values (near 100, often above ~61.8) mean choppy, sideways action - the
summed range far exceeds the net range; low values (near 0, often below ~38.2)
mean a strong directional trend. It signals *whether* to trend-trade, never which
way.
