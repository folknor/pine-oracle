---
title: Normalized ATR
aliases: normalized atr, natr, normalized average true range
---

# Normalized ATR

NATR expresses the Average True Range as a percentage of price, so volatility can
be compared across instruments and across time regardless of price level. There
is no `ta.natr()` builtin (it scales the `ta.atr` builtin).

## Recipe

```pine
//@version=6
indicator("Normalized ATR", "NATR")

length = input.int(14, "Length", minval = 1)

natr(len) =>
    100 * ta.atr(len) / close

plot(natr(length), "NATR")
```

## How to read it

A NATR of 2 means the average true range is 2% of price. Use it to size stops and
positions consistently across symbols (a $5 stock and a $5000 one can have the
same NATR), and to spot volatility regime shifts that raw ATR would obscure.
