---
title: TOS Standard Deviation All
aliases: tos_stdevall, tos stdev all, thinkorswim stdevall, stdevall
---

# TOS Standard Deviation All

A port of ThinkOrSwim's StandardDeviationAll: a linear-regression line fitted
over the lookback, flanked by bands set at multiples of the standard deviation.
It marks where price sits relative to its own regression channel. There is no
`ta.tos_stdevall()` builtin (it composes `ta.linreg` and `ta.stdev`).

## Recipe

```pine
//@version=6
indicator("TOS StdevAll", "STDEVALL", overlay = true)

length = input.int(30,  "Length", minval = 3)
mult   = input.float(2.0, "Std Devs", minval = 0.1, step = 0.5)
src    = input.source(close, "Source")

lr = ta.linreg(src, length, 0)
sd = ta.stdev(src, length)

plot(lr,             "LR",    color.blue)
plot(lr + mult * sd, "Upper", color.green)
plot(lr - mult * sd, "Lower", color.red)
```

## How to read it

Price riding the upper band signals a strong move that has stretched above its
regression trend; the lower band is the mirror case, and the center line is the
fair-value path. The pandas-ta source bands the standard deviation of price over
the window (not of the regression residuals, as the original TOS does), so this
recipe follows the port and uses `ta.stdev` of the source directly.
