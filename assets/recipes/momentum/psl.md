---
title: Psychological Line
aliases: psychological line, psl
---

# Psychological Line

The Psychological Line is the percentage of bars in the window that closed higher
than the previous bar. It is a simple sentiment gauge of how persistently buyers
have shown up. There is no `ta.psl()` builtin.

## Recipe

```pine
//@version=6
indicator("Psychological Line", "PSL")

length = input.int(12, "Length", minval = 1)
src    = input.source(close, "Source")

psl(source, len) =>
    up = source > source[1] ? 1.0 : 0.0
    100 * math.sum(up, len) / len

plot(psl(src, length), "PSL")
```

## How to read it

Readings near 100 mean almost every recent bar was an up bar (possibly
overbought); near 0 means almost all were down bars (possibly oversold); 50 is
neutral. It is a sentiment extreme gauge, not a precise timing tool.
