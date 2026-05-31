---
title: Declaration statements
source: https://www.tradingview.com/pine-script-docs/language/declaration-statements/
section: language
---

# Declaration statements {#declaration-statements}

## Introduction {#introduction}

In Pine Script®, a _declaration statement_ is a mandatory function call that declares the script’s _type_ and its _properties_ at _compile time_. The available declaration functions are [indicator()](https://www.tradingview.com/pine-script-reference/v6/#fun_indicator), [strategy()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy), and [library()](https://www.tradingview.com/pine-script-reference/v6/#fun_library). Each type of script has different capabilities and behaviors, the compiler uses different rules to compile them, and Pine’s runtime system also [executes](https://www.tradingview.com/pine-script-docs/language/execution-model/) them differently.

Every script must include exactly **one** declaration statement, and that statement must be in the script’s [global scope](https://www.tradingview.com/pine-script-docs/faq/programming/#what-does-scope-mean). Our [style guide](https://www.tradingview.com/pine-script-docs/writing/style-guide/#script-organization) recommends placing the statement directly below the `@version=` [compiler annotation](https://www.tradingview.com/pine-script-docs/language/script-structure/#compiler-annotations) at the top of the source code. For example:

```pine
//@version=6
indicator("My script") // Declares that the script is an indicator named "My script" with default properties.

// Plot the `close` series across the chart.
plot(close)
```

The parameters of a declaration statement define various script-wide properties and default behaviors. Only the `title` parameter, which sets the script’s _main title_, requires an argument. Supplying arguments to any other parameters is _optional_. Note that all parameters in each declaration statement require arguments qualified as “const”. They _cannot_ accept values with the “input”, “simple”, or “series” [type qualifier](https://www.tradingview.com/pine-script-docs/language/type-system/#qualifiers).

The [`indicator()`](https://www.tradingview.com/pine-script-docs/language/declaration-statements/#indicator), [`strategy()`](https://www.tradingview.com/pine-script-docs/language/declaration-statements/#strategy), and [`library()`](https://www.tradingview.com/pine-script-docs/language/declaration-statements/#library) sections below explain the parameters available for each declaration statement and how they affect a script, as well as various unique characteristics of each script type.

TipThe Pine Editor displays the [Reference Manual](https://www.tradingview.com/pine-script-reference/v6/) documentation for a declaration function and its parameters in a pop-up window as the user types the statement. To view the complete Reference Manual entry from inside the editor, press the CTRL or CMD key and click the [indicator()](https://www.tradingview.com/pine-script-reference/v6/#fun_indicator), [strategy()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy), or [library()](https://www.tradingview.com/pine-script-reference/v6/#fun_library) identifier.

## `indicator()` {#indicator}

The [indicator()](https://www.tradingview.com/pine-script-reference/v6/#fun_indicator) function declares that the script is an _indicator_. Indicators perform calculations across a dataset to generate [visuals](https://www.tradingview.com/pine-script-docs/visuals/overview/), [alerts](https://www.tradingview.com/pine-script-docs/concepts/alerts/), or [Pine Logs](https://www.tradingview.com/pine-script-docs/writing/debugging/#pine-logs). They are the most common type of scripts in Pine.

The built-in [Relative Strength Index (RSI)](https://www.tradingview.com/support/solutions/43000502338/) script is an example of an indicator. It calculates the RSI of a specified source series and plots the result in a separate pane. It can also plot a smoothed RSI, display divergence signals, and generate divergence alerts.

Indicators have several distinct characteristics, including the following:

-   Indicators are the _only_ scripts that can use alert triggers from calls to both the [alert()](https://www.tradingview.com/pine-script-reference/v6/#fun_alert) and [alertcondition()](https://www.tradingview.com/pine-script-reference/v6/#fun_alertcondition) functions.
-   Unlike [strategies](https://www.tradingview.com/pine-script-docs/concepts/strategies/), indicators _cannot_ use any `strategy.*` built-ins or simulate trades.
-   Unlike [libraries](https://www.tradingview.com/pine-script-docs/concepts/libraries/), indicators cannot _export_ code for use in other scripts. However, other scripts that include [source inputs](https://www.tradingview.com/pine-script-docs/concepts/inputs/#source-input) can retrieve values from an indicator’s _plots_ created by [plot()](https://www.tradingview.com/pine-script-reference/v6/#fun_plot) calls.
-   The [Pine Screener](https://www.tradingview.com/support/solutions/43000742436-tradingview-pine-screener-key-features-and-requirements/) is compatible with indicators only. The screener can display plotted values from an indicator’s [plot()](https://www.tradingview.com/pine-script-reference/v6/#fun_plot) function calls, and _filter_ the results using data from other [plot()](https://www.tradingview.com/pine-script-reference/v6/#fun_plot) or [alertcondition()](https://www.tradingview.com/pine-script-reference/v6/#fun_alertcondition) calls.
-   Indicators always execute _once per bar_ on historical bars, and _once per data feed update (tick)_ on [realtime bars](https://www.tradingview.com/pine-script-docs/language/execution-model/#realtime-bars).
-   Indicators must include _at least one_ call to a function that creates one of the following outputs: [plot visuals](https://www.tradingview.com/pine-script-docs/visuals/overview/#plot-visuals), [drawing visuals](https://www.tradingview.com/pine-script-docs/visuals/overview/#drawing-visuals), [alert triggers](https://www.tradingview.com/pine-script-docs/concepts/alerts/), or [Pine Logs](https://www.tradingview.com/pine-script-docs/writing/debugging/#pine-logs).

The signature for the [indicator()](https://www.tradingview.com/pine-script-reference/v6/#fun_indicator) function is as follows:

```
indicator(title, shorttitle, overlay, format, precision, scale, max_bars_back, timeframe, timeframe_gaps, explicit_plot_zorder, max_lines_count, max_labels_count, max_boxes_count, calc_bars_count, max_polylines_count, dynamic_requests, behind_chart) → void
```

The following sections explain the parameters of the [indicator()](https://www.tradingview.com/pine-script-reference/v6/#fun_indicator) declaration statement and how they work.

NoteAll the parameters described below, excluding [`timeframe` and `timeframe_gaps`](https://www.tradingview.com/pine-script-docs/language/declaration-statements/#timeframe-and-timeframe_gaps), also apply to the [strategy()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy) function.

### `title` and `shorttitle` {#title-and-shorttitle}

The required `title` parameter defines the script’s _main title_. The script displays the specified “string” title in all possible chart locations by default. Additionally, the “Publish script” window automatically suggests using that title for a [script publication](https://www.tradingview.com/pine-script-docs/writing/publishing/).

NoteIf the `title` argument is an empty string, the script uses “Study” as its main title on the chart, and the “Publish script” window does not suggest a publication title.

The optional `shorttitle` parameter defines a _short display title_ for the script. If the declaration statement includes a `shorttitle` argument that is not an empty string, the string’s text appears instead of the main title in multiple chart locations, including:

-   The script’s status line on the chart.
-   The chart’s object tree and data window.
-   The script’s “Settings” window.
-   The “Condition” section of the “Create alert” dialog box.
-   The listed alerts and logs for the script in the “Alerts” pane.
-   The [Pine Logs](https://www.tradingview.com/pine-script-docs/writing/debugging/#pine-logs) pane.

NoteThese parameters are **not related** to the name that the user assigns when saving or renaming a script project using the menu options in the Pine Editor. The text assigned from those options defines the name that the user searches in their _personal library_ to open the script in the editor, add the script to the chart, or run the script in the [Pine Screener](https://www.tradingview.com/support/solutions/43000742436-tradingview-pine-screener-key-features-and-requirements/).

The example script below plots an [Exponential Moving Average (EMA)](https://www.tradingview.com/support/solutions/43000592270-exponential-moving-average/) for a selected source series and length. The declaration statement sets the script’s main title to `"Exponential Moving Average indicator"`. However, because the declaration statement also includes the argument `shorttitle = "EMA"`, the script’s status line and the data window display “EMA” instead of the main title. Hovering over the short title in the status line reveals a tooltip containing the script’s main title:

![image](https://www.tradingview.com/pine-script-docs/_astro/Declaration-statements-Indicator-Title-and-shorttitle-1.Dbfz2H-5_1Wl4t7.webp)

```pine
//@version=6
indicator("Exponential Moving Average indicator", shorttitle = "EMA")

//@variable The source series for which to calculate the EMA.
float sourceInput = input.source(ohlc4, "Source")
//@variable The length value for the EMA's smoothing factor.
int lengthInput = input.int(20, "Length", minval = 1)

// Calculate the EMA for the specified series and length, and plot the result as a color-coded line.
float ema = ta.ema(sourceInput, lengthInput)
plot(ema, "EMA", ema > ema[1] ? color.green : color.red, 3)
```

TipThe `shorttitle` parameter is intended for creating an _abbreviated_ display title for a script. As such, the compiler raises a warning if the specified string contains more characters than the recommended limit. If you encounter the warning, we recommend reducing the string’s length. If you want the script to display a longer title, specify that title in the `title` argument and _remove_ the `shorttitle` argument.

### `overlay`, `scale`, and `behind_chart` {#overlay-scale-and-behind_chart}

The `overlay`, `scale`, and `behind_chart` parameters of the declaration statement configure where the script displays its chart outputs. They control the _global default_ display location and scaling behavior of the script’s [visuals](https://www.tradingview.com/pine-script-docs/visuals/overview/), separate to the individual properties of [plot visuals](https://www.tradingview.com/pine-script-docs/visuals/overview/#plot-visuals) or [drawing visuals](https://www.tradingview.com/pine-script-docs/visuals/overview/#drawing-visuals).

NoteThese parameters affect a script’s visuals only **once**, when the user first adds the script to their chart. If an instance of the script is already on the chart, changing the `overlay`, `scale`, or `behind_chart` arguments in the declaration statement **does not** affect that instance’s display location or scale. Programmers must _re-add_ the script to the chart to view the results of such changes.

The `overlay` parameter specifies which _default_ chart pane the script uses to display its visuals when the user adds the script to their chart. If the argument is `true`, the script’s visuals appear in the _main chart pane_ by default, or in another script’s pane if the user adds it to the chart via the “Add indicator/strategy on” option in the other script’s “More” menu. If the `overlay` argument is `false` (default), the script’s visuals occupy a _separate chart pane_ by default.

The `scale` parameter defines the location of the script’s _price scale_ and the scaling behavior of the script’s plots and drawings. The possible arguments are [scale.left](https://www.tradingview.com/pine-script-reference/v6/#const_scale.left), [scale.right](https://www.tradingview.com/pine-script-reference/v6/#const_scale.right), and [scale.none](https://www.tradingview.com/pine-script-reference/v6/#const_scale.none). The behaviors associated with this parameter are as follows:

-   If the declaration statement includes _any_ `scale` argument, the script scales its visuals _independently_ to fit the vertical range of the pane that it occupies.
-   If the argument is [scale.left](https://www.tradingview.com/pine-script-reference/v6/#const_scale.left) or [scale.right](https://www.tradingview.com/pine-script-reference/v6/#const_scale.right), and the script overlays on an existing pane, the script adds a _separate_ scale for its visuals on the specified side of that pane.
-   If the script occupies a separate pane, an argument of [scale.left](https://www.tradingview.com/pine-script-reference/v6/#const_scale.left) or [scale.right](https://www.tradingview.com/pine-script-reference/v6/#const_scale.right) moves that pane’s scale to the specified side without creating a new scale.
-   If the argument is [scale.none](https://www.tradingview.com/pine-script-reference/v6/#const_scale.none), which is valid only if the `overlay` argument is `true`, the script displays plotted numbers directly on the scale of an existing pane without creating a new scale. If the user moves the script to a separate pane, the script displays values on a new price scale in that pane.
-   If the statement does not include a `scale` argument, the script uses the main price scale for the pane it occupies, and it does _not_ scale its visuals separately if it overlays on an existing pane.

The following example indicator plots an [RSI](https://www.tradingview.com/support/solutions/43000502338-relative-strength-index-rsi/) as translucent, color-coded columns. The script displays the columns on the main chart pane because its declaration statement includes `overlay = true`. Additionally, the script adds a separate scale to the left side of the pane and scales its plotted values independently because the statement uses [scale.left](https://www.tradingview.com/pine-script-reference/v6/#const_scale.left) as the `scale` argument:

![image](https://www.tradingview.com/pine-script-docs/_astro/Declaration-statements-Indicator-Overlay-scale-and-behind-chart-1.BAAozNT0_Z2s50eG.webp)

```pine
//@version=6
indicator("`scale` demo", overlay = true, scale = scale.left, format = format.percent)

//@variable The RSI of the `close` series with a length of 14.
float rsi = ta.rsi(close, 14)

// Plot the RSI as translucent columns. Use an orange color for values of 50 or greater, and blue for others.
plot(rsi, "RSI", rsi <= 50 ? color.new(color.orange, 70) : color.new(color.blue, 75), 2, plot.style_columns)
```

Note that:

-   The script formats the plotted numbers and the values in the left-side scale as _percentages_ because the [indicator()](https://www.tradingview.com/pine-script-reference/v6/#fun_indicator) statement includes the argument `format = format.percent`. See the [`format` and `precision`](https://www.tradingview.com/pine-script-docs/language/declaration-statements/#format-and-precision) section below to learn more.

The `behind_chart` parameter determines the _visual order_ of the script’s plots and drawings relative to the main chart series. Specifying an argument for this parameter affects the script’s visuals only if the `overlay` argument is `true`, because the behavior does not apply to non-overlay scripts. If the `behind_chart` value is `true` (default), the script’s visuals appear _behind_ the main series. If the value is `false`, they appear _in front_ of the main series and can cover the chart’s bars.

NoteThe functions that create plots, background colors, and drawings include a `force_overlay` parameter. If a call to these functions specifies `true` as the `force_overlay` argument, its resulting visuals always appear over the _main_ chart pane and use that pane’s scale, regardless of the `overlay`, `behind_chart`, and `scale` arguments in the script’s declaration statement. See the [`overlay`](https://www.tradingview.com/pine-script-docs/visuals/overview/#overlay) section of the [Visuals overview](https://www.tradingview.com/pine-script-docs/visuals/overview/) page to learn more about this feature.

### `format` and `precision` {#format-and-precision}

The `format` and `precision` parameters of the declaration statement control the default appearance of _plotted numbers_ in the script’s status line, the price scales, and the data window.

The `precision` parameter determines the default number of _fractional digits_ that the script shows for plotted values and the numbers in the price scale. It accepts a value from 0 to 16. This parameter affects the appearance of all plotted numbers except for those formatted using [format.volume](https://www.tradingview.com/pine-script-reference/v6/#const_format.volume), because the decimal precision rules of the built-in volume format supersede other precision settings. If the declaration statement does not include a `precision` argument, the script inherits its default precision settings from the main chart series, or from another script if it accesses one of that script’s plots using a [source input](https://www.tradingview.com/pine-script-docs/concepts/inputs/#source-inputs).

The `format` parameter determines whether the script displays plotted numbers and the numbers in the price scale using a price, percentage, or volume format, or if it inherits formatting settings from the chart or another script. The possible arguments are [format.price](https://www.tradingview.com/pine-script-reference/v6/#const_format.price), [format.percent](https://www.tradingview.com/pine-script-reference/v6/#const_format.percent), [format.volume](https://www.tradingview.com/pine-script-reference/v6/#const_format.volume) and [format.inherit](https://www.tradingview.com/pine-script-reference/v6/#const_format.inherit). The default is [format.inherit](https://www.tradingview.com/pine-script-reference/v6/#const_format.inherit).

Below, we list how a script formats plotted values when using each of these arguments:

`format.price`

The script formats plotted values as whole numbers with two fractional digits by default. For example, a script that uses this argument and default precision settings formats a plot value of 122 as 122.0, and a value of 122.355 as 122.36. If a rounded value is greater than or equal to 1000, the script uses a comma as the thousands separator. For instance, it formats a value of 14489245 as 14,489,245.00. If a number is extremely large, the script formats it as a rounded value in [E notation](https://en.wikipedia.org/wiki/Scientific_notation#E_notation) (e.g., `1e+21`).

`format.percent`

The script applies similar formatting rules to those defined by [format.price](https://www.tradingview.com/pine-script-reference/v6/#const_format.price), and it appends a percent sign (`%`) to express plotted values as percentages. By default, the format rounds plotted numbers to two fractional digits. For example, it formats the value 39.787 as 39.79%. This format does _not_ recalculate values to express them as percentages. To represent a _ratio_ as a percentage when using this format, multiply the value by 100 before plotting it.

`format.volume`

The script formats plotted numbers as _abbreviated_ values that follow special precision rules. If a rounded value is greater than or equal to 1000, the script includes a letter representing a multiplied quantity: “K” for thousand, “M” for million, “B” for billion, or “T” for trillion. For example, it formats a plot value of 2474 as 2.74K, and a value of 14489245 as 14.49M. If a value is extremely large, the script displays a number with commas or E notation followed by “T”. For values less than 1000, the script displays those values rounded to the nearest whole number by default. Note that these formatting rules can apply to any plotted numbers; they are not limited to only volume values.

`format.inherit`

The script inherits the same formatting settings as those defined for the main chart series, or the global formatting settings for another script if it accesses one of the script’s plots using a [source input](https://www.tradingview.com/pine-script-docs/concepts/inputs/#source-input). For example, the script uses price formatting when applied to a stock chart series, and percentage formatting when applied to a bond chart series.

NoteIf the declaration statement uses [format.inherit](https://www.tradingview.com/pine-script-reference/v6/#const_format.inherit) as the `format` argument, changing the script’s _precision_ settings via the `precision` parameter or the “Precision” field in the script’s “Settings/Style” tab causes it to _ignore_ the inherited format and instead use [format.price](https://www.tradingview.com/pine-script-reference/v6/#const_format.price) settings with the specified precision, even if the inherited format uses [format.volume](https://www.tradingview.com/pine-script-reference/v6/#const_format.volume) rules.

The example indicator below plots [volume](https://www.tradingview.com/pine-script-reference/v6/#var_volume) values as color-coded columns, and it plots the average value over a specified number of bars as a line. The [indicator()](https://www.tradingview.com/pine-script-reference/v6/#fun_indicator) declaration statement includes [format.volume](https://www.tradingview.com/pine-script-reference/v6/#const_format.volume) as the `format` argument to apply the volume formatting rules described above to the script’s plots and scale. On our daily “NASDAQ:NFLX” chart, the current plotted values are in _millions_, so the script displays the numbers in an abbreviated format with “M” as the suffix:

![image](https://www.tradingview.com/pine-script-docs/_astro/Declaration-statements-Indicator-Format-and-precision-1.Be7Ya9Oi_ZUNfgG.webp)

```pine
//@version=6
indicator("`format.volume` demo", format = format.volume)

//@variable The number of bars over which to calculate the average volume.
int lengthInput = input.int(14, "Average length", minval = 1)

//@variable A red color if `close < open`, and green otherwise. The color is 50% transparent if `volume <= volume[1]`.
color volColor = switch
    volume > volume[1] => close < open ? color.red : color.green
    => close < open ? color.new(color.red, 50) : color.new(color.green, 50)

// Plot volume as columns, and the average volume as a line.
// Both of these plots automatically format numbers using `format.volume` rules.
plot(volume, "Volume", volColor, style = plot.style_columns)
plot(ta.sma(volume, lengthInput), "Avg volume", color.blue, linewidth = 2)
```

Note that the `plot*()` functions also include `format` and `precision` parameters, which enable scripts to define specific formatting behaviors for each separate plot. By default, a plot automatically inherits the default format and precision settings defined by the declaration statement, as demonstrated by the previous example. However, if a `plot*()` call includes `format` or `precision` arguments, those arguments _take precedence_ over the script’s default settings.

For example, in the script version below, we added the argument `format = format.price` to the [plot()](https://www.tradingview.com/pine-script-reference/v6/#fun_plot) call for the average volume display. With this change, the script formats the average volume values using the rules defined by [format.price](https://www.tradingview.com/pine-script-reference/v6/#const_format.price), while the volume plot and the price scale both continue to use the default [format.volume](https://www.tradingview.com/pine-script-reference/v6/#const_format.volume) rules specified by the declaration statement:

![image](https://www.tradingview.com/pine-script-docs/_astro/Declaration-statements-Indicator-Format-and-precision-2.CMMFQdQU_Z14pcEW.webp)

```pine
//@version=6
indicator("`format.volume` demo", format = format.volume)

//@variable The number of bars over which to calculate the average volume.
int lengthInput = input.int(14, "Average length", minval = 1)

//@variable A red color if `close < open`, and green otherwise. The color is 50% transparent if `volume <= volume[1]`.
color volColor = switch
    volume > volume[1] => close < open ? color.red : color.green
    => close < open ? color.new(color.red, 50) : color.new(color.green, 50)

// Because this call does not include a `format` argument, it inherits `format.volume` rules.
plot(volume, "Volume", volColor, style = plot.style_columns)
// By contrast, this call uses `format.price` rules, because the `format` argument here
// *overrides* the script's default plot format.
plot(ta.sma(volume, lengthInput), "Avg volume", color.blue, linewidth = 2, format = format.price)
```

### `max_bars_back` {#max_bars_back}

The `max_bars_back` parameter of the declaration statement, if it has a specified argument, sets the initial maximum _history-referencing length_ for each series in a script. It accepts an “int” value from 0 to 5000, representing the number of past data points maintained in memory for _all_ variables and expressions.

As a script executes, Pine’s runtime system stores data for each variable and expression across bars in fixed-length [historical buffers](https://www.tradingview.com/pine-script-docs/language/execution-model/#historical-buffers). The script can access _past bar data_ from these buffers by using the [`[]` history-referencing operator](https://www.tradingview.com/pine-script-docs/language/operators/#-history-referencing-operator) or the [built-in functions](https://www.tradingview.com/pine-script-docs/language/built-ins/#built-in-functions) that reference history internally. For example, the expression `close[10]` retrieves the last saved value of the [close](https://www.tradingview.com/pine-script-reference/v6/#var_close) variable from _10 bars back_.

By default, the system automatically sizes each historical buffer by analyzing the historical references that the script executes as it loads across historical bars. For resource efficiency, each buffer typically contains only enough past data to accommodate the script’s historical references, but _not more_. For instance, if a script requests the value of a variable from up to 500 bars back as it loads across a chart’s history, the buffer for that variable typically includes data for only the latest 500 past bars.

In most cases, this automatic sizing process accommodates a script’s historical references without issues. Therefore, manually setting the sizes of historical buffers is often **unnecessary**. However, in some cases, the system might fail to determine appropriate buffer sizes on its own, resulting in a [runtime error](https://www.tradingview.com/pine-script-docs/errors/RE10143/#the-requested-historical-offset-x-is-beyond-the-historical-buffers-limit-y). One possible way to resolve that error is to set the default size of the script’s historical buffers in advance by including a `max_bars_back` argument in the declaration statement.

Before using this parameter, ensure that _all_ or _most_ of the series in the script actually require historical buffers with the _same_ specific size. Manually setting buffers in a script to use a specific size when unnecessary can **negatively** impact the script’s performance. If only _specific_ series require manually sized buffers, either of the following approaches is far more efficient:

-   Use the [max\_bars\_back()](https://www.tradingview.com/pine-script-reference/v6/#fun_max_bars_back) _function_ to set the sizes of only the problematic historical buffers.
-   Structure the script’s history-referencing operations to request the _maximum_ required amount of history on the _first bar_.

See the [historical buffer limit](https://www.tradingview.com/pine-script-docs/errors/RE10143/#the-requested-historical-offset-x-is-beyond-the-historical-buffers-limit-y) error page to learn more. For _advanced_ details about the workings of historical buffers, refer to the [Historical buffers](https://www.tradingview.com/pine-script-docs/language/execution-model/#historical-buffers) section of the [Execution model](https://www.tradingview.com/pine-script-docs/language/execution-model/) page.

### `timeframe` and `timeframe_gaps` {#timeframe-and-timeframe_gaps}

The `timeframe` parameter of the [indicator()](https://www.tradingview.com/pine-script-reference/v6/#fun_indicator) declaration statement sets the script’s _main timeframe_. It enables the script to perform calculations on the data for a different timeframe than that of the chart without requiring `request.*()` function calls. The parameter accepts a valid [timeframe string](https://www.tradingview.com/pine-script-docs/concepts/timeframes/#timeframe-string-specifications), such as `"1D"` for the daily timeframe or `"30"` for the 30-minute timeframe. If an argument is not specified, or if the value is an empty string (`""`), the script executes on the data for the current chart’s timeframe.

TipScripts can retrieve a string representing the main timeframe specified in the declaration statement, even while executing other data requests with `request.*()` calls, by using the [timeframe.main\_period](https://www.tradingview.com/pine-script-reference/v6/#var_timeframe.main_period) variable.

The `timeframe_gaps` parameter determines how the script handles _time gaps_ when plotting data from a _higher timeframe_. It allows an argument only if the declaration statement also includes a `timeframe` argument. The parameter works similarly to the [`gaps`](https://www.tradingview.com/pine-script-docs/concepts/other-timeframes-and-data/#gaps) parameter of [request.security()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.security) and other `request.*()` functions. If the value is `true` (default), the script plots values only on the chart bars where new, _confirmed_ data is available from the specified timeframe, and displays [na](https://www.tradingview.com/pine-script-reference/v6/#var_na) results on other bars. If `false`, the script plots the _last retrieved values_ from the higher timeframe on the chart bars where new data is not available.

NoteThe [indicator()](https://www.tradingview.com/pine-script-reference/v6/#fun_indicator) declaration statement can include arguments for these parameters only if the script _does not_ use [drawing objects](https://www.tradingview.com/pine-script-docs/language/type-system/#drawing-types) or [alert()](https://www.tradingview.com/pine-script-reference/v6/#fun_alert) function calls, because scripts cannot evaluate outputs from such code on other datasets.

If the declaration statement specifies a `timeframe` argument, the script automatically adds a “Calculation” group with a “Timeframe” input to the “Settings/Inputs” tab. If the statement includes a `timeframe_gaps` argument, the script also adds a “Wait for timeframe closes” input below the “Timeframe” input. These inputs enable users to customize the script’s main timeframe and its gap-handling behavior without modifying the source code. To learn more about them, see the [Leveraging multi-timeframe analysis](https://www.tradingview.com/support/solutions/43000591555-leveraging-multi-timeframe-analysis/) article in our Help Center.

The following example demonstrates the behavior of both parameters. The indicator below calculates and plots the 14-bar average of [close](https://www.tradingview.com/pine-script-reference/v6/#var_close) values on a specified timeframe. The [indicator()](https://www.tradingview.com/pine-script-reference/v6/#fun_indicator) declaration statement includes `"1D"` as the `timeframe` argument, so it performs calculations using daily data for the current chart’s symbol by default, regardless of the chart’s timeframe. On an intraday chart, the script plots an “x-cross” shape only on the _last_ chart bar for each trading day by default, and [na](https://www.tradingview.com/pine-script-reference/v6/#var_na) on other bars, because the declaration statement also includes the argument `timeframe_gaps = true`:

![image](https://www.tradingview.com/pine-script-docs/_astro/Declaration-statements-Indicator-Timeframe-and-timeframe-gaps-1.C2MffQOi_ZrCG0M.webp)

```pine
//@version=6
indicator(
    "`timeframe` and `timeframe_gaps` demo",
    overlay = true, behind_chart = false,
    timeframe = "1D", timeframe_gaps = true // <- These arguments automatically add inputs to the "Settings/Inputs" tab.
)

//@variable The 14-bar average `close` value on the script's main timeframe ("1D" by default).
float avgClose = ta.sma(close, 14)

// Plot the `avgClose` series as "x-cross" shapes.
// By default, a new shape appears only on the last chart bar for each "1D" period. On other bars, the plot shows `na`.
plotshape(avgClose, "Avg close", shape.xcross, location.absolute, size = size.small)
```

Note that:

-   If the specified timeframe is _lower_ than or _equal_ to the chart’s timeframe, the script plots an “x-cross” shape on _every_ chart bar.
-   An alternative way to achieve this script’s default result without using these parameters is to plot the value returned by the call `request.security(syminfo.tickerid, "1D", ta.sma(close, 14), gaps = barmerge.gaps_on)`. Refer to the [Other timeframes and data](https://www.tradingview.com/pine-script-docs/concepts/other-timeframes-and-data/) page to learn more about `request.*()` functions.

### `explicit_plot_zorder` {#explicit_plot_zorder}

The `explicit_plot_zorder` parameter of the declaration statement determines the _visual order_ in which the script’s [plots](https://www.tradingview.com/pine-script-docs/visuals/plots/), [horizontal levels](https://www.tradingview.com/pine-script-docs/visuals/levels/), and [fills](https://www.tradingview.com/pine-script-docs/visuals/fills/) _stack_ on the chart.

If the value is `true`, the script visually stacks plots, levels, and fills based on the order of the `plot()*`, [hline()](https://www.tradingview.com/pine-script-reference/v6/#fun_hline), and [fill()](https://www.tradingview.com/pine-script-reference/v6/#fun_fill) function calls in the source code, where each written call’s output appears _on top_ of the outputs from the calls that _precede_ it. For example, if the code lists a [fill()](https://www.tradingview.com/pine-script-reference/v6/#fun_fill) call after a [plot()](https://www.tradingview.com/pine-script-reference/v6/#fun_plot) call, the resulting fill appears on top of the plot. Likewise, if the code lists a [plot()](https://www.tradingview.com/pine-script-reference/v6/#fun_plot) call after an [hline()](https://www.tradingview.com/pine-script-reference/v6/#fun_hline) call, the plot appears on top of the horizontal line.

If the value is `false` (default), the script visually stacks its plots, levels, and fills based on the order of those visuals in the [z-index](https://www.tradingview.com/pine-script-docs/visuals/overview/#z-index), regardless of the order in which the function calls for each type of output occur in the code. Horizontal levels always appear on top of plots, and plots always appear on top of fills. However, visual outputs of the _same_ type or group still stack on top of each other based on the order of their function calls. For example, if a script includes two calls to the [plot()](https://www.tradingview.com/pine-script-reference/v6/#fun_plot) function, the _second_ plot appears on top of the first.

NoteThis parameter **does not** affect visuals created by the [bgcolor()](https://www.tradingview.com/pine-script-reference/v6/#fun_bgcolor) function or [drawing objects](https://www.tradingview.com/pine-script-docs/language/type-system/#drawing-types). Background colors and drawings _always_ stack on the chart in the order of the _z-index_, regardless of the `explicit_plot_zorder` argument in the script’s declaration statement.

### `max_lines_count`, `max_labels_count`, `max_boxes_count`, and `max_polylines_count` {#max_lines_count-max_labels_count-max_boxes_count-and-max_polylines_count}

The `max_lines_count`, `max_labels_count`, `max_boxes_count`, and `max_polylines_count` parameters of the declaration statement limit the number of [line](https://www.tradingview.com/pine-script-reference/v6/#type_line), [label](https://www.tradingview.com/pine-script-reference/v6/#type_label), [box](https://www.tradingview.com/pine-script-reference/v6/#type_box), and [polyline](https://www.tradingview.com/pine-script-reference/v6/#type_polyline) drawing objects that the script can maintain in memory. As the script creates new objects of a given [drawing type](https://www.tradingview.com/pine-script-docs/language/type-system/#drawing-types), the runtime system _deletes_ the _oldest_ drawings of that type as necessary if the number of active objects exceeds the script’s limit.

The `max_lines_count`, `max_labels_count`, and `max_boxes_count` parameters accept an “int” value from 1 to 500, and the `max_polylines_count` parameter accepts an “int” value from 1 to 100. The default for each parameter is 50.

NoteThe limits defined by these parameters are approximate. The maximum number of active drawings can vary slightly across bars. Programmers can precisely limit the number of active drawings by using the `*.delete()` functions (e.g., [line.delete()](https://www.tradingview.com/pine-script-reference/v6/#fun_line.delete)) on the elements of the built-in `*.all` array for each drawing type (e.g., [line.all](https://www.tradingview.com/pine-script-reference/v6/#var_line.all)).

See the [Line, box, polyline, and label limits](https://www.tradingview.com/pine-script-docs/writing/limitations/#line-box-polyline-and-label-limits) section of the [Limitations](https://www.tradingview.com/pine-script-docs/writing/limitations/) page and the [Total number of objects](https://www.tradingview.com/pine-script-docs/visuals/lines-and-boxes/#total-number-of-objects) section of the [Lines and boxes](https://www.tradingview.com/pine-script-docs/visuals/lines-and-boxes/) page to learn more about drawing limits.

### `calc_bars_count` {#calc_bars_count}

The `calc_bars_count` parameter of the declaration statement sets the default _maximum_ number of most recent _historical bars_ that the script can access for its calculations. It accepts an “int” value that is greater than or equal to 0.

If the value is 0 (default), the script executes on _all_ the available bars in the dataset, starting from the first available bar. If the value is greater than 0, the script instead starts executions on the bar that is N bars before the _latest_ available bar at loading time, or on the dataset’s first bar if the value exceeds the number of available bars. Additionally, a positive `calc_bars_count` argument adds a “Calculation” group with a _“Calculated bars”_ input to the script’s “Settings/Inputs” tab, where users can adjust the number of historical bars available to the script without editing the source code.

The following example script plots the [close](https://www.tradingview.com/pine-script-reference/v6/#var_close) series across a limited number of historical bars and all realtime bars. The [indicator()](https://www.tradingview.com/pine-script-reference/v6/#fun_indicator) declaration statement includes the argument `calc_bars_count = 40`, which forces the script to treat the last 40 historical bars as the _only_ ones available in the dataset by default:

![image](https://www.tradingview.com/pine-script-docs/_astro/Declaration-statements-Indicator-Calc-bars-count-1.B4UsxCwI_Z22woEa.webp)

```pine
//@version=6

// The `calc_bars_count` argument in this declaration statement specifies that the script can use
// only the last 40 historical bars for its calculations by default. It also adds a "Calculated bars"
// input to the script's "Settings/Inputs" tab.
indicator("`calc_bars_count` demo", calc_bars_count = 40)

// Plot the `close` series on the specified number of recent historical bars and all realtime bars.
plot(close, "Close", linewidth = 2)
```

NoteLimiting the bars on which a script can execute with the `calc_bars_count` parameter also limits the data points available for [history-referencing](https://www.tradingview.com/pine-script-docs/language/operators/#-history-referencing-operator) operations and bar indexing. A script treats the first bar on which it _executes_ as the _earliest_ bar in the dataset, with a [bar\_index](https://www.tradingview.com/pine-script-reference/v6/#var_bar_index) value of 0, even if earlier bars are otherwise available from the data feed. Therefore, [\[\]](https://www.tradingview.com/pine-script-reference/v6/#op_[]) operations _cannot_ retrieve data from bars before the first bar determined by the `calc_bars_count` argument.

### `dynamic_requests` {#dynamic_requests}

The `dynamic_requests` parameter of the declaration statement specifies whether the script can use `request.*()` function calls to execute [dynamic requests](https://www.tradingview.com/pine-script-docs/concepts/other-timeframes-and-data/#dynamic-requests). If the value is `true` (default), the script can:

-   Include calls to `request.*()` functions inside the [local scopes](https://www.tradingview.com/pine-script-docs/concepts/other-timeframes-and-data/#in-local-scopes) of [conditional structures](https://www.tradingview.com/pine-script-docs/language/conditional-structures/) and [loops](https://www.tradingview.com/pine-script-docs/language/loops/), and in the operands of conditional expressions.
-   Use [“series” arguments](https://www.tradingview.com/pine-script-docs/concepts/other-timeframes-and-data/#series-arguments) that vary across bars to specify the ticker identifier, timeframe, and other settings of a `request.*()` call.
-   Execute [nested requests](https://www.tradingview.com/pine-script-docs/concepts/other-timeframes-and-data/#nested-requests), where one `request.*()` call evaluates another inside its context.

If the value is `false`, the script is more limited in how it can use `request.*()` functions:

-   All `request.*()` calls must execute in the script’s global scope, and outside the conditional operands of [ternary](https://www.tradingview.com/pine-script-docs/language/operators/#-ternary-operator) or [and](https://www.tradingview.com/pine-script-reference/v6/#kw_and)/[or](https://www.tradingview.com/pine-script-reference/v6/#kw_or) operations.
-   All `request.*()` parameters except for `expression` require arguments with the “simple” [type qualifier](https://www.tradingview.com/pine-script-docs/language/type-system/#qualifiers) or a weaker qualifier, meaning their values _cannot change_ across bars.
-   A `request.*()` call whose `expression` argument depends on another `request.*()` call _cannot_ evaluate the other call within its context.

The following example script calculates a [weighted moving average (WMA)](https://www.tradingview.com/support/solutions/43000594680-weighted-moving-average/) of [hl2](https://www.tradingview.com/pine-script-reference/v6/#var_hl2) values over a specified number of chart bars. It also uses a [request.security()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.security) call within an [if](https://www.tradingview.com/pine-script-reference/v6/#kw_if) structure to optionally calculate the latest confirmed WMA on a specified higher timeframe. The script can use the request inside the [if](https://www.tradingview.com/pine-script-reference/v6/#kw_if) structure because the [indicator()](https://www.tradingview.com/pine-script-reference/v6/#fun_indicator) declaration statement’s `dynamic_requests` argument is `true`:

![image](https://www.tradingview.com/pine-script-docs/_astro/Declaration-statements-Indicator-Dynamic-requests-1.CIUMbear_Z1e9J0S.webp)

```pine
//@version=6
indicator("Conditional dynamic requests demo", overlay = true, behind_chart = false, dynamic_requests = true)

//@variable The number of bars to use in the WMA calculation.
int lengthInput = input.int(5, "WMA length", minval = 1)
//@variable Specifies whether to retrieve a higher-timeframe WMA.
bool htfRequestInput = input.bool(true, "Show higher-timeframe WMA")
//@variable A higher-timeframe string for the data request.
string timeframeInput = input.timeframe("1W", "Higher timeframe", active = htfRequestInput)

//@variable The weighted moving average of `hl2` values over the specified length.
float chartWMA = ta.wma(hl2, lengthInput)

//@variable The WMA calculated on the higher timeframe if the `htfRequestInput` value is `true`, and `na` otherwise.
float requestedWMA = na

if htfRequestInput
    // Raise an error if the specified timeframe is *not* higher than the chart's timeframe.
    if timeframe.in_seconds(timeframeInput) <= timeframe.in_seconds(timeframe.period)
        runtime.error("The requested timeframe must be higher than the chart's timeframe.")

    // Execute the `request.security()` call for the higher-timeframe request.
    // The call works in this `if` structure because the declaration statement enables dynamic requests.
    // If we change the `dynamic_requests` argument to `false`, this call causes a *compilation error*.
    requestedWMA := request.security(
        syminfo.tickerid, timeframeInput, ta.wma(hl2, lengthInput)[1], lookahead = barmerge.lookahead_on
    )

// Plot the chart WMA and the optional HTF WMA.
plot(chartWMA,     "Chart WMA", color.teal,   4)
plot(requestedWMA, "HTF WMA",   color.purple, 4)
```

Note that:

-   The script behaves the same if we remove the `dynamic_requests` argument from the declaration statement, because the default argument is `true`.
-   If we change the `dynamic_requests` argument to `false`, the [request.security()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.security) call causes a _compilation error_ because the script cannot use it inside the [if](https://www.tradingview.com/pine-script-reference/v6/#kw_if) statement. To resolve the error without enabling dynamic requests, programmers must move the call to the _global scope_.
-   Users can change the “Higher timeframe” input in the “Settings/Inputs” tab only if they select the “Show higher-timeframe WMA” checkbox, because the [input.timeframe()](https://www.tradingview.com/pine-script-reference/v6/#fun_input.timeframe) call includes the argument `active = htfRequestInput` to control when the input is _active_. See the [Input function parameters](https://www.tradingview.com/pine-script-docs/concepts/inputs/#input-function-parameters) section of the [Inputs](https://www.tradingview.com/pine-script-docs/concepts/inputs/) page to learn more about `active` and other input parameters.

To learn more about the `request.*()` functions and the differences between dynamic and non-dynamic requests, refer to the [Other timeframes and data](https://www.tradingview.com/pine-script-docs/concepts/other-timeframes-and-data/) page.

## `strategy()` {#strategy}

The [strategy()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy) function declares that the script is a _strategy_. [Strategies](https://www.tradingview.com/pine-script-docs/concepts/strategies/) can simulate orders and trades across a dataset, enabling users to backtest and forward test their trading systems. They have many similar capabilities to indicators, while also providing the ability to analyze hypothetical trading performance in a dedicated tab.

The built-in [RSI Strategy](https://www.tradingview.com/support/solutions/43000645066-rsi-strategy/) script is an example of a simple strategy. The script simulates entering and exiting positions based on the RSI crossing the defined overbought and oversold levels. It displays trade markers directly on the chart and shows a detailed strategy report in a separate panel below the chart.

Scripts declared as strategies have several unique characteristics, including the following:

-   Strategies are the only scripts that can send [orders](https://www.tradingview.com/pine-script-docs/concepts/strategies/#orders-and-trades) to the [broker emulator](https://www.tradingview.com/pine-script-docs/concepts/strategies/#broker-emulator) and display simulated performance results using the [Strategy Tester](https://www.tradingview.com/pine-script-docs/concepts/strategies/#strategy-tester).
-   The “Settings” window for strategy scripts features a unique “Properties” tab, where users can customize the [properties](https://www.tradingview.com/support/solutions/43000628599-strategy-properties/) of the strategy simulation. Programmers can specify _default_ properties for this tab via the unique parameters in the [strategy()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy) statement.
-   Unlike indicators, strategies cannot run on data for other timeframes. They always use the same timeframe as the chart.
-   Strategies _cannot_ create alert triggers using the [alertcondition()](https://www.tradingview.com/pine-script-reference/v6/#fun_alertcondition) function, but they can create them by using calls to the [alert()](https://www.tradingview.com/pine-script-reference/v6/#fun_alert) function. Additionally, unlike indicators, they can generate special alerts from [order fill events](https://www.tradingview.com/pine-script-docs/concepts/alerts/#order-fill-events).
-   Unlike the plots created by indicators or [libraries](https://www.tradingview.com/pine-script-docs/concepts/libraries), strategy plots are _not_ accessible to [source inputs](https://www.tradingview.com/pine-script-docs/concepts/inputs/#source-input) in other scripts.
-   Strategies execute differently from indicators or libraries. By default, they execute strictly _once per closed bar_ and do _not_ execute on open bars. However, users can customize a strategy’s [calculation behavior](https://www.tradingview.com/pine-script-docs/concepts/strategies/#altering-calculation-behavior) to enable additional executions on open bars or after the broker emulator fills an order.
-   Strategies must include at least one call to an [order placement command](https://www.tradingview.com/pine-script-docs/concepts/strategies/#order-placement-and-cancellation), or to a function that creates [plot visuals](https://www.tradingview.com/pine-script-docs/visuals/overview/#plot-visuals), [drawing visuals](https://www.tradingview.com/pine-script-docs/visuals/overview/#drawing-visuals), [alert triggers](https://www.tradingview.com/pine-script-docs/concepts/alerts/), or [Pine Logs](https://www.tradingview.com/pine-script-docs/writing/debugging/#pine-logs).

The [strategy()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy) function has the following signature:

```
strategy(title, shorttitle, overlay, format, precision, scale, pyramiding, calc_on_order_fills, calc_on_every_tick, max_bars_back, backtest_fill_limits_assumption, default_qty_type, default_qty_value, initial_capital, currency, slippage, commission_type, commission_value, process_orders_on_close, close_entries_rule, margin_long, margin_short, explicit_plot_zorder, max_lines_count, max_labels_count, max_boxes_count, calc_bars_count, risk_free_rate, use_bar_magnifier, fill_orders_on_standard_ohlc, max_polylines_count, dynamic_requests, behind_chart) → void
```

Because strategies have many of the same features as indicators, the [strategy()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy) function includes most of the [indicator()](https://www.tradingview.com/pine-script-reference/v6/#fun_indicator) function’s parameters. The only exceptions are the [`timeframe` and `timeframe_gaps`](https://www.tradingview.com/pine-script-docs/language/declaration-statements/#timeframe-and-timeframe_gaps) parameters, because strategies cannot execute on other timeframes.

TipProgrammers can convert compatible indicator scripts into strategies by replacing the [indicator()](https://www.tradingview.com/pine-script-reference/v6/#fun_indicator) declaration statement with the [strategy()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy) declaration statement, using the same arguments, then adding calls to commands such as [strategy.entry()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.entry) and [strategy.exit()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.exit) to create orders. See an example in the [How can I turn my indicator into a strategy](https://www.tradingview.com/pine-script-docs/faq/strategies/#how-can-i-turn-my-indicator-into-a-strategy) section of the [Strategies FAQ page](https://www.tradingview.com/pine-script-docs/faq/strategies/).

The unique parameters in the [strategy()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy) declaration statement define the _default properties_ of the strategy simulation, including the initial simulated capital, default order sizes, hypothetical trading costs, and calculation behaviors. The sections below explain these unique parameters. To learn about the other parameters that are common to both indicators and strategies, see the [`indicator()`](https://www.tradingview.com/pine-script-docs/language/declaration-statements/#indicator) section above.

For detailed information about how to use the unique [strategy()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy) function parameters and the built-ins in the `strategy` namespace, refer to the [Strategies](https://www.tradingview.com/pine-script-docs/concepts/strategies/) page.

### `pyramiding` {#pyramiding}

The `pyramiding` parameter of the [strategy()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy) declaration statement accepts an “int” value specifying the default maximum number of _open trades_, from the orders created by [strategy.entry()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.entry) calls, that a strategy allows for a single position. The default argument is 1, meaning that the strategy can open only _one_ long or short trade at a time using orders from [strategy.entry()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.entry) calls and _cannot_ execute another entry order in the _same direction_ until after the existing trade closes. Users can adjust the script’s pyramiding limit without editing the code by using the “Pyramiding” input in the script’s “Settings/Properties” tab.

Note[Pyramiding](https://www.tradingview.com/pine-script-docs/concepts/strategies/#pyramiding) affects only entry orders from calls to the [strategy.entry()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.entry) command; it does **not** affect the behavior of orders from [strategy.order()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.order) calls.

The following example strategy uses two calls to the [strategy.entry()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.entry) command to create [market orders](https://www.tradingview.com/pine-script-docs/concepts/strategies/#market-orders) for entering long and short trades. The call for long orders executes once every five bars, excluding multiples of 30, and the one for short orders executes once every 30 bars. The [strategy()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy) declaration statement includes the argument `pyramiding = 3`, meaning that the strategy can enter up to _three trades_ for the same position using [strategy.entry()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.entry) calls by default.

As shown below, although the strategy’s long condition (highlighted by the purple background) occurs _five_ times before the short condition (highlighted by the orange background), the strategy executes only **three** entry orders for each long position instead of five. Once the number of open trades reaches three, it does not execute new long entry orders until after the short order _closes_ the existing long position:

![image](https://www.tradingview.com/pine-script-docs/_astro/Declaration-statements-Strategy-Pyramiding-1.Dj_MM0yD_ZiFz5w.webp)

```pine
//@version=6
strategy("Strategy `pyramiding` demo", overlay = true, pyramiding = 3, default_qty_value = 10)

// The `pyramiding = 3` argument above specifies that, by default, the strategy cannot use `strategy.entry()` calls to
// maintain an open position consisting of more than three trades.

//@variable The value for the short condition: `true` on every 30th bar, and `false` otherwise.
bool sellCondition = bar_index % 30 == 0
//@variable The value for the long condition: `true` on every 5th bar, excluding multiples of 30, and `false` otherwise.
bool buyCondition = bar_index % 5 == 0 and not sellCondition

if buyCondition
    // Place a market order named "buy" to close any short position and enter or add to a long position.
    strategy.entry("buy", strategy.long)
if sellCondition
    // Place a market order named "sell" to close any long position and enter or add to a short position.
    strategy.entry("sell", strategy.short)

// Highlight the background when the `buyCondition` or `sellCondition` value is `true`.
bgcolor(
    sellCondition ? color.new(color.orange, 80) : buyCondition ? color.new(color.purple, 85) : na,
    title = "Order conditions highlight"
)
```

Note that:

-   By default, the orders from the [strategy.entry()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.entry) command automatically close an existing position in the opposite direction and enter a new trade with the specified quantity. See the [Reversing positions](https://www.tradingview.com/pine-script-docs/concepts/strategies/#reversing-positions) section of the [Strategies](https://www.tradingview.com/pine-script-docs/concepts/strategies/) page for more information about this behavior.
-   The `default_qty_value` argument in the declaration statement specifies the initial default size of the strategy’s orders. See the [`default_qty_type` and `default_qty_value`](https://www.tradingview.com/pine-script-docs/language/declaration-statements/#default_qty_type-and-default_qty_value) section to learn more.
-   The strategy enters a new trade after every occurrence of the long condition only if the `pyramiding` value is at least 5.

### `calc_on_every_tick`, `calc_on_order_fills`, and `process_orders_on_close` {#calc_on_every_tick-calc_on_order_fills-and-process_orders_on_close}

The `calc_on_every_tick`, `calc_on_order_fills`, and `process_orders_on_close` parameters of the [strategy()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy) declaration statement specify the strategy’s default [calculation behaviors](https://www.tradingview.com/pine-script-docs/concepts/strategies/#altering-calculation-behavior). If the argument for each of these parameters is `false` (default), the strategy executes strictly _once per bar_, on each bar’s _closing tick_, and the [broker emulator](https://www.tradingview.com/pine-script-docs/concepts/strategies/#broker-emulator) fills each order from the strategy on the _open_ of the next available bar. Specifying a value of `true` for any of these parameters changes the strategy’s default execution and order-fill behaviors. Users can also change these behaviors via the “On every tick”, “After order is filled”, and “On bar close” checkboxes in the script’s “Settings/Properties” tab.

The `calc_on_every_tick` parameter specifies whether the strategy performs a _new execution_ on _each new tick_ of a [realtime bar](https://www.tradingview.com/pine-script-docs/language/execution-model/#realtime-bars) by default. If the value is `true`, the strategy executes once after _every update_ from the realtime data feed, similar to how an indicator executes, instead of waiting for each realtime bar to close. This parameter does _not_ affect the strategy’s executions on _historical bars_, because realtime tick information is not available on those bars.

The `calc_on_order_fills` parameter specifies whether the strategy can immediately recalculate and place additional orders on any bar where an _order fills_ by default. If the value is `true`, the strategy _re-executes_ on the next available tick following any tick where the broker emulator fills an order, even if that tick occurs during an open bar. This behavior enables the script to execute _more than once_ on any bar where an order fill occurs — up to four times per historical bar by default (at the open, high, low, and close), and up to once for each new tick on a realtime bar.

NoticeA strategy that enables recalculation on each tick or after order fills can behave _differently_ on realtime bars and historical bars, and therefore [repaint](https://www.tradingview.com/pine-script-docs/concepts/repainting/) after it reloads. Additionally, with recalculation after order fills enabled, the broker emulator can fill some historical orders at prices that are not typically possible in real-world trading, such as the exact high or low price of a bar. Therefore, when using either of these settings, exercise caution and examine the script’s behaviors carefully to avoid misleading results.

The `process_orders_on_close` parameter specifies whether the broker emulator can fill an order on the _same closing tick_ where the strategy creates the order by default. If the value is `false` (default), the earliest point at which the broker emulator can fill an order that occurs on a bar’s close is at the _open_ of the _following bar_, because that point is the next possible tick. If the value is `true`, the emulator fills the order _immediately_ on the bar’s close instead of waiting for the next bar’s opening tick.

For example, the following strategy simulates opening a position after one exponential moving average (EMA) crosses over another. On each bar where the EMAs cross, the script highlights the chart’s background, then creates a long or short [market order](https://www.tradingview.com/pine-script-docs/concepts/strategies/#market-orders) on that bar’s closing tick. With the default behavior defined by `process_orders_on_close = false`, the broker emulator does not fill each order on the same bar where the strategy creates it. Instead, it fills the order at the open of the following bar, because that point is the next available tick:

![image](https://www.tradingview.com/pine-script-docs/_astro/Declaration-statements-Strategy-Calc-on-every-tick-calc-on-order-fills-and-process-orders-on-close-1.BJHzvi6y_Z1cN16y.webp)

```pine
//@version=6
strategy("`process_orders_on_close` demo", overlay = true, process_orders_on_close = false)

// Calculate fast and slow moving averages.
float fastMA = ta.ema(close, 13)
float slowMA = ta.ema(close, 26)

// Set long and short order conditions based on crosses of the moving averages.
//@variable Is `true` if `fastMA` crosses above `slowMA`.
bool longCondition = ta.crossover(fastMA, slowMA)
//@variable Is `true` if `fastMA` crosses under `slowMA`.
bool shortCondition = ta.crossunder(fastMA, slowMA)
if longCondition
    strategy.entry("buy", strategy.long)
if shortCondition
    strategy.entry("sell", strategy.short)

// Plot the moving averages, and highlight the bars where order conditions occur.
plot(fastMA, "Fast MA", color.blue,   linewidth = 2)
plot(slowMA, "Slow MA", color.orange, linewidth = 2)
// Highlights background blue if long entry condition occurs, or orange if short entry condition occurs.
bgcolor(longCondition ? color.new(color.blue, 85) : shortCondition ? color.new(color.orange, 80) : na)
```

If we include `process_orders_on_close = true` in the [strategy()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy) declaration statement, the broker emulator is no longer limited to filling our strategy’s orders on the next available tick by default. Instead, it fills the orders immediately on each bar’s close:

![image](https://www.tradingview.com/pine-script-docs/_astro/Declaration-statements-Strategy-Calc-on-every-tick-calc-on-order-fills-and-process-orders-on-close-2.BUMMfyy5_1CMmT4.webp)

```pine
//@version=6
strategy("`process_orders_on_close` demo", overlay = true, process_orders_on_close = true)

// Calculate fast and slow moving averages.
float fastMA = ta.ema(close, 13)
float slowMA = ta.ema(close, 26)

// Set long and short order conditions based on crosses of the moving averages.
//@variable Is `true` if `fastMA` crosses above `slowMA`.
bool longCondition = ta.crossover(fastMA, slowMA)
//@variable Is `true` if `fastMA` crosses under `slowMA`.
bool shortCondition = ta.crossunder(fastMA, slowMA)
if longCondition
    strategy.entry("buy", strategy.long)
if shortCondition
    strategy.entry("sell", strategy.short)

// Plot the moving averages, and highlight the bars where order conditions occur.
plot(fastMA, "Fast MA", color.blue,   linewidth = 2)
plot(slowMA, "Slow MA", color.orange, linewidth = 2)
// Highlights background blue if long entry condition occurs, or orange if short entry condition occurs.
bgcolor(longCondition ? color.new(color.blue, 85) : shortCondition ? color.new(color.orange, 80) : na)
```

NoticeForcing orders to fill on a bar’s close can be helpful in some scenarios, such as when backtesting manual strategies where traders enter or exit positions immediately before the market closes. However, it’s crucial to understand that it can also cause _misleading_ results in some cases, because creating and filling orders on the same tick is _not_ typically possible in real-world trading.

See the [Altering calculation behavior](https://www.tradingview.com/pine-script-docs/concepts/strategies/#altering-calculation-behavior) section of the [Strategies](https://www.tradingview.com/pine-script-docs/concepts/strategies/) page to learn more about the `calc_on_every_tick`, `calc_on_order_fills`, and `process_orders_on_close` parameters. For detailed information about how scripts execute on historical and realtime bars, and how these parameters affect executions, refer to the [Execution model](https://www.tradingview.com/pine-script-docs/language/execution-model/) page.

### `slippage` and `backtest_fill_limits_assumption` {#slippage-and-backtest_fill_limits_assumption}

The `slippage` parameter of the `strategy()` declaration statement specifies the default fixed number of ticks that the strategy applies to the fill prices of _all_ [market orders](https://www.tradingview.com/pine-script-docs/concepts/strategies/#market-orders) and [stop orders](https://www.tradingview.com/pine-script-docs/concepts/strategies/#stop-and-stop-limit-orders) to simulate [slippage](https://www.tradingview.com/pine-script-docs/concepts/strategies/#slippage-and-unfilled-limits). If the argument is a positive “int” value, the strategy adds the specified number of ticks to the fill prices of long orders and subtracts it from the fill prices of short orders. This behavior helps simulate the disparity between expected and actual fill prices that might occur in real-world trading. If the `slippage` argument is 0 (default), the strategy fills orders at their expected prices without simulating any slippage. Users can change the specified slippage amount via the “Slippage” input in the strategy’s “Settings/Properties” tab.

The `backtest_fill_limits_assumption` parameter specifies the default number of ticks by which the market price must _exceed_ the prices of [limit orders](https://www.tradingview.com/pine-script-docs/concepts/strategies/#limit-orders) before the [broker emulator](https://www.tradingview.com/pine-script-docs/concepts/strategies/#broker-emulator) can fill the orders. If the argument is a positive “int” value, the broker emulator fills a limit order at the defined price only if the market price moves _past_ it by the specified number of ticks in the favorable direction. This behavior helps simulate the possibility of [unfilled limit orders](https://www.tradingview.com/pine-script-docs/concepts/strategies/#slippage-and-unfilled-limits), as filling limit orders in the real world requires sufficient liquidity and price action around the limit level. If the argument is 0 (default), the emulator fills orders as soon as the market price reaches the limit price or a more favorable value. Users can adjust a strategy’s limit verification requirements via the “Verify price for limit orders” input in the “Settings/Properties” tab.

NoticeLimit verification can cause order fills to occur at _different times_, depending on how long it takes for the market price to exceed limit levels by the specified amount. This tradeoff is necessary to enable filling limit orders at their verified prices without introducing lookahead bias in the simulation. However, in some cases, it can also cause some limit orders to fill at times that are not possible in the real world. We therefore recommend users understand this price-time tradeoff and analyze their strategies carefully when adding verification to limit orders.

### `default_qty_type` and `default_qty_value` {#default_qty_type-and-default_qty_value}

The `default_qty_type` and `default_qty_value` parameters of the [strategy()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy) declaration statement specify the initial _default order size_ for the [strategy.entry()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.entry) and [strategy.order()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.order) commands. If a call to either command does not specify an order size, the resulting order uses the default order size defined by these parameters. Users can adjust these properties via the “Default order size” inputs in the script’s “Settings/Properties” tab.

The `default_qty_type` parameter specifies the default _quantity type_ for each order from [strategy.entry()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.entry) and [strategy.order()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.order) calls. The possible arguments and their effects are as follows:

-   [strategy.fixed](https://www.tradingview.com/pine-script-reference/v6/#const_strategy.fixed) — The default order size is a fixed number of contracts, shares, lots, or units, depending on the instrument.
-   [strategy.cash](https://www.tradingview.com/pine-script-reference/v6/#const_strategy.cash) — The default size is a fixed number of units of the account currency specified by the [`currency`](https://www.tradingview.com/pine-script-docs/language/declaration-statements/#initial_capital-and-currency) argument.
-   [strategy.percent\_of\_equity](https://www.tradingview.com/pine-script-reference/v6/#const_strategy.percent_of_equity) — The default size is a fixed percentage of the strategy’s available equity.

The default argument is [strategy.fixed](https://www.tradingview.com/pine-script-reference/v6/#const_strategy.fixed).

The `default_qty_value` parameter accepts a “float” value that specifies the amount of the defined quantity type to use as the default order size. The default argument is 1, meaning that the strategy uses the default order size of one contract/share/lot/unit, one unit of the account currency, or one percent of the available equity, depending on the `default_qty_type` argument.

The specified default order size applies only to the orders from [strategy.entry()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.entry) and [strategy.order()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.order) calls that do _not_ include a `qty` argument. If a call to either command does include a `qty` argument, that call creates an order for the number of contracts/shares/lots/units specified by the argument instead of using the default quantity type and value. See the [Position sizing](https://www.tradingview.com/pine-script-docs/concepts/strategies/#position-sizing) section of the [Strategies](https://www.tradingview.com/pine-script-docs/concepts/strategies/) page for an example.

NoteThe `default_qty_type` and `default_qty_value` parameters do not affect orders from the [strategy.exit()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.exit) or [strategy.close()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.close) commands, because those commands create orders specifically for _closing_ trades. The default order size for those commands is the size of the trades to which they apply.

The following example demonstrates how different default order sizes can affect a strategy’s entry orders. The script below uses a [strategy.entry()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.entry) call, without a `qty` argument, to place a long [market order](https://www.tradingview.com/pine-script-docs/concepts/strategies/#market-orders) when the [close](https://www.tradingview.com/pine-script-reference/v6/#var_close) and [volume](https://www.tradingview.com/pine-script-reference/v6/#var_volume) values are rising over a specified number of bars, then uses a [strategy.close\_all()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.close_all) call to close the open position when the [close](https://www.tradingview.com/pine-script-reference/v6/#var_close) value is falling while the [volume](https://www.tradingview.com/pine-script-reference/v6/#var_volume) value is rising. It also plots the value of the [strategy.position\_size](https://www.tradingview.com/pine-script-reference/v6/#var_strategy.position_size) variable in a separate pane to visualize the size of each open position.

The [strategy()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy) statement in this example includes the arguments `default_qty_type = strategy.fixed` and `default_qty_value = 20`, which set the strategy’s default order size to 20 contracts/shares/lots/units. As shown by the trade markers and the plot on our NYSE:UBER chart below, each order from the [strategy.entry()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.entry) command consistently opens a 20-share trade:

![image](https://www.tradingview.com/pine-script-docs/_astro/Declaration-statements-Strategy-Default-qty-type-and-default-qty-value-1.C6t5EmMo_Z26Qktg.webp)

```pine
//@version=6
// The `default_qty_*` arguments in this declaration statement specify that, by default, `strategy.entry()` and
// `strategy.order()` calls create orders for 20 contracts/shares/lots/units if they do not specify a `qty` argument.
strategy(
    "`default_qty_type` and `default_qty_value` demo",
    default_qty_type = strategy.fixed, default_qty_value = 20
)

//@variable The number of bars for the `ta.rising()` and `ta.falling()` calculations.
int lengthInput = input.int(2, "Length", minval = 1, display = display.none)

// Determine if the `close` series is rising or falling over `lengthInput` bars, and if the `volume` series is rising.
bool risingClose  = ta.rising(close,  lengthInput)
bool fallingClose = ta.falling(close, lengthInput)
bool risingVolume = ta.rising(volume, lengthInput)

if risingVolume
    switch
        // Place a long market order if the `close` and `volume` values are both rising.
        risingClose  => strategy.entry("Long entry", strategy.long)
        // Place an order to close the position if the `close` value is falling while the `volume` value is rising.
        fallingClose => strategy.close_all()

// Plot the size of the current position. The plotted value is 0 if a position is not open.
plot(strategy.position_size, "Position size", style = plot.style_area)
```

If we edit the declaration statement to use the argument `default_qty_type = strategy.percent_of_equity`, the strategy sets the default size of each entry order to allocate 20% of its current available equity instead of the amount required to purchase 20 shares. Now, the trade markers and plot show _varying sizes_, because the number of shares that corresponds to the default order size varies with both the strategy’s available equity and the current market price:

![image](https://www.tradingview.com/pine-script-docs/_astro/Declaration-statements-Strategy-Default-qty-type-and-default-qty-value-2.DCBv2auy_ZrM34e.webp)

```pine
//@version=6
// The `default_qty_*` arguments in this declaration statement specify that, by default, `strategy.entry()` and
// `strategy.order()` calls create orders for 20% of the available equity if they do not specify a `qty` argument.
strategy(
    "`default_qty_type` and `default_qty_value` demo",
    default_qty_type = strategy.percent_of_equity, default_qty_value = 20
)

//@variable The number of bars for the `ta.rising()` and `ta.falling()` calculations.
int lengthInput = input.int(2, "Length", minval = 1, display = display.none)

// Determine if the `close` series is rising or falling over `lengthInput` bars, and if the `volume` series is rising.
bool risingClose  = ta.rising(close,  lengthInput)
bool fallingClose = ta.falling(close, lengthInput)
bool risingVolume = ta.rising(volume, lengthInput)

if risingVolume
    switch
        // Place a long market order if the `close` and `volume` values are both rising.
        risingClose  => strategy.entry("Long entry", strategy.long)
        // Place an order to close the position if the `close` value is falling while the `volume` value is rising.
        fallingClose => strategy.close_all()

// Plot the size of the current position. The plotted value is 0 if a position is not open.
plot(strategy.position_size, "Position size", style = plot.style_area)
```

### `initial_capital` and `currency` {#initial_capital-and-currency}

The `initial_capital` parameter of the `strategy()` declaration statement specifies the default _initial account balance_ for the strategy’s simulation, as a quantity of the account currency. It accepts a positive “int” or “float” argument. The default is 1000000. Users can change the strategy’s initial account balance by adjusting the “Initial capital” input in the script’s “Settings/Properties” tab.

The `currency` parameter specifies the strategy’s default _account currency_. It is the currency unit for the strategy’s initial capital and for the internal calculations in the simulation that express values as currency amounts (equity, profit and loss, commission, etc.). The parameter accepts a `currency.*` constant (e.g., [currency.USD](https://www.tradingview.com/pine-script-reference/v6/#const_currency.USD)) or a string representing a valid _currency code_, (e.g., `"USD"`). The default is [currency.NONE](https://www.tradingview.com/pine-script-reference/v6/#const_currency.NONE), which specifies that the strategy uses the _same currency_ as that of the quoted prices on the chart. Users can change the strategy’s account currency via the “Base currency” input in the “Settings/Properties” tab.

If the specified account currency differs from the chart’s currency, the strategy _converts_ monetary values in its calculations to express them in the account currency. However, the prices of the strategy’s orders remain expressed in the chart’s currency. To convert necessary monetary values to the account currency, the strategy typically uses the previous _daily_ value of a corresponding _currency pair_ as the conversion rate, or the value from a [spread](https://www.tradingview.com/support/solutions/43000502298-spread-charts-explained/) if no direct currency pair is available. See the [Currency](https://www.tradingview.com/pine-script-docs/concepts/strategies/#currency) section of the [Strategies](https://www.tradingview.com/pine-script-docs/concepts/strategies/) page for more information.

### `commission_type` and `commission_value` {#commission_type-and-commission_value}

The `commission_type` and `commission_value` parameters of the `strategy()` declaration statement specify the default commission fees that the broker emulator applies to the strategy’s simulated transactions. Users can customize the strategy’s commission settings via the “Commission” inputs in the “Settings/Properties” tab.

The `commission_type` parameter determines the default _commission type_ for each executed order. The possible arguments and their effects are as follows:

-   [strategy.commission.cash\_per\_order](https://www.tradingview.com/pine-script-reference/v6/#const_strategy.commission.cash_per_order) — The default commission for each transaction is a fixed number of units in the strategy’s [account currency](https://www.tradingview.com/pine-script-docs/language/declaration-statements/#initial_capital-and-currency).
-   [strategy.commission.cash\_per\_contract](https://www.tradingview.com/pine-script-reference/v6/#const_strategy.commission.cash_per_contract) — The commission is a fixed account currency amount for each traded contract/lot/share/unit.
-   [strategy.commission.percent](https://www.tradingview.com/pine-script-reference/v6/#const_strategy.commission.percent) — The commission is a fixed percentage of each transaction’s value.

The default argument is [strategy.commission.percent](https://www.tradingview.com/pine-script-reference/v6/#const_strategy.commission.percent).

The `commission_value` parameter accepts a positive “int” or “float” value specifying the default fee amount for the commission type. For example, if the value is 1, the strategy simulates a fee of one unit of the account currency per transaction, one unit of the account currency per contract/share/lot/unit, or one percent of each transaction’s size by default, depending on the `commission_type` value. The default argument is 0, meaning that the strategy does not simulate commission unless the user specifies a nonzero value for the first “Commission” input in the “Properties” tab.

### `close_entries_rule` {#close_entries_rule}

The `close_entries_rule` parameter of the [strategy()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy) declaration statement determines the order in which the strategy simulation closes the trades in an open market position. It accepts one of two “string” arguments: `"FIFO"` or `"ANY"`. If the value is `"FIFO"`, the [broker emulator](https://www.tradingview.com/pine-script-docs/concepts/strategies/#broker-emulator) follows _First In, First Out (FIFO)_ rules when closing market positions. Under these rules, the _earliest_ open trade is always the _first_ to close, regardless of the entry IDs specified by the script’s [strategy.exit()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.exit) or [strategy.close()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.close) calls. If the value is `"ANY"`, the broker emulator _ignores_ FIFO rules and closes the trades specified by the exit commands, even if an earlier trade with a different entry ID is open. The default is `"FIFO"`.

NoteUsers cannot customize a strategy’s exit order rules from the script’s “Settings/Properties” tab, unlike other strategy properties. The only way to change this property is by specifying a `close_entries_rule` argument in the [strategy()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy) statement.

Refer to the [Closing a market position](https://www.tradingview.com/pine-script-docs/concepts/strategies/#closing-a-market-position) section of the [Strategies](https://www.tradingview.com/pine-script-docs/concepts/strategies/) page for an example of how changing the `close_entries_rule` argument can affect a strategy’s exit behavior.

### `margin_long` and `margin_short` {#margin_long-and-margin_short}

The `margin_long` and `margin_short` parameters of the [strategy()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy) declaration statement specify the default [margin](https://www.tradingview.com/pine-script-docs/concepts/strategies/#margin) requirements for the strategy’s long and short positions, respectively. Users can adjust the strategy’s long and short margin requirements via the “Margin for long positions” and “Margin for short positions” inputs in the “Settings/Properties” tab.

Margin is the percentage of a position’s value that the simulated account must retain in its balance as _collateral_ for the [broker emulator](https://www.tradingview.com/pine-script-docs/concepts/strategies/#broker-emulator) to cover the rest of the position. It is the _inverse_ of _leverage_. For example, if the margin requirement for a long position is 50%, the strategy must maintain sufficient funds to cover _half_ of the open position. This level of margin means that the strategy’s leverage is 2:1. In other words, the strategy can risk up to _twice_ its available balance on a simulated trade.

The default `margin_long` and `margin_short` arguments are 100, meaning that the strategy must cover _100%_ of each long and short position using its simulated account balance.

If a strategy’s available funds drop below the required margin percentage, the broker emulator triggers a _margin call_, which forcibly _liquidates_ part or all of the simulated position to cover the loss. For detailed information about margin simulation and margin call events, refer to the [How to simulate trading with leverage in Pine Script](https://www.tradingview.com/support/solutions/43000717375-how-to-simulate-trading-with-leverage-in-pine-script/) article in our Help Center.

NoticeIf a strategy’s long or short margin percentage is _zero_, it effectively has _infinite_ leverage. It can open and maintain positions of _any size_, regardless of its simulated account balance. This behavior can cause **misleading** results, because real-world brokers require traders to fund at least part of their positions. Therefore, we do not recommend using a value of 0 as the `margin_long` or `margin_short` argument.

### `risk_free_rate` {#risk_free_rate}

The `risk_free_rate` parameter of the [strategy()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy) declaration statement specifies the annual percentage return of a hypothetical _risk-free_ investment. The strategy uses the specified risk-free rate to calculate the [Sharpe ratio](https://www.tradingview.com/support/solutions/43000681694-risk-performance-ratios-sharpe-ratio/) and [Sortino ratio](https://www.tradingview.com/support/solutions/43000681697-risk-performance-ratios-sortino-ratio/) metrics displayed in the “Strategy report” panel. The default value is 2, meaning that these metrics assess the strategy’s _risk-adjusted returns_ relative to a hypothetical 2% risk-free rate.

NoteUsers cannot adjust the risk-free rate from the “Settings/Properties” tab. The only way to change the value is by specifying a `risk_free_rate` argument in the [strategy()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy) statement.

### `use_bar_magnifier` {#use_bar_magnifier}

The `use_bar_magnifier` parameter of the [strategy()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy) declaration statement specifies whether the strategy enables the [Bar Magnifier](https://www.tradingview.com/pine-script-docs/concepts/strategies/#bar-magnifier) backtesting mode by default. Users can activate or deactivate the Bar Magnifier mode by selecting the “Using bar magnifier” checkbox in the strategy’s “Settings/Properties” tab. If the value is `true`, the broker emulator retrieves available prices from a _lower timeframe_ on historical bars by default for more precise intrabar order fills. If the argument is `false` (default), the broker emulator relies on default _assumptions_ about intrabar price movement instead of using prices from a lower timeframe. See the [Broker emulator](https://www.tradingview.com/pine-script-docs/concepts/strategies/#broker-emulator) section of the [Strategies](https://www.tradingview.com/pine-script-docs/concepts/strategies/) page to learn more.

NoteThe Bar Magnifier feature is available only to accounts with Premium and Ultimate plans.

### `fill_orders_on_standard_ohlc` {#fill_orders_on_standard_ohlc}

The `fill_orders_on_standard_ohlc` parameter of the [strategy()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy) declaration statement specifies whether the broker emulator fills the strategy’s orders using actual prices by default when the strategy executes on a [Heikin Ashi chart](https://www.tradingview.com/support/solutions/43000619436-understanding-heikin-ashi-charts/). Users can activate or deactivate the feature via the “Using standard OHLC” input in the strategy’s “Settings/Properties” tab. If the value is `false`, the emulator fills the strategy’s orders using the chart’s _synthetic prices_ by default. If `true`, it fills the orders using the _actual_ open, high, low, and close prices from a _standard chart_ dataset for more realistic results. The default argument is `false`.

NoticeThis feature **does not** affect backtests on other non-standard charts, such as [Renko](https://www.tradingview.com/support/solutions/43000502284-understanding-renko-charts/) or [Kagi](https://www.tradingview.com/support/solutions/43000502272-learn-to-use-kagi-charts/). A strategy always uses the chart’s synthetic prices when executing on those chart types, and therefore produces _unreliable_ results, regardless of the specified `fill_orders_on_standard_ohlc` argument. See the [Strategy produces unrealistic results on non-standard chart types](https://www.tradingview.com/support/solutions/43000481029-strategy-produces-unrealistic-results-on-non-standard-chart-types-heikin-ashi-renko-etc/) article in our Help Center to learn more.

## `library()` {#library}

The [library()](https://www.tradingview.com/pine-script-reference/v6/#fun_library) function declares that the script is a library. [Libraries](https://www.tradingview.com/pine-script-docs/concepts/libraries/) _export_ reusable [functions](https://www.tradingview.com/pine-script-docs/language/user-defined-functions/), [methods](https://www.tradingview.com/pine-script-docs/language/methods/#user-defined-methods), [user-defined types (UDTs)](https://www.tradingview.com/pine-script-docs/language/type-system/#user-defined-types), [enum types](https://www.tradingview.com/pine-script-docs/language/type-system/#enum-types), or [constant variables](https://www.tradingview.com/pine-script-docs/language/type-system/#const). Libraries can also include _non-exported_ code to demonstrate how they work and how to use them. Indicators, strategies, and other libraries can use the [import](https://www.tradingview.com/pine-script-reference/v6/#kw_import) keyword to import a [published](https://www.tradingview.com/pine-script-docs/writing/publishing/) library’s exported code components. Importing components from libraries often helps programmers streamline script creation and simplify source code.

The [VisibleChart](https://www.tradingview.com/script/j7vCseM2-VisibleChart/) publication from [PineCoders](https://www.tradingview.com/u/PineCoders/#published-scripts) is an example of a library. It exports functions that perform calculations on the chart’s visible bars. The example script in the FAQ entry [Can I create an indicator that plots like the built-in Volume or Volume Profile indicators](https://www.tradingview.com/pine-script-docs/faq/indicators/#can-i-create-an-indicator-that-plots-like-the-built-in-volume-or-volume-profile-indicators) demonstrates how scripts can import and use functions from this library.

Because the primary purpose of a library is to export components for other scripts, they have multiple unique characteristics, including the following:

-   Libraries are the only scripts that can use the [export](https://www.tradingview.com/pine-script-reference/v6/#kw_export) keyword.
-   Libraries _cannot_ directly create alert triggers, but they can _export_ custom functions that contain [alert()](https://www.tradingview.com/pine-script-reference/v6/#fun_alert) calls. Indicators and strategies can use the alert triggers from calls to those functions.
-   A library’s title acts similarly to a _namespace_ identifier when another script imports the library. Therefore, unlike indicators and strategies, libraries must follow [identifier](https://www.tradingview.com/pine-script-docs/language/identifiers/) naming rules in their titles.
-   [User-defined functions](https://www.tradingview.com/pine-script-docs/language/user-defined-functions/) and methods exported by libraries must prefix each declared parameter with a [type keyword](https://www.tradingview.com/pine-script-docs/language/user-defined-functions/#type-keywords).
-   The example code of a library executes similarly to an indicator. When applied to a chart, the code executes _once per bar_ on historical bars and _once per tick_ on [realtime bars](https://www.tradingview.com/pine-script-docs/language/execution-model/#realtime-bars).
-   Libraries use default indicator properties when their code executes on a chart. The declaration statement of a library does not include parameters for setting decimal precision, plot formatting, scales, drawing limits, or other script properties.
-   Unlike indicators, libraries can use the available `strategy.*` built-ins. However, unlike strategies, a library’s example code does _not_ display trade markers on the chart or generate a strategy report.
-   For a library to compile, it must use the [export](https://www.tradingview.com/pine-script-reference/v6/#kw_export) keyword to export _at least one_ function, method, enum, UDT, or “const” variable.

The [library()](https://www.tradingview.com/pine-script-reference/v6/#fun_library) function’s signature is as follows:

```
library(title, overlay, dynamic_requests) → void
```

The `overlay` parameter behaves the same as that of the [indicator()](https://www.tradingview.com/pine-script-reference/v6/#fun_indicator) and [strategy()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy) declaration statements. Refer to the [`overlay`, `scale`, and `behind_chart`](https://www.tradingview.com/pine-script-docs/language/declaration-statements/#overlay-scale-and-behind_chart) section above for information about this parameter.

The `title` and `dynamic_requests` parameters are also common to the [indicator()](https://www.tradingview.com/pine-script-reference/v6/#fun_indicator) and [strategy()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy) functions. However, they have some _unique_ characteristics in libraries, as explained in the sections below.

### `title` {#title}

The `title` parameter of the [library()](https://www.tradingview.com/pine-script-reference/v6/#fun_library) declaration statement specifies the library’s unique name, which other scripts _reference_ to import and use the library’s code. For example, if a `userName` user [publishes](https://www.tradingview.com/pine-script-docs/writing/publishing/) a library that uses `"foo"` as the `title` argument, another script imports version 1 of the library using the following [import](https://www.tradingview.com/pine-script-reference/v6/#kw_import) statement:

```pine
// Imports version 1 of the `foo` library from the `userName` user.
import userName/foo/1
```

The script can then use the library’s defined title (or a specified _alias_) similarly to a _namespace_ to access the imported components. For example, if the library exports a function named `bar()`, the script that imports the library references the library’s title using _dot notation_ when calling the function:

```pine
// Calls the `bar()` function from the `foo` library and assigns the result to a variable.
result = foo.bar()
```

Because a library’s title behaves as a _code identifier_ in other scripts, the `title` argument must follow identifier naming rules. The “string” argument can contain ASCII letters (`a-z` and `A-Z`), numeric digits (`0-9`), and underscores (`_`). The argument cannot be an empty string, cannot contain spaces or special characters, and cannot _start_ with a numeric digit. Special characters include any of the following:

-   Basic punctuation, including periods (`.`), commas (`,`), quotation marks (`"`), apostrophes (`'`), exclamation points (`!`), etc.
-   Symbols that scripts use for syntax, such as parentheses (`( )`), square brackets (`[ ]`), plus signs (`+`), hyphens (`-`), asterisks (`*`), slashes (`/`), and percent signs (`%`).
-   Currency symbols, such as `$` or `€`.
-   Non-ASCII letters and digits, such as the Unicode character `𝖠` (U+1D5A0).
-   Other Unicode characters, such as emoji or special-purpose symbols.

For example, a string such as `"Library_for_14_day_averages"` is a valid `title` argument for the [library()](https://www.tradingview.com/pine-script-reference/v6/#fun_library) declaration statement, but an argument such as `"Library for 14-day averages"` causes a _compilation error_.

NoteWhen [preparing a publication](https://www.tradingview.com/pine-script-docs/writing/publishing/#preparing-a-publication) for a library, the `title` argument appears as the _suggested title_ in the “Publish script” window. Programmers can specify a custom title for the publication if they wish. However, to import the library in another script, the [import](https://www.tradingview.com/pine-script-reference/v6/#kw_import) statement requires the `title` argument defined in the library’s declaration statement, **not** the publication’s custom title. Therefore, we recommend using the `title` argument as the title of a library publication for consistency.

If a user applies the library directly to their chart, the `title` argument’s text appears as the display name in all relevant chart locations, including the script’s status line, the data window, and the [Pine Logs](https://www.tradingview.com/pine-script-docs/writing/debugging/#pine-logs) pane.

### `dynamic_requests` {#dynamic_requests-1}

The `dynamic_requests` parameter of the [library()](https://www.tradingview.com/pine-script-reference/v6/#fun_library) declaration statement specifies whether the library can use [dynamic requests](https://www.tradingview.com/pine-script-docs/concepts/other-timeframes-and-data/#dynamic-requests). If the argument is `true` (default), the library can use `request.*()` function calls with [“series” arguments](https://www.tradingview.com/pine-script-docs/concepts/other-timeframes-and-data/#nested-requests) to define the requested ticker ID and timeframe, include `request.*()` calls [in the local scopes](https://www.tradingview.com/pine-script-docs/concepts/other-timeframes-and-data/#in-local-scopes) of [conditional structures](https://www.tradingview.com/pine-script-docs/language/conditional-structures/) or [loops](https://www.tradingview.com/pine-script-docs/language/loops/), and execute [nested requests](https://www.tradingview.com/pine-script-docs/concepts/other-timeframes-and-data/#nested-requests). Additionally, the library can _export_ [user-defined functions](https://www.tradingview.com/pine-script-docs/language/user-defined-functions/) and [methods](https://www.tradingview.com/pine-script-docs/language/methods/#user-defined-methods) that use `request.*()` calls within their [function scopes](https://www.tradingview.com/pine-script-docs/language/user-defined-functions/#function-scopes).

NoteAll `request.*()` calls in a library’s _exported_ functions **cannot** use `expression` arguments that _depend_ on any of the functions’ _parameters_. However, `request.*()` call arguments that define ticker ID, timeframe, and other settings of a request _can_ depend on exported function parameters.

If the `dynamic_requests` argument is `false`, the library allows `request.*()` calls only in the _global scope_ or within _non-exported_ functions, and those calls require arguments with “simple” or a weaker [type qualifier](https://www.tradingview.com/pine-script-docs/language/type-system/#qualifiers) for all parameters except for `expression`.

The example library below exports a custom `requestFinancialInsights()` function, which uses multiple `request.*()` calls to retrieve the quarterly Earnings Per Share (EPS), total revenue, total outstanding shares for a stock, and estimates the instrument’s market capitalization. The function returns a [tuple](https://www.tradingview.com/pine-script-docs/language/type-system/#tuples) containing all four values. The library can export this function because its declaration statement enables dynamic requests.

The library’s example code, listed below the user-defined function, demonstrates one way that programmers who import the library can use the function. The code creates a table and populates its cells with a `requestFinancialInsights()` call’s results on the last available bar:

![image](https://www.tradingview.com/pine-script-docs/_astro/Declaration-statements-Library-Dynamic-requests-1.hcbcmUzM_ZKLhjH.webp)

```pine
//@version=6

//@description This library exports a function that uses dynamic `request.*()` calls to retrieve multiple financial
//             metrics for a stock instrument.
library("FinancialInsights", overlay = true, dynamic_requests = true)

//#region --- Exported code ---

//@function      Requests the latest quarterly Earnings Per Share (EPS), total revenue, and total outstanding shares,
//               and calculates the latest market capitalization value for the specified stock.
//@param symbol  The symbol or ticker ID for the data requests. Requires an exchange prefix (e.g., `"NASDAQ:AAPL"`).
//@returns       A tuple containing the EPS, total revenue, outstanding shares, and market cap values, respectively.
export requestFinancialInsights(string symbol) =>
    //@variable The latest Earnings Per Share reported for the stock.
    float eps = request.earnings(symbol, earnings.actual)
    //@variable The quarterly total revenue reported for the issuing company.
    float totalRevenue = request.financial(symbol, "TOTAL_REVENUE", "FQ")
    //@variable The quarterly total number of outstanding shares reported for the stock.
    float totalSharesOutstanding = request.financial(symbol, "TOTAL_SHARES_OUTSTANDING", "FQ")
    //@variable The market capitalization, estimated by multiplying outstanding shares by current share price.
    float marketCap = totalSharesOutstanding * close
    // Return the four results in a tuple.
    [eps, totalRevenue, totalSharesOutstanding, marketCap]
//#endregion

//#region --- Example code ---

// The code defined below shows an example of *how to use* the library's exported function.
// This code is *not* exported; a script that imports the library cannot access it.

//@variable References a `table` object that displays financial insights for the stock represented on the chart.
var table tbl = table.new(position.top_right, 2, 5, color.yellow, border_color = color.gray, border_width = 1)

// Initialize row and column header cells in the table on the first bar.
if barstate.isfirst
    tbl.cell(0, 1, "Latest EPS"),                tbl.cell(0, 2, "Total revenue")
    tbl.cell(0, 3, "Total outstanding shares"),  tbl.cell(0, 4, "Market cap")
    tbl.cell(1, 0, str.format("{0} ({1})", syminfo.tickerid, syminfo.currency))
if barstate.islast
    // Call the `requestFinancialInsights()` function and declare a tuple of variables to store the data on the last bar.
    [currEPS, currRevenue, currShares, currMarketCap] = requestFinancialInsights(syminfo.tickerid)
    // Populate the remaining table cells with the retrieved results.
    tbl.cell(1, 1, str.tostring(currEPS,       "0.00"))
    tbl.cell(1, 2, str.tostring(currRevenue,   format.volume))
    tbl.cell(1, 3, str.tostring(currShares,    format.volume))
    tbl.cell(1, 4, str.tostring(currMarketCap, format.volume))
//#endregion
```

Note that:

-   We included `dynamic_requests = true` in the [library()](https://www.tradingview.com/pine-script-reference/v6/#fun_library) statement only to emphasize the `dynamic_requests` parameter. Specifying this argument is unnecessary; the value is `true` by default. A compilation error occurs if we change the value to `false`, because the library cannot export the custom function or call it within the example code’s [if](https://www.tradingview.com/pine-script-reference/v6/#kw_if) structure.
-   The `\\@description` [annotation](https://www.tradingview.com/pine-script-docs/language/script-structure/#compiler-annotations) at the top of the script sets a _default description_ for the library. Similarly, the `\\@function`, `\\@param`, and `\\@returns` annotations specify documentation for the exported function. Users who import this hypothetical library can hover over its identifiers to view the formatted text from these annotations. Additionally, the “Publish script” window uses these annotations to generate a default [publication description](https://www.tradingview.com/pine-script-docs/writing/publishing/#title-and-description).
-   The source code includes `//#region` and `//#endregion` annotations to define _collapsible regions_ that visually separate the library’s exported code from its non-exported code in the Pine Editor.

See the [Request](https://www.tradingview.com/script/Rpmobpw5-Request/) publication from the [TradingView](https://www.tradingview.com/u/TradingView/#published-scripts) account for an advanced example of a library that exports custom functions using dynamic requests.
