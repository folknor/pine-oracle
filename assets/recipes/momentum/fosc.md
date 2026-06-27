---
title: Forecast Oscillator
aliases: fosc, forecast oscillator
---

# Forecast Oscillator

Tushar Chande's Forecast Oscillator measures the percentage gap between the
actual close and the Time Series Forecast - the linear regression line projected
one bar ahead. Positive values mean price is running above its own forecast,
negative values below it. There is no `ta.fosc()` builtin (it composes
`ta.linreg`, whose one-bar-ahead projection is the TSF).

## Recipe

```pine
//@version=6
indicator("Forecast Oscillator", "FOSC")

length = input.int(14, "Length", minval = 1)
src    = input.source(close, "Source")

fosc(source, len) =>
    forecast = ta.linreg(source, len, -1)
    100.0 * (source - forecast) / source

plot(fosc(src, length), "FOSC", color.blue)
hline(0, "Zero", color.gray)
```

## How to read it

The oscillator swings around zero: above zero the close leads its forecast
(bullish stretch), below zero it lags (bearish stretch). Crossings of the zero
line flag a shift in which side of the regression the market is trading, and
large excursions warn the price has run far from its fitted trend. `ta.linreg`
with offset `-1` projects the fitted line one bar into the future, which is the
TSF the original computes.
