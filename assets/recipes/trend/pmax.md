---
title: Profit Maximizer
aliases: pmax, profit maximizer, price max
---

# Profit Maximizer

PMAX is a SuperTrend-style trailing stop that bands a moving average with ATR
instead of banding the midprice: a lower band at MA - mult * ATR and an upper
band at MA + mult * ATR. Price closing through a band flips the trend and the
plotted line follows the active band. There is no `ta.pmax()` builtin; it needs
`var` state and `[1]` feedback because each bar's band carries forward from the
last.

## Recipe

```pine
//@version=6
indicator("Profit Maximizer", "PMAX", overlay = true)

length = input.int(10,  "Length", minval = 1)
mult   = input.float(3.0, "Multiplier", minval = 0.1, step = 0.5)
src    = input.source(close, "Source")

pmax(source, len, m) =>
    atrv = ta.atr(len)
    base = ta.ema(source, len)
    up   = base - m * atrv
    dn   = base + m * atrv
    var float upBand = na
    var float dnBand = na
    var int   dir    = 1
    upBand := na(upBand[1]) ? up : (source[1] > upBand[1] ? math.max(up, upBand[1]) : up)
    dnBand := na(dnBand[1]) ? dn : (source[1] < dnBand[1] ? math.min(dn, dnBand[1]) : dn)
    dir    := source > dnBand[1] ? 1 : source < upBand[1] ? -1 : nz(dir[1], 1)
    dir == 1 ? upBand : dnBand

plot(pmax(src, length, mult), "PMAX", color.purple, linewidth = 2)
```

## How to read it

When the line sits below price the trend is up and the line is a trailing stop
to ride; when it flips above price the trend is down. A close crossing the line
marks the regime change. Raising the multiplier widens the bands, giving fewer
but later flips; lowering it tightens the stop and whipsaws more.
