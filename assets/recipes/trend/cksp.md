---
title: Chande Kroll Stop
aliases: chande kroll stop, cksp
---

# Chande Kroll Stop

The Chande Kroll Stop places volatility-based trailing stops on both sides of
price: a long stop below (from the highest high minus an ATR multiple) and a short
stop above (from the lowest low plus an ATR multiple), each smoothed by a second
extreme over `q` bars. Defaults shown are the TradingView variant. There is no
`ta.cksp()` builtin.

## Recipe

```pine
//@version=6
indicator("Chande Kroll Stop", "CKSP", overlay = true)

p = input.int(10, "ATR Period",  minval = 1)
x = input.float(1, "ATR Mult")
q = input.int(9,  "Stop Period", minval = 1)

cksp(pp, xx, qq) =>
    atr_      = ta.atr(pp)
    longStop  = ta.highest(ta.highest(high, pp) - xx * atr_, qq)
    shortStop = ta.lowest(ta.lowest(low, pp) + xx * atr_, qq)
    [longStop, shortStop]

[ls, ss] = cksp(p, x, q)
plot(ls, "Long Stop",  color.green)
plot(ss, "Short Stop", color.red)
```

## How to read it

In an uptrend, trail the long stop beneath price and exit on a close below it; in
a downtrend, use the short stop above. A close crossing from one stop to the other
flags a trend flip. The book variant uses (p=10, x=3, q=20) with an SMA-based ATR.
