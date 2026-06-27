---
title: Time Series Forecast
aliases: tsf, time series forecast, linear regression forecast
---

# Time Series Forecast

The Time Series Forecast fits a linear regression over the last `length` bars and
projects the line one bar into the future, so it reads as a least-squares moving
average that leans in the direction of the recent slope. It equals the linear
regression value extended by one bar. Pine has `ta.linreg`, which gives the
forecast directly at offset `-1`.

## Recipe

```pine
//@version=6
indicator("Time Series Forecast", "TSF", overlay = true)

length = input.int(14, "Length", minval = 1)
src    = input.source(close, "Source")

tsf = ta.linreg(src, length, -1)

plot(tsf, "TSF", color.orange, 2)
```

## How to read it

The TSF hugs price more tightly than an SMA and turns earlier because it carries
the regression slope forward. Price crossing the line, or the line's slope
changing sign, flags a shift in the short-term trend; the gap between price and
the forecast hints at how stretched the move is.
