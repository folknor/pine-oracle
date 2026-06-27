---
title: Mesa Sine Wave
aliases: msw, mesa sine wave
---

# Mesa Sine Wave

The Mesa Sine Wave runs a small discrete Fourier transform over the last `period`
bars to estimate the dominant cycle's phase, then emits two oscillators: the sine
of that phase and a lead line 45 degrees ahead. Their crossings mark cycle turns
before price confirms them. There is no `ta.msw()` builtin.

## Recipe

```pine
//@version=6
indicator("Mesa Sine Wave", "MSW")

period = input.int(5, "Period", minval = 2)
src    = input.source(close, "Source")

msw(source, len) =>
    tpi = 2 * math.pi
    rp  = 0.0
    ip  = 0.0
    for j = 0 to len - 1
        rp += source[j] * math.cos(tpi * j / len)
        ip += source[j] * math.sin(tpi * j / len)
    phase = math.abs(rp) > 0.001 ? math.atan(ip / rp) : math.pi * (ip < 0 ? -1 : 1)
    if rp < 0
        phase := phase + math.pi
    phase := phase + math.pi / 2
    if phase < 0
        phase := phase + tpi
    if phase > tpi
        phase := phase - tpi
    [math.sin(phase), math.sin(phase + math.pi / 4)]

[sine, lead] = msw(src, period)
plot(sine, "Sine", color.blue)
plot(lead, "Lead", color.orange)
```

## How to read it

When the lead line crosses above the sine line a new up-cycle is starting; a cross
below calls the down-cycle. The two lines run roughly parallel in a trend and
weave tightly together when no clear cycle is present, so divergence between them
is the cue that a tradeable cycle has formed.
