---
title: Relative Strength Xtra
aliases: rsx, relative strength xtra, jurik rsx
---

# Relative Strength Xtra

RSX is a Jurik-inspired smoother RSI. Instead of Wilder's averaging it runs the
price change through a cascade of paired exponential filters (each stage forms
`1.5*fast - 0.5*slow` to sharpen the response), tracks the absolute change the
same way, and rescales their ratio into a 0-100 oscillator. The result is far
less jagged than a plain RSI with only slight extra lag. There is no `ta.rsx()`
builtin.

## Recipe

```pine
//@version=6
indicator("Relative Strength Xtra", "RSX")

length = input.int(14, "Length", minval = 1)
src    = input.source(close, "Source")

rsx(source, len) =>
    f18 = 3.0 / (len + 2.0)
    f20 = 1.0 - f18
    v8  = 100.0 * nz(ta.change(source))

    var float f28 = 0.0
    var float f30 = 0.0
    var float f38 = 0.0
    var float f40 = 0.0
    var float f48 = 0.0
    var float f50 = 0.0
    var float f58 = 0.0
    var float f60 = 0.0
    var float f68 = 0.0
    var float f70 = 0.0
    var float f78 = 0.0
    var float f80 = 0.0

    f28 := f20 * f28 + f18 * v8
    f30 := f18 * f28 + f20 * f30
    vc  = 1.5 * f28 - 0.5 * f30
    f38 := f20 * f38 + f18 * vc
    f40 := f18 * f38 + f20 * f40
    v10 = 1.5 * f38 - 0.5 * f40
    f48 := f20 * f48 + f18 * v10
    f50 := f18 * f48 + f20 * f50
    v14 = 1.5 * f48 - 0.5 * f50
    f58 := f20 * f58 + f18 * math.abs(v8)
    f60 := f18 * f58 + f20 * f60
    v18 = 1.5 * f58 - 0.5 * f60
    f68 := f20 * f68 + f18 * v18
    f70 := f18 * f68 + f20 * f70
    v1c = 1.5 * f68 - 0.5 * f70
    f78 := f20 * f78 + f18 * v1c
    f80 := f18 * f78 + f20 * f80
    v20 = 1.5 * f78 - 0.5 * f80

    v4 = v20 > 1e-10 ? (v14 / v20 + 1.0) * 50.0 : 50.0
    math.max(0.0, math.min(100.0, v4))

plot(rsx(src, length), "RSX", color.blue)
hline(70, "Overbought", color.gray)
hline(30, "Oversold",   color.gray)
```

## How to read it

Read it like an RSI: above 70 is overbought, below 30 oversold, and the 50 line
splits bullish from bearish momentum, but the smoother curve makes crossings and
divergences cleaner to act on. Each `var float` holds one filter's previous-bar
state (Pine has no self-recursive functions, so the IIR cascade is unrolled with
`:=` over series history). The original seeds an explicit warmup counter; this
port lets the filters settle from zero, so the first few bars differ slightly.
