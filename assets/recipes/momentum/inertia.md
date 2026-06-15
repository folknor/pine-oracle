---
title: Inertia
aliases: inertia
---

# Inertia

Donald Dorsey's Inertia is the Relative Volatility Index smoothed by a linear
regression. It treats trend as "inertia" - a body in motion staying in motion -
reading positive inertia above 50 and negative below. There is no `ta.inertia()`
builtin (it composes RVI and `ta.linreg`).

## Recipe

```pine
//@version=6
indicator("Inertia", "INERTIA")

length = input.int(20, "Length",     minval = 1)
rviLen = input.int(14, "RVI Length", minval = 1)
src    = input.source(close, "Source")

inertia(source, len, rl) =>
    std    = ta.stdev(source, rl)
    up     = source > source[1] ? std : 0.0
    dn     = source < source[1] ? std : 0.0
    upAvg  = ta.ema(up, rl)
    dnAvg  = ta.ema(dn, rl)
    denom  = upAvg + dnAvg
    rviVal = denom != 0 ? 100 * upAvg / denom : 0.0
    ta.linreg(rviVal, len, 0)

plot(inertia(src, length, rviLen), "Inertia")
```

## How to read it

Above 50 signals positive inertia (an uptrend likely to persist); below 50,
negative inertia (downtrend). The linear-regression smoothing makes it slower and
steadier than the underlying RVI, so it is read as a trend-persistence gauge.
