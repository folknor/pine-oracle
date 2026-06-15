---
title: Know Sure Thing
aliases: know sure thing, kst, pring kst
---

# Know Sure Thing

Martin Pring's KST sums four smoothed rates of change, each over a different
horizon and weighted by speed (slowest weighted most), into a single momentum
oscillator with a signal line. There is no `ta.kst()` builtin.

## Recipe

```pine
//@version=6
indicator("Know Sure Thing", "KST")

r1 = input.int(10, "ROC 1", minval = 1)
r2 = input.int(15, "ROC 2", minval = 1)
r3 = input.int(20, "ROC 3", minval = 1)
r4 = input.int(30, "ROC 4", minval = 1)
s1 = input.int(10, "SMA 1", minval = 1)
s2 = input.int(10, "SMA 2", minval = 1)
s3 = input.int(10, "SMA 3", minval = 1)
s4 = input.int(15, "SMA 4", minval = 1)
sigLen = input.int(9, "Signal", minval = 1)
src    = input.source(close, "Source")

// User functions cannot read globals, so the four smoothed ROCs are computed
// inline rather than wrapped in a function.
m1 = ta.sma(ta.roc(src, r1), s1)
m2 = ta.sma(ta.roc(src, r2), s2)
m3 = ta.sma(ta.roc(src, r3), s3)
m4 = ta.sma(ta.roc(src, r4), s4)
kstLine   = m1 + 2 * m2 + 3 * m3 + 4 * m4
kstSignal = ta.sma(kstLine, sigLen)

plot(kstLine,   "KST",    color.blue)
plot(kstSignal, "Signal", color.orange)
```

## How to read it

The KST crossing its signal line, and crossing zero, are the standard triggers;
the weighting toward slower cycles makes it a smoother, more deliberate momentum
read than a single ROC. Divergence against price flags weakening trends.
