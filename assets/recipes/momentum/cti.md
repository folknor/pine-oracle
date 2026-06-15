---
title: Correlation Trend Indicator
aliases: correlation trend indicator, cti, ehlers cti
---

# Correlation Trend Indicator

John Ehlers' Correlation Trend Indicator scores how closely price has followed a
straight sloping line over the window: it is the Pearson correlation between price
and time, running from -1 (perfect downtrend) to +1 (perfect uptrend). There is
no `ta.cti()` builtin - it is `ta.correlation` of price against `bar_index`.

## Recipe

```pine
//@version=6
indicator("Correlation Trend Indicator", "CTI")

length = input.int(12, "Length", minval = 1)
src    = input.source(close, "Source")

cti(source, len) =>
    ta.correlation(source, bar_index, len)

plot(cti(src, length), "CTI")
```

## How to read it

Values near +1 mean a clean, straight uptrend; near -1 a clean downtrend; near 0
means price is wandering with no linear trend. Crossings of zero mark trend
births and deaths, and the magnitude measures trend quality.
