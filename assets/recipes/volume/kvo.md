---
title: Klinger Volume Oscillator
aliases: klinger volume oscillator, kvo, klinger oscillator
---

# Klinger Volume Oscillator

Stephen Klinger's oscillator signs each bar's volume by whether the typical price
(`hlc3`) rose or fell, then takes the difference of a fast and slow EMA of that
signed volume, with a signal line. It aims to predict reversals from volume
force. There is no `ta.kvo()` builtin.

## Recipe

```pine
//@version=6
indicator("Klinger Volume Oscillator", "KVO")

fast   = input.int(34, "Fast",   minval = 1)
slow   = input.int(55, "Slow",   minval = 1)
sigLen = input.int(13, "Signal", minval = 1)

kvo(f, s, sig) =>
    sv   = volume * math.sign(ta.change(hlc3))
    line = ta.ema(sv, f) - ta.ema(sv, s)
    [line, ta.ema(line, sig)]

[kvoLine, kvoSignal] = kvo(fast, slow, sigLen)
plot(kvoLine,   "KVO",    color.blue)
plot(kvoSignal, "Signal", color.orange)
```

## How to read it

Zero-line crosses and KVO/signal crossovers mark shifts in volume force. Its
headline use is divergence: price making a new extreme that the KVO does not
confirm warns the move is running out of volume.
