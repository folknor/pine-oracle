---
title: Bar coloring
source: https://www.tradingview.com/pine-script-docs/visuals/bar-coloring/
section: visuals
---

# Bar coloring {#bar-coloring}

The [barcolor()](https://www.tradingview.com/pine-script-reference/v6/#fun_barcolor) function colors bars on the main chart, regardless of whether the script is running in the main chart pane or a separate pane.

The function’s signature is:

```
barcolor(color, offset, editable, show_last, title, display) → void
```

The coloring can be conditional because the `color` parameter accepts “series color” arguments.

The following script renders _inside_ and _outside_ bars in different colors:

![image](https://www.tradingview.com/pine-script-docs/_astro/BarColoring-1.BVBRLjUu_24IaDi.webp)

```pine
//@version=6
indicator("barcolor example", overlay = true)
isUp = close > open
isDown = close <= open
isOutsideUp = high > high[1] and low < low[1] and isUp
isOutsideDown = high > high[1] and low < low[1] and isDown
isInside = high < high[1] and low > low[1]
barcolor(isInside ? color.yellow : isOutsideUp ? color.aqua : isOutsideDown ? color.purple : na)
```

Note that:

-   The [na](https://www.tradingview.com/pine-script-reference/v6/#var_na) value leaves bars as is.
-   In the [barcolor()](https://www.tradingview.com/pine-script-reference/v6/#fun_barcolor) call, we use embedded [?:](https://www.tradingview.com/pine-script-reference/v6/#op_?:) ternary operator expressions to select the color.
