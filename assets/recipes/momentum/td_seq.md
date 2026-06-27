---
title: TD Sequential
aliases: td_seq, td sequential, tom demark sequential, demark sequential
---

# TD Sequential

Tom DeMark's Sequential counts the setup phase that hunts for trend exhaustion.
A bar belongs to an up setup when its close is above the close four bars back, to
a down setup when it is below; the count runs up consecutively and resets the
moment the comparison fails. A maturing count (classically 6 through 9) warns the
move is stretched and a reversal is near. There is no `ta.td_seq()` builtin.

## Recipe

```pine
//@version=6
indicator("TD Sequential", "TD_SEQ")

tdSetup(up) =>
    cond = up ? close > close[4] : close < close[4]
    var int c = 0
    c := cond ? c + 1 : 0
    c > 0 ? math.min(c, 13) : na

upSeq = tdSetup(true)
dnSeq = tdSetup(false)
plot(upSeq, "Up Setup",   color.red,   style = plot.style_columns)
plot(dnSeq, "Down Setup", color.green, style = plot.style_columns)
hline(9, "Nine", color.gray)
```

## How to read it

A rising red count tallies an up setup (a potential top), a rising green count a
down setup (a potential bottom). The bars to watch are 6 through 9: a completed
9-count flags trend exhaustion and a likely reversal or pause. The count caps at
13, matching the source's 13-bar window; each setup resets to nothing the instant
the four-bar comparison breaks (Pine has no self-recursive functions, so the
running count is kept in a `var` updated over series history).
