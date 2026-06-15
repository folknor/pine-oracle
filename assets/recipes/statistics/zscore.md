---
title: Z-Score
aliases: z score, zscore, z-score
---

# Z-Score

The rolling Z-Score expresses how many standard deviations the current price is
from its moving average, standardising price into a mean-reverting oscillator.
There is no `ta.zscore()` builtin (it composes `ta.sma` and `ta.stdev`).

## Recipe

```pine
//@version=6
indicator("Z Score", "ZSCORE")

length = input.int(30, "Length", minval = 2)
src    = input.source(close, "Source")

zscore(source, len) =>
    (source - ta.sma(source, len)) / ta.stdev(source, len)

plot(zscore(src, length), "ZScore")
```

## How to read it

A Z-Score near 0 means price is at its average; +2 / -2 mark statistically
stretched moves (about the 95th percentile under a normal assumption) often used
as mean-reversion entry/exit thresholds. Sustained high readings instead indicate
a strong trend.
