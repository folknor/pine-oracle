---
title: BRAR
aliases: brar, ar indicator, br indicator, bull bear ratio
---

# BRAR

BRAR is a pair of Chinese-market sentiment gauges. AR (the "popularity"/energy
ratio) compares each bar's high-open span to its open-low span; BR (the
"willingness"/buy-sell ratio) compares the high above the prior close to the
prior close above the low. Both are summed over the window and scaled. There is
no `ta.brar()` builtin.

## Recipe

```pine
//@version=6
indicator("BRAR", "BRAR")

length = input.int(26, "Length", minval = 1)

brar(len) =>
    ho     = high - open
    ol     = open - low
    hcy    = math.max(high - close[1], 0)
    cyl    = math.max(close[1] - low,  0)
    arLine = 100 * math.sum(ho, len)  / math.sum(ol, len)
    brLine = 100 * math.sum(hcy, len) / math.sum(cyl, len)
    [arLine, brLine]

[ar, br] = brar(length)
plot(ar, "AR", color.blue)
plot(br, "BR", color.orange)
```

## How to read it

Both oscillate around 100 (balance). Readings well above 100 indicate strong
buying energy that can mark an overbought top; readings well below indicate
exhaustion that can mark a bottom. AR and BR diverging from each other is the
classic warning the two factions disagree.
