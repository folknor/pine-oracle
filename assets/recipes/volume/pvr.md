---
title: Price Volume Rank
aliases: price volume rank, pvr, pv rank
---

# Price Volume Rank

Anthony Macek's Price Volume Rank is a discrete 1-4 score from whether price and
volume each rose or fell on the bar: 1 = price up on rising volume, 2 = price up
on falling volume, 3 = price down on rising volume, 4 = price down on falling
volume. Designed to be computable by hand, it is a coarse classifier of the
price-volume regime. There is no `ta.pvr()` builtin.

## Recipe

```pine
//@version=6
indicator("Price Volume Rank", "PVR")

pvr() =>
    dC = nz(ta.change(close))
    dV = nz(ta.change(volume))
    dC >= 0 and dV >= 0 ? 1 : dC >= 0 and dV < 0 ? 2 : dC < 0 and dV >= 0 ? 3 : 4

plot(pvr(), "PVR", style = plot.style_stepline)
```

## How to read it

The classic rule is to buy when the rank sits below 2.5 (the price-up regimes)
and sell when it is above 2.5 (the price-down regimes). Smoothing the raw rank
with a moving average turns the four steps into a tradeable line.
