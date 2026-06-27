---
title: Historical Volatility
aliases: historical volatility, hvol, hv, annualized volatility
---

# Historical Volatility

Historical Volatility is the annualized standard deviation of logarithmic
returns over a window, expressed as a percentage. It answers "how much has this
instrument actually moved lately?" in a unit you can compare across symbols and
timeframes. The annualization factor is an input (252 trading days, 52 weeks, 12
months). There is no `ta.hvol()` builtin (it composes `ta.stdev` of log returns).

## Recipe

```pine
//@version=6
indicator("Historical Volatility", "HVOL")

length = input.int(20, "Length", minval = 1)
annual = input.float(252, "Annualization", minval = 1)
src    = input.source(close, "Source")

hvol(source, len, ann) =>
    logRet = math.log(source / source[1])
    100 * ta.stdev(logRet, len, false) * math.sqrt(ann)

plot(hvol(src, length, annual), "HVOL")
```

## How to read it

A reading of 30 means roughly a 30% annualized standard deviation of returns. Use
it to gauge regime: low values flag complacent, range-bound markets ripe for an
expansion; high values flag stress and wide swings. `ta.stdev` is called with
`biased = false` to match the sample (N-1) standard deviation of the source.
