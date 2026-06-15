---
title: Percent Return
aliases: percent return, pctret, percentage return
---

# Percent Return

The percent return is the simple fractional change in price over `length` bars.
It is the everyday "how much did it move" measure. There is no
`ta.percent_return()` builtin.

## Recipe

```pine
//@version=6
indicator("Percent Return", "PCTRET")

length = input.int(1, "Length", minval = 1)
src    = input.source(close, "Source")

plot(src / src[length] - 1, "Percent Return", style = plot.style_histogram)
```

## How to read it

A value of 0.05 means price is up 5% over the window. Simple returns are intuitive
and correct for single-period P&L, but unlike log returns they do not add across
periods - use log returns for multi-period statistical work.
