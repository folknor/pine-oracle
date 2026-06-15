---
title: Value at Risk
aliases: value at risk, var, parametric var, cvar, expected shortfall, conditional value at risk
---

# Value at Risk

Parametric (Gaussian) Value at Risk estimates the loss a position would not exceed
over one bar at a chosen confidence level, from the mean and standard deviation of
recent log returns. Conditional VaR (Expected Shortfall) is the *average* loss in
the tail beyond VaR. There is no `ta.var()`/risk builtin.

## Recipe

```pine
//@version=6
indicator("Value at Risk", "VaR")

length = input.int(100, "Lookback", minval = 2)
conf   = input.string("95%", "Confidence", options = ["90%", "95%", "99%"])

// Confidence -> normal z-score and tail probability (avoids needing erf inline)
z    = conf == "99%" ? 2.3263 : conf == "95%" ? 1.6449 : 1.2816
tail = conf == "99%" ? 0.01   : conf == "95%" ? 0.05   : 0.10

r     = math.log(close / close[1])     // log returns
mu    = ta.sma(r, length)
sigma = ta.stdev(r, length)

phi      = math.exp(-z * z / 2.0) / math.sqrt(2.0 * math.pi)  // normal pdf at z
varFrac  = z * sigma - mu                                     // VaR loss fraction
cvarFrac = sigma * phi / tail - mu                            // expected shortfall

plot(100 * varFrac,  "VaR %",  color.orange)
plot(100 * cvarFrac, "CVaR %", color.red)
```

## How to read it

VaR % is the worst single-bar loss expected at the chosen confidence (e.g. 95% VaR
of 2% means: on 95% of bars the loss should be under 2%). CVaR is larger - the
mean loss on the bad ~5% of bars - and is the more conservative risk figure. Both
assume normally-distributed returns; fat-tailed markets will exceed them more often
than the model implies. Harvested as a self-contained leaf from RicardoSantos-style
distribution libraries; a full erf-based version would let confidence be any value
rather than presets.
