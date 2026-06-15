---
title: Elder Force Index
aliases: elder force index, force index, efi
---

# Elder Force Index

Alexander Elder's Force Index multiplies the bar-to-bar price change by volume,
then smooths it - capturing the power behind a move from its direction, size, and
volume together. There is no `ta.efi()` builtin.

## Recipe

```pine
//@version=6
indicator("Elder Force Index", "EFI")

length = input.int(13, "Length", minval = 1)

plot(ta.ema(ta.change(close) * volume, length), "EFI")
```

## How to read it

Above zero means bulls have force (rising price on volume); below, bears. A short
EFI (length 2) is used to time entries within a trend; a longer EFI (13) gauges
the trend's underlying strength. Divergence flags exhaustion.
