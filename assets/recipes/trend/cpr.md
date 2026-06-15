---
title: Central Pivot Range
aliases: central pivot range, cpr
---

# Central Pivot Range

The Central Pivot Range plots three levels from the prior period's high/low/close:
the pivot, plus a top (TC) and bottom (BC) central line that bracket it. The width
between TC and BC gauges expected trend vs range for the session. This recipe uses
classic daily pivots. There is no `ta.cpr()` builtin.

## Recipe

```pine
//@version=6
indicator("Central Pivot Range", "CPR", overlay = true)

[ph, pl, pc] = request.security(syminfo.tickerid, "1D", [high[1], low[1], close[1]], lookahead = barmerge.lookahead_on)
pivot = (ph + pl + pc) / 3
bc    = (ph + pl) / 2
tc    = pivot - bc + pivot   // mirror of BC across the pivot

plot(tc,    "TC",    color.blue)
plot(pivot, "Pivot", color.orange)
plot(bc,    "BC",    color.blue)
```

## How to read it

A narrow CPR (TC and BC close together) signals an expected trending day; a wide
CPR signals range-bound conditions. Price above the whole range is bullish, below
is bearish, and inside is indecision. Using the *prior* day's data with
`lookahead_on` is the standard, repaint-free way to draw today's pivots.
