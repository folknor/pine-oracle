---
title: Squeeze Pro
aliases: squeeze_pro, squeeze pro, ttm squeeze pro, sqzpro
---

# Squeeze Pro

Squeeze Pro extends John Carter's TTM Squeeze with three Keltner widths instead
of one. A "squeeze" is on when the Bollinger Bands sit fully inside a Keltner
Channel - volatility is compressed and a move is coiling. Testing the bands
against narrow, normal, and wide Keltner channels grades how tight the coil is,
while a smoothed momentum histogram shows which way it is loading. There is no
`ta.squeeze()` builtin.

## Recipe

```pine
//@version=6
indicator("Squeeze Pro", "SQZPRO")

bbLen  = input.int(20,    "BB Length",  minval = 1)
bbStd  = input.float(2.0, "BB Std")
kcLen  = input.int(20,    "KC Length",  minval = 1)
wide   = input.float(2.0, "KC Wide")
normal = input.float(1.5, "KC Normal")
narrow = input.float(1.0, "KC Narrow")
momLen = input.int(12,    "Mom Length", minval = 1)
momSm  = input.int(6,     "Mom Smooth", minval = 1)

basis = ta.sma(close, bbLen)
dev   = bbStd * ta.stdev(close, bbLen)
bbU   = basis + dev
bbL   = basis - dev

kcBas = ta.sma(close, kcLen)
rng   = ta.sma(ta.tr(true), kcLen)
kcUw  = kcBas + wide   * rng
kcLw  = kcBas - wide   * rng
kcUn  = kcBas + normal * rng
kcLn  = kcBas - normal * rng
kcUr  = kcBas + narrow * rng
kcLr  = kcBas - narrow * rng

momo  = ta.sma(ta.mom(close, momLen), momSm)

onWide   = bbL > kcLw and bbU < kcUw
onNormal = bbL > kcLn and bbU < kcUn
onNarrow = bbL > kcLr and bbU < kcUr
offWide  = bbL < kcLw and bbU > kcUw

sqzColor = onNarrow ? color.red : onNormal ? color.orange : onWide ? color.yellow : offWide ? color.green : color.gray
plot(momo, "Momentum", momo >= 0 ? color.aqua : color.purple, style = plot.style_columns)
plot(0,    "Squeeze",  sqzColor, style = plot.style_circles, linewidth = 3)
```

## How to read it

The dots on the zero line read the coil tightness: red (narrow) is the tightest
squeeze, orange (normal) and yellow (wide) progressively looser, green means the
squeeze has fired (bands outside the wide Keltner), and gray is neutral. The
histogram is the loaded direction - trade the break in the histogram's direction
when the dots flip from a squeeze colour to green. True range (`ta.tr(true)`)
feeds the Keltner width, matching the source default.
