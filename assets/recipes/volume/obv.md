---
title: On-Balance Volume
aliases: on-balance volume, on balance volume, obv
---

# On-Balance Volume

On-Balance Volume is a cumulative running total of volume signed by the direction
of the close: add the bar's volume when price closes up, subtract it when price
closes down. The level itself is arbitrary; its slope and divergences are the
signal. Despite being a TradingView staple, Pine has no `ta.obv()` builtin, so
build it from `ta.cum` of signed volume.

## Recipe

```pine
//@version=6
indicator("On-Balance Volume", "OBV")

obv() =>
    ta.cum(math.sign(ta.change(close)) * volume)

plot(obv(), "OBV")
```

## How to read it

OBV rising while price rises confirms the move is backed by volume; the headline
use is divergence: price making a new high that OBV does not confirm warns the
advance lacks participation and may reverse.
