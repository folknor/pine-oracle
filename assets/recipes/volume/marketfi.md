---
title: Market Facilitation Index
aliases: market facilitation index, marketfi, bw mfi
---

# Market Facilitation Index

Bill Williams' Market Facilitation Index measures how much price moved per unit of
volume - the range divided by volume. Read with volume, it classifies bars into
four states (the "MFI squares"). There is no `ta.marketfi()` builtin.

## Recipe

```pine
//@version=6
indicator("Market Facilitation Index", "MARKETFI")

marketfi = (high - low) / volume
plot(marketfi, "MARKETFI")
```

## How to read it

Williams reads MFI together with volume: MFI up + volume up ("green") means a
strong, facilitated move; MFI up + volume down ("fake") is a move on thin
participation; MFI down + volume down ("squat") often precedes a turn; MFI down +
volume up ("fade") signals a contested bar.
