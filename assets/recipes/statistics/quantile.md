---
title: Rolling Quantile
aliases: quantile, rolling quantile, qtl, percentile
---

# Rolling Quantile

The rolling quantile reports the value below which a given fraction of the last
`length` samples fall: q = 0.5 is the rolling median, q = 0.25 the lower
quartile. It composes the `ta.percentile_linear_interpolation()` builtin, which
takes a 0-100 percentage, so the recipe just scales the 0-1 quantile fraction up
by 100.

## Recipe

```pine
//@version=6
indicator("Quantile", "QTL")

length = input.int(30,  "Length", minval = 1)
q      = input.float(0.5, "Quantile", minval = 0.0, maxval = 1.0, step = 0.05)
src    = input.source(close, "Source")

quantile(source, len, frac) =>
    ta.percentile_linear_interpolation(source, len, frac * 100)

plot(quantile(src, length, q), "Quantile")
```

## How to read it

The line traces a chosen rank of the recent distribution rather than its mean,
so it is robust to outliers: the median (q = 0.5) ignores a single spike that
would drag an average. Pair a low and a high quantile (say 0.1 and 0.9) to bracket
where price normally sits and flag excursions beyond that envelope.
