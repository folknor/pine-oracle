---
title: Fibonacci Weighted Moving Average
aliases: fibonacci weighted moving average
---

# Fibonacci Weighted Moving Average

The FWMA is a weighted moving average whose weights are the Fibonacci numbers,
with the largest weight on the most recent bar. Because Fibonacci numbers grow
quickly, recent prices dominate even more than in a linear WMA. There is no
`ta.fwma()` builtin.

## Recipe

```pine
//@version=6
indicator("Fibonacci Weighted Moving Average", "FWMA", overlay = true)

length = input.int(10, "Length", minval = 1)
src    = input.source(close, "Source")

fwma(source, len) =>
    float prev  = 0.0
    float cur   = 1.0
    float total = 0.0
    float[] f = array.new_float(len)
    for i = 0 to len - 1
        array.set(f, i, cur)
        total += cur
        float nxt = prev + cur
        prev := cur
        cur  := nxt
    float num = 0.0
    for i = 0 to len - 1
        num += array.get(f, len - 1 - i) * source[i]
    num / total

plot(fwma(src, length), "FWMA", color.orange, 2)
```

## How to read it

The Fibonacci weights front-load recency more aggressively than a linear WMA, so
the FWMA is fast and hugs price closely. The first loop builds the Fibonacci
weights (oldest to newest); the second applies the largest weight to the current
bar. Read it as a responsive trend line.
