---
title: Projection Oscillator
aliases: projection oscillator, po
---

# Projection Oscillator

The Projection Oscillator measures the percentage deviation of price from its
linear-regression trend line, flagging when price has stretched away from where
the regression sits. There is no `ta.po()` builtin.

## Recipe

```pine
//@version=6
indicator("Projection Oscillator", "PO")

length = input.int(14, "Length", minval = 1)
src    = input.source(close, "Source")

po(source, len) =>
    lr = ta.linreg(source, len, 0)
    lr != 0 ? 100 * (source - lr) / lr : 0.0

plot(po(src, length), "PO")
```

## How to read it

Positive values mean price is above its regression line, negative below;
extremes flag overbought/oversold relative to the trend. It differs from the
Chande Forecast Oscillator only in the denominator (the regression value rather
than price).
