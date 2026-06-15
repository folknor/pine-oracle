---
title: Drawdown
aliases: drawdown, dd
---

# Drawdown

Drawdown measures the decline from the running peak: how far price has fallen
below its highest value so far, as a percentage. It is the core risk metric for
peak-to-trough pain. There is no `ta.drawdown()` builtin (`ta.max` supplies the
running peak).

## Recipe

```pine
//@version=6
indicator("Drawdown", "DD")

maxClose = ta.max(close)          // running all-time high
ddPct    = 1 - close / maxClose   // fraction below the peak

plot(ddPct, "Drawdown %", color.red, style = plot.style_area)
```

## How to read it

Zero means price is at a new high; a value of 0.20 means price is 20% below its
peak. The maximum drawdown over a period is the worst loss an entry at the top
would have suffered - a key gauge of strategy and instrument risk.
