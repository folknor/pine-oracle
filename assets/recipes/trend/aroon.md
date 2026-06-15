---
title: Aroon
aliases: aroon, aroon indicator, aroon oscillator
---

# Aroon

Aroon measures how recently the highest high and lowest low occurred within the
window. Aroon Up is high when a new high is recent; Aroon Down is high when a new
low is recent; the oscillator is their difference. There is no `ta.aroon()`
builtin (it is built from `ta.highestbars`/`ta.lowestbars`).

## Recipe

```pine
//@version=6
indicator("Aroon", "Aroon")

length = input.int(14, "Length", minval = 1)

aroon(len) =>
    up   = 100 * (ta.highestbars(high, len + 1) + len) / len
    down = 100 * (ta.lowestbars(low,  len + 1) + len) / len
    [up, down, up - down]

[aUp, aDown, aOsc] = aroon(length)
plot(aUp,   "Aroon Up",   color.green)
plot(aDown, "Aroon Down", color.red)
plot(aOsc,  "Oscillator", color.gray)
```

## How to read it

Aroon Up above 70 with Aroon Down below 30 marks a strong uptrend (and vice
versa); the two crossing signals a possible trend change. The oscillator above
zero leans bullish, below zero bearish. Both lines low means no trend - a range.
