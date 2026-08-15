---
title: Chart information
source: https://www.tradingview.com/pine-script-docs/concepts/chart-information/
section: concepts
---

# Chart information {#chart-information}

## Introduction {#introduction}

Scripts can retrieve multiple types of information about the current chart and its dataset by using a subset of [built-in variables](https://www.tradingview.com/pine-script-docs/language/built-ins/#built-in-variables). The chart data that scripts can access using these variables includes the following:

-   The available [prices and volume](https://www.tradingview.com/pine-script-docs/concepts/chart-information/#prices-and-volume)
-   The chart’s [timeframe](https://www.tradingview.com/pine-script-docs/concepts/chart-information/#chart-timeframe)
-   The dataset’s [session information](https://www.tradingview.com/pine-script-docs/concepts/chart-information/#session-information)
-   [Symbol information](https://www.tradingview.com/pine-script-docs/concepts/chart-information/#symbol-information)
-   [Time series information](https://www.tradingview.com/pine-script-docs/concepts/chart-information/#time-series-information)
-   The chart’s [type and color](https://www.tradingview.com/pine-script-docs/concepts/chart-information/#chart-type-and-color)

The following sections on this page list the variables that can access chart information, along with examples demonstrating how to use them. To learn more about all the built-in variables available in Pine Script®, refer to the [Built-ins](https://www.tradingview.com/pine-script-docs/language/built-ins/) page in this manual and the “Variables” section of our [Reference Manual](https://www.tradingview.com/pine-script-reference/v6/). 

> [!NOTE]
> Several variables described on this page behave differently in _data requests_. For most of these variables, if a script uses them in the `expression` arguments of `request.*()` calls, or if the script is an indicator that includes a `timeframe` argument in its [declaration statement](https://www.tradingview.com/pine-script-docs/language/declaration-statements/), they represent information from the _requested dataset_ rather than the current chart. For example, if a [request.security()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.security) call includes `"NASDAQ:AAPL"` as its `symbol` argument, the value of the [syminfo.prefix](https://www.tradingview.com/pine-script-reference/v6/#var_syminfo.prefix) variable is `"NASDAQ"` inside that request’s context, regardless of the symbol used by the current chart. Likewise, the [close](https://www.tradingview.com/pine-script-reference/v6/#var_close) value inside the request refers to the share price for NASDAQ:AAPL stock. See the [Other timeframes and data](https://www.tradingview.com/pine-script-docs/concepts/other-timeframes-and-data/) page to learn more.

## Prices and volume {#prices-and-volume}

Most chart datasets include _OHLCV_ (open, high, low, close, and volume) values for each available bar. The chart displays the _final_ values for each _closed_ bar, and the _developing_ values for an _open realtime bar_. See the section [The basics](https://www.tradingview.com/pine-script-docs/language/execution-model/#the-basics) in the [Execution model](https://www.tradingview.com/pine-script-docs/language/execution-model/) page to learn more about this behavior.

The variables that store final or developing OHLCV data for the current bar are as follows:

-   [open](https://www.tradingview.com/pine-script-reference/v6/#var_open): Stores the current bar’s opening price. The value does not fluctuate while the bar is open.
-   [high](https://www.tradingview.com/pine-script-reference/v6/#var_high): Stores the current bar’s highest price. If the bar is open, the value represents the bar’s highest price as of the current tick.
-   [low](https://www.tradingview.com/pine-script-reference/v6/#var_low): Stores the current bar’s lowest price. If the bar is open, the value represents the bar’s lowest price as of the current tick.
-   [close](https://www.tradingview.com/pine-script-reference/v6/#var_close): Stores the current bar’s final closing price, or the _latest_ available price if the current bar is open.
-   [volume](https://www.tradingview.com/pine-script-reference/v6/#var_volume): Stores the trading volume reported for the current bar. If the bar is open, the value represents the total volume accumulated from the bar’s opening tick to the current tick. The unit that the volume value uses varies by instrument. For example, the unit is typically shares for stocks, lots for Forex pairs, the base currency for cryptocurrency pairs, and contracts for futures and other derivatives.

Pine Script also includes multiple variables that store values _derived_ from available OHLC data, including the following:

-   [hl2](https://www.tradingview.com/pine-script-reference/v6/#var_hl2): Stores the average of the bar’s [high](https://www.tradingview.com/pine-script-reference/v6/#var_high) and [low](https://www.tradingview.com/pine-script-reference/v6/#var_low) values (`(high + low) / 2`).
-   [hlc3](https://www.tradingview.com/pine-script-reference/v6/#var_hlc3): Stores the average of the bar’s [high](https://www.tradingview.com/pine-script-reference/v6/#var_high), [low](https://www.tradingview.com/pine-script-reference/v6/#var_low), and [close](https://www.tradingview.com/pine-script-reference/v6/#var_close) values (`(high + low + close) / 3`).
-   [ohlc4](https://www.tradingview.com/pine-script-reference/v6/#var_ohlc4): Stores the average of the bar’s [open](https://www.tradingview.com/pine-script-reference/v6/#var_open), [high](https://www.tradingview.com/pine-script-reference/v6/#var_high), [low](https://www.tradingview.com/pine-script-reference/v6/#var_low), and [close](https://www.tradingview.com/pine-script-reference/v6/#var_close) values (`(open + high + low + close) / 4`).
-   [hlcc4](https://www.tradingview.com/pine-script-reference/v6/#var_hlcc4): Stores a weighted average of the bar’s [high](https://www.tradingview.com/pine-script-reference/v6/#var_high), [low](https://www.tradingview.com/pine-script-reference/v6/#var_low), and [close](https://www.tradingview.com/pine-script-reference/v6/#var_close) values (`(high + low + close + close) / 4`).

> [!NOTE]
> If a bar contains only a _single_ price rather than complete OHLC prices, each of these price-based variables stores that value. If no volume data is available for a bar, the value of the [volume](https://www.tradingview.com/pine-script-reference/v6/#var_volume) variable is [na](https://www.tradingview.com/pine-script-reference/v6/#var_na).
>
> Some chart types do not _display_ OHLC prices, but their datasets still _contain_ those prices. For example, a [line chart](https://www.tradingview.com/support/solutions/43000745271/) displays only one price per bar. However, the variables that access bar prices still use the available OHLC values from the underlying dataset.
>
> On _non-standard_ charts, such as [Heikin Ashi](https://www.tradingview.com/support/solutions/43000619436/) or [Renko](https://www.tradingview.com/support/solutions/43000502284/), variables that access price data store the chart’s _synthetic_ prices, not the instrument’s _actual_ prices. Therefore, logic that uses these variables can yield different results on these charts.

On tick charts that use the “1T” timeframe, scripts can also use the [bid](https://www.tradingview.com/pine-script-reference/v6/#var_bid) and [ask](https://www.tradingview.com/pine-script-reference/v6/#var_ask) variables to access the current _bid and ask_ prices. The _bid_ is the _highest_ price that an active buyer is willing to pay for the instrument at its current value, and the _ask_ is the _lowest_ price that an active seller is willing to accept at the current value. On timeframes higher than “1T”, the value of these variables is [na](https://www.tradingview.com/pine-script-reference/v6/#var_na).

All of these price and volume variables are of the “series float” [qualified type](https://www.tradingview.com/pine-script-docs/language/type-system/), because they store floating-point values that can vary from bar to bar. Scripts can use the [`[]` history-referencing operator](https://www.tradingview.com/pine-script-docs/language/operators/#-history-referencing-operator) to retrieve the past values of these variables from previous bars. For example, the expression `close[1]` retrieves the _previous bar’s_ closing price. Multiple [built-in functions](https://www.tradingview.com/pine-script-docs/language/built-ins/#built-in-functions) also access past values internally. For instance, the expression `ta.change(ohlc4, 20)` is equivalent to `ohlc4 - ohlc4[20]`; both expressions calculate the difference between the current [ohlc4](https://www.tradingview.com/pine-script-reference/v6/#var_ohlc4) value and the value from _20 bars back_.

The following example uses the prices and volume of current and previous bars on the chart to calculate a condition for a dynamic background color. The script colors the chart’s background green only if the current values of the [volume](https://www.tradingview.com/pine-script-reference/v6/#var_volume) and [close](https://www.tradingview.com/pine-script-reference/v6/#var_close) variables are greater than the previous values, and the current [close](https://www.tradingview.com/pine-script-reference/v6/#var_close) value is greater than its 10-bar moving average. The script also plots the moving average for visual reference:

```pine
//@version=6
indicator("Price and volume variables demo", overlay = true, behind_chart = false)

//@variable The 10-bar moving average of the `close` series.
float ma = ta.sma(close, 10)

//@variable Is `true` if the current volume is greater than the previous bar's volume, and `false` otherwise. 
bool risingVolume = volume > volume[1]
//@variable Is `true` if the current price is greater than the closing price of the previous bar, and `false` otherwise. 
bool risingPrice = close > close[1]
//@variable Is `true` if the current price is above the current `ma` value, and `false` otherwise. 
bool closeAboveMA = close > ma

// Plot the `ma` series on the chart. 
plot(ma, "10-bar MA", linewidth = 3)
// Highlight the background in green when all three conditions are true.
bgcolor(risingVolume and risingPrice and closeAboveMA ? #4caf5080 : na, title = "Condition highlight")
```

## Chart timeframe {#chart-timeframe}

Scripts can retrieve the _timeframe_ of the current chart by using the [timeframe.period](https://www.tradingview.com/pine-script-reference/v6/#var_timeframe.period) or [timeframe.main\_period](https://www.tradingview.com/pine-script-reference/v6/#var_timeframe.main_period) variable. Both variables hold a “simple string” value representing the analyzed timeframe:

-   The value of the [timeframe.period](https://www.tradingview.com/pine-script-reference/v6/#var_timeframe.period) variable represents the timeframe of a specific context. If used outside the `expression` argument of a `request.*()` call, the value represents the _chart’s timeframe_, or the script’s _main timeframe_ if the script is an indicator whose declaration statement includes a `timeframe` argument. When used in the `expression` argument of a `request.*()` call, the value represents the timeframe of the _requested dataset_.
-   The value of the [timeframe.main\_period](https://www.tradingview.com/pine-script-reference/v6/#var_timeframe.main_period) variable _always_ represents the chart’s timeframe or the script’s main timeframe, even if the script uses it inside a `request.*()` call. This behavior is often useful for [nested requests](https://www.tradingview.com/pine-script-docs/concepts/other-timeframes-and-data/#nested-requests) that require information from the chart’s timeframe in their logic.

The timeframe strings stored by these variables contain a number representing a _quantity (multiplier)_ followed by a single letter representing the _time unit_. For all intraday timeframes that Pine expresses in _minutes_, the timeframe string contains a multiplier _without_ a unit postfix. For example, `"1D"` represents the one-day timeframe, `"5"` represents the five-minute timeframe, `"60"` represents the one-hour (60-minute) timeframe, and `"3M"` represents the three-month timeframe. See the [Timeframe string specifications](https://www.tradingview.com/pine-script-docs/concepts/timeframes/#timeframe-string-specifications) section of the [Timeframes](https://www.tradingview.com/pine-script-docs/concepts/timeframes/) page to learn more.

Multiple built-in functions feature a `timeframe` parameter that accepts a valid timeframe string. Scripts can pass the [timeframe.period](https://www.tradingview.com/pine-script-reference/v6/#var_timeframe.period) or [timeframe.main\_period](https://www.tradingview.com/pine-script-reference/v6/#var_timeframe.main_period) variable to this parameter to use the chart’s timeframe in the calculations.

For example, the following script uses the [timeframe.period](https://www.tradingview.com/pine-script-reference/v6/#var_timeframe.period) variable in calls to the [`time()` and `time_close()` functions](https://www.tradingview.com/pine-script-docs/concepts/time/#time-and-time_close-functions) to retrieve the [UNIX timestamps](https://www.tradingview.com/pine-script-docs/concepts/time/#unix-timestamps) of the current bar’s opening time and the previous bar’s closing time, then measures the difference between the two timestamps to identify time gaps in the chart’s bars. It also uses the variable in a call to [timeframe.in\_seconds()](https://www.tradingview.com/pine-script-reference/v6/#fun_timeframe.in_seconds) to retrieve the typical number of seconds represented by the timeframe, then uses the result to express the time difference as an approximate number of bars. Each time that the script detects a gap, it displays [formatted text](https://www.tradingview.com/pine-script-docs/concepts/strings/#formatting-strings) containing the gap’s size in minutes and bars, the [timeframe.period](https://www.tradingview.com/pine-script-reference/v6/#var_timeframe.period) value, and the number of bars elapsed since the previous gap in a [label](https://www.tradingview.com/pine-script-docs/visuals/text-and-shapes/#labels) at the current bar’s high:

```pine
//@version=6
indicator("`timeframe.period` demo", overlay = true, behind_chart = false)

//@variable The previous bar's closing UNIX timestamp.
int prevCloseTime = time_close(timeframe = timeframe.period, timeframe_bars_back = 1)
//@variable The current bar's opening UNIX timestamp.
int currOpenTime = time(timeframe = timeframe.period)

//@variable The number of seconds elapsed between the `prevCloseTime` and `currOpenTime` timestamps.
int timeDiff = (currOpenTime - prevCloseTime) / 1000
//@variable The approximate span of the time difference in bars.
int gapBarLength = int(timeDiff / timeframe.in_seconds(timeframe.period))
//@variable Is `true` if the difference is greater than zero.
bool hasGap = timeDiff > 0
//@variable The number of bars since the `hasGap` value was last `true`. 
int barsSinceLastGap = ta.barssince(hasGap)

// If a time gap occurs, display the gap in minutes, the current timeframe, and the bars since the previous gap
// in a label at the current bar's high.
if hasGap
    label.new(
        bar_index, high, 
        text = str.format(
            "{0}-minute gap (~{1} bars) on ''{2}'' timeframe\nBars since previous gap: {3}", 
            timeDiff / 60, gapBarLength, timeframe.period, barsSinceLastGap[1] + 1
        )
    )
```

Note that:

-   Programmers can also use an _empty string_ (`""`) as a `timeframe` argument to specify the same timeframe as [timeframe.period](https://www.tradingview.com/pine-script-reference/v6/#var_timeframe.period). For instance, our example script yields the same results if we use `""` instead of the variable in the [time()](https://www.tradingview.com/pine-script-reference/v6/#fun_time), [time\_close()](https://www.tradingview.com/pine-script-reference/v6/#fun_time_close), and [timeframe.in\_seconds()](https://www.tradingview.com/pine-script-reference/v6/#fun_timeframe.in_seconds) calls.

Scripts can use the [timeframe.multiplier](https://www.tradingview.com/pine-script-reference/v6/#var_timeframe.multiplier) variable to retrieve a “simple int” value representing the _multiplier_ of the timeframe referenced by [timeframe.period](https://www.tradingview.com/pine-script-reference/v6/#var_timeframe.period). For example, if the timeframe is “2D”, the [timeframe.multiplier](https://www.tradingview.com/pine-script-reference/v6/#var_timeframe.multiplier) value is `2`. If the timeframe is “30S”, the variable’s value is `30`.

The following `timeframe.*` variables store “simple bool” values to indicate the _unit_ of the timeframe referenced by [timeframe.period](https://www.tradingview.com/pine-script-reference/v6/#var_timeframe.period):

-   [timeframe.isticks](https://www.tradingview.com/pine-script-reference/v6/#var_timeframe.isticks): Stores `true` if the current timeframe is a tick-based timeframe (e.g., `"10T"`), and `false` otherwise.
-   [timeframe.isseconds](https://www.tradingview.com/pine-script-reference/v6/#var_timeframe.isseconds): Stores `true` if the current timeframe is a second-based timeframe (e.g., `"30S"`), and `false` otherwise.
-   [timeframe.isminutes](https://www.tradingview.com/pine-script-reference/v6/#var_timeframe.isminutes): Stores `true` if the current timeframe is an intraday timeframe in minutes (`"1"` to `"1440"`), and `false` otherwise.
-   [timeframe.isintraday](https://www.tradingview.com/pine-script-reference/v6/#var_timeframe.isintraday): Stores `true` if the current timeframe is any intraday timeframe (minutes, seconds, or ticks), and `false` otherwise.
-   [timeframe.isdaily](https://www.tradingview.com/pine-script-reference/v6/#var_timeframe.isdaily): Stores `true` if the current timeframe is day-based (`"1D"` to `"365D"`), and `false` otherwise.
-   [timeframe.isweekly](https://www.tradingview.com/pine-script-reference/v6/#var_timeframe.isweekly): Stores `true` if the current timeframe is week-based (`"1W"` to `"52W"`), and `false` otherwise.
-   [timeframe.ismonthly](https://www.tradingview.com/pine-script-reference/v6/#var_timeframe.ismonthly): Stores `true` if the current timeframe is month-based (`"1M"` to `"12M"`), and `false` otherwise.
-   [timeframe.isdwm](https://www.tradingview.com/pine-script-reference/v6/#var_timeframe.isdwm): Stores `true` if the current timeframe is expressed in days, weeks, or months, and `false` otherwise.

The example below uses these variables to construct a custom representation of the chart’s timeframe. On the first bar, the script uses multiple `timeframe.is*` variables in a [switch](https://www.tradingview.com/pine-script-reference/v6/#kw_switch) statement to retrieve a string representing the chart timeframe’s unit, then creates a formatted string using the result and the value of [timeframe.multiplier](https://www.tradingview.com/pine-script-reference/v6/#var_timeframe.multiplier). It displays the final text in a single-cell [table](https://www.tradingview.com/pine-script-docs/visuals/tables/) in the chart’s top-right corner:

```pine
//@version=6
indicator("Timeframe multiplier and unit variables demo", overlay = true, behind_chart = false)

//@variable References a single-cell `table` object that displays the chart's timeframe in the top-right corner.
var table displayTable = table.new(position.top_right, 1, 1, color.blue)

if barstate.isfirst
    //@variable A string that describes the unit of the chart's timeframe.
    string unitsStr = switch
        timeframe.isticks   => "tick"
        timeframe.isseconds => "second"
        timeframe.isminutes => "minute"
        timeframe.isdaily   => "day"
        timeframe.isweekly  => "week"
        => "month"
    //@variable A formatted string that contains the timeframe's multiplier and unit descriptor.
    string displayStr = str.format(
        "Chart timeframe: {0,number,#} {1}{2}", 
        timeframe.multiplier, unitsStr, timeframe.multiplier > 1 ? "s" : ""
    )
    // Initialize the table's cell to display the text from the `displayStr` value. 
    displayTable.cell(0, 0, displayStr, text_color = color.white, text_size = 24)
```

Refer to the [Timeframes](https://www.tradingview.com/pine-script-docs/concepts/timeframes/) page to learn more about the `timeframe.*` built-ins and how to use them.

## Session information {#session-information}

Pine Script includes multiple built-in variables that can retrieve information about an intraday dataset’s _session_, which refers to the days and the times of day in which trading data is available. These variables represent session information for the current chart’s dataset, or for a requested dataset if the script uses them in the `expression` argument of a `request.*()` function call.

Scripts can access the [named session](https://www.tradingview.com/pine-script-docs/concepts/sessions/#named-sessions) for the current chart’s dataset by using the [syminfo.session](https://www.tradingview.com/pine-script-reference/v6/#var_syminfo.session) variable. The variable holds a “simple string” value representing the session’s name. In most cases, the string matches the value of either of the following `session.*` constants by default:

-   [session.regular](https://www.tradingview.com/pine-script-reference/v6/#const_session.regular): Stores the string for the instrument’s default trading session (`"regular"`). The default session varies with the instrument. For instance, on the charts for US equities, the string typically corresponds to regular trading hours (RTH). By contrast, on the charts for several futures contracts, it corresponds to electronic trading hours (ETH), because ETH is enabled by default. On timeframes higher than or equal to 1D, the values of [syminfo.session](https://www.tradingview.com/pine-script-reference/v6/#var_syminfo.session) and [session.regular](https://www.tradingview.com/pine-script-reference/v6/#const_session.regular) are always equal.
-   [session.extended](https://www.tradingview.com/pine-script-reference/v6/#const_session.extended): Stores the string for the instrument’s extended session (`"extended"`), which includes data from pre- and post-market hours. Outside data requests, the [syminfo.session](https://www.tradingview.com/pine-script-reference/v6/#var_syminfo.session) variable holds this string only if the chart includes the option for extended sessions and the user selects that option in the chart’s “Session” settings.

The [syminfo.session](https://www.tradingview.com/pine-script-reference/v6/#var_syminfo.session) variable can also hold _other_ unique strings for specific subsessions defined by the exchange or data provider. For instance, the value is `"us_regular"` on a CME futures chart that uses the RTH session, and `"24h"` on an equities chart that includes _overnight (24-hour)_ sessions. Refer to the [Retrieving named sessions](https://www.tradingview.com/pine-script-docs/concepts/sessions/#retrieving-named-sessions) section of the [Sessions](https://www.tradingview.com/pine-script-docs/concepts/sessions/) page to learn more about named session strings.

Programmers can use the string from this variable to create session-specific logic in their scripts, or pass the string to the `session` parameter of the [ticker.new()](https://www.tradingview.com/pine-script-reference/v6/#fun_ticker.new) or [ticker.modify()](https://www.tradingview.com/pine-script-reference/v6/#fun_ticker.modify) functions to create ticker identifiers for requesting data using the same session as the chart. See the [Custom contexts](https://www.tradingview.com/pine-script-docs/concepts/other-timeframes-and-data/#custom-contexts) section of the [Other timeframes and data](https://www.tradingview.com/pine-script-docs/concepts/other-timeframes-and-data/) page for more information about these `ticker.*()` functions.

Additional variables in the `session` namespace hold “series bool” values that indicate the current [market state](https://www.tradingview.com/pine-script-docs/concepts/sessions/#market-states) or track the [first and last bars](https://www.tradingview.com/pine-script-docs/concepts/sessions/#first-and-last-bars) in named sessions:

-   [session.ismarket](https://www.tradingview.com/pine-script-reference/v6/#var_session.ismarket): Stores `true` if the current bar belongs to the _regular_ (default) session, and `false` otherwise. The value is typically always `true` on timeframes higher than or equal to 1D.
-   [session.ispremarket](https://www.tradingview.com/pine-script-reference/v6/#var_session.ispremarket): Stores `true` if the current bar is a _pre-market_ bar, and `false` otherwise. The value is always `false` on timeframes higher than or equal to 1D.
-   [session.ispostmarket](https://www.tradingview.com/pine-script-reference/v6/#var_session.ispostmarket): Stores `true` if the current bar is a _post-market_ bar, and `false` otherwise. The value is always `false` on timeframes higher than or equal to 1D.
-   [session.isfirstbar](https://www.tradingview.com/pine-script-reference/v6/#var_session.isfirstbar): Stores `true` if the current bar is the first bar of the daily session, and `false` otherwise. If the dataset uses only the regular session, the value is `true` on the first bar in that session. If the dataset uses extended sessions, the value is `true` on the first _pre-market_ bar. If the dataset uses 24-hour sessions, the value is `true` on the first _overnight_ bar.
-   [session.isfirstbar\_regular](https://www.tradingview.com/pine-script-reference/v6/#var_session.isfirstbar_regular): Stores `true` if the current bar is the first bar of the instrument’s _regular_ session, and `false` otherwise, regardless of the dataset’s session settings.
-   [session.islastbar](https://www.tradingview.com/pine-script-reference/v6/#var_session.islastbar): Stores `true` if the current bar is the last bar of the daily session, and `false` otherwise. If the dataset uses only regular trading hours, the value is `true` on the last bar in that session. If the dataset uses extended or overnight sessions, the value is `true` on the last _post-market_ bar.
-   [session.islastbar\_regular](https://www.tradingview.com/pine-script-reference/v6/#var_session.islastbar_regular): Stores `true` if the current bar is the last bar in the instrument’s regular session, and `false` otherwise, regardless of the dataset’s session settings.

The following example demonstrates the behavior of these variables. The script below calculates and plots the total volume for each subsession on an intraday chart that includes extended or overnight sessions. The script declares four persistent variables to store the total volume for regular, pre-market, post-market, and overnight hours. Then, inside the [if](https://www.tradingview.com/pine-script-reference/v6/#kw_if) structure, it uses [session.ismarket](https://www.tradingview.com/pine-script-reference/v6/#var_session.ismarket), [session.ispremarket](https://www.tradingview.com/pine-script-reference/v6/#var_session.ispremarket), and [session.ispostmarket](https://www.tradingview.com/pine-script-reference/v6/#var_session.ispostmarket) as conditions for resetting or incrementing the value of each variable based on the current session state. The script also uses one of the `session.isfirstbar*` or `session.islastbar*` variables, depending on the selected inputs, as a condition to color the background of specific session bars. Additionally, the script checks the value of the [syminfo.session](https://www.tradingview.com/pine-script-reference/v6/#var_syminfo.session) variable to confirm that these calculations are compatible with the chart. It raises a custom _runtime error_ if the value is not `"extended"` or `"24h"`, indicating that the chart is day-based or does not use the “Extended” or “24 hour” session setting:

```pine
//@version=6
indicator("`syminfo.session` and `session.*` demo", format = format.volume)

// Create inputs for highlighting the first or last bar in a given session type.
string firstLastInput  = input.string("First", "Highlight", ["First", "Last"], inline = "0")
string regularExtendInput = input.string(
    "Regular Session", "Bar In ", ["Regular Session", "Extended/Overnight Hours"], inline = "0"
)

// Raise an error if the current chart does not use an "Extended" or "24 hour" session.
if not (syminfo.session == session.extended or syminfo.session == "24h")
    runtime.error("Open an intraday chart with an 'Extended' or '24 hour' session to use this script.")

// Declare persistent variables to store the total volume for regular, pre-market, post-market, and overnight hours.
var float regularTotal    = 0.0
var float premarketTotal  = 0.0
var float postmarketTotal = 0.0
var float overnightTotal  = 0.0

// For each session, accumulate the current session's total and reset the previous session's total to 0.
if session.ismarket
    regularTotal += volume
    premarketTotal := 0.0
else if session.ispremarket
    premarketTotal += volume
    postmarketTotal := 0.0
    overnightTotal := 0.0
else if session.ispostmarket
    postmarketTotal += volume
    regularTotal := 0.0
else
    overnightTotal += volume
    postmarketTotal := 0.0

//@variable Is `true` for the first or last bar in the specified session type, and `false` for all others. 
bool highlightBar = if firstLastInput == "First"
    regularExtendInput == "Regular Session" ? session.isfirstbar_regular : session.isfirstbar
else
    regularExtendInput == "Regular Session" ? session.islastbar_regular : session.islastbar

// Plot the total volume for each session. 
plot(regularTotal    == 0 ? na : regularTotal,    "Regular total volume",     #ff623b, style = plot.style_areabr)
plot(premarketTotal  == 0 ? na : premarketTotal,  "Pre-market total volume",  #ff9800, style = plot.style_areabr)
plot(postmarketTotal == 0 ? na : postmarketTotal, "Post-market total volume", #2962ff, style = plot.style_areabr)
plot(overnightTotal  == 0 ? na : overnightTotal,  "Overnight total volume",   #d500f9, style = plot.style_areabr)

// Highlight the background of the specified first/last bar in gray.
bgcolor(highlightBar ? #787b8680 : na, title = "First/Last bar highlight")
```

Refer to the [Sessions](https://www.tradingview.com/pine-script-docs/concepts/sessions/) page to learn more about market sessions and the session-related built-ins.

## Symbol information {#symbol-information}

The built-in variables in the `syminfo` namespace hold essential information about the chart’s symbol and the underlying instrument. Most of these variables, excluding [syminfo.main\_tickerid](https://www.tradingview.com/pine-script-reference/v6/#var_syminfo.main_tickerid), can also represent information relating to a requested dataset if a script uses them as the `expression` argument in a `request.*()` function call. Most `syminfo.*` variables have the “simple” [type qualifier](https://www.tradingview.com/pine-script-docs/language/type-system/#qualifiers), because their values do not change after the first bar. However, the variables relating to analyst recommendations and targets have the “series” qualifier, because they store dynamic data that can change over time.

The available `syminfo.*` variables include the following:

-   [syminfo.ticker](https://www.tradingview.com/pine-script-reference/v6/#var_syminfo.ticker): Stores a string representing the dataset’s symbol without the exchange prefix. For example, the value is `"AAPL"` for a NASDAQ:AAPL chart, `"BTCUSD"` for a BITSTAMP:BTCUSD chart, and `"ES1!"` for a CME\_MINI\_DL:ES1! chart.
-   [syminfo.prefix](https://www.tradingview.com/pine-script-reference/v6/#var_syminfo.prefix): Stores a string representing the symbol’s broker/exchange identifier. For example, the value is `"NASDAQ"` for a NASDAQ:AAPL chart, `"BITSTAMP"` for a BITSTAMP:BTCUSD chart, and `"CME_MINI_DL"` for a CME\_MINI\_DL:ES1! chart.
-   [syminfo.root](https://www.tradingview.com/pine-script-reference/v6/#var_syminfo.root): Stores a string representing the instrument’s _root code_ if the symbol refers to a futures contract or another applicable derivative. Otherwise, it stores the same value as [syminfo.ticker](https://www.tradingview.com/pine-script-reference/v6/#var_syminfo.ticker). For example, the value is `"ZW"` for wheat futures symbols such as ZW1! and ZWU2026.
-   [syminfo.tickerid](https://www.tradingview.com/pine-script-reference/v6/#var_syminfo.tickerid): Stores a string representing the _ticker identifier_ (ticker ID) of the chart’s dataset, or of a requested dataset if the script uses it in the `expression` argument of a `request.*()` call. The ticker ID contains the dataset’s symbol _with_ the exchange prefix (e.g., `"NASDAQ:AAPL"`). The string can also contain information about dataset _modifiers_, such as extended hours, dividend adjustments, and currency conversion. Programmers can retrieve the dataset’s ticker ID without modifiers by passing this variable to the [ticker.standard()](https://www.tradingview.com/pine-script-reference/v6/#fun_ticker.standard) function.
-   [syminfo.main\_tickerid](https://www.tradingview.com/pine-script-reference/v6/#var_syminfo.main_tickerid): Stores a string representing the ticker ID of the _main dataset_ on which the script runs. Unlike [syminfo.tickerid](https://www.tradingview.com/pine-script-reference/v6/#var_syminfo.tickerid), this variable does _not_ store a different value when used in a `request.*()` call’s `expression` argument. Therefore, scripts can use this variable to retrieve the current chart’s ticker ID while executing data requests.
-   [syminfo.basecurrency](https://www.tradingview.com/pine-script-reference/v6/#var_syminfo.basecurrency): Stores a string representing the instrument’s _base currency_ if the current symbol refers to a Forex pair, a cryptocurrency pair, or a derivative instrument based on a currency pair. Otherwise, it stores an empty string. For example, the variable’s value is `"EUR"` for any EURJPY pair, `"BTC"` for any BTCUSDT pair, `"CAD"` for CME:6C1! futures, and `""` for NASDAQ:AAPL stock.
-   [syminfo.currency](https://www.tradingview.com/pine-script-reference/v6/#var_syminfo.currency): Stores a string representing the currency of the instrument’s quoted prices, or `"NONE"` if the dataset’s values do not represent currency amounts. For example, the value is `"JPY"` for a EURJPY pair, `"USD"` for NASDAQ:AAPL stock, and `"NONE"` for the TVC:US10Y bond yield dataset.
-   [syminfo.country](https://www.tradingview.com/pine-script-reference/v6/#var_syminfo.country): Stores a string representing the two-letter country code of the instrument’s exchange or data provider, or an empty string if the exchange is not linked to a specific country. For example, the value is `"US"` for NASDAQ:AAPL, `"GB"` for LSE:AAPL, and `""` for BITSTAMP:BTCUSD.
-   [syminfo.timezone](https://www.tradingview.com/pine-script-reference/v6/#var_syminfo.timezone): Stores a string representing the dataset’s _exchange time zone_ in the [IANA time zone database](https://en.wikipedia.org/wiki/List_of_tz_database_time_zones) format. For example, the value is `"America/New_York"` for stocks traded on the NASDAQ exchange. See the [Time zone strings](https://www.tradingview.com/pine-script-docs/concepts/time/#time-zone-strings) section of the [Time](https://www.tradingview.com/pine-script-docs/concepts/time/) page to learn more about IANA identifiers.
-   [syminfo.session](https://www.tradingview.com/pine-script-reference/v6/#var_syminfo.session): Stores a string representing the dataset’s session setting. See the [Session information](https://www.tradingview.com/pine-script-docs/concepts/chart-information/#session-information) section above for more information.
-   [syminfo.current\_contract](https://www.tradingview.com/pine-script-reference/v6/#var_syminfo.current_contract): Stores the “string” ticker identifier of the underlying contract if the current symbol refers to a continuous futures dataset, and an empty string otherwise.
-   [syminfo.description](https://www.tradingview.com/pine-script-reference/v6/#var_syminfo.description): Stores a string containing the description or extended name of the current instrument or dataset.
-   [syminfo.employees](https://www.tradingview.com/pine-script-reference/v6/#var_syminfo.employees): Stores the issuing company’s reported number of employees if the current symbol refers to a stock, and [na](https://www.tradingview.com/pine-script-reference/v6/#var_na) otherwise.
-   [syminfo.shareholders](https://www.tradingview.com/pine-script-reference/v6/#var_syminfo.shareholders): Stores an “int” value representing the total number of reported shareholders if the current instrument is a stock, and [na](https://www.tradingview.com/pine-script-reference/v6/#var_na) otherwise.
-   [syminfo.shares\_outstanding\_float](https://www.tradingview.com/pine-script-reference/v6/#var_syminfo.shares_outstanding_float): Stores the total reported number of outstanding shares, excluding any restricted shares, if the symbol refers to a stock. For other symbols, the value is [na](https://www.tradingview.com/pine-script-reference/v6/#var_na).
-   [syminfo.shares\_outstanding\_total](https://www.tradingview.com/pine-script-reference/v6/#var_syminfo.shares_outstanding_total): Stores the total reported number of outstanding shares, including restricted shares held by insiders, major shareholders, and employees, if the symbol refers to a stock. For other symbols, the value is [na](https://www.tradingview.com/pine-script-reference/v6/#var_na).
-   [syminfo.expiration\_date](https://www.tradingview.com/pine-script-reference/v6/#var_syminfo.expiration_date): Stores an “int” [UNIX timestamp](https://www.tradingview.com/pine-script-docs/concepts/time/#unix-timestamps) representing the start of the last day of the current contract if the symbol refers to a non-continuous futures dataset. On other datasets, the value is [na](https://www.tradingview.com/pine-script-reference/v6/#var_na).
-   [syminfo.isin](https://www.tradingview.com/pine-script-reference/v6/#var_syminfo.isin): Holds a string representing the International Securities Identification Number (ISIN) for the underlying instrument, or an empty string if no ISIN information is available for the instrument.
-   [syminfo.mincontract](https://www.tradingview.com/pine-script-reference/v6/#var_syminfo.mincontract): Stores a “float” value representing the minimum number of contracts/lots/shares/units required for a trade, as set by the exchange. For many instruments, the value is 1. Additionally, note that the value is 1 if the symbol does not refer to a tradable instrument.
-   [syminfo.minmove](https://www.tradingview.com/pine-script-reference/v6/#var_syminfo.minmove): Stores a whole number for calculating the smallest increment by which the instrument’s prices change. It is the _numerator_ of the [syminfo.mintick](https://www.tradingview.com/pine-script-reference/v6/#var_syminfo.mintick) formula: `syminfo.mintick = syminfo.minmove / syminfo.pricescale`.
-   [syminfo.pricescale](https://www.tradingview.com/pine-script-reference/v6/#var_syminfo.pricescale): Stores a whole number for calculating the smallest increment by which the instrument’s prices change. It is the _denominator_ of the [syminfo.mintick](https://www.tradingview.com/pine-script-reference/v6/#var_syminfo.mintick) formula: `syminfo.mintick = syminfo.minmove / syminfo.pricescale`.
-   [syminfo.mintick](https://www.tradingview.com/pine-script-reference/v6/#var_syminfo.mintick): Stores the dataset’s minimum tick size, i.e., the smallest increment by which the instrument’s recorded prices change. For example, the value is `0.00001` for the OANDA:EURUSD pair, `0.01` for NASDAQ:AAPL stock, and `0.25` for CME\_MINI\_DL:ES1! futures.
-   [syminfo.pointvalue](https://www.tradingview.com/pine-script-reference/v6/#var_syminfo.pointvalue): Stores the instrument’s point value, which represents a multiplier of the instrument’s price for determining the value of a single contract. For most instruments, the variable’s value is typically 1, which means the instrument’s price directly represents the value of a contract, share, etc. However, for some futures instruments, the price on the chart represents the value per _index point_ or _unit_ of the underlying _commodity_. For example, the point value for COMEX:GC1! futures is 100, because the standard size of a single contract is _100 troy ounces_ of gold. Therefore, the cost of purchasing one contract is _100 times_ the price on the chart.
-   [syminfo.type](https://www.tradingview.com/pine-script-reference/v6/#var_syminfo.type): Stores a string representing the instrument’s type. Possible values include `"stock"`, `"futures"`, `"index"`, `"forex"`, `"crypto"`, `"fund"`, `"dr"`, `"cfd"`, `"bond"`, `"warrant"`, `"structured"`, and `"right"`.
-   [syminfo.volumetype](https://www.tradingview.com/pine-script-reference/v6/#var_syminfo.volumetype): Stores a string indicating the type of volume reported for the instrument. Possible values include `"base"`, `"quote"`, `"tick"`, and `"n/a"`.
-   [syminfo.sector](https://www.tradingview.com/pine-script-reference/v6/#var_syminfo.sector): Stores a string representing the sector associated with the underlying instrument, or an empty string if there is no associated sector. The sector refers to a broad classification of the associated economy. For example, the value is `"Electronic Technology"` for NASDAQ:AAPL, `"Technology Services"` for NYSE:IBM, and `"Miscellaneous"` for AMEX:SPY.
-   [syminfo.industry](https://www.tradingview.com/pine-script-reference/v6/#var_syminfo.industry): Stores a string representing the industry associated with the underlying instrument, or an empty string if there is no associated industry. The industry associated with an instrument refers to a subset of the corresponding sector. For example, the value is `"Telecommunications Equipment"` for NASDAQ:AAPL, `"Information Technology Services"` for NYSE:IBM, `"Investment Trusts/Mutual Funds"` for AMEX:SPY, and `""` for SP:SPX.
-   [syminfo.recommendations\_buy](https://www.tradingview.com/pine-script-reference/v6/#var_syminfo.recommendations_buy): Stores the total number of analysts who gave the current instrument a “Buy” rating.
-   [syminfo.recommendations\_buy\_strong](https://www.tradingview.com/pine-script-reference/v6/#var_syminfo.recommendations_buy_strong): Stores the total number of analysts who gave the current instrument a “Strong Buy” rating.
-   [syminfo.recommendations\_sell](https://www.tradingview.com/pine-script-reference/v6/#var_syminfo.recommendations_sell): Stores the total number of analysts who gave the current instrument a “Sell” rating.
-   [syminfo.recommendations\_sell\_strong](https://www.tradingview.com/pine-script-reference/v6/#var_syminfo.recommendations_sell_strong): Stores the total number of analysts who gave the current instrument a “Strong Sell” rating.
-   [syminfo.recommendations\_hold](https://www.tradingview.com/pine-script-reference/v6/#var_syminfo.recommendations_hold): Stores the total number of analysts who gave the current instrument a “Hold” rating.
-   [syminfo.recommendations\_total](https://www.tradingview.com/pine-script-reference/v6/#var_syminfo.recommendations_total): Stores the total number of recommendations for the current instrument.
-   [syminfo.recommendations\_date](https://www.tradingview.com/pine-script-reference/v6/#var_syminfo.recommendations_date): Stores an “int” UNIX timestamp representing the starting date of the latest set of recommendations for the current instrument.
-   [syminfo.target\_price\_average](https://www.tradingview.com/pine-script-reference/v6/#var_syminfo.target_price_average): Stores the average of the last yearly analyst price targets for the instrument.
-   [syminfo.target\_price\_high](https://www.tradingview.com/pine-script-reference/v6/#var_syminfo.target_price_high): Stores the last highest yearly analyst price target for the instrument.
-   [syminfo.target\_price\_low](https://www.tradingview.com/pine-script-reference/v6/#var_syminfo.target_price_low): Stores the last lowest yearly analyst price target for the instrument.
-   [syminfo.target\_price\_median](https://www.tradingview.com/pine-script-reference/v6/#var_syminfo.target_price_median): Stores the median of the last yearly analyst price targets for the instrument.
-   [syminfo.target\_price\_date](https://www.tradingview.com/pine-script-reference/v6/#var_syminfo.target_price_date): Stores an “int” UNIX timestamp representing the starting date of the last analyst price target prediction for the current instrument.
-   [syminfo.target\_price\_estimates](https://www.tradingview.com/pine-script-reference/v6/#var_syminfo.target_price_estimates): Stores the latest total number of analyst price target predictions for the current instrument.

The example script below displays a [table](https://www.tradingview.com/pine-script-docs/visuals/tables/) containing a simple summary of symbol and instrument information from the chart. On the first bar, the script creates two “string” [arrays](https://www.tradingview.com/pine-script-docs/language/arrays/) using the [array.from](https://www.tradingview.com/pine-script-reference/v6/#fun_array.from) function. The first array contains titles for the table’s first column. The second array contains corresponding strings from multiple `syminfo.*` variables for the second column. The script iterates through the arrays and populates the cells on each table row within a [for](https://www.tradingview.com/pine-script-reference/v6/#kw_for) loop:

```pine
//@version=6
indicator("`syminfo.*` variables demo", overlay = true, behind_chart = false)

if barstate.isfirst
    //@variable References a `table` object that displays symbol and instrument information on the chart.
    table displayTable = table.new(position.middle_right, 2, 9, border_color = chart.fg_color, border_width = 1)
    //@variable References an array of "string" titles for the table's first column.
    array<string> titles = array.from(
        "Ticker ID", "Symbol", "Prefix", "Type", "Description", "ISIN", 
        "Currency", "Tick size", "Point value"
    )
    //@variable References an array of "string" representations of `syminfo.*` values for the second column.
    array<string> values = array.from(
        syminfo.tickerid, syminfo.ticker, syminfo.prefix, syminfo.type, syminfo.description, syminfo.isin, 
        syminfo.currency, str.tostring(syminfo.mintick), str.tostring(syminfo.pointvalue)
    )
    // Loop through the arrays and populate the rows with the corresponding `titles` and `values` elements. 
    for i = 0 to 8
        displayTable.cell(0, i, titles.get(i), text_color = chart.fg_color)
        displayTable.cell(1, i, values.get(i), text_color = chart.fg_color)
```

Note that:

-   The script initializes and populates the table only on the _first_ bar because the values of the `syminfo.*` variables used in the code do not change from bar to bar. After the script creates the table and sets its cells on the first bar, the table’s output persists on the right side of the chart.
-   The script uses the [chart.fg\_color](https://www.tradingview.com/pine-script-reference/v6/#var_chart.fg_color) variable to set the color of the table’s borders and text. The variable’s value changes based on the color of the chart’s _background_. See the [Chart type and color](https://www.tradingview.com/pine-script-docs/concepts/chart-information/#chart-type-and-color) section below for more information.

## Time series information {#time-series-information}

Two built-in variables store information about the _bar indices_ in the [time series](https://www.tradingview.com/pine-script-docs/language/execution-model/#time-series) for the current chart, or for a requested dataset if used in the `expression` argument of a `request.*()` function call:

-   [bar\_index](https://www.tradingview.com/pine-script-reference/v6/#var_bar_index): Stores a “series int” value representing the time series index for the current bar. The value is 0 on the first available bar, 1 on the second bar, and so on. The value on the last available bar is one less than the _total_ number of bars.
-   [last\_bar\_index](https://www.tradingview.com/pine-script-reference/v6/#var_last_bar_index): Stores a “series int” value representing the time series index of the last available bar. The value is consistently one less than the total number of available bars, even while the script executes on the first bar.

Several variables in the `barstate` namespace hold “series bool” values to indicate the _states_ of each bar in the chart’s dataset or a requested dataset. These variables include the following:

-   [barstate.isfirst](https://www.tradingview.com/pine-script-reference/v6/#var_barstate.isfirst): Stores `true` if the current bar is the first available bar, and `false` otherwise. The value is equivalent to the result of `bar_index == 0`.
-   [barstate.islast](https://www.tradingview.com/pine-script-reference/v6/#var_barstate.islast): Stores `true` if the current bar is the last available bar, and `false` otherwise. The value is equivalent to the result of `bar_index == last_bar_index`.
-   [barstate.isnew](https://www.tradingview.com/pine-script-reference/v6/#var_barstate.isnew): Stores `true` on all historical bars and on the _first tick_ of an open realtime bar. On subsequent ticks within an open bar, the value is `false`.
-   [barstate.isconfirmed](https://www.tradingview.com/pine-script-reference/v6/#var_barstate.isconfirmed): Stores `true` if the current bar is _closed_ (confirmed), and `false` if the bar is _open_. The runtime system _commits (saves)_ a script’s calculated data to the time series when the value is `true`.
-   [barstate.ishistory](https://www.tradingview.com/pine-script-reference/v6/#var_barstate.ishistory): Stores `true` if the current bar is historical, meaning that it closed _before_ the script loaded on the dataset, and `false` otherwise.
-   [barstate.isrealtime](https://www.tradingview.com/pine-script-reference/v6/#var_barstate.isrealtime): The _opposite_ of [barstate.ishistory](https://www.tradingview.com/pine-script-reference/v6/#var_barstate.ishistory). Stores `true` if the current bar is a realtime bar, which closes _after_ the script loads, and `false` otherwise.
-   [barstate.islastconfirmedhistory](https://www.tradingview.com/pine-script-reference/v6/#var_barstate.islastconfirmedhistory): Stores `true` on the last available _historical_ bar, and `false` otherwise.

Refer to the [Bar states](https://www.tradingview.com/pine-script-docs/concepts/bar-states/) page to learn more about these variables and how they work. For detailed information about how scripts execute across historical and realtime bars, and how they manage data in the time series based on bar states, refer to the [Execution model](https://www.tradingview.com/pine-script-docs/language/execution-model/) page.

The following example calculates a volume-weighted average price (VWAP) over periods spanning a specified number of bars. The script resets the VWAP calculation on each bar whose [bar\_index](https://www.tradingview.com/pine-script-reference/v6/#var_bar_index) value is divisible by the specified period. For instance, with the default input value of 100, the calculation resets on bar 0, 100, 200, and so on. The script plots the VWAP series and highlights the background of each bar on which the reset occurs. Additionally, the script uses the [bar\_index](https://www.tradingview.com/pine-script-reference/v6/#var_bar_index), [last\_bar\_index](https://www.tradingview.com/pine-script-reference/v6/#var_last_bar_index), and [barstate.ishistory](https://www.tradingview.com/pine-script-reference/v6/#var_barstate.ishistory) variables to calculate the total number of historical bars, realtime bars, and completed periods, then displays the results in a single-cell [table](https://www.tradingview.com/pine-script-docs/visuals/tables/) on the last bar:

```pine
//@version=6
indicator("`*bar_index` and `barstate.*` demo", overlay = true, behind_chart = false)

//@variable The total number of bars in each VWAP calculation.
int periodInput = input.int(100, "VWAP period", 2)

//@variable Is `true` once every `periodInput` bars, starting from bar 0, and `false` otherwise. 
bool resetVWAP = bar_index % periodInput == 0
//@variable The VWAP for the current period. The calculation resets each time the specified number of bars elapses.
float vwap = ta.vwap(hlc3, resetVWAP)

// This `if` structure's scope executes on the last bar.
// The condition `bar_index == last_bar_index` is equivalent to `barstate.islast`.
if bar_index == last_bar_index
    //@variable References a single-cell table that displays bar and period information in the top-right corner.
    var table infoTable = table.new(position.top_right, 1, 1)
    //@variable The initial total bars if the last bar is historical, and one less than the total otherwise.
    var int historicalBars = barstate.ishistory ? bar_index + 1 : bar_index
    //@variable A formatted string containing bar and VWAP period information.
    string displayText = str.format(
        "Historical bars: {0}\nRealtime bars: {1}\nVWAP period: {2} bars\nComplete periods on chart: {3}", 
        historicalBars, bar_index + 1 - historicalBars, periodInput, int(bar_index / periodInput)
    )
    // Display the text from the `displayText` string in the table's cell.
    infoTable.cell(0, 0, displayText, text_color = #000000, bgcolor = #2196f3)

// Plot the `vwap` series and highlight the background on each calculation reset.
plot(vwap, "Periodic VWAP", linewidth = 3)
bgcolor(resetVWAP ? #ff98004d : na, title = "VWAP reset highlight")
```

Note that:

-   On the first bar where the [bar\_index](https://www.tradingview.com/pine-script-reference/v6/#var_bar_index) and [last\_bar\_index](https://www.tradingview.com/pine-script-reference/v6/#var_last_bar_index) values are equal, the script checks the value of the [barstate.ishistory](https://www.tradingview.com/pine-script-reference/v6/#var_barstate.ishistory) variable to determine whether that bar is historical. If the value is `true`, the total number of historical bars is one greater than the bar index on that bar. Otherwise, the number of historical bars equals the bar index. As new bars become available, the script counts the number of realtime bars by subtracting the historical total from the value of `bar_index + 1`.
-   The script counts the total number of completed VWAP periods by dividing the latest [bar\_index](https://www.tradingview.com/pine-script-reference/v6/#var_bar_index) value by the input period, then rounding the result down to the nearest integer.

Pine Script also features several built-in variables that access _time_ information for the bars on the chart or a requested dataset:

-   The [time](https://www.tradingview.com/pine-script-reference/v6/#var_time) and [time\_close](https://www.tradingview.com/pine-script-reference/v6/#var_time_close) variables hold [UNIX timestamps](https://www.tradingview.com/pine-script-docs/concepts/time/#unix-timestamps) representing the current bar’s opening and closing times, respectively.
-   The [last\_bar\_time](https://www.tradingview.com/pine-script-reference/v6/#var_last_bar_time) variable stores a UNIX timestamp representing the opening time of the last available bar.
-   The [time\_tradingday](https://www.tradingview.com/pine-script-reference/v6/#var_time_tradingday) variable holds a UNIX timestamp representing the starting time of the trading day to which the current bar belongs.
-   The [timenow](https://www.tradingview.com/pine-script-reference/v6/#var_timenow) variable stores the UNIX timestamp of the script’s latest execution.
-   The [year](https://www.tradingview.com/pine-script-reference/v6/#var_year), [month](https://www.tradingview.com/pine-script-reference/v6/#var_month), [weekofyear](https://www.tradingview.com/pine-script-reference/v6/#var_weekofyear), [dayofmonth](https://www.tradingview.com/pine-script-reference/v6/#var_dayofmonth), [dayofweek](https://www.tradingview.com/pine-script-reference/v6/#var_dayofweek), [hour](https://www.tradingview.com/pine-script-reference/v6/#var_hour), [minute](https://www.tradingview.com/pine-script-reference/v6/#var_minute), and [second](https://www.tradingview.com/pine-script-reference/v6/#var_second) variables store [calendar-based](https://www.tradingview.com/pine-script-docs/concepts/time/#calendar-based-variables) values derived from the current bar’s opening time. The values are expressed in the [exchange time zone](https://www.tradingview.com/pine-script-docs/concepts/time/#time-zones).
-   The [chart.left\_visible\_bar\_time](https://www.tradingview.com/pine-script-reference/v6/#var_chart.left_visible_bar_time) and [chart.right\_visible\_bar\_time](https://www.tradingview.com/pine-script-reference/v6/#var_chart.right_visible_bar_time) variables store UNIX timestamps representing the opening times of the leftmost and rightmost visible chart bars.

Refer to the [Time](https://www.tradingview.com/pine-script-docs/concepts/time/) page for detailed information about these variables and examples of how they work.

## Chart type and color {#chart-type-and-color}

Multiple built-in variables in the `chart` namespace hold “simple bool” values to indicate the type of chart on which the script runs. These variables can also indicate a requested chart dataset’s type when used in the `expression` argument of a `request.*()` function call:

-   [chart.is\_standard](https://www.tradingview.com/pine-script-reference/v6/#var_chart.is_standard): Stores `true` if the chart is any of the _standard_ types, including [line charts](https://www.tradingview.com/support/solutions/43000745271/), [bar charts](https://www.tradingview.com/support/solutions/43000672403/), [candlestick charts](https://www.tradingview.com/support/solutions/43000745269/), and other chart types that use the instrument’s actual OHLC prices, as opposed to calculated prices. Otherwise, the value is `false`.
-   [chart.is\_heikinashi](https://www.tradingview.com/pine-script-reference/v6/#var_chart.is_heikinashi): Stores `true` if the chart type is [Heikin Ashi](https://www.tradingview.com/support/solutions/43000619436/), and `false` otherwise.
-   [chart.is\_linebreak](https://www.tradingview.com/pine-script-reference/v6/#var_chart.is_linebreak): Stores `true` if the chart type is [line break](https://www.tradingview.com/support/solutions/43000502273/), and `false` otherwise.
-   [chart.is\_pnf](https://www.tradingview.com/pine-script-reference/v6/#var_chart.is_pnf): Stores `true` if the chart type is [point & figure](https://www.tradingview.com/support/solutions/43000502276/), and `false` otherwise.
-   [chart.is\_kagi](https://www.tradingview.com/pine-script-reference/v6/#var_chart.is_kagi): Stores `true` if the chart type is [Kagi](https://www.tradingview.com/support/solutions/43000502272/), and `false` otherwise.
-   [chart.is\_range](https://www.tradingview.com/pine-script-reference/v6/#var_chart.is_range): Stores `true` if the chart type is [range](https://www.tradingview.com/support/solutions/43000474007/), and `false` otherwise.
-   [chart.is\_renko](https://www.tradingview.com/pine-script-reference/v6/#var_chart.is_renko): Stores `true` if the chart type is [Renko](https://www.tradingview.com/support/solutions/43000502284/), and `false` otherwise.

These `chart.is_*` variables are typically useful when a script’s logic must respond differently on non-standard charts. For example, the following script demonstrates a simple [strategy](https://www.tradingview.com/pine-script-docs/concepts/strategies/) that places [market orders](https://www.tradingview.com/pine-script-docs/concepts/strategies/#market-orders) to enter trades based on the crossing of two moving averages. On a non-standard chart, these orders can generate _misleading_ results, because Pine’s [broker emulator](https://www.tradingview.com/pine-script-docs/concepts/strategies/#broker-emulator) fills them at the chart’s _calculated_ prices rather than using the instrument’s actual prices. To prevent such results, the script allows orders only on standard chart types by using [chart.is\_standard](https://www.tradingview.com/pine-script-reference/v6/#var_chart.is_standard) in the conditions that control the [strategy.entry()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.entry) commands. As shown below, if the script runs on a non-standard chart, it does not generate any orders or display performance data in the [strategy report](https://www.tradingview.com/pine-script-docs/concepts/strategies/#strategy-report):

```pine
//@version=6
strategy("`chart.is_standard` demo", overlay = true, behind_chart = false)

// Calculate two moving averages for the cross signal.
float ma1 = ta.sma(close, 10)
float ma2 = ta.sma(close, 20)

if chart.is_standard
    // This logic executes only on standard charts, preventing misleading trade data on non-standard charts. 

    // Close any short position and open a new long position if the first MA crosses over the second MA.
    if ta.crossover(ma1, ma2)
        strategy.entry("Buy", strategy.long)
    // Close any long position and open a new short position if the first MA crosses under the second MA.
    if ta.crossunder(ma1, ma2)
        strategy.entry("Sell", strategy.short)

// Plot the moving averages on the chart.
plot(ma1, "Fast MA", color.orange, 2)
plot(ma2, "Slow MA", color.blue, 3)

// Color the background translucent red if the chart is not a standard type.
bgcolor(not chart.is_standard ? color.new(color.red, 60) : na, title = "Non-standard chart highlight")
```

Note that:

-   An alternative way to avoid misleading trade prices on _Heikin Ashi_ charts is to include `fill_orders_on_standard_ohlc = true` in the [strategy()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy) declaration statement. This argument configures the broker emulator to fill orders using _standard_ chart prices by default. See the [Strategies](https://www.tradingview.com/pine-script-docs/concepts/strategies/) page to learn more about strategy scripts.

The `chart` namespace also features the following variables that store “input color” values based on the background color defined in the chart’s settings:

-   [chart.bg\_color](https://www.tradingview.com/pine-script-reference/v6/#var_chart.bg_color): Stores the value of the chart’s background color, as defined by the “Background” inputs in the “Canvas” tab of the chart’s settings.
-   [chart.fg\_color](https://www.tradingview.com/pine-script-reference/v6/#var_chart.fg_color): Stores a grayscale color that provides high contrast with most chart background colors. For dark backgrounds, the color is `#dbdbdb`. For light backgrounds, the color is `#0f0f0f`.

The following script creates a single-cell [table](https://www.tradingview.com/pine-script-docs/visuals/tables/) to indicate whether the chart’s background is light or dark, based on the value of [chart.fg\_color](https://www.tradingview.com/pine-script-reference/v6/#var_chart.fg_color). If the value is `#0f0f0f`, the table’s text states that the background is considered light. Otherwise, it states that the background is considered dark. The script colors the table’s background using the foreground color, and it sets the text color using the value of [chart.bg\_color](https://www.tradingview.com/pine-script-reference/v6/#var_chart.bg_color). The script also sets the table’s frame color using the middle value of a [gradient](https://www.tradingview.com/pine-script-docs/visuals/colors/#colorfrom_gradient) from the background color to the foreground color:

```pine
//@version=6
indicator("`chart.bg_color` and `chart.fg_color` demo")

if barstate.isfirst
    //@variable The color from the midpoint of a gradient using the chart's background and foreground colors.
    color middleColor = color.from_gradient(0.5, 0, 1, chart.bg_color, chart.fg_color)
    //@variable References a table with `chart.fg_color` as the background color and `middleColor` as the frame color.
    table displayTable = table.new(position.middle_center, 1, 1, chart.fg_color, middleColor, 2)

    //@variable Is `true` if the chart's foreground color is `#0f0f0f`, indicating a light background.
    bool isLight = chart.fg_color == #0f0f0f
    //@variable A string indicating whether the chart's background is considered light or dark.
    string displayText = "The chart's background is considered " + (isLight ? "light." : "dark.")
    // Initialize the table's cell to display the string's text colored using the chart's background color. 
    displayTable.cell(0, 0, displayText, text_color = chart.bg_color, text_size = 30)
```
