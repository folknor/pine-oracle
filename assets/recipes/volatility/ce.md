---
title: Chandelier Exit
aliases: chandelier exit, ce
---

# Chandelier Exit

Chuck LeBeau's Chandelier Exit sets a trailing stop an ATR multiple away from the
highest high (for longs) or lowest low (for shorts) over the window. It "hangs"
the stop from the extreme like a chandelier from a ceiling. There is no `ta.ce()`
builtin.

## Recipe

```pine
//@version=6
indicator("Chandelier Exit", "CE", overlay = true)

length = input.int(22, "Length", minval = 1)
mult   = input.float(3.0, "ATR Mult")

ce(len, m) =>
    atr_ = ta.atr(len)
    [ta.highest(high, len) - m * atr_, ta.lowest(low, len) + m * atr_]

[ceLong, ceShort] = ce(length, mult)
plot(ceLong,  "Long Exit",  color.green)
plot(ceShort, "Short Exit", color.red)
```

## How to read it

In a long trade, trail the long-exit line and close on a daily close below it; in
a short, use the short-exit line. The ATR multiple sets how much room you give the
trend - wider survives noise but gives back more profit at the turn.
