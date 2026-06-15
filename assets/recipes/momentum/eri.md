---
title: Elder Ray Index
aliases: elder ray index, elder ray, bull power, bear power
---

# Elder Ray Index

Alexander Elder's Elder Ray splits momentum into Bull Power (how far the high
extends above an EMA) and Bear Power (how far the low extends below it),
exposing who controls each bar relative to consensus value. There is no
`ta.eri()` builtin.

## Recipe

```pine
//@version=6
indicator("Elder Ray Index", "ERI")

length = input.int(13, "Length", minval = 1)

eri(len) =>
    e = ta.ema(close, len)
    [high - e, low - e]

[bull, bear] = eri(length)
plot(bull, "Bull Power", color.green, style = plot.style_histogram)
plot(bear, "Bear Power", color.red,   style = plot.style_histogram)
```

## How to read it

Elder reads it alongside a trend filter: in an uptrend, buy when Bear Power is
negative but rising (a dip into value); in a downtrend, sell when Bull Power is
positive but falling. Bull Power below zero or Bear Power above zero is unusual
and signals a strong one-sided bar.
