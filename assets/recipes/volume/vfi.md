---
title: Volume Flow Indicator
aliases: volume flow indicator, vfi
---

# Volume Flow Indicator

The Volume Flow Indicator accumulates signed volume to track money flowing into
or out of an instrument, but only counts bars whose price change clears a
volatility threshold (small wiggles are ignored) and caps each bar's volume so a
single spike cannot dominate. The running sum is normalized by average volume and
smoothed. There is no `ta.vfi()` builtin.

## Recipe

```pine
//@version=6
indicator("Volume Flow Indicator", "VFI")

length = input.int(130, "Length", minval = 1)
pcoef  = input.float(0.2, "Price cutoff coef")
vcoef  = input.float(2.5, "Volume cap coef")

vfi(len, pc, vc) =>
    vave   = ta.sma(volume, len)[1]
    vmax   = vave * vc
    vcVol  = math.min(volume, vmax)
    inter  = ta.change(close)
    cutoff = pc * close
    mf     = math.abs(inter) > cutoff ? inter : 0.0
    vcp    = vcVol * mf
    raw    = math.sum(vcp, len) / ta.sma(vave, len)
    ta.ema(raw, 3)

plot(vfi(length, pcoef, vcoef), "VFI")
```

## How to read it

VFI above zero and rising marks net accumulation (money flowing in); below zero
and falling marks distribution. As with OBV, divergence against price is the
prime tell. This ports the pandas-ta-classic variant, which thresholds the raw
close change against `coef * close`; the canonical LazyBear VFI instead uses the
`hlc3` log return scaled by its own standard deviation.
