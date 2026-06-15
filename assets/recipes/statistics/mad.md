---
title: Mean Absolute Deviation
aliases: mean absolute deviation, mad
---

# Mean Absolute Deviation

MAD is the average absolute distance of price from its mean over the window - a
robust dispersion measure that, unlike standard deviation, does not square the
deviations and so is less sensitive to outliers. There is no `ta.mad()` builtin.

## Recipe

```pine
//@version=6
indicator("Mean Absolute Deviation", "MAD")

length = input.int(30, "Length", minval = 1)
src    = input.source(close, "Source")

mad(source, len) =>
    mean = ta.sma(source, len)
    ta.sma(math.abs(source - mean), len)

plot(mad(src, length), "MAD")
```

## How to read it

Higher MAD means price is dispersed (volatile); lower means tightly clustered. It
is the dispersion term in the CCI and a robust alternative to standard deviation
for bands. (This is the common Pine approximation: deviations are taken from each
bar's rolling mean rather than a single window mean.)
