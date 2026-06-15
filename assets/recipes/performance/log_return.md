---
title: Log Return
aliases: log return, logarithmic return, logret
---

# Log Return

The log return is the natural log of the price ratio over `length` bars. Log
returns are time-additive (they sum across periods) and roughly symmetric, which
makes them the preferred return measure for statistics. There is no
`ta.log_return()` builtin.

## Recipe

```pine
//@version=6
indicator("Log Return", "LOGRET")

length = input.int(1, "Length", minval = 1)
src    = input.source(close, "Source")

plot(math.log(src / src[length]), "Log Return", style = plot.style_histogram)
```

## How to read it

Positive means price rose over the window, negative means it fell; the magnitude
is the continuously-compounded rate. Because log returns add up over time, they
feed cleanly into volatility, Sharpe, and other statistical measures where
percent returns would compound awkwardly.
