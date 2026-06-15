---
title: Elder Thermometer
aliases: elder thermometer, thermo, market thermometer
---

# Elder Thermometer

Alexander Elder's Thermometer measures volatility as the larger of the change in
high or the change in low from the prior bar, smoothed by an EMA. It "takes the
market's temperature" - quiet bars run cool, expansion runs hot. There is no
`ta.thermo()` builtin.

## Recipe

```pine
//@version=6
indicator("Elder Thermometer", "THERMO")

length = input.int(20, "Length", minval = 1)

thermo(len) =>
    tl = math.abs(low[1] - low)
    th = math.abs(high - high[1])
    t  = math.max(th, tl)
    [t, ta.ema(t, len)]

[therm, thermMa] = thermo(length)
plot(therm,   "Thermo",    color.gray, style = plot.style_histogram)
plot(thermMa, "Thermo MA", color.orange)
```

## How to read it

Elder's rule: only enter when the thermometer is cool (below its EMA), and expect
a trend move when a hot bar (well above the EMA) appears. Spikes mark volatility
expansion - often the start of a strong move or a climax.
