---
title: Other data and timeframes
source: https://www.tradingview.com/pine-script-docs/faq/other-data-and-timeframes/
section: faq
---

# Other data and timeframes {#other-data-and-timeframes}

## What kinds of data can I get from a higher timeframe? {#what-kinds-of-data-can-i-get-from-a-higher-timeframe}

Generally speaking, the [request.security()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.security) function can get the same kinds of data from another timeframe that is available on the chart timeframe. Scripts can retrieve built-in variables like [open](https://www.tradingview.com/pine-script-reference/v6/#var_open), [high](https://www.tradingview.com/pine-script-reference/v6/#var_high), [low](https://www.tradingview.com/pine-script-reference/v6/#var_low), [close](https://www.tradingview.com/pine-script-reference/v6/#var_close), [volume](https://www.tradingview.com/pine-script-reference/v6/#var_volume), and [bar\_index](https://www.tradingview.com/pine-script-reference/v6/#var_bar_index).

The [request.security()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.security) function can also evaluate built-in or user-defined functions in the requested _context_ (timeframe and symbol). For example, the following example script retrieves the [Average True Range](https://www.tradingview.com/support/solutions/43000501823-average-true-range-atr/) (ATR) value from the daily (`1D`) timeframe by passing the [ta.atr()](https://www.tradingview.com/pine-script-reference/v6/#fun_ta.atr) function as the `expression` argument.

```pine
//@version=6
indicator("HTF ATR")
float higherTfAtr = request.security(symbol = syminfo.tickerid, timeframe = "1D", expression = ta.atr(14))
plot(higherTfAtr)
```

NoticeWhile the above script executes on realtime bars, its requested data includes _unconfirmed_ values from developing daily bars. Only the _confirmed_ values for the closed bars and the latest value for the open bar remain available after the script reloads. To learn how to request _non-repainting_ values from another context, see the [Avoiding repainting](https://www.tradingview.com/pine-script-docs/concepts/other-timeframes-and-data/#avoiding-repainting) section of the [Other timeframes and data](https://www.tradingview.com/pine-script-docs/concepts/other-timeframes-and-data/) page.

## Which `request.*()` function should I use for lower timeframes? {#which-request-function-should-i-use-for-lower-timeframes}

The [request.security()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.security) function is intended for accessing data at timeframes that are equal to or higher than the chart’s current timeframe. It is _possible_ to retrieve lower-timeframe (LTF) data using this function. However, the function returns the value from only _one_ LTF bar within the chart’s current bar (the last bar, by default).

If the script supplies the `expression` as a variable or simple calculation, directly or within a function, the data that [request.security()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.security) returns from a lower timeframe is generally of limited use (see the first script in [this section](https://www.tradingview.com/pine-script-docs/faq/other-data-and-timeframes/#how-can-i-plot-a-moving-average-only-when-the-charts-timeframe-is-1d-or-higher) for an example). It is possible, however, to construct a function that performs meaningful calculations on the LTF bars and then returns the result on the last bar. The following example script counts the number of LTF bars in a chart bar and returns this value on the last LTF bar. For simplicity, the timeframes are hardcoded to `"1D"` and `"1W"` and the script should therefore be run from a chart on the weekly timeframe.

```pine
//@version=6
indicator("Counting intrabars using `request.security()`")

// @function    Calculates the quantity of 1D bars in a week of trading.
// @returns     (int) The number of intrabars within the current weekly bar up to the current moment.
qtyIntrabars() =>
    var int count = 0
    count := timeframe.change("W") ? 1 : count + 1

int qtyIntrabars = request.security(syminfo.tickerid, "1D", qtyIntrabars())

plot(qtyIntrabars, "qtyIntrabars", style=plot.style_histogram)
```

When using the [request.security()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.security) function on a lower timeframe, all calculations that reference individual LTF bars must be done _within the requested context_, and only the _result_ of the calculation is returned. Using the [request.security\_lower\_tf()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.security_lower_tf) function for intrabar analysis is usually easier and more powerful, because it returns an [array](https://www.tradingview.com/pine-script-reference/v6/#type_array) of data from _all_ available intrabars within a chart bar. Returning the data for each bar allows scripts to perform calculations on specific bars or all bars in the main script context.

In the following version of our example script, we use [request.security\_lower\_tf()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.security_lower_tf) to perform the same calculations. With this approach, we do not need to explicitly define the current chart’s timeframe, nor do we need a custom function.

```pine
//@version=6
indicator("Counting intrabars using `request.security_lower_tf()`")

// Count the number of elements in the array of close prices for each LTF bar in the current chart's bar.
int qtyIntrabars = array.size(request.security_lower_tf(syminfo.tickerid, "1D", close))

plot(qtyIntrabars, "qtyIntrabars", style=plot.style_histogram)
```

NoteAlthough this approach is simpler to implement than the previous, it is also more computationally expensive, because it retrieves a _new array_ instead of a single value on each execution. If either approach produces the same result — as is the case in our examples above — it is often more optimal to use the first approach, especially if the script is large and performs many intrabar calculations.

See the sections in the User Manual page “Other timeframes and Data” about [`request.security_lower_tf()`](https://www.tradingview.com/pine-script-docs/concepts/other-timeframes-and-data/#lower-timeframes) and using [`request.security()` on lower timeframes](https://www.tradingview.com/pine-script-docs/concepts/other-timeframes-and-data/#requestsecurity_lower_tf) to learn more about the differences between running these functions on a lower timeframe.

## How to avoid repainting when using the `request.security()` function? {#how-to-avoid-repainting-when-using-the-requestsecurity-function}

[Repainting](https://www.tradingview.com/pine-script-docs/concepts/repainting/) can be a problem when retrieving data from higher or lower timeframes using [request.security()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.security).

Retrieving data from a different symbol on the chart’s timeframe does not risk repainting. Requesting data from the chart’s own symbol and timeframe does not result in repainting either, but it is usually unnecessary to use [request.security()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.security) rather than simply referencing the chart’s own values (except when modifying the chart’s ticker using `ticker.*()` functions). When using the chart’s timeframe, there is no need to offset the data, change the default `lookahead` value, or use [barmerge.lookahead\_on](https://www.tradingview.com/pine-script-reference/v6/#const_barmerge.lookahead_on) in order to avoid repainting.

### Higher timeframes {#higher-timeframes}

Values from a _higher timeframe_ (HTF) often repaint because a [historical bar](https://www.tradingview.com/pine-script-docs/language/execution-model/#executions-on-historical-bars) on the chart might include data from a [realtime bar](https://www.tradingview.com/pine-script-docs/language/execution-model/#executions-on-realtime-bars) on the HTF. Realtime values can change throughout the bar; for example, the [close](https://www.tradingview.com/pine-script-reference/v6/#var_close) price reflects the _latest_ price update in a realtime bar. When the HTF bar closes and its values become fixed, the relevant historical chart bars _change_ to adjust to the fixed HTF values. This behavior is described in the [Historical and realtime behavior](https://www.tradingview.com/pine-script-docs/concepts/other-timeframes-and-data/#historical-and-realtime-behavior) section of the User Manual. Users expect historical bars not to change, which is one reason why repainting is such a concern.

To prevent repainting, use confirmed values that remain consistent across all bars. The most robust method is to offset all expressions by 1. For example, instead of `close`, which is equivalent to `close[0]`, use `close[1]`. The [request.security()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.security) call must also use [barmerge.lookahead\_on](https://www.tradingview.com/pine-script-reference/v6/#const_barmerge.lookahead_on). This method returns data that is up to one HTF bar “late”, and is thus not subject to change.

NoticeNeglecting to offset the `expression` argument in an HTF request causes **lookahead bias** on historical bars. See the [lookahead](https://www.tradingview.com/pine-script-docs/concepts/other-timeframes-and-data/#lookahead) section of the [Other timeframes and data](https://www.tradingview.com/pine-script-docs/concepts/other-timeframes-and-data/) page and the [Repainting](https://www.tradingview.com/pine-script-docs/concepts/repainting/) page’s section about [future leak](https://www.tradingview.com/pine-script-docs/concepts/repainting/#future-leak-with-requestsecurity) to learn more.

The following example script demonstrates the use of a single bar offset to the `expression` argument and [barmerge.lookahead\_on](https://www.tradingview.com/pine-script-reference/v6/#const_barmerge.lookahead_on) in [request.security()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.security) to ensure that the data behaves the same on historical and realtime bars. The script calls [runtime.error()](https://www.tradingview.com/pine-script-reference/v6/#fun_runtime.error) to trigger a custom runtime error if the chart’s timeframe exceeds or matches the daily timeframe, to prevent the return of inaccurate values.

```pine
//@version=6
indicator("HTF close" , overlay = true)
float dailyClose = request.security(syminfo.tickerid, "1D", close[1], lookahead = barmerge.lookahead_on)
plot(dailyClose)
if timeframe.in_seconds() >= timeframe.in_seconds("1D")
    runtime.error("Chart timeframe must be less than 1D.")
```

See the [Avoiding repainting](https://www.tradingview.com/pine-script-docs/concepts/other-timeframes-and-data/#avoiding-repainting) section of the User Manual for more information.

### Lower timeframes {#lower-timeframes}

Although the [request.security()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.security) function is intended to operate on timeframes greater than or equal to the chart timeframe, it _can_ request data from a [lower timeframe](https://www.tradingview.com/pine-script-docs/concepts/other-timeframes-and-data/#lower-timeframes) (LTF), with limitations. When accessing data from a LTF, the function evaluates the given expression in the LTF context and returns the result from a _single_ LTF bar per chart bar. The specific LTF bar returned depends on the [lookahead](https://www.tradingview.com/pine-script-docs/concepts/other-timeframes-and-data/#lookahead) parameter:

-   [barmerge.lookahead\_on](https://www.tradingview.com/pine-script-reference/v6/#const_barmerge.lookahead_on) returns the _first_ intrabar of the period historically, but the _last_ intrabar in realtime.
-   [barmerge.lookahead\_off](https://www.tradingview.com/pine-script-reference/v6/#const_barmerge.lookahead_off) always returns the last intrabar for both historical and realtime data. To prevent [repainting](https://www.tradingview.com/pine-script-docs/concepts/repainting/) (in this case, inconsistent results between realtime and historical data) use [barmerge.lookahead\_off](https://www.tradingview.com/pine-script-reference/v6/#const_barmerge.lookahead_off) for lower timeframe data requests.

In most cases, using the [request.security\_lower\_tf()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.security_lower_tf) function is more suitable for lower timeframes, as it returns an [array](https://www.tradingview.com/pine-script-docs/language/arrays/) containing data from _all_ available intrabars within a chart bar. See the section on [`request.security_lower_tf()`](https://www.tradingview.com/pine-script-docs/concepts/other-timeframes-and-data/#requestsecurity_lower_tf) to learn more.

## How can I convert the chart’s timeframe into a numeric format? {#how-can-i-convert-the-charts-timeframe-into-a-numeric-format}

The [timeframe.in\_seconds()](https://www.tradingview.com/pine-script-reference/v6/#fun_timeframe.in_seconds) function converts a timeframe specified in [timeframe.period](https://www.tradingview.com/pine-script-reference/v6/#var_timeframe.period) format into an equivalent number of seconds. Having the timeframe in a numeric format means that scripts can calculate the number of time units within a specific timeframe, or perform operations that adjust the timeframe used in HTF calls in relation to the chart’s timeframe, as described in [this FAQ entry](https://www.tradingview.com/pine-script-docs/faq/other-data-and-timeframes/#how-do-i-define-a-higher-timeframe-that-is-a-multiple-of-the-chart-timeframe).

In this script example, we use the [timeframe.in\_seconds()](https://www.tradingview.com/pine-script-reference/v6/#fun_timeframe.in_seconds) function to determine the chart’s timeframe, measured in seconds. Since no specific `timeframe` argument is specified, the function defaults to using `timeframe.period`, which returns the chart’s current timeframe. The script then converts the timeframe in seconds into various other units of time, including minutes, hours, and days, and displays the original string and converted numeric values in a [table](https://www.tradingview.com/pine-script-docs/visuals/tables/):

```pine
//@version=6
indicator("Timeframe to value")

tfInSec  = timeframe.in_seconds()
tfInMin  = tfInSec / 60
tfInHrs  = tfInMin / 60
tfInDays = tfInHrs / 24

if barstate.islastconfirmedhistory
    var table displayTable = table.new(position.top_right, 2, 5, na, color.gray, 1, color.gray, 1)
    table.cell(displayTable, 0, 0, "Original TF string",   text_color = chart.fg_color)
    table.cell(displayTable, 1, 0, "\"" + timeframe.period + "\"", text_color = chart.fg_color)
    table.cell(displayTable, 0, 1, "Timeframe in seconds", text_color = chart.fg_color)
    table.cell(displayTable, 1, 1, str.tostring(tfInSec),  text_color = chart.fg_color)
    table.cell(displayTable, 0, 2, "Timeframe in minutes", text_color = chart.fg_color)
    table.cell(displayTable, 1, 2, str.tostring(tfInMin),  text_color = chart.fg_color)
    table.cell(displayTable, 0, 3, "Timeframe in hours",   text_color = chart.fg_color)
    table.cell(displayTable, 1, 3, str.tostring(tfInHrs),  text_color = chart.fg_color)
    table.cell(displayTable, 0, 4, "Timeframe in days",    text_color = chart.fg_color)
    table.cell(displayTable, 1, 4, str.tostring(tfInDays), text_color = chart.fg_color)
```

## How can I convert a timeframe in “float” minutes into a string usable with `request.security()`? {#how-can-i-convert-a-timeframe-in-float-minutes-into-a-string-usable-with-requestsecurity}

The built-in function [timeframe.from\_seconds()](https://www.tradingview.com/pine-script-reference/v6/#fun_timeframe.from_seconds) function converts a number of seconds into a timeframe string that is compatible with [request.security()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.security).

The example script below converts a user-defined number of minutes into a timeframe string using the [timeframe.from\_seconds()](https://www.tradingview.com/pine-script-reference/v6/#fun_timeframe.from_seconds) function. The script then requests the close price from that timeframe using [request.security()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.security) and plots it. Additionally, we display the resulting timeframe string in a [table](https://www.tradingview.com/pine-script-docs/visuals/tables/) on the chart’s top right corner:

```pine
//@version=6
indicator("Target TF in string from float minutes", "", true)
float tfInMinInput = input.float(1440, "Minutes in target timeframe (<= 0.0167 [1 sec.])", minval = 0.0167)
// Convert target TF in minutes from input into string.
string targetTfString = timeframe.from_seconds(int(tfInMinInput * 60))
// Fetch target timeframe's close.
float targetTfClose = request.security(syminfo.tickerid, targetTfString, close)
// Plot target timeframe close.
plot(targetTfClose, "Target TF close")
// Display the target timeframe string in a table cell at the chart's top right.
if barstate.islastconfirmedhistory
    var table displayTable = table.new(position.top_right, 1, 1, color.new(color.yellow, 70), color.gray, 1, color.gray, 1)
    table.cell(displayTable, 0, 0, str.format("Target TF (string): {0}", targetTfString), text_color = chart.fg_color)
```

## How do I define a higher timeframe that is a multiple of the chart timeframe? {#how-do-i-define-a-higher-timeframe-that-is-a-multiple-of-the-chart-timeframe}

This example script uses the [timeframe.in\_seconds()](https://www.tradingview.com/pine-script-reference/v6/#fun_timeframe.in_seconds) and [timeframe.from\_seconds()](https://www.tradingview.com/pine-script-reference/v6/#fun_timeframe.from_seconds) functions to calculate a higher timeframe that is a fixed multiple of the chart’s current timeframe. Using the input for the multiplier, the user can define the ratio between the chart’s timeframe and the higher timeframe. The script then calculates the [Relative Strength Index](https://www.tradingview.com/support/solutions/43000502338-relative-strength-index-rsi/) (RSI) for both the chart’s timeframe and the higher timeframe, plotting both in a separate pane for comparison. We display the calculated higher timeframe string in a [table](https://www.tradingview.com/pine-script-docs/visuals/tables/) on the main chart pane by using `force_overlay`:

```pine
//@version=6
indicator("Multiple of current TF", overlay = false)

// Provide an input to specify the multiple to apply to the chart's timeframe.
float tfMult = input.float(4, minval = 1)

// Get multiple of current timeframe.
string targetTfString = timeframe.from_seconds(int(timeframe.in_seconds() * tfMult))
// Create RSI from the current timeframe.
float myRsi = ta.rsi(close, 14)
plot(myRsi, "Current TF RSI", color = color.silver)
// Non-repainting HTF RSI.
float myRsiHtf = request.security(syminfo.tickerid, targetTfString, myRsi[1], lookahead = barmerge.lookahead_on)
plot(myRsiHtf, "Non-repainting HTF RSI", color = color.green)

// Display the calculated timeframe at the top right of the main chart pane. 
if barstate.islastconfirmedhistory
    var table displayTable = table.new(position.top_right, 1, 1, color.new(color.yellow, 70), color.gray, 1, color.gray, 1, force_overlay = true)
    table.cell(displayTable, 0, 0, str.format("Target TF (string): {0}", targetTfString), text_color = chart.fg_color)
```

## How can I plot a moving average only when the chart’s timeframe is 1D or higher? {#how-can-i-plot-a-moving-average-only-when-the-charts-timeframe-is-1d-or-higher}

To plot a moving average on a chart only if it has a timeframe of daily (“1D”) or higher, scripts can use the [timeframe.in\_seconds()](https://www.tradingview.com/pine-script-reference/v6/#fun_timeframe.in_seconds) function to convert the chart’s current timeframe into seconds. Since a day consists of 86400 seconds, any timeframe equal to or exceeding this value corresponds to a daily or longer duration.

The example script below calculates and plots a [Simple Moving Average](https://www.tradingview.com/support/solutions/43000696841-simple-moving-average/) (SMA) of the closing prices over the last 200 bars. The script uses a [ternary operator](https://www.tradingview.com/pine-script-docs/language/operators/#-ternary-operator) to return the moving average on timeframes of 1D or greater, or [na](https://www.tradingview.com/pine-script-reference/v6/#var_na) if the timeframe is shorter than one day. Because [plot()](https://www.tradingview.com/pine-script-reference/v6/#fun_plot) calls cannot be in a local scope, scripts cannot conditionally call this function. Passing an [na](https://www.tradingview.com/pine-script-reference/v6/#var_na) value as the `series` argument is an effective way to not plot anything. Note that plotting an [na](https://www.tradingview.com/pine-script-reference/v6/#var_na) value _does_ count towards the script’s [plot limit](https://www.tradingview.com/pine-script-docs/writing/limitations/#plot-limits).

```pine
//@version=6
indicator("Timeframe-dependent MA", overlay = true)
bool tfIsDailyOrGreater = timeframe.in_seconds() >= 86400
float ma = ta.sma(close, 200)
plot(tfIsDailyOrGreater ? ma : na, "MA", color.aqua)
```

## What happens if I plot a moving average from the 1H timeframe on a different timeframe? {#what-happens-if-i-plot-a-moving-average-from-the-1h-timeframe-on-a-different-timeframe}

The [request.security()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.security) function can access data from a different _context_, such as a different symbol or timeframe. There are different considerations when accessing data from a timeframe _higher_ or _lower_ than the chart timeframe.

First, let’s consider an example of plotting data from a _lower_ timeframe. The following script plots a 21-period [Exponential Moving Average](https://www.tradingview.com/support/solutions/43000592270-exponential-moving-average/) (EMA) derived from the 1-hour (1H) timeframe on any chart, irrespective of the timeframe of that chart:

```pine
//@version=6
indicator("1hr EMA", overlay = true)
plot(request.security(syminfo.tickerid, "60", ta.ema(close, 21)), color = color.orange)
```

Assuming that we run this script on a chart with a daily timeframe, we encounter the following problems:

-   For each daily bar, the chart can plot only 1 of the 24 MA values theoretically available. The plot misses out the intraday fluctuations and trends that a 1H moving average (MA) is typically used to identify.
-   The script above displays only the EMA value calculated for the _final_ 1-hour bar of each day. In realtime, the plot displays the most recently known value.

Unlike [request.security()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.security), the [request.security\_lower\_tf()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.security_lower_tf) function is intended for use on lower timeframes. It returns an [array](https://www.tradingview.com/pine-script-docs/language/arrays/) containing data from all available intrabars within a chart bar. See [this section](https://www.tradingview.com/pine-script-docs/concepts/other-timeframes-and-data/#requestsecurity_lower_tf) of the User Manual to learn more.

We could rewrite the script to use [request.security\_lower\_tf()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.security_lower_tf), but plotting a moving average from a lower timeframe is still not very practical.

A more logical approach is to plot MAs from a _higher_ timeframe. This strategy shows broader market trends within the context of shorter-term price movements. For example, plotting a daily MA on a 1H chart provides insights into how intraday prices are trending relative to the longer-term daily average.

In the following example script, we plot the 21 EMA calculated at the 1H timeframe, but only when the chart’s timeframe is _equal to or lower than_ 1H. We call the [request.security()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.security) function [in the recommended way](https://www.tradingview.com/pine-script-docs/faq/other-data-and-timeframes/#higher-timeframes) to avoid repainting.

```pine
//@version=6
indicator("HTF EMA", overlay = true)

// Input to specify the timeframe for `request.security() call.
string tfinput      = input.timeframe("60", "Timeframe for MA")

// @function            A wrapper for the `request.security()` function for non-repainting calls to HTFs.
// @param timeframe     Timeframe of the requested data. 
//                      To use the chart's timeframe, use an empty string or the `timeframe.period` variable.
// @param expression    An expression to calculate and returne from the request.security() call's context.
// @returns             The result of the calculated expression.
htfSecurity(string timeframe, expression) =>
    result = request.security(syminfo.tickerid, timeframe, expression[1], lookahead = barmerge.lookahead_on)

// Calculate the moving average in the chart context.
float ma = ta.ema(close, 21)
// Calculate the moving average in the specified `tfInput` timeframe.
float htfMA = htfSecurity(tfinput, ma)

// Check whether the requested timeframe is greater or less than the chart's timeframe.
bool tfIsGreater = timeframe.in_seconds() < timeframe.in_seconds(tfinput)
bool tfIsLess    = timeframe.in_seconds() > timeframe.in_seconds(tfinput)

// Plot the HTF MA, the chart MA, or nothing, depending on the timeframe.
float maPlot = tfIsGreater ? htfMA : tfIsLess ? na : ma
plot(maPlot, "Requested MA", color.orange)

// Display a message in a table indicating that the requested timeframe is lower than the chart's timeframe, if applicable.
if barstate.islastconfirmedhistory and tfIsLess
    var table displayTable = table.new(position.bottom_right, 1, 1, color.new(color.yellow, 70))
    table.cell(displayTable, 0, 0, "Requested TF is lower than chart's TF\nNo MA displayed", text_color = color.red)
```

## Why do intraday price and volume values differ from values retrieved with `request.security()` at daily timeframes and higher? {#why-do-intraday-price-and-volume-values-differ-from-values-retrieved-with-requestsecurity-at-daily-timeframes-and-higher}

Intraday [open](https://www.tradingview.com/pine-script-reference/v6/#var_open), [high](https://www.tradingview.com/pine-script-reference/v6/#var_high), [low](https://www.tradingview.com/pine-script-reference/v6/#var_low), [close](https://www.tradingview.com/pine-script-reference/v6/#var_close), and [volume](https://www.tradingview.com/pine-script-reference/v6/#var_volume) (OHLCV) values can be different from those from [request.security()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.security) at daily timeframes and higher for a number of reasons, including the following:

-   **Different data feeds:** Certain trades (like block trades and OTC trades, especially in stocks) are recorded only at the end of the trading day, so their volume affects the End-of-Day (EOD) feed but not the intraday feed.
-   **Price discrepancies:** There can be slight differences in prices between EOD and intraday data. For example, an EOD high might not match any intraday highs due to variations in data feeds.
-   **Extended hours data:** EOD data feeds do not include information from trading outside regular hours, unlike some intraday feeds. For instance, the bars of an hourly chart might straddle the open of a session, mixing data from pre-market and regular trading.

For an extended list of factors with detailed explanations, refer to the [Data feeds](https://www.tradingview.com/pine-script-docs/concepts/other-timeframes-and-data/#data-feeds) section in the User Manual.
