---
title: Fisher Transform
aliases: fisher transform, ehlers fisher transform
---

# Fisher Transform

John Ehlers' Fisher Transform reshapes price into a near-Gaussian distribution so
that turning points stand out as sharp extremes. Price is normalized to its
recent range, then passed through the Fisher transform and smoothed recursively;
a signal line (the prior Fisher value) crosses it for triggers. There is no
`ta.fisher()` builtin. It is also exported as `ft()` by TradingView's `ta`
library:
https://pine-facade.tradingview.com/pine-facade/lib_list/?lib_id_prefix=TradingView/ta/12

## Recipe

```pine
//@version=6
indicator("Fisher Transform", "FISHER")

length = input.int(9, "Length", minval = 1)
signal = input.int(1, "Signal", minval = 1)

fisher(len, sig) =>
    hh  = ta.highest(hl2, len)
    ll  = ta.lowest(hl2, len)
    hlr = math.max(hh - ll, 0.001)
    pos = (hl2 - ll) / hlr - 0.5
    var float v = 0.0
    raw = 0.66 * pos + 0.67 * nz(v[1])
    v := raw > 0.99 ? 0.999 : raw < -0.99 ? -0.999 : raw
    var float fish = 0.0
    fish := 0.5 * (math.log((1 + v) / (1 - v)) + nz(fish[1]))
    [fish, fish[sig]]

[fishLine, sigLine] = fisher(length, signal)
plot(fishLine, "Fisher", color.blue)
plot(sigLine,  "Signal", color.orange)
```

## How to read it

Sharp spikes to high or low extremes flag stretched price likely to reverse; the
actionable signal is the Fisher line crossing its signal line. The clamp to
+-0.999 keeps the logarithm finite when price sits at the edge of its range.
