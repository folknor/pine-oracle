---
title: Vortex Indicator
aliases: vortex indicator, vortex, vi
---

# Vortex Indicator

The Vortex Indicator captures directional trend movement with two lines: VI+
measures upward movement (this high vs the prior low) and VI- measures downward
movement (this low vs the prior high), each normalized by true range. There is no
`ta.vortex()` builtin. It is also exported as `vi()` by TradingView's `ta`
library:
https://pine-facade.tradingview.com/pine-facade/lib_list/?lib_id_prefix=TradingView/ta/12

## Recipe

```pine
//@version=6
indicator("Vortex", "VTX")

length = input.int(14, "Length", minval = 1)

vortex(len) =>
    trSum = math.sum(ta.tr(true), len)
    vmp   = math.abs(high - low[1])
    vmm   = math.abs(low - high[1])
    [math.sum(vmp, len) / trSum, math.sum(vmm, len) / trSum]

[vip, vim] = vortex(length)
plot(vip, "VI+", color.green)
plot(vim, "VI-", color.red)
```

## How to read it

When VI+ crosses above VI-, an uptrend is signalled; the reverse signals a
downtrend. The wider the gap between the lines, the stronger the trend; them
converging and crossing marks the turn.
