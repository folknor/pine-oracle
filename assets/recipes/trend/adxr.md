---
title: ADXR
aliases: adxr, average directional movement index rating
---

# ADXR

ADXR smooths the ADX by averaging the current ADX with the ADX from `length - 1`
bars ago, giving a steadier read of trend strength. The DI+ / DI- lines come
along for direction. It builds on `ta.dmi` (there is no separate `ta.adxr`).

## Recipe

```pine
//@version=6
indicator("ADXR", "ADXR")

length = input.int(14, "Length", minval = 1)

adxr(len) =>
    [diPlus, diMinus, adxVal] = ta.dmi(len, len)
    adxrVal = (adxVal + adxVal[len - 1]) / 2
    [adxrVal, diPlus, diMinus]

[adxrVal, dip, dim] = adxr(length)
plot(adxrVal, "ADXR", color.blue)
plot(dip,     "DI+",  color.green)
plot(dim,     "DI-",  color.red)
```

## How to read it

ADXR above ~25 confirms a trend strong enough to follow; below ~20 signals a weak
or ranging market. It lags the ADX slightly but filters its whipsaws, so it is
often used as the trend-strength gate while DI+/DI- give direction.
