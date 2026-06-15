---
title: Price Distance
aliases: price distance, pdist
---

# Price Distance

Price Distance estimates how much ground price actually covered in a bar,
combining the high-low range with the body and the gap from the prior close. It is
a single-bar volatility/movement measure. There is no `ta.pdist()` builtin.

## Recipe

```pine
//@version=6
indicator("Price Distance", "PDIST")

pdist = 2 * (high - low) + math.abs(open - close[1]) - math.abs(close - open)
plot(pdist, "PDIST")
```

## How to read it

Larger values mean a bar travelled more (wide range, big gap, or both) - higher
realized movement; small values mean a quiet bar. It is typically smoothed and
used as a volatility input rather than read bar by bar.
