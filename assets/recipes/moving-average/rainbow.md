---
title: Rainbow Moving Average
aliases: rainbow, rainbow moving average, rainbow charts, rainbow ma
---

# Rainbow Moving Average

The Rainbow is a ribbon of moving averages where each band smooths the previous
band instead of price: SMA1 over the source, SMA2 over SMA1, and so on. Stacking
ten short SMAs this way fans them into a "rainbow" whose width and order read off
trend strength and turns. There is no `ta.rainbow()` builtin, but each band is
just a chained `ta.sma`.

## Recipe

```pine
//@version=6
indicator("Rainbow Moving Average", "Rainbow", overlay = true)

length = input.int(2, "Length", minval = 1)
src    = input.source(close, "Source")

r1  = ta.sma(src, length)
r2  = ta.sma(r1,  length)
r3  = ta.sma(r2,  length)
r4  = ta.sma(r3,  length)
r5  = ta.sma(r4,  length)
r6  = ta.sma(r5,  length)
r7  = ta.sma(r6,  length)
r8  = ta.sma(r7,  length)
r9  = ta.sma(r8,  length)
r10 = ta.sma(r9,  length)

plot(r1,  "MA1",  color.red)
plot(r2,  "MA2",  color.orange)
plot(r3,  "MA3",  color.yellow)
plot(r4,  "MA4",  color.green)
plot(r5,  "MA5",  color.teal)
plot(r6,  "MA6",  color.aqua)
plot(r7,  "MA7",  color.blue)
plot(r8,  "MA8",  color.navy)
plot(r9,  "MA9",  color.purple)
plot(r10, "MA10", color.fuchsia)
```

## How to read it

A wide, fanned-out and orderly ribbon signals a strong trend; a narrow, tangled
ribbon signals consolidation. Watch for the bands compressing and crossing
together: that knot is where trend strength is fading and a reversal often
follows.
