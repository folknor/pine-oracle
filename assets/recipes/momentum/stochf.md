---
title: Fast Stochastic
aliases: stochf, fast stochastic, fast stochastic oscillator
---

# Fast Stochastic

The Fast Stochastic is the raw form of the Stochastic oscillator: %K is the
close's position within the high-low range of the last N bars, and %D is a short
moving average of %K. Skipping the extra smoothing of the "slow" variant makes it
quicker but noisier. There is no `ta.stochf()` builtin (it composes `ta.stoch`
for %K and a short `ta.sma` for %D).

## Recipe

```pine
//@version=6
indicator("Fast Stochastic", "STOCHF")

fastK = input.int(5, "Fast %K", minval = 1)
fastD = input.int(3, "Fast %D", minval = 1)

stochf(kLen, dLen) =>
    k = ta.stoch(close, high, low, kLen)
    [k, ta.sma(k, dLen)]

[kLine, dLine] = stochf(fastK, fastD)
plot(kLine, "%K", color.blue)
plot(dLine, "%D", color.orange)
hline(80, "Overbought", color.gray)
hline(20, "Oversold",   color.gray)
```

## How to read it

Values above 80 are overbought and below 20 oversold; %K crossing %D gives the
trade trigger, earlier here than on the slow Stochastic at the cost of more false
signals. As with any oscillator its strongest use is divergence against price.
