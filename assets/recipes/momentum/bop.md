---
title: Balance of Power
aliases: balance of power, bop
---

# Balance of Power

Balance of Power gauges the strength of buyers against sellers within each bar by
comparing the close-open move to the high-low range. It runs between -1 and +1.
There is no `ta.bop()` builtin.

## Recipe

```pine
//@version=6
indicator("Balance of Power", "BOP")

hlRange = high - low
bop = hlRange != 0 ? (close - open) / hlRange : 0.0

plot(bop, "BOP", style = plot.style_histogram)
```

## How to read it

Positive values mean buyers closed the bar near its high (control); negative
values mean sellers pushed it toward the low. It is noisy bar-to-bar, so it is
usually smoothed (e.g. an SMA of BOP) and read for its trend and zero crossings.
