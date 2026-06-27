---
title: Archer Moving Averages Trends
aliases: amat, archer moving averages trends, archer ma trends
---

# Archer Moving Averages Trends

AMAT reads trend direction from a fast and a slow moving average, emitting two
binary flags. The long-run flag fires when the fast MA is rising while the slow
MA is either rising or turning up off a bottom; the short-run flag mirrors it for
a falling fast MA. "Rising" and "falling" are measured over a short lookback.
There is no `ta.amat()` builtin (it composes two `ta.ema` calls).

## Recipe

```pine
//@version=6
indicator("Archer MA Trends", "AMAT")

fast     = input.int(8,  "Fast", minval = 1)
slow     = input.int(21, "Slow", minval = 1)
lookback = input.int(2,  "Lookback", minval = 1)
src      = input.source(close, "Source")

amat(source, f, s, lb) =>
    fastMa = ta.ema(source, f)
    slowMa = ta.ema(source, s)
    fastUp = fastMa > fastMa[lb]
    fastDn = fastMa < fastMa[lb]
    slowUp = slowMa > slowMa[lb]
    slowDn = slowMa < slowMa[lb]
    longRun  = (fastUp and slowDn) or (fastUp and slowUp)
    shortRun = (fastDn and slowUp) or (fastDn and slowDn)
    [longRun ? 1 : 0, shortRun ? 1 : 0]

[lr, sr] = amat(src, fast, slow, lookback)
plot(lr, "Long Run",  color.green)
plot(sr, "Short Run", color.red)
```

## How to read it

A long-run reading of 1 marks a bullish regime (fast MA leading up, whether the
slow MA is confirming or just bottoming); a short-run reading of 1 marks the
bearish mirror. The two are mutually exclusive in a clean trend; both reading 0
means the averages are crossing or flat, i.e. no committed direction.
