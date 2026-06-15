---
title: Q Stick
aliases: q stick, qstick
---

# Q Stick

Tushar Chande's Q Stick quantifies candlestick trend by averaging the body
(close minus open) over a window. It is a numeric read of whether recent candles
have been predominantly up or down. There is no `ta.qstick()` builtin.

## Recipe

```pine
//@version=6
indicator("Q Stick", "QStick")

length = input.int(10, "Length", minval = 1)

qstick(len) =>
    ta.sma(close - open, len)

plot(qstick(length), "QStick", style = plot.style_histogram)
```

## How to read it

Above zero means up candles dominate (bullish pressure), below zero means down
candles dominate. Zero-line crossings are the basic signal; pairing Q Stick with
price can flag divergences. Swap the `ta.sma` for an EMA to make it more
responsive.
