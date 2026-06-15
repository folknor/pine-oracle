---
title: McGinley Dynamic
aliases: mcginley dynamic, mcginley dynamic line
---

# McGinley Dynamic

The McGinley Dynamic is a self-adjusting average that automatically speeds up in
down moves and slows in up moves, aiming to track price without the lag or the
whipsaw of a fixed moving average. It is a single recursive line with no `ta.*`
builtin.

## Recipe

```pine
//@version=6
indicator("McGinley Dynamic", "MD", overlay = true)

length = input.int(14, "Length", minval = 1)
src    = input.source(close, "Source")

mcginley(source, len) =>
    var float md = na
    md := na(md[1]) ? source : md[1] + (source - md[1]) / (len * math.pow(source / md[1], 4))
    md

plot(mcginley(src, length), "McGinley Dynamic", color.orange, 2)
```

## How to read it

The McGinley line is designed to stay closer to price than a same-length EMA and
to avoid the separation that opens up in fast moves, so it is read much like a
moving average but tolerates a longer length without falling behind. The
`source / md` ratio is the self-adjusting term; guard against `md` starting at
`na` (the recipe seeds it with the first price).
