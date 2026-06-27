---
title: Beta
aliases: beta, market beta
---

# Beta

Beta measures how sensitively an instrument's returns track a benchmark's
returns: a beta of 1 moves with the benchmark, above 1 is more volatile, below 1
less. It is the covariance of the two return streams divided by the variance of
the benchmark returns, over a rolling window. There is no `ta.beta()` builtin
(pull the benchmark with `request.security` and compose `ta.sma`).

## Recipe

```pine
//@version=6
indicator("Beta", "BETA")

length = input.int(30, "Length", minval = 2)
sym    = input.symbol("SPY", "Benchmark")
src    = input.source(close, "Source")

bench = request.security(sym, timeframe.period, close)

beta(source, market, len) =>
    r    = source / source[1] - 1
    rb   = market / market[1] - 1
    cov  = ta.sma(r * rb, len) - ta.sma(r, len) * ta.sma(rb, len)
    varb = ta.sma(rb * rb, len) - math.pow(ta.sma(rb, len), 2)
    cov / varb

plot(beta(src, bench, length), "Beta")
hline(1, "Market", color.gray)
```

## How to read it

Beta near 1 means the instrument swings in step with the benchmark; above 1
amplifies the benchmark's moves (higher systematic risk) and below 1 dampens
them. A negative beta moves opposite the benchmark. The sample-vs-population
divisor cancels in the covariance/variance ratio, so the population form used
here matches the standard financial beta.
