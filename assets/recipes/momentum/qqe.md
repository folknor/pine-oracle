---
title: Quantitative Qualitative Estimation
aliases: qqe, quantitative qualitative estimation, smoothed rsi trailing
---

# Quantitative Qualitative Estimation

QQE is a SuperTrend-style trailing stop built on a smoothed RSI rather than on
price. It takes an EMA of the RSI, derives an adaptive band width from a double
Wilder-smoothed average of that RSI's bar-to-bar range scaled by a factor, and
trails a long or short line that flips when the smoothed RSI crosses the opposite
band. There is no `ta.qqe()` builtin.

## Recipe

```pine
//@version=6
indicator("QQE", "QQE")

length = input.int(14, "RSI Length", minval = 1)
smooth = input.int(5,  "RSI Smooth", minval = 1)
factor = input.float(4.236, "Factor")
src    = input.source(close, "Source")

qqe(source, len, sm, fct) =>
    rsiMa = ta.ema(ta.rsi(source, len), sm)
    wild  = 2 * len - 1
    dar   = fct * ta.ema(ta.ema(math.abs(ta.change(rsiMa)), wild), wild)
    ub    = rsiMa + dar
    lb    = rsiMa - dar

    var float longLine  = 0.0
    var float shortLine = 0.0
    var int   dir       = 1

    cLong  = nz(longLine[1])
    cShort = nz(shortLine[1])
    pLong  = nz(longLine[2])
    pShort = nz(shortLine[2])
    cRsi   = rsiMa
    pRsi   = nz(rsiMa[1])

    longLine  := (pRsi > cLong  and cRsi > cLong)  ? math.max(cLong,  lb) : lb
    shortLine := (pRsi < cShort and cRsi < cShort) ? math.min(cShort, ub) : ub

    if (cRsi > cShort and pRsi < pShort) or (cRsi <= cShort and pRsi >= pShort)
        dir := 1
    else if (cRsi > cLong and pRsi < pLong) or (cRsi <= cLong and pRsi >= pLong)
        dir := -1

    qqeLine = dir == 1 ? longLine : shortLine
    [rsiMa, qqeLine]

[basis, trail] = qqe(src, length, smooth, factor)
plot(basis, "RSI MA",   color.blue)
plot(trail, "QQE Line", color.orange)
hline(50, "Midline", color.gray)
```

## How to read it

The smoothed RSI (`RSI MA`) crossing its QQE trailing line marks a momentum flip:
above the line is a long regime, below it a short. Because the band trails like a
SuperTrend, the line clings to the RSI in trends and only releases on a genuine
reversal, filtering the whipsaws a bare RSI would give. The recursive trailing
math is unrolled here with series history (`[1]` / `[2]` and `var`), since Pine
has no self-recursive functions.
