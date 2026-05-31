---
title: Overview
source: https://www.tradingview.com/pine-script-docs/visuals/overview/
section: visuals
---

# Overview {#overview}

## Introduction {#introduction}

Well-designed visuals make indicators and strategies easier to use and less cluttered. Each visual element presents data differently:

-   [Plot visuals](https://www.tradingview.com/pine-script-docs/visuals/overview/#plot-visuals) include all `plot*()` functions, horizontal levels, background and bar coloring, and fills.
-   [Drawing visuals](https://www.tradingview.com/pine-script-docs/visuals/overview/#drawing-visuals) include lines, polylines, linefills, boxes, labels, and tables.

Scripts can configure where and how the visual elements appear by using the [script-wide visual settings](https://www.tradingview.com/pine-script-docs/visuals/overview/#script-wide-visual-settings).

By understanding [when to use](https://www.tradingview.com/pine-script-docs/visuals/overview/#when-to-use) each tool effectively, programmers can pick the best visual for the task to make the most of the power of Pine Script®.

NoteUsers can draw directly on TradingView charts using the [Drawing Tools](https://www.tradingview.com/support/solutions/43000703396-drawing-tools-available-on-tradingview/). While such drawings might sometimes resemble visuals created by Pine scripts, they are **unrelated** entities. Pine scripts cannot interact with drawing tools from the chart interface, and cursor actions do not modify Pine drawing objects.

This page describes plots and drawings, and what their differences are. It includes all the available visual constructs and examples of their use in built-in indicators (for more details about a specific visual element, refer to its User Manual page).

## Script-wide visual settings {#script-wide-visual-settings}

Some visual settings control how all of the script’s outputs _collectively_ appear on the chart, regardless of their individual properties. These script-wide visual settings are parameters in the [indicator()](https://www.tradingview.com/pine-script-reference/v6/#fun_indicator) or [strategy()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy) declaration statement.

### `overlay` {#overlay}

The `overlay` parameter controls whether the script’s outputs appear in the main pane or a separate pane. By default, its value is `false`, so adding a script to the chart displays its visual outputs in a _separate_ pane to the main chart series.

Whereas the `overlay` parameter affects the script as a whole, the `force_overlay` parameter controls the pane location for _individual_ elements. Using `force_overlay = true` displays the specified element in the main pane, even if the script’s `overlay` argument is `false`. This allows a script occupying a separate pane to overlay only some visuals on the main chart. The `force_overlay` parameter is available for all `plot*()` functions, [bgcolor()](https://www.tradingview.com/pine-script-reference/v6/#fun_bgcolor), and all drawing `*.new()` constructor functions ([box.new()](https://www.tradingview.com/pine-script-reference/v6/#fun_box.new), [label.new()](https://www.tradingview.com/pine-script-reference/v6/#fun_label.new), etc.).

For example, the built-in [Seasonality](https://www.tradingview.com/support/solutions/43000723025-seasonality/) indicator uses `overlay = false` to display in a separate pane, where it displays its primary visual of a [table](https://www.tradingview.com/pine-script-docs/visuals/tables/), but draws [boxes](https://www.tradingview.com/pine-script-docs/visuals/lines-and-boxes/#boxes) on the main chart with `force_overlay = true`:

![image](https://www.tradingview.com/pine-script-docs/_astro/Visuals-overview-Script-wide-visual-settings-1.C9OzYRWa_tOtaJ.webp)

### `scale` {#scale}

A script’s `scale` parameter specifies the y-axis scale that its pane visuals use. By default, scripts overlayed in the main pane use the existing chart scale ([scale.none](https://www.tradingview.com/pine-script-reference/v6/#const_scale.none)). Specifying a [scale.right](https://www.tradingview.com/pine-script-reference/v6/#const_scale.right) or [scale.left](https://www.tradingview.com/pine-script-reference/v6/#const_scale.left) argument in overlayed scripts generates a _new scale_ distinct from the main chart’s price scale. Scripts displaying in a separate pane generate their own scale by default, which they can also set to the left or right position. For instance, this image shows an overlayed indicator using a distinct right-side scale, and a separate pane indicator using a left-side scale:

![image](https://www.tradingview.com/pine-script-docs/_astro/Visuals-overview-Script-wide-visual-settings-2.8V2mdHkD_Z2pw9Rx.webp)

### `behind_chart` {#behind_chart}

The `behind_chart` parameter specifies whether a script’s visuals appear behind or in front of the main chart series. By default, its value is `true`, so visuals overlayed in the main pane appear behind the chart bars. When `behind_chart` is `false`, visuals appear _in front of_ the bars, which may obscure bars, depending on the type of visual and its color transparency:

![image](https://www.tradingview.com/pine-script-docs/_astro/Visuals-overview-Script-wide-visual-settings-3.CGH8GQWk_ZAvDKL.webp)

### Changing settings {#changing-settings}

To adjust the visual settings of a script on the chart, click the “More” menu (three dots icon) in the script’s status line. Options are available to adjust the script’s visual order, move it to another pane, and change its y-axis scale:

![image](https://www.tradingview.com/pine-script-docs/_astro/Visuals-overview-Script-wide-visual-settings-4.CgYJxP3X_Z2g79EG.webp)

NoticeScripts evaluate the visual settings in the [indicator()](https://www.tradingview.com/pine-script-reference/v6/#fun_indicator) or [strategy()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy) declaration statement _only once_, when the script first loads on the chart. Updating parameters like `overlay` or `scale` in the code of a script instance that is already active on the chart does **not** update its existing display. Add a new script instance to the chart to apply updated visual settings.

## Plot visuals {#plot-visuals}

The outputs of the following functions are classified as plot visuals:

-   All `plot*()` functions:
    -   Data series [plots](https://www.tradingview.com/pine-script-docs/visuals/plots/) using [plot()](https://www.tradingview.com/pine-script-reference/v6/#fun_plot)
    -   [Shape](https://www.tradingview.com/pine-script-docs/visuals/text-and-shapes/#plotshape) plots using [plotshape()](https://www.tradingview.com/pine-script-reference/v6/#fun_plotshape)
    -   [Character](https://www.tradingview.com/pine-script-docs/visuals/text-and-shapes/#plotchar) plots using [plotchar()](https://www.tradingview.com/pine-script-reference/v6/#fun_plotchar)
    -   [Arrow](https://www.tradingview.com/pine-script-docs/visuals/text-and-shapes/#plotarrow) plots using [plotarrow()](https://www.tradingview.com/pine-script-reference/v6/#fun_plotarrow)
    -   [Bar](https://www.tradingview.com/pine-script-docs/visuals/bar-plotting/#plotting-bars-with-plotbar) plots using [plotbar()](https://www.tradingview.com/pine-script-reference/v6/#fun_plotbar)
    -   [Candle](https://www.tradingview.com/pine-script-docs/visuals/bar-plotting/#plotting-candles-with-plotcandle) plots using [plotcandle()](https://www.tradingview.com/pine-script-reference/v6/#fun_plotcandle)
-   [Bar coloring](https://www.tradingview.com/pine-script-docs/visuals/bar-coloring/) using [barcolor()](https://www.tradingview.com/pine-script-reference/v6/#fun_barcolor)
-   [Background coloring](https://www.tradingview.com/pine-script-docs/visuals/backgrounds/) using [bgcolor()](https://www.tradingview.com/pine-script-reference/v6/#fun_bgcolor)
-   [Horizontal levels](https://www.tradingview.com/pine-script-docs/visuals/levels/) using [hline()](https://www.tradingview.com/pine-script-reference/v6/#fun_hline)
-   [Fills](https://www.tradingview.com/pine-script-docs/visuals/fills/#plot-and-hline-fills) for plots and horizontal levels using [fill()](https://www.tradingview.com/pine-script-reference/v6/#fun_fill)

Plots are _serial_ visuals that always return a result on each bar — although the result can be [na](https://www.tradingview.com/pine-script-reference/v6/#var_na). One plot therefore forms a _series_. By contrast, [drawing visuals](https://www.tradingview.com/pine-script-docs/visuals/overview/#drawing-visuals) instantiate _individual objects_. A single plot visual function call can display results on all the bars in the main series, no matter how many bars display in the series, while drawings adhere to a [drawing limit](https://www.tradingview.com/pine-script-docs/writing/limitations/#line-box-polyline-and-label-limits) of approximately ~500 objects.

A script creates plot visuals sequentially as it executes across the chart bars, so it cannot draw them into the past or future all at once like drawings. For example, `plot(close)` plots the _current_ [close](https://www.tradingview.com/pine-script-reference/v6/#var_close) on the _current_ bar. Pine’s [execution model](https://www.tradingview.com/pine-script-docs/language/execution-model/) then repeats this for every bar in the dataset.

Scripts create plots with [offsets](https://www.tradingview.com/pine-script-docs/visuals/plots/#offsets) in exactly the same way. They appear to end at past or future bars because the script executes the same plot call on each bar and simply displays each result the same _fixed_ number of bars forwards or backwards.

### Display in other locations {#display-in-other-locations}

Plots can display results in locations other than the chart pane, unlike drawings. The last numeric value of a plot can display in the price scale. The script’s status line and the Data Window can display plot values for specific bars, and the values update as the user hovers over different bars:

![image](https://www.tradingview.com/pine-script-docs/_astro/Visuals-overview-Plot-visuals-UI-integration-and-display-1.wierKQQW_Z2gr137.webp)

In one script, plots can display their results in different places by using different arguments for the `display` parameter for each plot function. For example, a script can display one plot’s results in all locations, display another plot everywhere but the status line, and create a third plot with no visible display.

The `plot*()` functions accept multiple `display.*` arguments and support addition and subtraction to combine arguments for further customization. Other, numerically simpler plot visuals like [horizontal levels](https://www.tradingview.com/pine-script-docs/visuals/levels/), [fills](https://www.tradingview.com/pine-script-docs/visuals/overview/#fills), and [coloring functions](https://www.tradingview.com/pine-script-docs/visuals/overview/#background-and-bar-coloring) have only two display states: they either display a pane visual ([display.all](https://www.tradingview.com/pine-script-reference/v6/#const_display.all)) or are hidden ([display.none](https://www.tradingview.com/pine-script-reference/v6/#const_display.none)).

This simple demonstration script uses various plot visuals and `display` locations to plot the [open](https://www.tradingview.com/pine-script-reference/v6/#var_open) and [close](https://www.tradingview.com/pine-script-reference/v6/#var_close) prices, plot the difference between them (`barCO`), and to signal when that difference is greater than 5:

![image](https://www.tradingview.com/pine-script-docs/_astro/Visuals-overview-Plot-visuals-UI-integration-and-display-2.DvJhT7q-_Z10iJYv.webp)

```pine
//@version=6
indicator("Plot visuals display demo", overlay = false)

//@variable The difference between the bar's `open` and `close` prices.  
float barCO = close - open

// Horizontal lines and fills have only two possible `display` states. 
h1 = hline(125, "Level 125", linewidth = 2, display = display.none)
h2 = hline(100, "Level 100", linewidth = 2, display = display.all)
fill(h1, h2, color = color.new(color.blue, 90), display = display.all)

// `plot*()` visuals accept multiple `display` options and support addition and subtraction.
plot(close, "Close", color.blue,   3, display = display.all)
plot(open,  "Open",  color.orange, 3, display = display.all - display.pane)
plotarrow(barCO, "Bar CO", color.green, color.red, display = display.status_line + display.data_window)
plotshape(barCO > 5, "Large CO", shape.circle, location.abovebar, color.fuchsia, display = display.pane)
```

Note that:

-   Although there are no arrows visible in the script pane, the [plotarrow()](https://www.tradingview.com/pine-script-reference/v6/#fun_plotarrow) call still calculates and plots the `barCO` values on every bar, as indicated by the “Bar CO” result in the Data Window and the matching green result in the status line.
-   Since `plotshape(barCO > 5)` uses a “bool” series, the plot’s numeric results can only be 1 or 0 on any bar. We set it to display only in the chart pane because that’s our most useful visual signal for this plot. Being selective with display options can help to keep results in any one location free from clutter.

TipScripts can also use `display.*` arguments to show or hide [input](https://www.tradingview.com/pine-script-docs/concepts/inputs/) values next to the script title in the status line and Data Window.

The `format` and `precision` parameters of `plot*()` functions can further customize how numeric results appear in the status line, price scale, and Data Window. The `format` parameter specifies whether to format plot values as prices, percentages, or volume. The `precision` parameter specifies the number of decimal digits that plot values include for non-volume formats. See the [\`plot()\` parameters](https://www.tradingview.com/pine-script-docs/visuals/plots/#plot-parameters) section of the [Plots](https://www.tradingview.com/pine-script-docs/visuals/plots/) page to learn more.

Additionally, users can manage whether numeric plot results are visible for a given indicator or chart by using settings at both the indicator and chart level, without editing any source code (see the Help Center article on how to [hide values of indicators](https://www.tradingview.com/support/solutions/43000659666-i-want-to-hide-the-values-of-individual-indicators/) for more). An indicator’s settings control whether any plot values appear in _that indicator’s_ status line and price scale. A chart’s settings control whether status line and price scale values appear _at all_ in any indicators on that chart. Disabling the indicator settings overrides the script’s per-plot `display` properties, while the chart settings override both.

Users can also customize the visibility, color, and style of plot visuals without needing to create new inputs or edit the script. Settings are automatically generated in the indicator’s “Style” tab for every plot visual in the script, regardless of their `display` state:

![image](https://www.tradingview.com/pine-script-docs/_astro/Visuals-overview-Plot-visuals-UI-integration-and-display-3.CZik0MJA_2mUA62.webp)

Note that if the script generates any _dynamic colors_, the color pickers in the “Style” settings do not display. See the [Maintaining automatic color selectors](https://www.tradingview.com/pine-script-docs/visuals/colors/#maintaining-automatic-color-selectors) section of the [Colors](https://www.tradingview.com/pine-script-docs/visuals/colors/) page to learn more.

The `display.*` arguments represent the _default state_ of the script’s plot visuals. Disabling a plot from the indicator’s “Style” settings and then reactivating it causes the plot to revert to [display.all](https://www.tradingview.com/pine-script-reference/v6/#const_display.all), unless the indicator is reset to its default settings.

TipTo prevent users from changing a plot’s properties from the script’s “Settings/Style” tab, set the `plot*()` call’s `editable` argument to `false`.

The ability to display outputs in several locations and to visually track a series across the chart bars makes plot visuals useful debugging tools. See the [Plots and chart colors](https://www.tradingview.com/pine-script-docs/writing/debugging/#plots-and-chart-colors) section of the [Debugging](https://www.tradingview.com/pine-script-docs/writing/debugging/) page for more information.

### External uses: exports, alerts, and more {#external-uses-exports-alerts-and-more}

Unlike drawings, plots have uses outside the script: exporting data, creating [alerts](https://www.tradingview.com/pine-script-docs/concepts/alerts/), setting another indicator’s [source input](https://www.tradingview.com/pine-script-docs/concepts/inputs/#source-input), and scanning watchlists using the [Pine Screener](https://www.tradingview.com/pine-screener/).

These uses for plot results function regardless of a plot’s `display.*` state on the chart and do not require special code for the outputs. Indeed, when creating plots for use in alerts or data exports, using [display.none](https://www.tradingview.com/pine-script-reference/v6/#const_display.none) can keep a script’s visuals clutter-free and avoid distorting the chart scale.

Users can export plots using the “Export chart data” feature, which generates a comma-separated values (CSV) file of the chart data (see the section on [exporting indicator data to a file](https://www.tradingview.com/pine-script-docs/faq/indicators/#is-it-possible-to-export-indicator-data-to-a-file) in the [Indicators FAQ](https://www.tradingview.com/pine-script-docs/faq/indicators/) page). The exported data includes the symbol’s OHLC ([open](https://www.tradingview.com/pine-script-reference/v6/#var_open), [high](https://www.tradingview.com/pine-script-reference/v6/#var_high), [low](https://www.tradingview.com/pine-script-reference/v6/#var_low), and [close](https://www.tradingview.com/pine-script-reference/v6/#var_close)) values and any numeric plot results generated by active scripts on the chart, including those displayed only in the Data Window or status line. Drawings and hidden scripts are excluded from exports.

An alert can use any `plot*()` call executing on the chart as its trigger condition. Users can create alerts based on plots even if the script does not include any alert-specific code such as [alert()](https://www.tradingview.com/pine-script-reference/v6/#fun_alert) or [alertcondition()](https://www.tradingview.com/pine-script-reference/v6/#fun_alertcondition). See the Help Center article on [how to create alerts](https://www.tradingview.com/support/solutions/43000520149-tradingview-alerts-how-to-get-notifications-immediately/#How-to-create-alerts-on-TradingView) from the user interface. Users can also include the dynamic results from up to 20 `plot*()` series in an alert’s message using [placeholders](https://www.tradingview.com/pine-script-docs/concepts/alerts/#placeholders), as explained in the Help Center article on [using variable values in alerts](https://www.tradingview.com/support/solutions/43000531021/).

A script can use plots that are output by other indicators on the chart as a _source input_. The [input.source()](https://www.tradingview.com/pine-script-reference/v6/#fun_input.source) function creates a “Source” dropdown in the script’s “Inputs” settings, from which users can then select any plots displayed on the chart as the input source. Any calculated plots can act as source inputs even if they are hidden from the current chart display (e.g., the unseen `plotarrow(barCO)` plot from the example in the [display in other locations](https://www.tradingview.com/pine-script-docs/visuals/overview/#display-in-other-locations) section above, or any hidden indicators). Using a source input links both scripts, so changes to the original plot subsequently alter the input plot, and removing the source indicator from the chart removes the dependent script.

The [Pine Screener](https://www.tradingview.com/pine-screener/) uses an indicator’s plots to scan a watchlist of symbols. It generates columns showing the results of the indicator’s [plot()](https://www.tradingview.com/pine-script-reference/v6/#fun_plot) and [alertcondition()](https://www.tradingview.com/pine-script-reference/v6/#fun_alertcondition) calls for each symbol. Users can also choose to filter screener results based on plot conditions. See this Help Center article on [the Pine Screener](https://www.tradingview.com/support/solutions/43000742436-tradingview-pine-screener-key-features-and-requirements/) to learn more.

### Limitations {#limitations}

Scripts can plot visuals only in the _global_ scope. Unlike drawings, plots cannot be included in the local scopes of [loops](https://www.tradingview.com/pine-script-docs/language/loops/), [conditional structures](https://www.tradingview.com/pine-script-docs/language/conditional-structures/), or [user-defined functions](https://www.tradingview.com/pine-script-docs/language/user-defined-functions/) and [methods](https://www.tradingview.com/pine-script-docs/language/methods/#user-defined-methods), and plot calls can only accept variables and literals that are declared globally. However, a script can still create visuals that [plot conditionally](https://www.tradingview.com/pine-script-docs/visuals/plots/#plotting-conditionally) by using [na](https://www.tradingview.com/pine-script-reference/v6/#var_na) values for a plot’s `series` or `color` arguments, thus hiding the plot on certain bars.

While plot visuals are well suited for displaying dynamically-calculated series, those that support text, like [plotshape()](https://www.tradingview.com/pine-script-reference/v6/#fun_plotshape) and [plotchar()](https://www.tradingview.com/pine-script-reference/v6/#fun_plotchar), cannot display dynamic text. The parameters of these functions accept “const string” arguments, so the same text displays on all the bars, and it cannot change during execution or be an [input](https://www.tradingview.com/pine-script-docs/concepts/inputs/) value, unlike the text supported in [drawing visuals](https://www.tradingview.com/pine-script-docs/visuals/overview/#drawing-visuals).

Plots can be offset into the past or future, but only by a fixed number of bars. This makes plotted shapes, for example, suitable for marking [Williams fractals](https://www.tradingview.com/support/solutions/43000591663/), which confirm after a known number of bars, but unsuitable for marking more complex types of events that confirm after an arbitrary number of bars.

Each script instance can create a maximum of 64 plots. Depending on the complexity of the plot and its arguments, one function call can count _more than once_ towards the [plot count limit](https://www.tradingview.com/pine-script-docs/visuals/plots/#plot-count-limit). See the [plot limits](https://www.tradingview.com/pine-script-docs/writing/limitations/#plot-limits) section of the [Limitations](https://www.tradingview.com/pine-script-docs/writing/limitations/) page for more information.

## Drawing visuals {#drawing-visuals}

Pine drawings display in a script’s pane, and provide the flexibility to represent graphical data beyond plotting series. The following elements are classified as drawing visuals:

-   [Lines](https://www.tradingview.com/pine-script-docs/visuals/lines-and-boxes/#lines)
-   [Polylines](https://www.tradingview.com/pine-script-docs/visuals/lines-and-boxes/#polylines)
-   [Linefills](https://www.tradingview.com/pine-script-docs/visuals/fills/#line-fills)
-   [Boxes](https://www.tradingview.com/pine-script-docs/visuals/lines-and-boxes/#boxes)
-   [Labels](https://www.tradingview.com/pine-script-docs/visuals/text-and-shapes/#labels)
-   [Tables](https://www.tradingview.com/pine-script-docs/visuals/tables/)

Drawings are _objects_, unlike [plots](https://www.tradingview.com/pine-script-docs/visuals/overview/#plot-visuals), which are _serial_ visuals, so calling a drawing function **does not** create a visual that always returns a persistent result on every bar in the dataset. Instead, a drawing function references _one instance_ of a drawing object, which can be at an arbitrary location relative to the bar on which the script called the function.

Since drawings are not serialized, scripts can call the same drawing function several times on one bar to create multiple drawings at different locations on the chart at once.

Each drawing visual has its own namespace with built-in functions for creating and managing the drawing objects. Most drawing parameters accept “series” types, which allows the visuals to use dynamic positions, colors, styles, etc. Drawing parameters support [input](https://www.tradingview.com/pine-script-docs/concepts/inputs/) values and complex expressions as arguments, and can update these arguments as the script executes from bar to bar. Drawings like labels, boxes, and tables can also display dynamic text.

Scripts can create and manage drawing visuals from _local_ scopes, so programmers can include drawing calls in [conditional structures](https://www.tradingview.com/pine-script-docs/language/conditional-structures/), [loops](https://www.tradingview.com/pine-script-docs/language/loops/), and [user-defined functions](https://www.tradingview.com/pine-script-docs/language/user-defined-functions/) or [methods](https://www.tradingview.com/pine-script-docs/language/methods/#user-defined-methods), unlike plot calls. While scripts _can_ call drawing functions globally, it’s rarely necessary to execute drawings on every bar. Further, because scripts that create drawing objects on each bar are likely to reach the [limit](https://www.tradingview.com/pine-script-docs/writing/limitations/#line-box-polyline-and-label-limits) for that drawing type, it’s more usual to create drawings in local scopes.

The ability of drawing functions to display dynamic data at any available chart location and to run in local scopes makes them useful debugging tools. See the [Pine drawings](https://www.tradingview.com/pine-script-docs/writing/debugging/#pine-drawings) section of the [Debugging](https://www.tradingview.com/pine-script-docs/writing/debugging/) page for more information.

### Display and customization {#display-and-customization}

Unlike plots, drawings do not [display in other locations](https://www.tradingview.com/pine-script-docs/visuals/overview/#display-in-other-locations) — they display a visual only in the chart pane. Therefore, they cannot show any numeric results in the script’s status line, price scale, or Data Window, or by hovering over the drawing. Likewise, using drawings in a script does not automatically generate color/style customization options in the indicator’s “Style” tab.

Instead, the “Style” settings generate a checkbox for each drawing _type_ used by a script, which toggles the visibility of **all** objects of that type in that indicator:

![image](https://www.tradingview.com/pine-script-docs/_astro/Visuals-overview-Drawing-visuals-1.DHW8vFIY_2mH3BP.webp)

However, since drawings accept “series” arguments, scripts can use [inputs](https://www.tradingview.com/pine-script-docs/concepts/inputs/) to create fully customizable drawing visuals. For example, this script uses [string inputs](https://www.tradingview.com/pine-script-docs/concepts/inputs/#string-input), [color inputs](https://www.tradingview.com/pine-script-docs/concepts/inputs/#color-input), and [integer inputs](https://www.tradingview.com/pine-script-docs/concepts/inputs/#integer-input) to allow users to easily customize the appearance of the [table](https://www.tradingview.com/pine-script-docs/visuals/tables/) and [label](https://www.tradingview.com/pine-script-docs/visuals/text-and-shapes/#labels) visuals from the indicator’s “Inputs” tab:

![image](https://www.tradingview.com/pine-script-docs/_astro/Visuals-overview-Drawing-visuals-2.BGEZYA-4_Z26FHFE.webp)

```pine
//@version=6
indicator("Customizable drawings demo", overlay = true)

// Input `group` headers to distinguish the table style inputs and the label style inputs.
const string G1 = "Table Style"
const string G2 = "Label Style"
// Create user inputs for customizing `table` style (position, colors, text size).
string tbVerticalInput   = input.string("Top", "Position", ["Top", "Middle", "Bottom"], inline = "Pos", group = G1)
string tbHorizontalInput = input.string("Right", "Center", ["Left", "Center", "Right"], inline = "Pos", group = G1)
color  tbBackgroundInput = input.color(#ffeb3bb3,  "Background color", inline = "Col", group = G1)
color  tbBorderInput     = input.color(color.white,"Border color",     inline = "Col", group = G1)
string tbTextSizeInput   = input.string(size.large, "Text size", inline = "Txt", group = G1,
     options = [size.tiny, size.small, size.normal, size.large, size.huge, size.auto])
color tbTextColorInput = input.color(color.black, "Text color", inline = "Txt", group = G1)
// Create user inputs for customizing `label` style (size, colors).
int   lblSizeInput      = input.int(16, "Label size", minval = 0, inline = "Lbl", group = G2)
color lblColorInput     = input.color(color.orange, "Label color", inline = "Lbl", group = G2)
color lblTextColorInput = input.color(color.white, "Text color", group = G2)

// On last confirmed bar, draw a table to show the `open` and `close` prices, and a label to show their difference.
if barstate.islastconfirmedhistory
    //@variable The table's `position` argument based on the values of `tbVerticalInput` and `tbHorizontalInput`.
    string tbPos = switch
        tbVerticalInput == "Top"    and tbHorizontalInput == "Left"   => position.top_left
        tbVerticalInput == "Top"    and tbHorizontalInput == "Center" => position.top_center
        tbVerticalInput == "Top"    and tbHorizontalInput == "Right"  => position.top_right
        tbVerticalInput == "Middle" and tbHorizontalInput == "Left"   => position.middle_left
        tbVerticalInput == "Middle" and tbHorizontalInput == "Center" => position.middle_center
        tbVerticalInput == "Middle" and tbHorizontalInput == "Right"  => position.middle_right
        tbVerticalInput == "Bottom" and tbHorizontalInput == "Left"   => position.bottom_left
        tbVerticalInput == "Bottom" and tbHorizontalInput == "Center" => position.bottom_center
        tbVerticalInput == "Bottom" and tbHorizontalInput == "Right"  => position.bottom_right
    //@variable A table showing the last confirmed bar's `open` and `close` prices. Inputs customize the table's style.
    var table displayTable = table.new(tbPos, 2, 2, tbBackgroundInput, border_color = tbBorderInput, border_width = 1) 
    displayTable.cell(0, 0, "Open",              text_color = tbTextColorInput, text_size = tbTextSizeInput)
    displayTable.cell(1, 0, str.tostring(open),  text_color = tbTextColorInput, text_size = tbTextSizeInput)
    displayTable.cell(0, 1, "Close",             text_color = tbTextColorInput, text_size = tbTextSizeInput)
    displayTable.cell(1, 1, str.tostring(close), text_color = tbTextColorInput, text_size = tbTextSizeInput)
    //@variable The label text, containing the difference between the bar's `open` and `close` prices.
    string lblText = "Bar body = " + str.tostring(close - open)
    label.new(bar_index, high, lblText, color = lblColorInput, textcolor = lblTextColorInput, size = lblSizeInput)
```

### Limitations {#limitations-1}

There are [limits](https://www.tradingview.com/pine-script-docs/writing/limitations/#line-box-polyline-and-label-limits) to the total number of drawing visuals a script can display on the chart. A single script instance can draw a maximum of approximately 500 lines, boxes, and labels, and a maximum of 100 polylines. If the number of drawings exceeds the limit, a garbage collection mechanism deletes the oldest drawings to keep only the most recent visuals on the chart.

The `max_lines_count`, `max_boxes_count`, `max_labels_count`, and `max_polylines_count` parameters in the [indicator()](https://www.tradingview.com/pine-script-reference/v6/#fun_indicator) or [strategy()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy) declaration statement control the total number of drawings the script can display for each object type. The default value for each `max_*_count` parameter is 50, so if a script does not specify this parameter, it displays the 50 most recent drawings of each type.

Most drawing types have x and y coordinates, so drawing objects move as the user scrolls the chart or zooms in or out. The only exception is _tables_, which are anchored to one of nine fixed positions in the pane itself. See the [Tables](https://www.tradingview.com/pine-script-docs/visuals/overview/#tables) section below for more details about their unique characteristics.

The leftmost (earliest) x coordinate of a drawing object can be no more than approximately 9999 bars before or 500 bars after the bar on which the script draws it. See [this entry](https://www.tradingview.com/pine-script-docs/faq/techniques/#how-can-i-prevent-the-bar-index-value-of-the-x-argument-is-too-far-from-the-current-bar-index-try-using-time-instead-and-objects-positioned-using-xlocbar_index-cannot-be-drawn-further-than-x-bars-into-the-future-errors) in the [Techniques](https://www.tradingview.com/pine-script-docs/faq/techniques/) FAQ to learn how to work around this issue.

Unlike plots, Pine drawings do not have [external uses](https://www.tradingview.com/pine-script-docs/visuals/overview/#external-uses-exports-alerts-and-more) like creating alerts or exporting data.

## Z-index {#z-index}

All visual elements on the chart occupy a position along the z-axis, meaning that some elements appear on top of others. The _z-index_ is a value that represents the relative position of elements on the z-axis. Elements with a higher z-index appear on top of elements with a lower z-index.

Pine elements are divided into z-index groups based on their visual type. Each group has its own position in the z-space, and **within the same group**, elements created _last_ in the script’s logic appear on top of other elements from the same group.

This list orders the visual element groups by _ascending_ z-index, i.e., background colors are always at the bottom of z-space, and tables always appear on top of all other elements:

1.  Background colors
2.  Fills
3.  Plots
4.  Horizontal levels
5.  Linefills
6.  Lines
7.  Boxes
8.  Labels
9.  Tables

An element cannot be placed outside the region of z-space that its group occupies — for example, a plot can never appear on top of a table, because tables have the highest z-index. The sole exception to this rule is that programmers can choose to arrange `plot*()`, [hline()](https://www.tradingview.com/pine-script-reference/v6/#fun_hline), and [fill()](https://www.tradingview.com/pine-script-reference/v6/#fun_fill) visuals (and only these types of visuals) in z-space in the order in which they appear in the script, by using `explicit_plot_zorder = true` in [indicator()](https://www.tradingview.com/pine-script-reference/v6/#fun_indicator) or [strategy()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy) declaration statements.

## When to use {#when-to-use}

Knowing the strengths of each type of visual element, and how they compare to each other, helps programmers develop efficient scripts that look good. The sections below describe some useful features of each visual element and spotlight a few built-in use cases. For more details about a specific visual element, refer to its User Manual page.

### `plot()` {#plot}

The [plot()](https://www.tradingview.com/pine-script-reference/v6/#fun_plot) function displays a data series across the chart. A single [plot()](https://www.tradingview.com/pine-script-reference/v6/#fun_plot) visual registers one value for every bar in the main series.

Unlike [line and polyline](https://www.tradingview.com/pine-script-docs/visuals/overview/#lines-and-polylines) drawings, which connect two or more [chart points](https://www.tradingview.com/pine-script-docs/language/type-system/#chart-points) independent of the bar series, each data point in a [plot()](https://www.tradingview.com/pine-script-reference/v6/#fun_plot) series relates to a specific chart bar, and only one point can exist per bar within the same plot series. Plotted “int” and “float” series can represent a variety of [constant](https://www.tradingview.com/pine-script-docs/language/type-system/#const) values, [inputs](https://www.tradingview.com/pine-script-docs/concepts/inputs/), built-in series like [close](https://www.tradingview.com/pine-script-reference/v6/#var_close), and dynamically-calculated results like [ta.sma()](https://www.tradingview.com/pine-script-reference/v6/#fun_ta.sma).

The function offers multiple plot styles, including lines, step lines, histograms, areas, crosses, and circles (see the [\`plot()\` parameters](https://www.tradingview.com/pine-script-docs/visuals/plots/#plot-parameters) section of the [Plots](https://www.tradingview.com/pine-script-docs/visuals/plots/) page for all available `style` options). Like other [plot visuals](https://www.tradingview.com/pine-script-docs/visuals/overview/#plot-visuals), [plot()](https://www.tradingview.com/pine-script-reference/v6/#fun_plot) outputs can display numeric results in locations other than the main chart pane, such as the status line, price scale, and Data Window.

Most built-in indicators generate plots in their outputs, e.g., [RSI](https://www.tradingview.com/support/solutions/43000502338/), [EMA](https://www.tradingview.com/support/solutions/43000592270/), and [Bollinger Bands](https://www.tradingview.com/support/solutions/43000501840-bollinger-bands-bb/). Indicators can use several plot styles in the same script to display different kinds of data simultaneously, like the [MACD](https://www.tradingview.com/support/solutions/43000502344-macd-moving-average-convergence-divergence/) indicator does with its line and histogram plots:

![image](https://www.tradingview.com/pine-script-docs/_astro/Visuals-overview-When-to-use-Plot-1.CsWmkbLb_ZYYHpU.webp)

Scripts can also use [plot()](https://www.tradingview.com/pine-script-reference/v6/#fun_plot) to create horizontal [levels](https://www.tradingview.com/pine-script-docs/visuals/plots/#levels) in cases where the dedicated [hline()](https://www.tradingview.com/pine-script-reference/v6/#fun_hline) function is not suitable, for example, to display a dynamically-calculated level, or to create a [fill](https://www.tradingview.com/pine-script-docs/visuals/fills/#plot-and-hline-fills) between a horizontal line and a fluctuating series.

Unlike [\`plotshape()\` and \`plotchar()\`](https://www.tradingview.com/pine-script-docs/visuals/overview/#plotshape-and-plotchar), the [plot()](https://www.tradingview.com/pine-script-reference/v6/#fun_plot) function cannot display text and doesn’t support “bool” series. However, it can create [conditional plots](https://www.tradingview.com/pine-script-docs/visuals/plots/#plotting-conditionally) by setting the plot’s series values or colors to [na](https://www.tradingview.com/pine-script-reference/v6/#var_na) on certain bars.

### `plotshape()` and `plotchar()` {#plotshape-and-plotchar}

The [plotshape()](https://www.tradingview.com/pine-script-reference/v6/#fun_plotshape) and [plotchar()](https://www.tradingview.com/pine-script-reference/v6/#fun_plotchar) functions plot a series across the chart, like [plot()](https://www.tradingview.com/pine-script-reference/v6/#fun_plot), but using a wide range of shapes and characters.

The [plotshape()](https://www.tradingview.com/pine-script-reference/v6/#fun_plotshape) function displays specific `shape.*` styles like crosses, circles, and triangles, while [plotchar()](https://www.tradingview.com/pine-script-reference/v6/#fun_plotchar) displays any single alphanumeric or symbol Unicode character. See the table in the [\`plotshape()\`](https://www.tradingview.com/pine-script-docs/visuals/text-and-shapes/#plotshape) section of the [Text and shapes](https://www.tradingview.com/pine-script-docs/visuals/text-and-shapes/) page for all available `shape.*` styles.

Like other [plot visuals](https://www.tradingview.com/pine-script-docs/visuals/overview/#plot-visuals), these plots are connected to the main series. They produce one plot value per bar, which can also appear in the status line and Data Window. Both functions accept “int” and “float” series, like [plot()](https://www.tradingview.com/pine-script-reference/v6/#fun_plot), and additionally support “bool” series to display conditional plots.

For instance, the built-in [Moon phases](https://www.tradingview.com/support/solutions/43000599884-moon-phases/) indicator uses [plotshape()](https://www.tradingview.com/pine-script-reference/v6/#fun_plotshape) to conditionally draw circles above or below the chart bars, which represent when a new or full moon occurs:

![image](https://www.tradingview.com/pine-script-docs/_astro/Visuals-overview-When-to-use-Plotshape-and-plotchar-1.DR9HL3-c_1gHOW8.webp)

Both [plotshape()](https://www.tradingview.com/pine-script-reference/v6/#fun_plotshape) and [plotchar()](https://www.tradingview.com/pine-script-reference/v6/#fun_plotchar) have several `location` options, which can use either _relative_ or _absolute_ chart positions:

-   They can plot graphics at _absolute_ price positions, corresponding to each `series` value.
-   They can position graphics _near each bar_ in the main series, either above or below the bars.
-   They can anchor graphics to the _pane_ itself, either at the top or bottom of the pane.

The Moon Phases indicator above uses [location.abovebar](https://www.tradingview.com/pine-script-reference/v6/#const_location.abovebar) and [location.belowbar](https://www.tradingview.com/pine-script-reference/v6/#const_location.belowbar) arguments to position the circle plots _near_ each bar at an automatic, consistent distance, regardless of the bar’s price fluctuation or the [plotshape()](https://www.tradingview.com/pine-script-reference/v6/#fun_plotshape) `series` value.

Relative positioning also makes [plotchar()](https://www.tradingview.com/pine-script-reference/v6/#fun_plotchar) and [plotshape()](https://www.tradingview.com/pine-script-reference/v6/#fun_plotshape) useful for [debugging](https://www.tradingview.com/pine-script-docs/writing/debugging/#plotting-and-coloring-conditions) numeric values or conditions. These functions can plot `series` values at a different scale than the chart bars without interfering with the chart scale, unlike [plot()](https://www.tradingview.com/pine-script-reference/v6/#fun_plot) series. Hovering over a bar can verify its numeric `series` value in the status line or Data Window — these locations show `0` as the numeric result if there is no visual marker on this particular bar. The functions do not display a visual marker when the `series` value is `false` or [na](https://www.tradingview.com/pine-script-reference/v6/#var_na), and they also hide the marker for a `0` value in “int”/“float” series when using relative positioning.

For example, suppose we have a script overlayed in the main pane, and part of its logic generates an “int” series of `0` or `1` values based on some `testCondition`. Using [plotchar()](https://www.tradingview.com/pine-script-reference/v6/#fun_plotchar) with a relative `location` argument quickly verifies that the condition occurs where expected as the function plots a visual marker _only_ when the `series` value is `1`. Otherwise, plotting with the absolute `series` locations would distort the main price scale to accommodate a marker appearing on _every_ bar at the low price levels `0.00` and `1.00`:

```pine
//@variable An "int" series where the value is either `0` or `1`.
int mySeries = testCondition ? 1 : 0 

// To verify `mySeries`, plot a "!" character at the bottom of the pane only if `mySeries` is `1`.
plotchar(mySeries, "Debugging series", "!", location = location.bottom)
```

The [plotshape()](https://www.tradingview.com/pine-script-reference/v6/#fun_plotshape) and [plotchar()](https://www.tradingview.com/pine-script-reference/v6/#fun_plotchar) functions can also display text alongside their shapes. Unlike for [labels](https://www.tradingview.com/pine-script-docs/visuals/overview/#labels), the string must be of type “const”, so the value cannot be dynamic and cannot represent series: the _same_ text appears for all the points in the plot.

### `plotarrow()` {#plotarrow}

Similar to [\`plotshape()\` and \`plotchar()\`](https://www.tradingview.com/pine-script-docs/visuals/overview/#plotshape-and-plotchar), the [plotarrow()](https://www.tradingview.com/pine-script-reference/v6/#fun_plotarrow) function plots a series across the chart that presents graphic information using an arrow shape.

A single [plotarrow()](https://www.tradingview.com/pine-script-reference/v6/#fun_plotarrow) call plots an arrow on every bar, setting each arrow’s direction, position, and length based on the bar’s value in the plot `series`. Like other [plot visuals](https://www.tradingview.com/pine-script-docs/visuals/overview/#plot-visuals), an arrow’s numeric value can also display in the script’s status line and Data Window.

The [plotarrow()](https://www.tradingview.com/pine-script-reference/v6/#fun_plotarrow) function is useful for visualizing changes in the directionality and magnitude of “int” or “float” series values across the chart. The underlying `series` can be at a different scale than the chart bars without visually distorting the main chart scale.

Unlike [plotchar()](https://www.tradingview.com/pine-script-reference/v6/#fun_plotchar) or [plotshape()](https://www.tradingview.com/pine-script-reference/v6/#fun_plotshape), the [plotarrow()](https://www.tradingview.com/pine-script-reference/v6/#fun_plotarrow) function cannot display text and doesn’t accept “bool” series. However, the function can still achieve a conditional arrow plot by using [na](https://www.tradingview.com/pine-script-reference/v6/#var_na) values for its `series` on certain bars.

This simple example indicator uses [plotarrow()](https://www.tradingview.com/pine-script-reference/v6/#fun_plotarrow) to visualize a `barGap` series, where each arrow represents the price difference between the current bar’s [open](https://www.tradingview.com/pine-script-reference/v6/#var_open) and the previous bar’s [close](https://www.tradingview.com/pine-script-reference/v6/#var_close). The function call automatically sets the locations of all the arrows, plotting positive-value arrows below bars and negative-value arrows above bars, and adjusts their lengths relative to the other values in the `barGap` series:

![image](https://www.tradingview.com/pine-script-docs/_astro/Visuals-overview-When-to-use-Plotarrow-1.Cwp6FkbG_HyaqR.webp)

```pine
//@version=6
indicator("`plotarrow()` demo", overlay = true)

//@variable The difference between the current `open` and previous `close`.
float barGap = open - close[1]
plotarrow(barGap, "Bar gap", color.rgb(0, 187, 212, 40), color.rgb(223, 64, 251, 40))
```

### `plotbar()` and `plotcandle()` {#plotbar-and-plotcandle}

The [plotbar()](https://www.tradingview.com/pine-script-reference/v6/#fun_plotbar) and [plotcandle()](https://www.tradingview.com/pine-script-reference/v6/#fun_plotcandle) functions create custom bar or candle sets on the chart. One call to either function registers four values — the bar or candle’s `open`, `high`, `low`, and `close` values — on every bar of the main chart series. As a result, a single [plotbar()](https://www.tradingview.com/pine-script-reference/v6/#fun_plotbar) or [plotcandle()](https://www.tradingview.com/pine-script-reference/v6/#fun_plotcandle) call generates _at least four_ plots counting towards a script’s total [plot limit](https://www.tradingview.com/pine-script-docs/writing/limitations/#plot-limits).

Indicators can use these functions to plot a new series separate from the main series, or to build new visuals for the main series itself, like the built-in [Bollinger Bars](https://www.tradingview.com/support/solutions/43000742575-bollinger-bars/) indicator does to create candles with thicker wicks:

![image](https://www.tradingview.com/pine-script-docs/_astro/Visuals-overview-When-to-use-Plotbar-and-plotcandle-1.C-euBaEJ_Z178dzn.webp)

As with other [plot visuals](https://www.tradingview.com/pine-script-docs/visuals/overview/#plot-visuals), the [plotbar()](https://www.tradingview.com/pine-script-reference/v6/#fun_plotbar) and [plotcandle()](https://www.tradingview.com/pine-script-reference/v6/#fun_plotcandle) outputs can display in other locations: their numeric results in the script’s status line and Data Window (four values per plot) and their latest `close` value on the price scale.

See the [Bar plotting](https://www.tradingview.com/pine-script-docs/visuals/bar-plotting/) page for more information about these functions.

### Horizontal levels {#horizontal-levels}

The [hline()](https://www.tradingview.com/pine-script-reference/v6/#fun_hline) function creates a [horizontal level](https://www.tradingview.com/pine-script-docs/visuals/levels/) across the script pane at a defined price. The horizontal level extends fully across the visible space of the chart in both directions.

Unlike other [plot visuals](https://www.tradingview.com/pine-script-docs/visuals/overview/#plot-visuals), a horizontal level’s only output is the line drawn in the script pane; it does not display values in the status line, price scale, or Data Window.

This visual element is useful for displaying minimum or maximum prices, thresholds, or support and resistance levels. Many built-in indicators like [RSI](https://www.tradingview.com/support/solutions/43000502338-relative-strength-index-rsi/), [CCI](https://www.tradingview.com/support/solutions/43000502001-commodity-channel-index-cci/), and [Stochastic](https://www.tradingview.com/support/solutions/43000502332-stochastic-stoch/) use horizontal levels to represent fixed boundaries for oscillator plots. For example, in the RSI indicator, the horizontal levels are upper and lower bands that represent the oversold and overbought boundaries:

![image](https://www.tradingview.com/pine-script-docs/_astro/Visuals-overview-When-to-use-Horizontal-levels-1.aUvsy9vC_2lD2gU.webp)

Some built-in indicators also use horizontal levels with [fills](https://www.tradingview.com/pine-script-docs/visuals/fills/#plot-and-hline-fills) to create colored bands, which can help to visually distinguish the typical value ranges from outlier ranges, as seen above.

A horizontal level uses a _single, fixed_ price value, so it cannot use a dynamically-calculated value or a “series” type like [close](https://www.tradingview.com/pine-script-reference/v6/#var_close). Instead, scripts can use [plot()](https://www.tradingview.com/pine-script-reference/v6/#fun_plot) to produce similar horizontal lines for dynamically-calculated [levels](https://www.tradingview.com/pine-script-docs/visuals/plots/#levels).

Because an [hline()](https://www.tradingview.com/pine-script-reference/v6/#fun_hline) call plots only a fixed level in a single color, it is often more performant than similar [plot()](https://www.tradingview.com/pine-script-reference/v6/#fun_plot) lines. Adding a horizontal level does not count towards a script’s [plot limit](https://www.tradingview.com/pine-script-docs/writing/limitations/#plot-limits) because the [hline()](https://www.tradingview.com/pine-script-reference/v6/#fun_hline) function doesn’t create a plot series internally or externally to generate its visual output.

### Background and bar coloring {#background-and-bar-coloring}

The [bgcolor()](https://www.tradingview.com/pine-script-reference/v6/#fun_bgcolor) function sets the [background color](https://www.tradingview.com/pine-script-docs/visuals/backgrounds/) of the chart space behind a bar, while the [barcolor()](https://www.tradingview.com/pine-script-reference/v6/#fun_barcolor) function sets the [body color](https://www.tradingview.com/pine-script-docs/visuals/bar-coloring/) of a candle.

The functions accept both [constant colors](https://www.tradingview.com/pine-script-docs/visuals/colors/#constant-colors) and dynamically-calculated colors, so they can use [conditional coloring](https://www.tradingview.com/pine-script-docs/visuals/colors/#conditional-coloring) for bars or backgrounds. For instance, the built-in [Moon Phases](https://www.tradingview.com/support/solutions/43000599884-moon-phases/) indicator uses [bgcolor()](https://www.tradingview.com/pine-script-reference/v6/#fun_bgcolor) to conditionally set the background color of the bars to highlight waxing and waning moon phases:

![image](https://www.tradingview.com/pine-script-docs/_astro/Visuals-overview-When-to-use-Plotshape-and-plotchar-1.DR9HL3-c_1gHOW8.webp)

A [bgcolor()](https://www.tradingview.com/pine-script-reference/v6/#fun_bgcolor) call, like most visuals, affects the _script pane_ by default. It sets the background color behind the main bar series only when it’s overlayed in the main pane — when `overlay = true` for the script or `force_overlay = true` for [bgcolor()](https://www.tradingview.com/pine-script-reference/v6/#fun_bgcolor) — otherwise it sets the background for the equivalent space in a separate pane.

By contrast, the [barcolor()](https://www.tradingview.com/pine-script-reference/v6/#fun_barcolor) function **always** colors the main bar series in the _main_ pane, even when called by a script executing in a separate pane.

As [barcolor()](https://www.tradingview.com/pine-script-reference/v6/#fun_barcolor) only affects the main chart series, scripts cannot use it to alter the colors of new bars or candles created using [plotbar()](https://www.tradingview.com/pine-script-reference/v6/#fun_plotbar) or [plotcandle()](https://www.tradingview.com/pine-script-reference/v6/#fun_plotcandle).

This simple example uses arbitrary [bar\_index](https://www.tradingview.com/pine-script-reference/v6/#var_bar_index) and price conditions to set conditional background and bar colors:

![image](https://www.tradingview.com/pine-script-docs/_astro/Visuals-overview-When-to-use-Background-and-bar-coloring-2.DoJ1F0LQ_1aWetg.webp)

```pine
//@version=6
indicator("`bgcolor()` and `barcolor()` demo")

// Plot a new candle series for this script, separate to the main pane. Candles are set to main series OHLC values.
plotcandle(open, high, low, close, color = color.silver)

// Set the background color of the script pane. Color is set conditionally depending on divisibility of `bar_index` by 10 or 6.
bgcolor(bar_index % 10 == 0 ? color.new(color.purple, 60) : bar_index % 6 == 0 ? color.new(color.teal, 60) : na)   

// Set the bar color for the main series. Although script executes in a separate pane, this call executes on the main pane.
// Bar's body color is set conditionally to highlight bars with price move of 10 or more.
barcolor(math.abs(close - open) >= 10  ? color.orange : color.white)
```

Note that:

-   The script executes in a separate pane, but the [barcolor()](https://www.tradingview.com/pine-script-reference/v6/#fun_barcolor) function colors the main series.
-   The [barcolor()](https://www.tradingview.com/pine-script-reference/v6/#fun_barcolor) call does _not_ affect the new candles plotted in the script pane.

### Fills {#fills}

Scripts can use [fills](https://www.tradingview.com/pine-script-docs/visuals/fills/#plot-and-hline-fills) to set the background color of the space between a pair of [plots](https://www.tradingview.com/pine-script-docs/visuals/plots/) or [horizontal levels](https://www.tradingview.com/pine-script-docs/visuals/levels/). The [fill()](https://www.tradingview.com/pine-script-reference/v6/#fun_fill) function accepts both constant and dynamically-calculated colors. There is also a [fill()](https://www.tradingview.com/pine-script-reference/v6/#fun_fill-2) function overload that can create color gradient fills.

Fills between plots are commonly used in built-in indicators to visualize calculated channels or bands, like those used in the [Bollinger Bands](https://www.tradingview.com/support/solutions/43000501840-bollinger-bands-bb/) indicator, which signify the upper and lower standard deviations from its [SMA](https://www.tradingview.com/support/solutions/43000696841/) line:

![image](https://www.tradingview.com/pine-script-docs/_astro/Visuals-overview-When-to-use-Fills-1.I46aGwse_Z14jft6.webp)

Fills between horizontal levels are often used in built-in oscillators to highlight chart regions of interest or to differentiate between typical and outlier ranges. For example, the [Stochastic Momentum Index (SMI)](https://www.tradingview.com/support/solutions/43000707882-stochastic-momentum-index-smi/) indicator fills the background between horizontal levels that signify overbought and oversold boundaries, which can help easily identify signs of bullish or bearish trends beyond the filled regions:

![image](https://www.tradingview.com/pine-script-docs/_astro/Visuals-overview-When-to-use-Fills-2.Cz5iMt-W_Z1jIw4V.webp)

The SMI indicator also uses the [fill()](https://www.tradingview.com/pine-script-reference/v6/#fun_fill-2) function’s color gradient overload to gradually color the space within the plot lines green or red as they enter the overbought or oversold zones respectively.

Other Pine visuals have their own dedicated fills, like [linefills](https://www.tradingview.com/pine-script-docs/visuals/fills/#line-fills) for setting the fill color between two [lines](https://www.tradingview.com/pine-script-docs/visuals/lines-and-boxes/#lines), and built-in fill color arguments for drawing objects like [boxes](https://www.tradingview.com/pine-script-docs/visuals/lines-and-boxes/#boxes) and [polylines](https://www.tradingview.com/pine-script-docs/visuals/lines-and-boxes/#polylines). See the [Fills](https://www.tradingview.com/pine-script-docs/visuals/fills/) page for more information about the different fill mechanisms available.

NoteA single [fill()](https://www.tradingview.com/pine-script-reference/v6/#fun_fill) function call cannot color the space between a plot and a horizontal level. It can only color the region between either a pair of [plots](https://www.tradingview.com/pine-script-docs/visuals/plots/) or a pair of [horizontal levels](https://www.tradingview.com/pine-script-docs/visuals/levels/).

### Lines and polylines {#lines-and-polylines}

Scripts can draw [lines](https://www.tradingview.com/pine-script-docs/visuals/lines-and-boxes/#lines) to visually connect any two points on the chart horizontally, vertically, or diagonally.

Like other [drawing visuals](https://www.tradingview.com/pine-script-docs/visuals/overview/#drawing-visuals), lines are independent from the main series, so scripts can draw them at any available chart locations from any bar.

Programmers can specify a line’s start and end coordinates using any of the following:

-   A [bar\_index](https://www.tradingview.com/pine-script-reference/v6/#var_bar_index) x-coordinate and price y-coordinate.
-   A [UNIX timestamp](https://www.tradingview.com/pine-script-docs/concepts/time/#unix-timestamps) x-coordinate and price y-coordinate.
-   A [chart point](https://www.tradingview.com/pine-script-docs/language/type-system/#chart-points) object, where the x-coordinate is a bar index or time value.

Lines can also extend to the left or right of the chart, like those used in the built-in [Auto Fib Extension](https://www.tradingview.com/support/solutions/43000612397-auto-fib-extension/) indicator to visualize projected price levels:

![image](https://www.tradingview.com/pine-script-docs/_astro/Visuals-overview-When-to-use-Lines-and-polylines-1.DZGk_5wC_Z1h8Rjh.webp)

Scripts can specify line coordinates at dynamic offsets from the bars on which they’re calculated, to draw lines at varying lengths and distances. For instance, the built-in [Zig Zag](https://www.tradingview.com/support/solutions/43000591664-zig-zag/) indicator draws straight, angled lines to connect calculated high and low pivots alternatingly across the chart, connecting the last leg to the last available bar. The indicator confirms a point as a high/low pivot only when the price reverses by a specified percentage over time. Therefore, it always draws its lines _into the past_ from a different bar than that of the pivot point, and the number of bars between two sequential pivots is not predictable or consistent:

![image](https://www.tradingview.com/pine-script-docs/_astro/Visuals-overview-When-to-use-Lines-and-polylines-2.MNTPcVWF_ZiC3DH.webp)

While a [line](https://www.tradingview.com/pine-script-reference/v6/#type_line) object can connect only two points with a straight line, a [polyline](https://www.tradingview.com/pine-script-docs/visuals/lines-and-boxes/#polylines) can connect _multiple_ points on the chart consecutively to create a straight or _curved_ line drawing. A polyline uses an [array](https://www.tradingview.com/pine-script-docs/language/arrays/) of [chart points](https://www.tradingview.com/pine-script-docs/language/type-system/#chart-points) to set the coordinates of its sequential line segments, which can contain up to 10,000 chart points.

Polylines can create more complex graphic formations than lines or [boxes](https://www.tradingview.com/pine-script-docs/visuals/lines-and-boxes/#boxes). A script can connect chart points together with [closed polylines](https://www.tradingview.com/pine-script-docs/visuals/lines-and-boxes/#closed-shapes) to draw polygons, or leave them open-ended to draw geometric series across the chart. Scripts can also use open-ended, curved polylines to draw chart patterns like the [Cup and Handle](https://www.tradingview.com/support/solutions/43000732556-chart-pattern-cup-and-handle/) pattern, which identifies a U-shape price trend that is difficult to produce with other drawing visuals:

![image](https://www.tradingview.com/pine-script-docs/_astro/Visuals-overview-When-to-use-Lines-and-polylines-3.DGxUi3hc_Zmhowz.webp)

A script can replicate the visuals made by drawing several sequential [line](https://www.tradingview.com/pine-script-reference/v6/#type_line) objects with just one [polyline](https://www.tradingview.com/pine-script-reference/v6/#type_polyline) object instead. Using polylines can thus help a script to stay under the [limits](https://www.tradingview.com/pine-script-docs/writing/limitations/#line-box-polyline-and-label-limits) for the total number of lines.

For example, we can use a simplified version of the [Zig Zag](https://www.tradingview.com/support/solutions/43000591664-zig-zag/) indicator’s logic to illustrate this. Here, we use one polyline drawing to connect pivot points across the chart. The script stores the high and low pivots together in one [chart.point](https://www.tradingview.com/pine-script-reference/v6/#type_chart.point) array, and creates the [polyline](https://www.tradingview.com/pine-script-reference/v6/#type_polyline) object only on the last confirmed historical bar, using [barstate.islastconfirmedhistory](https://www.tradingview.com/pine-script-reference/v6/#var_barstate.islastconfirmedhistory), drawing it retrospectively across the chart:

![image](https://www.tradingview.com/pine-script-docs/_astro/Visuals-overview-When-to-use-Lines-and-polylines-4.CwILzGqz_iEBT5.webp)

```pine
//@version=6
indicator("Polyline drawing demo", overlay = true)

//@variable The left and right strength of the pivot.
int pivotLegsInput = input.int(5, "Pivot leg length", minval = 1)
//@variable Switches the `polyline` drawing to a straight or curved line drawing.
bool isCurvedPolyline = input.bool(false, "Use curved polyline")

//@variable A persistent array that stores high and low pivots for the polyline.
var array<chart.point> pointsArray = array.new<chart.point>()
// Calculate the high and low pivot prices using `ta.pivot*()` functions.
float pivotHigh = ta.pivothigh(pivotLegsInput, pivotLegsInput)
float pivotLow = ta.pivotlow(pivotLegsInput, pivotLegsInput)
// Add all high and low pivot points sequentially to `pointsArray`, and draw labels at pivots to show prices.
if not na(pivotHigh)
    chart.point highPoint = chart.point.from_index(bar_index - pivotLegsInput, pivotHigh)
    pointsArray.push(highPoint)
    label.new(highPoint, "Pivot: " + str.tostring(pivotHigh, "##.##"))
if not na(pivotLow)
    chart.point lowPoint = chart.point.from_index(bar_index - pivotLegsInput, pivotLow)
    pointsArray.push(lowPoint)
    label.new(lowPoint, "Pivot: " + str.tostring(pivotLow, "##.##"), style = label.style_label_up)

// On the last confirmed bar, draw a polyline across the chart to connect all pivots in `pointsArray`. 
if barstate.islastconfirmedhistory
    // First, remove chart points that are too far from the current bar, to prevent errors.
    // Iterate backwards to avoid index shifting issues when removing items.
    for i = (pointsArray.size() - 1) to 0
        chart.point point = pointsArray.get(i)
        if (bar_index - point.index) > 9999
            pointsArray.remove(i)

    polyline.new(pointsArray, curved = isCurvedPolyline, line_color = color.purple, line_width = 4)
    // For reference, display the total number of polyline drawings created by the script in a table cell on the chart.
    table displayTable = table.new(position.bottom_right, 1, 1, color.purple)
    displayTable.cell(0, 0, "Total polyline drawings: " + str.tostring(array.size(polyline.all)), 
         text_color = color.white, text_size = size.large) 
```

Note that:

-   To avoid [runtime errors](https://www.tradingview.com/pine-script-docs/faq/techniques/#how-can-i-prevent-the-bar-index-value-of-the-x-argument-is-too-far-from-the-current-bar-index-try-using-time-instead-and-objects-positioned-using-xlocbar_index-cannot-be-drawn-further-than-x-bars-into-the-future-errors) due to the polyline trying to draw points more than approximately 9999 bars back from the current bar, one alternative is to use [chart.point.from\_time()](https://www.tradingview.com/pine-script-reference/v6/#fun_chart.point.from_time) to set x-coordinates with [UNIX timestamps](https://www.tradingview.com/pine-script-docs/concepts/time/#unix-timestamps). Here, we instead use a [loop](https://www.tradingview.com/pine-script-docs/language/loops/) to remove [chart.point](https://www.tradingview.com/pine-script-reference/v6/#type_chart.point) objects that are too far from the current bar, before drawing the polyline. Note that to accurately remove more than one element from an array using a loop, scripts must iterate _backwards_ through the array.
-   A polyline’s `curved` parameter accepts a “series” argument, so scripts can use [Boolean inputs](https://www.tradingview.com/pine-script-docs/concepts/inputs/#boolean-input) like `isCurvedPolyline` in our example to easily switch between straight or curved line drawings from an indicator’s settings.

Scripts can fill the closed space of a polyline drawing using the [polyline.new()](https://www.tradingview.com/pine-script-reference/v6/#fun_polyline.new) function’s `fill_color` parameter. To fill the space between two [lines](https://www.tradingview.com/pine-script-docs/visuals/lines-and-boxes/#lines) with a specified color, use linefill objects, which are described in the next section.

### Linefills {#linefills}

A [linefill](https://www.tradingview.com/pine-script-docs/visuals/fills/#line-fills) is a [drawing object](https://www.tradingview.com/pine-script-docs/language/type-system/#drawing-types), unlike the [fills](https://www.tradingview.com/pine-script-docs/visuals/overview/#fills) for plots and horizontal levels. Calling the [linefill.new()](https://www.tradingview.com/pine-script-reference/v6/#fun_linefill.new) function instantiates an _object_ of type “linefill”. Scripts can store linefill objects and manipulate them with functions, e.g., to set the associated fill color or retrieve the pair of lines.

Similar to plot fills, linefills are useful for highlighting regions of interest, like calculated channels or trend zones, between two lines on the chart. For example, the built-in [Linear Regression](https://www.tradingview.com/support/solutions/43000644936-linear-regression/) indicator uses two linefills between its baseline and its support and resistance lines, which signify the expected price movement ranges. Highlighting the upper and lower channels can make it easier to visually register the price reversal signals:

![image](https://www.tradingview.com/pine-script-docs/_astro/Visuals-overview-When-to-use-Linefills-1.C7iSPLcw_ZzK6ud.webp)

The exact dimensions occupied by a linefill object are defined by the pair of lines it’s attached to. Moving one line farther away, for example, automatically widens the attached linefill. Only one linefill instance can exist between a pair of lines, and it covers only the common space between them. If a pair of lines both extend in the same direction, the linefill can also extend infinitely, as seen in the [Auto Pitchfork](https://www.tradingview.com/support/solutions/43000657911-auto-pitchfork/) indicator:

![image](https://www.tradingview.com/pine-script-docs/_astro/Visuals-overview-When-to-use-Linefills-2.nLJWt_Bh_289aq8.webp)

Linefills can fill the space only between two “line” objects. For [polylines](https://www.tradingview.com/pine-script-docs/visuals/lines-and-boxes/#polylines), the [polyline.new()](https://www.tradingview.com/pine-script-reference/v6/#fun_polyline.new) function has a `fill_color` parameter to fill the polyline drawing’s closed space.

### Boxes {#boxes}

Scripts can use [boxes](https://www.tradingview.com/pine-script-docs/visuals/lines-and-boxes/#boxes) to create custom rectangle drawings on the chart. Like other [drawing visuals](https://www.tradingview.com/pine-script-docs/visuals/overview/#drawing-visuals), a box is a flexible object type, not a series visual, so a script can draw multiple boxes on the same bar, and can set box coordinates at any allowed chart locations ahead or behind the current bar.

Programmers can specify box coordinates using either two diagonal corner points or all four edges of the box, and can define the x-coordinates using [bar\_index](https://www.tradingview.com/pine-script-reference/v6/#var_bar_index) or [UNIX timestamp](https://www.tradingview.com/pine-script-docs/concepts/time/#unix-timestamps) values.

Boxes can be useful for highlighting chart areas of interest, showing price ranges, or visually grouping bars. For example, the built-in [Multi-time period charts](https://www.tradingview.com/support/solutions/43000502591-multi-time-period-charts/) indicator overlays boxes on the current chart to visualize the corresponding higher timeframe candles:

![image](https://www.tradingview.com/pine-script-docs/_astro/Visuals-overview-When-to-use-Boxes-1.Bbk02t8y_ZxcVET.webp)

Boxes can also display text as part of their drawings, as shown in the [Seasonality](https://www.tradingview.com/support/solutions/43000723025-seasonality/) indicator below. Scripts can customize a box’s [text formatting](https://www.tradingview.com/pine-script-docs/visuals/text-and-shapes/#text-formatting), alignment, and wrapping, with auto-scaling and auto-wrapping options available to design boxes that are responsive to a user’s chart adjustments:

![image](https://www.tradingview.com/pine-script-docs/_astro/Visuals-overview-When-to-use-Boxes-2.BmeKcoBW_ZdIzH3.webp)

### Labels {#labels}

[Labels](https://www.tradingview.com/pine-script-docs/visuals/text-and-shapes/#labels) are drawing objects that can display dynamic text on the chart. They accept “series string” arguments, so they can use changeable text values that aren’t known at the start of execution, like [inputs](https://www.tradingview.com/pine-script-docs/concepts/inputs/) or conditionally-calculated expressions, unlike the text displayed by [\`plotshape()\` and \`plotchar()\`](https://www.tradingview.com/pine-script-docs/visuals/overview/#plotshape-and-plotchar).

Scripts can manage labels in local scopes and draw them at historical or future positions, like other [drawing visuals](https://www.tradingview.com/pine-script-docs/visuals/overview/#drawing-visuals). Each label’s position is anchored to the chart’s x and y scales at a specific price and bar/time value. However, this position is flexible, as a script can modify a label’s coordinates any number of times.

In the built-in [Zig Zag](https://www.tradingview.com/support/solutions/43000591664-zig-zag/) indicator, text labels display the calculated pivot prices and, depending on the selected inputs, can also display the reversal price and cumulative volume data within these same labels. The indicator takes advantage of several dynamic label features when building the concatenated label text and setting each label’s high/low position, color, and variable offset:

![image](https://www.tradingview.com/pine-script-docs/_astro/Visuals-overview-When-to-use-Labels-1.BeK-v0k1_Z190Bhu.webp)

Many `label.style_*` options are available to customize a label’s visual appearance, including standard pointing labels and shape-based labels like crosses, triangles, arrows, or flags. The indicator above uses the [label.style\_none](https://www.tradingview.com/pine-script-reference/v6/#const_label.style_none) style to display the text on the chart without a visible label shape or outline. See the table in the [positioning labels](https://www.tradingview.com/pine-script-docs/visuals/text-and-shapes/#positioning-labels) section of the [Text and shapes](https://www.tradingview.com/pine-script-docs/visuals/text-and-shapes/) page for all available label styles.

The versatility of labels also makes them particularly useful for [debugging](https://www.tradingview.com/pine-script-docs/writing/debugging/#labels) scripts. A label can easily show calculated numeric values, strings, or test conditions directly on the chart with little extra code. Scripts can even display empty labels without text to create quick visual markers, for example, to verify that conditions occur on their expected bars.

### Tables {#tables}

[Tables](https://www.tradingview.com/pine-script-docs/visuals/tables/) are special drawing objects useful for displaying customized, organized information that isn’t connected to the chart’s price or bar scales.

Tables are anchored to the _pane space_ itself, not to any x or y chart coordinates. As such, they remain fixed in size and position when zooming into or scrolling across the chart, even if they are overlayed in the main pane. Like other [drawing visuals](https://www.tradingview.com/pine-script-docs/visuals/overview/#drawing-visuals), tables do not change the data they display when the user hovers over different bars.

Scripts can draw tables in one of _nine_ fixed pane positions, specified by the `top`, `middle`, or `bottom` vertical region of the pane and the corresponding `left`, `center`, or `right` horizontal region:

![image](https://www.tradingview.com/pine-script-docs/_astro/Visuals-overview-When-to-use-Tables-1.DB-j0ufk_Z1sUXM4.webp)

If a script displays more than one table in the same location, the table that is drawn latest in the code replaces any previous tables.

Similar to other drawings, tables have various features that scripts can modify during execution using setter functions. These include _table-specific_ features like the frame, border, and height/width in the pane, as well as _cell-specific_ features like background color, alignment, and [text formatting](https://www.tradingview.com/pine-script-docs/visuals/text-and-shapes/#text-formatting).

A customization feature unique to tables is that, within the same table object, _each cell_ can have different visual properties.

For example, the built-in [Performance](https://www.tradingview.com/support/solutions/43000736064-performance/) indicator shows the price percentage change at multiple timeframes for a group of symbols. It uses a variable color intensity for the cell background colors to represent each value’s absolute strength. The tabular format and dynamic cell colors make it easy to compare values across symbols and timeframes at a glance:

![image](https://www.tradingview.com/pine-script-docs/_astro/Visuals-overview-When-to-use-Tables-2.BJSAv3Vi_Z1fxEpW.webp)

Unlike for [lines](https://www.tradingview.com/pine-script-docs/visuals/lines-and-boxes/#lines), [boxes](https://www.tradingview.com/pine-script-docs/visuals/lines-and-boxes/#boxes), and [labels](https://www.tradingview.com/pine-script-docs/visuals/text-and-shapes/#labels), scripts **cannot** use getter functions to retrieve properties for tables drawn on the chart. To refer to an attribute of a table later in a script, first store the value in a separate variable.

The Performance indicator above draws its table only _once_ during initial execution, on the last bar. This improves script performance and is recommended because a table only displays its _last state_. Tables are thus useful for displaying annotations or general information that won’t change during execution, like selected settings, release notes, misconfigurations, etc.

The following example script displays labels for the start and end of each daily trading session. As such, it supports only intraday data and does not display any labels on a “1D” timeframe or higher. The script displays a single-cell table if [timeframe.isdwm](https://www.tradingview.com/pine-script-reference/v6/#var_timeframe.isdwm) is true, to notify users of this information:

![image](https://www.tradingview.com/pine-script-docs/_astro/Visuals-overview-When-to-use-Tables-3.d9t_lRT9_2bPKck.webp)

```pine
//@version=6
indicator("Timeframe warning table demo", overlay = true, max_labels_count = 500, behind_chart = false) 

// Input for the trading session (e.g., "0930-1600" for US stocks regular hours)
sessionInput = input.session("0930-1600", "Trading Session")

// Display `warningTable` on last bar if the chart timeframe is not intraday.
if barstate.islastconfirmedhistory and timeframe.isdwm
    //@variable A single-cell `table` that inform users about the unsupported timeframe.
    var table warningTable = table.new(position.middle_center, 1, 1, color.yellow)
    warningTable.cell(0, 0, 
         "Warning: This indicator supports only intraday timeframes.\n Switch to a lower timeframe to see output.", 
         text_size = size.large)

// Plot a label at the opening price when the session starts
if timeframe.isintraday and na(time("", sessionInput)[1]) and not na(time("", sessionInput))
    label.new(bar_index, open, "Session Open: " + str.tostring(open, "#.##"), yloc = yloc.abovebar, 
      color = color.green, style = label.style_label_down)

// Plot a label at the closing price on the last bar of the session
if timeframe.isintraday and not na(time("", sessionInput)[1]) and na(time("", sessionInput))
    label.new(bar_index[1], close[1], "Session Close: " + str.tostring(close[1], "#.##"), yloc = yloc.belowbar, 
      color = color.red, style = label.style_label_up)
```

Note that:

-   Using a table in this case ensures that users clearly see the warning, because it appears directly in the chart pane regardless of how their chart is scaled.

Lastly, a table’s organized format and fixed pane positions also makes it useful for debugging scripts. See the [Tables](https://www.tradingview.com/pine-script-docs/writing/debugging/#tables) section of the [Debugging](https://www.tradingview.com/pine-script-docs/writing/debugging/) page for more details.
