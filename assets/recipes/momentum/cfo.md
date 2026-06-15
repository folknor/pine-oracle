---
title: Chande Forecast Oscillator
aliases: chande forecast oscillator, forecast oscillator, cfo
---

# Chande Forecast Oscillator

The CFO measures the percentage difference between price and its time-series
forecast (the endpoint of a linear regression line). It shows how far price has
diverged from where the regression "expects" it. There is no `ta.cfo()` builtin.

## Recipe

```pine
//@version=6
indicator("Chande Forecast Oscillator", "CFO")

length = input.int(9, "Length", minval = 1)
src    = input.source(close, "Source")

cfo(source, len) =>
    tsf = ta.linreg(source, len, 0)
    100 * (source - tsf) / source

plot(cfo(src, length), "CFO")
```

## How to read it

A positive CFO means price is trading above its regression forecast; negative
means below. Persistent one-sided readings indicate a strong trend, while
oscillation around zero indicates price tracking its regression closely. The
`ta.linreg` endpoint stands in for the time-series forecast.
