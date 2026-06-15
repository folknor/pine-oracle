---
title: Entropy
aliases: entropy, shannon entropy, entp
---

# Entropy

Shannon entropy measures the unpredictability of price over the window: treat each
bar's share of the window's total as a probability and sum `-p*log(p)`. Higher
entropy means more disorder. There is no `ta.entropy()` builtin.

## Recipe

```pine
//@version=6
indicator("Entropy", "ENTP")

length = input.int(10, "Length", minval = 1)
base   = input.float(2.0, "Base")
src    = input.source(close, "Source")

entropy(source, len, b) =>
    p = source / math.sum(source, len)
    -math.sum(p * math.log(p) / math.log(b), len)

plot(entropy(src, length, base), "Entropy")
```

## How to read it

Rising entropy signals an increasingly disordered, unpredictable market (often
ranging/choppy); falling entropy signals order, which frequently accompanies a
developing trend. It is a regime gauge rather than a directional signal.
