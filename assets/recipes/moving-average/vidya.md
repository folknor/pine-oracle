---
title: Variable Index Dynamic Average
aliases: variable index dynamic average
---

# Variable Index Dynamic Average

Tushar Chande's VIDYA is an EMA whose smoothing constant scales with volatility,
measured by the absolute Chande Momentum Oscillator (CMO). When momentum is
strong the average speeds up; when price chops it slows down. There is no
`ta.vidya()` builtin.

## Recipe

```pine
//@version=6
indicator("Variable Index Dynamic Average", "VIDYA", overlay = true)

length = input.int(14, "Length", minval = 1)
src    = input.source(close, "Source")

vidya(source, len) =>
    alpha = 2.0 / (len + 1)
    mom   = ta.change(source)
    pos   = math.sum(mom > 0 ?  mom : 0.0, len)
    neg   = math.sum(mom < 0 ? -mom : 0.0, len)
    denom = pos + neg
    cmo   = denom != 0 ? math.abs((pos - neg) / denom) : 0.0
    var float result = na
    result := na(result[1]) ? ta.sma(source, len) : alpha * cmo * source + result[1] * (1 - alpha * cmo)
    result

plot(vidya(src, length), "VIDYA", color.orange, 2)
```

## How to read it

A rising-then-flattening VIDYA marks momentum fading into a range; a steadily
sloping VIDYA confirms a trend. Because the effective length stretches when the
CMO is small, VIDYA stops chasing price in noise, which makes it a useful trend
filter rather than a fast signal line.
