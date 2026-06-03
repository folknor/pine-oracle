---
title: Other timeframes and data
source: https://www.tradingview.com/pine-script-docs/concepts/other-timeframes-and-data/
section: concepts
---

# Other timeframes and data {#other-timeframes-and-data}

## Introduction {#introduction}

Pine Script® allows users to request data from sources and contexts other than those their charts use. The functions we present on this page can fetch data from a variety of alternative sources:

-   [request.security()](https://www.tradingview.com/pine-script-docs/concepts/other-timeframes-and-data/#requestsecurity) retrieves data from another symbol, timeframe, or other context.
-   [request.security\_lower\_tf()](https://www.tradingview.com/pine-script-docs/concepts/other-timeframes-and-data/#requestsecurity_lower_tf) retrieves _intrabar_ data, i.e., data from a timeframe lower than the chart timeframe.
-   [request.currency\_rate()](https://www.tradingview.com/pine-script-docs/concepts/other-timeframes-and-data/#requestcurrency_rate) requests a _daily rate_ to convert a value expressed in one currency to another.
-   [request.dividends(), request.splits(), and request.earnings()](https://www.tradingview.com/pine-script-docs/concepts/other-timeframes-and-data/#requestdividends-requestsplits-and-requestearnings) respectively retrieve information about an issuing company’s dividends, splits, and earnings.
-   [request.financial()](https://www.tradingview.com/pine-script-docs/concepts/other-timeframes-and-data/#requestfinancial) retrieves financial data from [FactSet](https://www.factset.com/).
-   [request.economic()](https://www.tradingview.com/pine-script-docs/concepts/other-timeframes-and-data/#requesteconomic) retrieves economic and industry data.
-   [request.footprint()](https://www.tradingview.com/pine-script-docs/concepts/other-timeframes-and-data/#requestfootprint) retrieves _volume footprint_ data.
-   [request.seed()](https://www.tradingview.com/pine-script-docs/concepts/other-timeframes-and-data/#requestseed) retrieves data from a _user-maintained_ GitHub repository.

> [!NOTE]
> Throughout this page, and in other parts of our documentation that discuss `request.*()` functions, we often use the term _“context”_ to describe the symbol, timeframe, and any modifications — such as price adjustments, session settings, and non-standard chart types — that apply to a chart or the data retrieved by a script.

These are the signatures of the functions in the `request.*` namespace:

```
request.security(symbol, timeframe, expression, gaps, lookahead, ignore_invalid_symbol, currency, calc_bars_count) → series <type>

request.security_lower_tf(symbol, timeframe, expression, ignore_invalid_symbol, currency, ignore_invalid_timeframe, calc_bars_count) → array<type>

request.currency_rate(from, to, ignore_invalid_currency) → series float

request.dividends(ticker, field, gaps, lookahead, ignore_invalid_symbol, currency) → series float

request.splits(ticker, field, gaps, lookahead, ignore_invalid_symbol) → series float

request.earnings(ticker, field, gaps, lookahead, ignore_invalid_symbol, currency) → series float

request.financial(symbol, financial_id, period, gaps, ignore_invalid_symbol, currency) → series float

request.economic(country_code, field, gaps, ignore_invalid_symbol) → series float

request.footprint(ticks_per_row, va_percent, imbalance_percent) → series footprint

request.seed(source, symbol, expression, ignore_invalid_symbol, calc_bars_count) → series <type>
```

The `request.*()` family of functions has numerous potential applications. Throughout this page, we discuss in detail these functions and some of their typical use cases.

> [!TIP]
> Programmers can also enable compatible scripts to perform calculations on data from another timeframe, without requiring `request.*()` calls, by supplying an argument to the `timeframe` parameter of the [indicator()](https://www.tradingview.com/pine-script-reference/v6/#fun_indicator) declaration statement.

## Common characteristics {#common-characteristics}

Many functions in the `request.*()` namespace share some common properties and parameters. Before we explore each function in depth, let’s familiarize ourselves with these characteristics.

### Behavior {#behavior}

All `request.*()` functions have similar internal behavior, even though they do not all share the same required parameters. Every unique `request.*()` call in a script requests a dataset from a defined _context_ (i.e., ticker ID and timeframe) and evaluates an _expression_ across the retrieved data.

The [request.security()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.security) and [request.security\_lower\_tf()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.security_lower_tf) functions allow programmers to specify the context of a request and the expression directly via the `symbol`, `timeframe`, and `expression` parameters, making them suitable for a wide range of data requests.

For example, the [request.security()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.security) call in this simple script requests daily “AMEX:SPY” data, and it calculates the slope of a 20-bar linear regression line using the retrieved [hl2](https://www.tradingview.com/pine-script-reference/v6/#var_hl2) prices. The first two arguments specify the context of the request, and the third specifies the expression to evaluate across the requested data:

```pine
//@version=6
indicator("Behavior of `request.security()` demo")

//@variable The 20-bar linear regression slope of `hl2` prices from the "AMEX:SPY" symbol on the "1D" timeframe.
float requestedSlope = request.security("AMEX:SPY", "1D", ta.linreg(hl2, 20, 0) - ta.linreg(hl2, 20, 1))

//@variable Is `color.teal` when the `requestedSlope` is positive, and `color.maroon` otherwise.
color plotColor = requestedSlope > 0 ? color.teal : color.maroon

// Plot the `requestedSlope` with the `plotColor`.
plot(requestedSlope, "Requested slope", plotColor, 1, plot.style_area)
```

Other functions within the `request.*()` namespace _do not_ allow programmers to directly define the full context of a request or the evaluated expression. Instead, these functions determine some of the necessary information _internally_ because they perform only specific types of requests.

For instance, [request.financial()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.financial) exclusively retrieves periodic financial data. Its required parameters (`symbol`, `financial_id`, and `period`) all define parts of a specific financial _ticker ID_. The function does not allow specification of the timeframe or expression, as it determines these details internally. The script below demonstrates a simple call to this function that retrieves the annual cost of goods data for the chart symbol’s issuing company:

```pine
//@version=6
indicator("Behavior of `request.financial()` demo", format = format.volume)

//@variable The annual cost of goods sold by the chart symbol's issuing company.
float costOfGoods = request.financial(syminfo.tickerid, "COST_OF_GOODS", "FY")

// Plot the `costOfGoods`.
plot(costOfGoods, "Cost of goods", color.purple, 3, plot.style_stepline_diamond)
```

Scripts can perform up to 40 unique requests using any combination of `request.*()` function calls, or up to 64 if the user has the [Ultimate plan](https://www.tradingview.com/pricing/). Unique `request.*()` calls count toward this limit because they are the only calls that fetch _new data_. By contrast, redundant calls to the same `request.*()` function with identical arguments _do not_ typically perform new requests. Instead, they _reuse_ the data from the first executed call. See the [`request.*()` calls](https://www.tradingview.com/pine-script-docs/writing/limitations/#request-calls) section of the [Limitations](https://www.tradingview.com/pine-script-docs/writing/limitations/) page for more information.

### `gaps` {#gaps}

When using a `request.*()` function to retrieve data from another context, the data might not come in on each new bar as it would with the current chart. The `gaps` parameter of a `request.*()` function controls how the function responds to nonexistent values in the requested series.

> [!NOTE]
> The `timeframe_gaps` parameter of the [indicator()](https://www.tradingview.com/pine-script-reference/v6/#fun_indicator) declaration statement is similar to the `gaps` parameter for `request.*()` functions. When the declaration statement includes a `timeframe` argument, causing the script to evaluate its code using data from a specific timeframe, the `timeframe_gaps` parameter specifies how the script handles nonexistent values on each chart bar.

Suppose we have a script that requests hourly data for the chart’s symbol using [request.security()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.security) executing on a 1-minute chart. The function call returns new values only on the 1-minute bars that cover the opening or closing times of the symbol’s hourly bars. On other chart bars, we can decide whether the function returns [na](https://www.tradingview.com/pine-script-reference/v6/#var_na) values or the last available values via the `gaps` parameter.

If the `gaps` parameter uses [barmerge.gaps\_on](https://www.tradingview.com/pine-script-reference/v6/#var_barmerge.gaps_on), the function returns [na](https://www.tradingview.com/pine-script-reference/v6/#var_na) results on all chart bars where new data is not yet confirmed from the requested context. Otherwise, if the parameter uses [barmerge.gaps\_off](https://www.tradingview.com/pine-script-reference/v6/#var_barmerge.gaps_off), the function fills the gaps in the requested data with the last confirmed values on historical bars and the most recent developing values on realtime bars.

The script below demonstrates the difference in behavior by plotting the results from two [request.security()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.security) calls that fetch the [close](https://www.tradingview.com/pine-script-reference/v6/#var_close) price of the current symbol from the hourly timeframe on a 1-minute chart. The first call uses `gaps = barmerge.gaps_off` and the second uses `gaps = barmerge.gaps_on`:

```pine
//@version=6
indicator("`gaps` demo", overlay = true)

//@variable The `close` requested from the hourly timeframe without gaps.
float dataWithoutGaps = request.security(syminfo.tickerid, "60", close, gaps = barmerge.gaps_off)
//@variable The `close` requested from the hourly timeframe with gaps.
float dataWithGaps = request.security(syminfo.tickerid, "60", close, gaps = barmerge.gaps_on)

// Plot the requested data.
plot(dataWithoutGaps, "Data without gaps", color.blue, 3, plot.style_linebr)
plot(dataWithGaps, "Data with gaps", color.purple, 15, plot.style_linebr)

// Highlight the background for realtime bars.
bgcolor(barstate.isrealtime ? color.new(color.aqua, 70) : na, title = "Realtime bar highlight")
```

Note that:

-   [barmerge.gaps\_off](https://www.tradingview.com/pine-script-reference/v6/#var_barmerge.gaps_off) is the default value for the `gaps` parameter in all applicable `request.*()` functions.
-   The script plots the requested series as lines with breaks ([plot.style\_linebr](https://www.tradingview.com/pine-script-reference/v6/#var_plot.style_linebr)), which do not bridge over [na](https://www.tradingview.com/pine-script-reference/v6/#var_na) values as the default style ([plot.style\_line](https://www.tradingview.com/pine-script-reference/v6/#var_plot.style_line)) does.
-   When using [barmerge.gaps\_off](https://www.tradingview.com/pine-script-reference/v6/#var_barmerge.gaps_off), the [request.security()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.security) function returns the last confirmed [close](https://www.tradingview.com/pine-script-reference/v6/#var_close) from the hourly timeframe on all historical bars. When running on _realtime bars_ (the bars with the [color.aqua](https://www.tradingview.com/pine-script-reference/v6/#var_color.aqua) background in this example), it returns the symbol’s current [close](https://www.tradingview.com/pine-script-reference/v6/#var_close) value, regardless of confirmation. For more information, see the [Historical and realtime behavior](https://www.tradingview.com/pine-script-docs/concepts/other-timeframes-and-data/#historical-and-realtime-behavior) section of this page.

### `ignore_invalid_symbol` {#ignore_invalid_symbol}

The `ignore_invalid_symbol` parameter of `request.*()` functions determines how a function handles invalid data requests, e.g.:

-   Using a `request.*()` function with a nonexistent ticker ID as the `symbol/ticker` parameter.
-   Using [request.financial()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.financial) to retrieve information that does not exist for the specified `symbol` or `period`.
-   Using [request.economic()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.economic) to request a `field` that does not exist for a `country_code`.

A `request.*()` function call produces a _runtime error_ and halts the execution of the script when making an erroneous request if its `ignore_invalid_symbol` parameter is `false`. When this parameter’s value is `true`, the function returns [na](https://www.tradingview.com/pine-script-reference/v6/#var_na) values in such a case instead of raising an error.

This example uses `request.*()` calls within a [user-defined function](https://www.tradingview.com/pine-script-docs/language/user-defined-functions/) to retrieve data for estimating an instrument’s market capitalization (market cap). The user-defined `calcMarketCap()` function calls [request.financial()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.financial) to retrieve the total shares outstanding for a symbol and [request.security()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.security) to retrieve a tuple containing the symbol’s [close](https://www.tradingview.com/pine-script-reference/v6/#var_close) and [syminfo.currency](https://www.tradingview.com/pine-script-reference/v6/#var_syminfo.currency) values. We’ve included `ignore_invalid_symbol = true` in both of these `request.*()` calls to prevent runtime errors for invalid requests.

The script displays a [formatted string](https://www.tradingview.com/pine-script-docs/concepts/strings/#formatting-strings) representing the symbol’s estimated market cap value and currency in a [table](https://www.tradingview.com/pine-script-reference/v6/#type_table) on the chart and uses a [plot()](https://www.tradingview.com/pine-script-reference/v6/#fun_plot) call to visualize the `marketCap` history:

```pine
//@version=6
indicator("`ignore_invalid_symbol` demo", "Market cap estimate", format = format.volume)

//@variable The symbol to request data from.
string symbol = input.symbol("TSX:SHOP", "Symbol")

//@function Estimates the market capitalization of the specified `tickerID` if the data exists.
calcMarketCap(simple string tickerID) =>
    //@variable The quarterly total shares outstanding for the `tickerID`. Returns `na` when the data isn't available.
    float tso = request.financial(tickerID, "TOTAL_SHARES_OUTSTANDING", "FQ", ignore_invalid_symbol = true)
    //@variable The `close` price and currency for the `tickerID`. Returns `[na, na]` when the `tickerID` is invalid.
    [price, currency] = request.security(
         tickerID, timeframe.period, [close, syminfo.currency], ignore_invalid_symbol = true
     )
    // Return a tuple containing the market cap estimate and the quote currency.
    [tso * price, currency]

//@variable A `table` object with a single cell that displays the `marketCap` and `quoteCurrency`.
var table infoTable = table.new(position.top_right, 1, 1)
// Initialize the table's cell on the first bar.
if barstate.isfirst
    table.cell(infoTable, 0, 0, "", text_color = color.white, text_size = size.huge, bgcolor = color.teal)

// Get the market cap estimate and quote currency for the `symbol`.
[marketCap, quoteCurrency] = calcMarketCap(symbol)

if barstate.islast
    //@variable The formatted text displayed inside the `infoTable`.
    string tableText = str.format("Market cap:\n{0} {1}", str.tostring(marketCap, format.volume), quoteCurrency)
    // Update the `infoTable`.
    table.cell_set_text(infoTable, 0, 0, tableText)

// Plot the `marketCap` value.
plot(marketCap, "Market cap", color.new(color.purple, 60), style = plot.style_area)
```

Note that:

-   The `calcMarketCap()` function only returns non-na values on valid instruments with total shares outstanding data, such as the one we selected for this example. It returns [na](https://www.tradingview.com/pine-script-reference/v6/#var_na) on others that do not have financial data, including forex, crypto, and derivatives.
-   Not all issuing companies publish quarterly financial reports. If the issuing company of the `symbol` does not report on a quarterly basis, change the “FQ” value in this script to the company’s minimum reporting period. See the [request.financial()](https://www.tradingview.com/pine-script-docs/concepts/other-timeframes-and-data/#requestfinancial) section for more information.
-   We included [format.volume](https://www.tradingview.com/pine-script-reference/v6/#var_format.volume) in the [indicator()](https://www.tradingview.com/pine-script-reference/v6/#fun_indicator) and [str.tostring()](https://www.tradingview.com/pine-script-reference/v6/#fun_str.tostring) calls to specify that the y-axis of the chart pane represents volume-formatted values and the “string” representation of the `marketCap` value shows as volume-formatted text.
-   For efficiency, this script creates a [table](https://www.tradingview.com/pine-script-reference/v6/#type_table) and initializes its cell on the _first_ chart bar, then updates the cell’s text on the _last_ bar. To learn more about working with tables, see the [Tables](https://www.tradingview.com/pine-script-docs/visuals/tables/) page.

### `currency` {#currency}

The `currency` parameter of a `request.*()` function enables programmers to specify the currency of the requested data. If this parameter’s value differs from the symbol’s [syminfo.currency](https://www.tradingview.com/pine-script-reference/v6/#var_syminfo.currency) value, the function converts the requested values to express them in the specified currency. The `currency` parameter accepts a built-in constant from the `currency.*` namespace, such as [currency.JPY](https://www.tradingview.com/pine-script-reference/v6/#var_currency.JPY), or a string representing a valid currency code (e.g., “JPY”). By default, this parameter accepts a “series” argument that can change across executions. However, if [dynamic requests](https://www.tradingview.com/pine-script-docs/concepts/other-timeframes-and-data/#dynamic-requests) are not enabled, it accepts only a value with the “simple” [qualifier](https://www.tradingview.com/pine-script-docs/language/type-system/#qualifiers) or a weaker one.

The conversion rate between the [syminfo.currency](https://www.tradingview.com/pine-script-reference/v6/#var_syminfo.currency) of the requested data and the specified `currency` depends on the _previous daily value_ of the corresponding currency pair from the most popular exchange. If no exchange provides the rate directly, the function derives the rate using a [spread symbol](https://www.tradingview.com/support/solutions/43000502298/).

> [!NOTE]
> Not all `request.*()` function calls return values expressed as a currency amount. Therefore, currency conversion is _not_ always necessary. For example, some of the series that the [request.financial()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.financial) function can retrieve — such as the “PIOTROSKI\_F\_SCORE” and “NUMBER\_OF\_EMPLOYEES” metrics — use units other than currency. It is up to programmers to determine when currency conversion is appropriate for their data requests.

### `lookahead` {#lookahead}

The `lookahead` parameter in [request.security()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.security), [request.dividends()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.dividends), [request.splits()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.splits), and [request.earnings()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.earnings) specifies the lookahead behavior of the function call. Its default value is [barmerge.lookahead\_off](https://www.tradingview.com/pine-script-reference/v6/#var_barmerge.lookahead_off).

When requesting data from a higher-timeframe (HTF) context, the `lookahead` value determines whether the `request.*()` function can return values from times _beyond_ those of the historical bars it executes on. In other words, the `lookahead` paremeter determines whether the requested data may contain _lookahead bias_ on historical bars.

When requesting data from a lower-timeframe (LTF) context, the `lookahead` parameter determines whether the function requests values from the first or last _intrabar_ (LTF bar) of each chart-timeframe bar.

**Programmers should exercise extreme caution when using lookahead in their requests, especially when requesting data from higher timeframes.** When using [barmerge.lookahead\_on](https://www.tradingview.com/pine-script-reference/v6/#var_barmerge.lookahead_on) as the `lookahead` value, ensure that it does not compromise the integrity of the script’s logic by leaking _future data_ into historical chart bars.

The following scenarios are cases where enabling lookahead is acceptable in a `request.*()` call:

-   The `expression` argument in a [request.security()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.security) call includes a _historical offset_ (e.g., `close[1]`), which prevents the function from requesting future values that it would **not** have access to on a realtime basis.
-   The `timeframe` argument of the call represents the same timeframe as that of the chart on which the script executes, i.e., [timeframe.period](https://www.tradingview.com/pine-script-reference/v6/#var_timeframe.period).
-   The function call requests data from an intrabar timeframe, i.e., a timeframe smaller than the [timeframe.period](https://www.tradingview.com/pine-script-reference/v6/#var_timeframe.period). See the [Lower-timeframes](https://www.tradingview.com/pine-script-docs/concepts/other-timeframes-and-data/#lower-timeframes) section for more information.

> [!IMPORTANT]
> Scripts that use [request.security()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.security) calls with lookahead to leak future data into the past are extremely **misleading**. As such, they are **not allowed** as script publications. Although the results of such a script might look great across history due to its apparent aquisition of prescience, those results are _unrealistic_ because the retrieved data was not knowable at the time of each bar. Furthermore, the same behavior is _impossible_ to reproduce on realtime bars. Therefore, before [publishing a script](https://www.tradingview.com/pine-script-docs/writing/publishing/) to share it with others, ensure that its requests **do not** mislead traders by using future data on historical bars.

This example demonstrates how the `lookahead` parameter affects the behavior of higher-timeframe data requests and why enabling lookahead in [request.security()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.security) without offsetting the `expression` is misleading. The script calls [request.security()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.security) to get the HTF [high](https://www.tradingview.com/pine-script-reference/v6/#var_high) price for the current chart’s symbol in three different ways and [plots](https://www.tradingview.com/pine-script-docs/visuals/plots/) the resulting series on the chart for comparison.

The first call uses [barmerge.lookahead\_off](https://www.tradingview.com/pine-script-reference/v6/#var_barmerge.lookahead_off) (default), and the others use [barmerge.lookahead\_on](https://www.tradingview.com/pine-script-reference/v6/#var_barmerge.lookahead_on). However, the third [request.security()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.security) call also _offsets_ its `expression` using the history-referencing operator [\[\]](https://www.tradingview.com/pine-script-reference/v6/#op_[]) to avoid leaking future data into the past.

As we see on the chart, the [plot](https://www.tradingview.com/pine-script-reference/v6/#fun_plot) of the series requested using [barmerge.lookahead\_on](https://www.tradingview.com/pine-script-reference/v6/#var_barmerge.lookahead_on) without an offset ([fuchsia](https://www.tradingview.com/pine-script-reference/v6/#var_color.fuchsia) line) shows final HTF [high](https://www.tradingview.com/pine-script-reference/v6/#var_high) prices _before_ they are actually available on historical bars, whereas the other two calls do not:

```pine
//@version=6
indicator("`lookahead` demo", overlay = true)

//@variable The timeframe to request the data from.
string timeframe = input.timeframe("30", "Timeframe")

//@variable The requested `high` price from the current symbol on the `timeframe` without lookahead bias.
//          On realtime bars, it returns the current `high` of the `timeframe`.
float lookaheadOff = request.security(syminfo.tickerid, timeframe, high, lookahead = barmerge.lookahead_off)

//@variable The requested `high` price from the current symbol on the `timeframe` with lookahead bias.
//          Returns values that should NOT be accessible yet on historical bars.
float lookaheadOn = request.security(syminfo.tickerid, timeframe, high, lookahead = barmerge.lookahead_on)

//@variable The requested `high` price from the current symbol on the `timeframe` without lookahead bias or repainting.
//          Behaves the same on historical and realtime bars.
float lookaheadOnOffset = request.security(syminfo.tickerid, timeframe, high[1], lookahead = barmerge.lookahead_on)

// Plot the values.
plot(lookaheadOff, "High, no lookahead bias", color.new(color.blue, 40), 5)
plot(lookaheadOn, "High with lookahead bias", color.fuchsia, 3)
plot(lookaheadOnOffset, "High, no lookahead bias or repaint", color.aqua, 3)
// Highlight the background on realtime bars.
bgcolor(barstate.isrealtime ? color.new(color.orange, 60) : na, title = "Realtime bar highlight")
```

Note that:

-   The series requested using [barmerge.lookahead\_off](https://www.tradingview.com/pine-script-reference/v6/#var_barmerge.lookahead_off) has a new historical value at the _end_ of each HTF period, and both series requested using [barmerge.lookahead\_on](https://www.tradingview.com/pine-script-reference/v6/#var_barmerge.lookahead_on) have new historical data at the _start_ of each period.
-   On realtime bars, the plot of the series without lookahead ([blue](https://www.tradingview.com/pine-script-reference/v6/#var_color.blue)) and the series with lookahead and no historical offset ([fuchsia](https://www.tradingview.com/pine-script-reference/v6/#var_color.fuchsia)) show the _same value_ (i.e., the HTF period’s unconfirmed [high](https://www.tradingview.com/pine-script-reference/v6/#var_high) price), as no data exists beyond those points to leak into the past. Both of these plots _repaint_ their results after the user reloads the script, because the _elapsed_ realtime bars from the previous run become _historical_ bars in the new run.
-   The series that uses lookahead and a historical offset ([aqua](https://www.tradingview.com/pine-script-reference/v6/#var_color.aqua)) _does not_ repaint its results, because it always uses the last _confirmed_ value from the higher timeframe. See the [Avoiding repainting](https://www.tradingview.com/pine-script-docs/concepts/other-timeframes-and-data/#avoiding-repainting) section of this page for more information.

> [!IMPORTANT]
> In Pine Script versions 1 and 2, the `security()` function did not include a `lookahead` parameter. However, the request behaved the same as those with `lookahead = barmerge.lookahead_on` in later versions of Pine, meaning that it systematically accessed future data from a higher timeframe on historical bars. Therefore, _exercise caution_ with Pine v1 or v2 scripts that use HTF `security()` calls, unless those calls offset the requested series with the [\[\]](https://www.tradingview.com/pine-script-reference/v6/#op_[]) operator.

### Dynamic requests {#dynamic-requests}

By default, unlike all previous Pine Script versions, `request.*()` function calls in Pine Script v6 are _dynamic_.

In contrast to non-dynamic requests, dynamic requests can:

-   Access data from different data feeds using a single `request.*()` instance with [“series” arguments](https://www.tradingview.com/pine-script-docs/concepts/other-timeframes-and-data/#series-arguments).
-   Execute within the [local scopes](https://www.tradingview.com/pine-script-docs/concepts/other-timeframes-and-data/#in-local-scopes) of [conditional structures](https://www.tradingview.com/pine-script-docs/language/conditional-structures/), [loops](https://www.tradingview.com/pine-script-docs/language/loops/), and [exported functions](https://www.tradingview.com/pine-script-docs/concepts/libraries/#library-functions).
-   Execute [nested requests](https://www.tradingview.com/pine-script-docs/concepts/other-timeframes-and-data/#nested-requests).

Aside from the features listed above, there are insignificant differences in the behavior of dynamic and non-dynamic requests. However, for backward compatibility, programmers can deactivate dynamic requests by specifying `dynamic_requests = false` in the [indicator()](https://www.tradingview.com/pine-script-reference/v6/#fun_indicator), [strategy()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy), or [library()](https://www.tradingview.com/pine-script-reference/v6/#fun_library) declaration statement.

> [!NOTE]
> In Pine Script v5, it is possible for scripts to call [user-defined functions](https://www.tradingview.com/pine-script-docs/language/user-defined-functions/) or [methods](https://www.tradingview.com/pine-script-docs/language/methods/#user-defined-methods) containing `request.*()` calls inside loops or conditional structures _without_ enabling dynamic requests. However, those wrapped requests are **not** truly dynamic, and they still require **“simple”** or weaker [qualifiers](https://www.tradingview.com/pine-script-docs/language/type-system/#qualifiers) for all arguments that define the requested context.
>
> In Pine Script v6, scripts **cannot** use wrapped `request.*()` calls within the local blocks of these structures without enabling dynamic requests.

#### ”series” arguments {#series-arguments}

Scripts without dynamic requests enabled cannot use “series” arguments for most `request.*()` function parameters, which means the argument values _cannot change_. The only exception is the `expression` parameter in [request.security()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.security), [request.security\_lower\_tf()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.security_lower_tf), and [request.seed()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.seed), which _always_ allows “series” values.

In contrast, when a script allows dynamic requests, all `request.*()` function parameters that define parts of the ticker ID or timeframe of a request accept “series” arguments that _can change_ with each script execution. In other words, with dynamic requests, it’s possible for a single `request.*()` instance to fetch data from _different contexts_ in different executions. Some other optional parameters, such as `ignore_invalid_symbol`, can also accept “series” arguments, allowing additional flexibility in `request.*()` call behaviors.

The following script declares a `symbolSeries` variable that is assigned four different symbol strings in 20-bar cycles, with its value changing after every five bars. The [request.security()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.security) call uses this variable as the `symbol` argument. The script plots the `requestedClose` values, which therefore represent a different symbol’s [close](https://www.tradingview.com/pine-script-reference/v6/#var_close) prices for each five-bar period.

```pine
//@version=6
indicator("'series' arguments demo")
 
//@variable A "series" that cycles through four different symbol strings. Its value changes every five bars.  
string symbolSeries = switch int(bar_index / 5) % 4
    1 => "NASDAQ:MSFT"
    2 => "NASDAQ:AMD"
    3 => "NASDAQ:INTC"
    =>   "AMEX:SPY"

//@variable The requested `close` value from one of the four `symbolSeries` values on the chart's timeframe.
float requestedClose = request.security(symbolSeries, timeframe.period, close)

// Plot the `requestedClose`.
plot(requestedClose, "Requested close", color.purple, 3)

// Draw a label displaying the requested symbol each time the `symbolSeries` changes.
if symbolSeries != symbolSeries[1]
    label.new(bar_index, requestedClose, symbolSeries, textcolor = color.white)
```

Note that:

-   The script draws a [label](https://www.tradingview.com/pine-script-reference/v6/#type_label) every time the `symbolSeries` changes, to signify which symbol’s data the `requestedClose` currently represents.
-   Pine v6 scripts enable dynamic requests by default, allowing this script to use a “series string” `symbol` argument in its [request.security()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.security) call without error. If the dynamic behavior is disabled by including `dynamic_requests = false` in the [indicator()](https://www.tradingview.com/pine-script-reference/v6/#fun_indicator) declaration, then the “series” argument causes a compilation error.

An important limitation is that when using dynamic `request.*()` calls with “series” arguments or within [local scopes](https://www.tradingview.com/pine-script-docs/concepts/other-timeframes-and-data/#in-local-scopes), scripts must request all required datasets while executing on **historical bars**. All `request.*()` calls on _realtime_ bars can retrieve data from the datasets that the script previously accessed on historical bars, but they **cannot** request a new context or evaluate a new expression.

To illustrate this limitation, let’s revisit the above script. Notice that it requests [close](https://www.tradingview.com/pine-script-reference/v6/#var_close) data for all four symbols on the chart’s timeframe during its historical executions. The external datasets for those four contexts are the **only** ones that any `request.*()` call on realtime bars can access.

Below, we changed the `timeframe` argument in the script’s [request.security()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.security) call to specify that it requests `symbolSeries` data from the chart’s timeframe on historical bars and the “240” (240 minutes = 4H) timeframe on realtime bars. This version raises a runtime error on the first realtime tick, if it is run on any timeframe other than the 4H timeframe, because it **cannot** access the 4H data feeds without requesting them on historical bars first:

```pine
//@version=6
indicator("Invalid realtime request demo")
 
//@variable A "series" that cycles through four different symbol strings. Its value changes every five bars.  
string symbolSeries = switch int(bar_index / 5) % 4
    1 => "NASDAQ:MSFT"
    2 => "NASDAQ:AMD"
    3 => "NASDAQ:INTC"
    =>   "AMEX:SPY"

// Request the `close` of the `symbolSeries` from the chart's timeframe on historical bars and the "240" (4H) timeframe 
// on realtime bars. Causes a runtime error on the first realtime tick because the script did not previously access 
// data from the "240" timeframe on any historical bars. 
float requestedClose = request.security(symbolSeries, barstate.isrealtime ? "240" : timeframe.period, close)

// Plot the `requestedClose`.
plot(requestedClose, "Requested close", color.purple, 3)

// Draw a label displaying the requested symbol each time the `symbolSeries` changes.
if symbolSeries != symbolSeries[1]
    label.new(bar_index, requestedClose, symbolSeries, textcolor = color.white)
```

#### In local scopes {#in-local-scopes}

When scripts do not allow dynamic requests, all `request.*()` calls execute once on _every_ bar or realtime tick, which prevents their use within most local scopes. The only exception is for `request.*()` calls in the scopes of _non-exported_ [functions](https://www.tradingview.com/pine-script-docs/language/user-defined-functions/) and [methods](https://www.tradingview.com/pine-script-docs/language/methods/#user-defined-methods), because the Pine Script compiler extracts such calls into the _global scope_ during translation.

Scripts that allow dynamic requests _do not_ restrict the execution of `request.*()` calls to the global scope. They can call `request.*()` functions directly within the scopes of [conditional structures](https://www.tradingview.com/pine-script-docs/language/conditional-structures/) and [loops](https://www.tradingview.com/pine-script-docs/language/loops/), meaning that each `request.*()` instance in the code can activate zero, one, or several times on each script execution.

The following example uses a single [request.security()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.security) instance within a loop to request data from multiple forex data feeds. The script declares an [array](https://www.tradingview.com/pine-script-reference/v6/#type_array) of `symbols` on the first chart bar, which it iterates through on all bars using a [for…in](https://www.tradingview.com/pine-script-reference/v6/#kw_for...in) loop. Each loop iteration calls [request.security()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.security) to retrieve the [volume](https://www.tradingview.com/pine-script-reference/v6/#var_volume) value for one of the symbols and pushes the result into the `requestedData` array. After the loop terminates, the script calculates the average, maximum, and minimum values from the `requestedData` array using built-in [methods](https://www.tradingview.com/pine-script-docs/language/methods/), then plots the results on the chart:

```pine
//@version=6
indicator("In local scopes demo", format = format.volume)

//@variable An array of "string" values representing different symbols to request. 
var array<string> symbols = array.from(
     "EURUSD", "USDJPY", "GBPUSD", "AUDUSD", "USDCAD", "USDCHF", "NZDUSD", "EURJPY", "GBPJPY", "EURGBP"
 )

//@variable An array containing the data retrieved for each requested symbol.  
array<float> requestedData = array.new<float>()

// Retrieve `volume` data for each symbol in the `symbols` array and push the results into the `requestedData` array. 
for symbol in symbols
    float data = request.security("OANDA:" + symbol, timeframe.period, volume)
    requestedData.push(data)

// Calculate the average, maximum, and minimum tick volume in the `requestedData`.
float avgVolume = requestedData.avg()
float maxVolume = requestedData.max()
float minVolume = requestedData.min()

// Plot the `avgVolume`, `maxVolume`, and `minVolume`. 
plot(avgVolume, "Average volume", color.gray,   3)
plot(maxVolume, "Highest volume", color.teal,   3)
plot(minVolume, "Lowest volume",  color.maroon, 3)
```

Notice that the `expression` argument in the above example ([volume](https://www.tradingview.com/pine-script-reference/v6/#var_volume)) is _loop-invariant_, i.e., it does not change on any loop iteration. When using `request.*()` calls within a loop, all parameters defining parts of the requested _context_ can accept arguments that depend on variables from the loop’s header or mutable variables that change within the loop’s local scope. However, a `request.*()` call’s evaluated expression **cannot** depend on the values of those variables.

Here, we modified the above script to use the _second form_ of the [for…in](https://www.tradingview.com/pine-script-reference/v6/#kw_for...in) loop statement, which creates a [tuple](https://www.tradingview.com/pine-script-docs/language/type-system/#tuples) containing the index and value of each element in the `symbols` array. The [request.security()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.security) instance in this version uses the index (`i`) in its `expression` argument, resulting in a _compilation error_:

```pine
//@version=6
indicator("Loop-dependent expression demo", format = format.volume)

//@variable An array of "string" values representing different symbols to request. 
var array<string> symbols = array.from(
     "EURUSD", "USDJPY", "GBPUSD", "AUDUSD", "USDCAD", "USDCHF", "NZDUSD", "EURJPY", "GBPJPY", "EURGBP"
 )

//@variable An array containing the data retrieved for each requested symbol.  
array<float> requestedData = array.new<float>()

// Retrieve `volume` data for each symbol in the `symbols` array, weighted using the element index.
// Causes a compilation error because the `expression` in `request.security()` cannot depend on loop variables 
// or mutable variables that change within the loop's scope. 
for [i, symbol] in symbols
    float data = request.security("OANDA:" + symbol, timeframe.period, volume * (10 - i))
    requestedData.push(data)

// Calculate the average, maximum, and minimum tick volume in the `requestedData`.
float avgVolume = requestedData.avg()
float maxVolume = requestedData.max()
float minVolume = requestedData.min()

// Plot the `avgVolume`, `maxVolume`, and `minVolume`. 
plot(avgVolume, "Average volume", color.gray,   3)
plot(maxVolume, "Highest volume", color.teal,   3)
plot(minVolume, "Lowest volume",  color.maroon, 3)
```

#### In libraries {#in-libraries}

[Libraries](https://www.tradingview.com/pine-script-docs/concepts/libraries/) with dynamic requests enabled can _export_ [functions](https://www.tradingview.com/pine-script-docs/language/user-defined-functions/) and [methods](https://www.tradingview.com/pine-script-docs/language/methods/#user-defined-methods) that utilize `request.*()` calls within their local scopes, provided that the evaluated expressions **do not** depend on any exported function parameters.

For example, this simple library exports an `htfPrices()` function that requests a [tuple](https://www.tradingview.com/pine-script-docs/language/type-system/#tuples) of confirmed [open](https://www.tradingview.com/pine-script-reference/v6/#var_open), [high](https://www.tradingview.com/pine-script-reference/v6/#var_high), [low](https://www.tradingview.com/pine-script-reference/v6/#var_low), and [close](https://www.tradingview.com/pine-script-reference/v6/#var_close) prices using a specified `tickerID` and `timeframe`. If we publish this library, another script can _import_ the function to request higher-timeframe prices without explicitly calling [request.security()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.security).

```pine
//@version=6
library("DynamicRequests")

//@function        Requests a tuple containing confirmed HTF OHLC data for a specified `tickerID` and `timeframe`.
//@param tickerID  The ticker identifier to request data for. 
//@param timeframe The timeframe of the requested data.
//@returns         A tuple containing the last confirmed `open`, `high`, `low`, and `close` from the requested context.
export htfPrices(string tickerID, string timeframe) =>
    if timeframe.in_seconds() >= timeframe.in_seconds(timeframe)
        runtime.error("The `timeframe` argument of `getHTFPrices()` must be higher than the chart's timeframe.")
    request.security(tickerID, timeframe, [open[1], high[1], low[1], close[1]], lookahead = barmerge.lookahead_on)
```

Note that:

-   The tuple that the [request.security()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.security) call includes as the `expression` argument _does not_ depend on the `htfPrices()` parameters.
-   The `htfPrices()` function includes a [runtime.error()](https://www.tradingview.com/pine-script-reference/v6/#fun_runtime.error) call that raises a custom runtime error when the `timeframe` argument does not represent a higher timeframe than the chart’s timeframe. See the [higher timeframes](https://www.tradingview.com/pine-script-docs/concepts/other-timeframes-and-data/#higher-timeframes) section for more information.
-   The [request.security()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.security) call uses [barmerge.lookahead\_on](https://www.tradingview.com/pine-script-reference/v6/#const_barmerge.lookahead_on) and offsets each item in the tuple by one bar. This is the only recommended method to [avoid repainting](https://www.tradingview.com/pine-script-docs/concepts/other-timeframes-and-data/#avoiding-repainting).

#### Nested requests {#nested-requests}

Scripts can use dynamic requests to execute _nested requests_, i.e., `request.*()` calls that dynamically evaluate other `request.*()` calls that their `expression` arguments depend on.

When a [request.security()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.security) or [request.security\_lower\_tf()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.security_lower_tf) call uses an empty string or [syminfo.tickerid](https://www.tradingview.com/pine-script-reference/v6/#var_syminfo.tickerid) for its `symbol` argument, or if it uses an empty string or [timeframe.period](https://www.tradingview.com/pine-script-reference/v6/#var_timeframe.period) for the `timeframe` argument, the requested ticker ID or timeframe _depends_ on the context where the call executes. This context is normally the ticker ID or timeframe of the chart that the script is running on. However, if such a [request.security()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.security) or [request.security\_lower\_tf()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.security_lower_tf) function call is evaluated by another `request.*()` call, the nested request _inherits_ that `request.*()` call’s ticker ID or timeframe information.

For example, the script below contains two [request.security()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.security) calls and uses [Pine Logs](https://www.tradingview.com/pine-script-docs/writing/debugging/#pine-logs) to display their results. The first call uses empty strings as its `symbol` and `timeframe` arguments, meaning that the requested context depends on where the call executes. It evaluates a concatenated string containing the call’s requested ticker ID and timeframe, and the script assigns its result to the `info1` variable.

The second call requests data for a specific `symbol` and `timeframe` using the `info1` variable as its `expression` argument. Since the `info1` variable depends on the first [request.security()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.security) call, the second call evaluates the first call _within_ its own context. Therefore, the first call adopts the second call’s ticker ID and timeframe while executing within that context, resulting in a different returned value:

```pine
//@version=6
indicator("Nested requests demo")

//@variable A concatenated string containing the current `syminfo.tickerid` and `timeframe.period`.
string info1 = request.security("", "", syminfo.tickerid + "_" + timeframe.period)
//@variable A concatenated string representing the `info1` value calculated within the "NASDAQ:AAPL, 240" context.
//          This call evaluates the call on line 5 within its context to determine its result because the script 
//          allows dynamic requests.
string info2 = request.security("NASDAQ:AAPL", "240", info1)

// Log the results from both calls in the Pine Logs pane on the last historical bar. 
if barstate.islastconfirmedhistory
    log.info("First request: {0}", info1)
    log.info("Second request: {0}", info2)
```

This script allows the execution of the first [request.security()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.security) call within the context of the second call because Pine v6 scripts enable dynamic `request.*()` calls by default. We can disable this behavior by including `dynamic_requests = false` in the [indicator()](https://www.tradingview.com/pine-script-reference/v6/#fun_indicator) declaration statement. Without dynamic requests enabled, the script evaluates each call _independently_, passing the first call’s calculated value directly into the second call rather than executing the first call within the second context. Consequently, the second call’s returned value is the _same_ as the first call’s value, as we see below:

```pine
//@version=6
indicator("Nested requests demo", dynamic_requests = false)

//@variable A concatenated string containing the current `syminfo.tickerid` and `timeframe.period`.
string info1 = request.security("", "", syminfo.tickerid + "_" + timeframe.period)
//@variable The same value as `info1`. This call does not evalutate the call on line 5 because dynamic requests aren't 
//          allowed. Instead, it only uses the value of `info1`, meaning its result does not change. 
string info2 = request.security("NASDAQ:AAPL", "240", info1)

// Log the results from both calls in the Pine Logs pane on the last historical bar. 
if barstate.islastconfirmedhistory
    log.info("First request: {0}", info1)
    log.info("Second request: {0}", info2)
```

## Data feeds {#data-feeds}

TradingView’s data providers supply different data feeds that scripts can access to retrieve information about an instrument, including:

-   Intraday historical data (for timeframes < 1D)
-   End-of-day (EOD) historical data (for timeframes >= 1D)
-   Realtime data (which may be delayed, depending on your account type and extra data services)
-   Extended hours data

Not all of these data feed types exist for every instrument. For example, the symbol “BNC:BLX” only has EOD data available.

For some instruments with intraday and EOD historical feeds, volume data may not be the same since some trades (block trades, OTC trades, etc.) may only be available at the _end_ of the trading day. Consequently, the EOD feed will include this volume data, but the intraday feed will not. Differences between EOD and intraday volume feeds are almost nonexistent for instruments such as cryptocurrencies, but they are commonplace in stocks.

Slight price discrepancies may also occur between EOD and intraday feeds. For example, the high value on one EOD bar may not match any intraday high values supplied by the data provider for that day.

Another distinction between EOD and intraday data feeds is that EOD feeds do not contain information from _extended hours_.

When retrieving information on realtime bars with `request.*()` functions, it’s important to note that historical and realtime data reported for an instrument often rely on _different_ data feeds. A broker/exchange may retroactively modify values reported on realtime bars, which the data will only reflect after refreshing the chart or restarting the script.

Another important consideration is that the chart’s data feeds and feeds requested from providers by the script are managed by _independent_, concurrent processes. Consequently, in some _rare_ cases, it’s possible for races to occur where requested results temporarily fall out of synch with the chart on a realtime bar, which a script retroactively adjusts after restarting its executions.

These points may account for variations in the values retrieved by `request.*()` functions when requesting data from other contexts. They may also result in discrepancies between data received on realtime bars and historical bars. There are no steadfast rules about the variations one may encounter in their requested data feeds.

> [!NOTE]
> As a rule, TradingView _does not_ generate data; it relies on its data providers for the information displayed on charts and accessed by scripts.

When using data feeds requested from other contexts, it’s also crucial to consider the _time axis_ differences between the chart the script executes on and the requested feeds since `request.*()` functions adapt the returned series to the chart’s time axis. For example, requesting “BTCUSD” data on the “SPY” chart with [request.security()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.security) will only show new values when the “SPY” chart has new data as well. Since “SPY” is not a 24-hour symbol, the “BTCUSD” data returned will contain gaps that are otherwise not present when viewing its chart directly.

## `request.security()` {#requestsecurity}

The [request.security()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.security) function allows scripts to request data from other contexts than the chart the script executes on, such as:

-   Other symbols, including [spread symbols](https://www.tradingview.com/support/solutions/43000502298/)
-   Other timeframes (see our User Manual’s page on [Timeframes](https://www.tradingview.com/pine-script-docs/concepts/timeframes/) to learn about timeframe specifications in Pine Script)
-   [Custom contexts](https://www.tradingview.com/pine-script-docs/concepts/other-timeframes-and-data/#custom-contexts), including alternative sessions, price adjustments, chart types, etc. using `ticker.*()` functions

This is the function’s signature:

```
request.security(symbol, timeframe, expression, gaps, lookahead, ignore_invalid_symbol, currency, calc_bars_count) → series <type>
```

The `symbol` value is the ticker identifier representing the symbol to fetch data from. This parameter accepts values in any of the following formats:

-   A “string” representing a symbol (e.g., “IBM” or “EURUSD”) or an _“Exchange:Symbol” pair_ (e.g., “NYSE:IBM” or “OANDA:EURUSD”). When the value does not contain an exchange prefix, the function selects the exchange automatically. We recommend specifying the exchange prefix when possible for consistent results. Users can also pass an empty string to this parameter, which prompts the function to use the current chart’s symbol.
-   A “string” representing a [spread symbol](https://www.tradingview.com/support/solutions/43000502298/) (e.g., “AMD/INTC”). Note that “Bar Replay” mode does not work with these symbols.
-   The [syminfo.ticker](https://www.tradingview.com/pine-script-reference/v6/#var_syminfo.ticker) or [syminfo.tickerid](https://www.tradingview.com/pine-script-reference/v6/#var_syminfo.tickerid) built-in variables, which return the symbol or the “Exchange:Symbol” pair that the current chart references. We recommend using [syminfo.tickerid](https://www.tradingview.com/pine-script-reference/v6/#var_syminfo.tickerid) to avoid ambiguity unless the exchange information does not matter in the data request. For more information on `syminfo.*` variables, see [this](https://www.tradingview.com/pine-script-docs/concepts/chart-information/#symbol-information) section of our [Chart information](https://www.tradingview.com/pine-script-docs/concepts/chart-information/) page.
-   A custom ticker identifier created using `ticker.*()` functions. Ticker IDs constructed from these functions may contain additional settings for requesting data using [non-standard chart](https://www.tradingview.com/pine-script-docs/concepts/non-standard-charts-data/) calculations, alternative sessions, and other contexts. See the [Custom contexts](https://www.tradingview.com/pine-script-docs/concepts/other-timeframes-and-data/#custom-contexts) section for more information.

The `timeframe` value specifies the timeframe of the requested data. This parameter accepts “string” values in our [timeframe specification](https://www.tradingview.com/pine-script-docs/concepts/timeframes/#timeframe-string-specifications) format (e.g., a value of “1D” represents the daily timeframe). To request data from the same timeframe as the chart the script executes on, use the [timeframe.period](https://www.tradingview.com/pine-script-reference/v6/#var_timeframe.period) variable or an empty string.

The `expression` parameter of the [request.security()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.security) function determines the data it retrieves from the specified context. This versatile parameter accepts “series” values of [int](https://www.tradingview.com/pine-script-docs/language/type-system/#int), [float](https://www.tradingview.com/pine-script-docs/language/type-system/#float), [bool](https://www.tradingview.com/pine-script-docs/language/type-system/#bool), [color](https://www.tradingview.com/pine-script-docs/language/type-system/#color), [string](https://www.tradingview.com/pine-script-docs/language/type-system/#string), and [chart.point](https://www.tradingview.com/pine-script-docs/language/type-system/#chart-points) types. It can also accept [tuples](https://www.tradingview.com/pine-script-docs/language/type-system/#tuples), [collections](https://www.tradingview.com/pine-script-docs/language/type-system/#collections), [user-defined types](https://www.tradingview.com/pine-script-docs/language/type-system/#user-defined-types), and the outputs of function and [method](https://www.tradingview.com/pine-script-docs/language/methods/) calls. For more details on the data one can retrieve, see the [Requestable data](https://www.tradingview.com/pine-script-docs/concepts/other-timeframes-and-data/#requestable-data) section below.

> [!IMPORTANT]
> If a `request.*()` call uses the value from a [source input](https://www.tradingview.com/pine-script-docs/concepts/inputs/#source-input) in its `expression` argument, and that input accesses a plotted series from another indicator, the request evaluates that series using the data for the **chart’s symbol**, and **not** the data for the specified symbol. This behavior occurs because `request.*()` functions cannot evaluate the scopes required by an external series. Therefore, some `request.*()` calls that use a `symbol` argument and request the value of a source input can return unintended results.

### Timeframes {#timeframes}

The [request.security()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.security) function can request data from any available timeframe, regardless of the chart the script executes on. The timeframe of the data retrieved depends on the `timeframe` argument in the function call, which may represent a higher timeframe (e.g., using “1D” as the `timeframe` value while running the script on an intraday chart) or the chart’s timeframe (i.e., using [timeframe.period](https://www.tradingview.com/pine-script-reference/v6/#var_timeframe.period) or an empty string as the `timeframe` argument).

Scripts can also request _limited_ data from lower timeframes with [request.security()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.security) (e.g., using “1” as the `timeframe` argument while running the script on a 60-minute chart). However, we don’t typically recommend using this function for LTF data requests. The [request.security\_lower\_tf()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.security_lower_tf) function is more optimal for such cases.

#### Higher timeframes {#higher-timeframes}

Most use cases of [request.security()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.security) involve requesting data from a timeframe higher than or the same as the chart timeframe. For example, this script retrieves the [hl2](https://www.tradingview.com/pine-script-reference/v6/#var_hl2) price from a requested `higherTimeframe`. It [plots](https://www.tradingview.com/pine-script-docs/visuals/plots/) the resulting series on the chart alongside the current chart’s [hl2](https://www.tradingview.com/pine-script-reference/v6/#var_hl2) for comparison:

```pine
//@version=6
indicator("Higher timeframe security demo", overlay = true)

//@variable The higher timeframe to request data from.
string higherTimeframe = input.timeframe("240", "Higher timeframe")

//@variable The `hl2` value from the `higherTimeframe`. Combines lookahead with an offset to avoid repainting.
float htfPrice = request.security(syminfo.tickerid, higherTimeframe, hl2[1], lookahead = barmerge.lookahead_on)

// Plot the `hl2` from the chart timeframe and the `higherTimeframe`.
plot(hl2, "Current timeframe HL2", color.teal, 2)
plot(htfPrice, "Higher timeframe HL2", color.purple, 3)
```

Note that:

-   We’ve included an offset to the `expression` argument and used [barmerge.lookahead\_on](https://www.tradingview.com/pine-script-reference/v6/#var_barmerge.lookahead_on) in [request.security()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.security) to ensure the series returned behaves the same on historical and realtime bars. See the [Avoiding repainting](https://www.tradingview.com/pine-script-docs/concepts/other-timeframes-and-data/#avoiding-repainting) section for more information.

Notice that in the above example, it is possible to select a `higherTimeframe` value that actually represents a _lower timeframe_ than the one the chart uses, as the code does not prevent it. When designing a script to work specifically with higher timeframes, we recommend including conditions to prevent it from accessing lower timeframes, especially if you intend to [publish](https://www.tradingview.com/pine-script-docs/writing/publishing/) it.

Below, we’ve added an [if](https://www.tradingview.com/pine-script-reference/v6/#kw_if) structure to our previous example. If the `higherTimeframe` value represents a timeframe that is smaller than the chart’s timeframe, the script calls [runtime.error()](https://www.tradingview.com/pine-script-reference/v6/#fun_runtime.error) within the structure’s local block to raise a custom runtime error, effectively preventing the script from requesting LTF data:

```pine
//@version=6
indicator("Higher timeframe security demo", overlay = true)

//@variable The higher timeframe to request data from.
string higherTimeframe = input.timeframe("240", "Higher timeframe")

// Raise a runtime error when the `higherTimeframe` is smaller than the chart's timeframe.
if timeframe.in_seconds() > timeframe.in_seconds(higherTimeframe)
    runtime.error("The requested timeframe is smaller than the chart's timeframe. Select a higher timeframe.")

//@variable The `hl2` value from the `higherTimeframe`. Combines lookahead with an offset to avoid repainting.
float htfPrice = request.security(syminfo.tickerid, higherTimeframe, hl2[1], lookahead = barmerge.lookahead_on)

// Plot the `hl2` from the chart timeframe and the `higherTimeframe`.
plot(hl2, "Current timeframe HL2", color.teal, 2)
plot(htfPrice, "Higher timeframe HL2", color.purple, 3)
```

#### Lower timeframes {#lower-timeframes}

Although the [request.security()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.security) function is intended to operate on timeframes greater than or equal to the chart timeframe, it _can_ request data from lower timeframes as well, with limitations. When calling this function to access a lower timeframe, it will evaluate the `expression` from the LTF context. However, it returns the results from only a _single_ intrabar (LTF bar) on each chart bar.

The intrabar that the function returns data from on each historical chart bar depends on the `lookahead` value in the function call. When using [barmerge.lookahead\_on](https://www.tradingview.com/pine-script-reference/v6/#var_barmerge.lookahead_on), it will return the _first_ available intrabar from the chart period. When using [barmerge.lookahead\_off](https://www.tradingview.com/pine-script-reference/v6/#var_barmerge.lookahead_off), it will return the _last_ intrabar from the chart period. On realtime bars, it returns the last available value of the `expression` from the timeframe, regardless of the `lookahead` value, as the realtime intrabar information retrieved by the function is not yet sorted.

This script retrieves [close](https://www.tradingview.com/pine-script-reference/v6/#var_close) data from the valid timeframe closest to a fourth of the size of the chart timeframe. It makes two calls to [request.security()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.security) with different `lookahead` values. The first call uses [barmerge.lookahead\_on](https://www.tradingview.com/pine-script-reference/v6/#var_barmerge.lookahead_on) to access the first intrabar value in each chart bar. The second uses the default `lookahead` value ([barmerge.lookahead\_off](https://www.tradingview.com/pine-script-reference/v6/#var_barmerge.lookahead_off)), which requests the last intrabar value assigned to each chart bar. The script [plots](https://www.tradingview.com/pine-script-docs/visuals/plots/) the outputs of both calls on the chart to compare the difference:

```pine
//@version=6
indicator("Lower timeframe security demo", overlay = true)

//@variable The valid timeframe closest to 1/4 the size of the chart timeframe.
string lowerTimeframe = timeframe.from_seconds(int(timeframe.in_seconds() / 4))

//@variable The `close` value on the `lowerTimeframe`. Represents the first intrabar value on each chart bar.
float firstLTFClose = request.security(syminfo.tickerid, lowerTimeframe, close, lookahead = barmerge.lookahead_on)
//@variable The `close` value on the `lowerTimeframe`. Represents the last intrabar value on each chart bar.
float lastLTFClose = request.security(syminfo.tickerid, lowerTimeframe, close)

// Plot the values.
plot(firstLTFClose, "First intrabar close", color.teal, 3)
plot(lastLTFClose, "Last intrabar close", color.purple, 3)
// Highlight the background on realtime bars.
bgcolor(barstate.isrealtime ? color.new(color.orange, 70) : na, title = "Realtime background highlight")
```

Note that:

-   The script determines the value of the `lowerTimeframe` by calculating the number of seconds in the chart timeframe with [timeframe.in\_seconds()](https://www.tradingview.com/pine-script-reference/v6/#fun_timeframe.in_seconds), then dividing by four and converting the result to a [valid timeframe string](https://www.tradingview.com/pine-script-docs/concepts/timeframes/#timeframe-string-specifications) via [timeframe.from\_seconds()](https://www.tradingview.com/pine-script-reference/v6/#fun_timeframe.from_seconds).
-   The plot of the series without lookahead ([purple](https://www.tradingview.com/pine-script-reference/v6/#var_color.purple)) aligns with the [close](https://www.tradingview.com/pine-script-reference/v6/#var_close) value on the chart timeframe, as this is the last intrabar value in the chart bar.
-   Both [request.security()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.security) calls return the _same_ value (the current [close](https://www.tradingview.com/pine-script-reference/v6/#var_close)) on each _realtime_ bar, as shown on the bars with the [orange](https://www.tradingview.com/pine-script-reference/v6/#var_color.orange) background.
-   Scripts can retrieve up to 200,000 intrabars from a lower-timeframe context. The number of chart bars with available intrabar data varies with the requested lower timeframe, the `calc_bars_count` value, and the user’s plan. For more information, see [this](https://www.tradingview.com/pine-script-docs/writing/limitations/#intrabars) section of the [Limitations](https://www.tradingview.com/pine-script-docs/writing/limitations/) page.

> [!TIP]
> While scripts can use [request.security()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.security) to retrieve limited intrabar data, we recommend using [request.security\_lower\_tf()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.security_lower_tf) function for such requests in most cases. Instead of retrieving data for only a single LTF bar on each chart bar, it returns an [array](https://www.tradingview.com/pine-script-reference/v6/#type_array) containing the data for _all_ available LTF bars in the chart bar. See the [`request.security_lower_tf()` section](https://www.tradingview.com/pine-script-docs/concepts/other-timeframes-and-data/#requestsecurity_lower_tf) below to learn more.

### Requestable data {#requestable-data}

The [request.security()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.security) function is quite versatile, as it can retrieve values of any fundamental type ([int](https://www.tradingview.com/pine-script-docs/language/type-system/#int), [float](https://www.tradingview.com/pine-script-docs/language/type-system/#float), [bool](https://www.tradingview.com/pine-script-docs/language/type-system/#bool), [color](https://www.tradingview.com/pine-script-docs/language/type-system/#color), or [string](https://www.tradingview.com/pine-script-docs/language/type-system/#string)). It can also request the IDs of data structures and built-in or [user-defined types](https://www.tradingview.com/pine-script-docs/language/type-system/#user-defined-types) that reference fundamental types. The data this function requests depends on its `expression` parameter, which accepts any of the following arguments:

-   [Built-in variables and function calls](https://www.tradingview.com/pine-script-docs/concepts/other-timeframes-and-data/#built-in-variables-and-functions)
-   [Variables declared by the script](https://www.tradingview.com/pine-script-docs/concepts/other-timeframes-and-data/#declared-variables)
-   [Tuples](https://www.tradingview.com/pine-script-docs/concepts/other-timeframes-and-data/#tuples)
-   [Calls to user-defined functions](https://www.tradingview.com/pine-script-docs/concepts/other-timeframes-and-data/#user-defined-functions)
-   [Chart points](https://www.tradingview.com/pine-script-docs/concepts/other-timeframes-and-data/#chart-points)
-   [Collections](https://www.tradingview.com/pine-script-docs/concepts/other-timeframes-and-data/#collections)
-   [User-defined types](https://www.tradingview.com/pine-script-docs/concepts/other-timeframes-and-data/#user-defined-types)

#### Built-in variables and functions {#built-in-variables-and-functions}

A frequent use case of [request.security()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.security) is requesting the output of a built-in variable or function/[method](https://www.tradingview.com/pine-script-docs/language/methods/) call from another symbol or timeframe.

For example, suppose we want to calculate the 20-bar SMA of a symbol’s [ohlc4](https://www.tradingview.com/pine-script-reference/v6/#var_ohlc4) prices from the daily timeframe while on an intraday chart. We can accomplish this task with a single line of code:

```pine
float ma = request.security(syminfo.tickerid, "1D", ta.sma(ohlc4, 20))
```

The above line calculates the value of `ta.sma(ohlc4, 20)` on the current symbol’s data from the daily timeframe.

It’s important to note that newcomers to Pine might sometimes confuse the above line of code as being equivalent to the following:

```pine
float ma = ta.sma(request.security(syminfo.tickerid, "1D", ohlc4), 20)
```

However, this line returns an entirely _different_ result. Rather than requesting a 20-bar SMA from the daily timeframe, it requests the [ohlc4](https://www.tradingview.com/pine-script-reference/v6/#var_ohlc4) price from the daily timeframe and calclates the [ta.sma()](https://www.tradingview.com/pine-script-reference/v6/#fun_ta.sma) of the results over 20 **chart bars**.

In essence, when the intention is to request the results of an expression from other contexts, pass the expression _directly_ to the `expression` parameter in the [request.security()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.security) call, as demonstrated in the initial example.

Let’s expand on this concept. The script below calculates a multi-timeframe (MTF) ribbon of moving averages, where each moving average in the ribbon calculates over the same number of bars on its respective timeframe. Each [request.security()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.security) call uses a [ta.sma()](https://www.tradingview.com/pine-script-reference/v6/#fun_ta.sma) call as its `expression` argument to return a `length`\-bar SMA from the specified timeframe:

```pine
//@version=6
indicator("Requesting built-ins demo", "MTF Ribbon", true)

//@variable The length of each moving average.
int length = input.int(20, "Length", 1)

//@variable The number of seconds in the chart timeframe.
int chartSeconds = timeframe.in_seconds()

// Calculate the higher timeframes closest to 2, 3, and 4 times the size of the chart timeframe.
string htf1 = timeframe.from_seconds(chartSeconds * 2)
string htf2 = timeframe.from_seconds(chartSeconds * 3)
string htf3 = timeframe.from_seconds(chartSeconds * 4)

// Calculate the `length`-bar moving averages from each timeframe.
float chartAvg = ta.sma(ohlc4, length)
float htfAvg1  = request.security(syminfo.tickerid, htf1, ta.sma(ohlc4, length))
float htfAvg2  = request.security(syminfo.tickerid, htf2, ta.sma(ohlc4, length))
float htfAvg3  = request.security(syminfo.tickerid, htf3, ta.sma(ohlc4, length))

// Plot the results.
plot(chartAvg, "Chart timeframe SMA", color.red, 3)
plot(htfAvg1, "Double timeframe SMA", color.orange, 3)
plot(htfAvg2, "Triple timeframe SMA", color.green, 3)
plot(htfAvg3, "Quadruple timeframe SMA", color.blue, 3)

// Highlight the background on realtime bars.
bgcolor(barstate.isrealtime ? color.new(color.aqua, 70) : na, title = "Realtime highlight")
```

Note that:

-   The script calculates the ribbon’s higher timeframes by multiplying the chart’s [timeframe.in\_seconds()](https://www.tradingview.com/pine-script-reference/v6/#fun_timeframe.in_seconds) value by 2, 3, and 4, then converting each result into a [valid timeframe string](https://www.tradingview.com/pine-script-docs/concepts/timeframes/#timeframe-string-specifications) using [timeframe.from\_seconds()](https://www.tradingview.com/pine-script-reference/v6/#fun_timeframe.from_seconds).
-   Instead of calling [ta.sma()](https://www.tradingview.com/pine-script-reference/v6/#fun_ta.sma) within each [request.security()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.security) call, one could use the `chartAvg` variable as the `expression` in each call to achieve the same result. See the [next section](https://www.tradingview.com/pine-script-docs/concepts/other-timeframes-and-data/#declared-variables) for more information.
-   On realtime bars, this script also tracks _unconfirmed_ SMA values from each higher timeframe. See the [Historical and realtime behavior](https://www.tradingview.com/pine-script-docs/concepts/other-timeframes-and-data/#historical-and-realtime-behavior) section to learn more.

#### Declared variables {#declared-variables}

The [request.security()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.security) function’s `expression` parameter can accept declared variables that are accessible to the scope from which the function call executes. When using a declared variable as the `expression` argument, the function call _duplicates_ all _preceding code_ that determines the assigned value or reference. This duplication allows the function to evaluate necessary calculations and logic in the requested context without affecting the original variable.

For instance, this line of code declares a `priceReturn` variable that holds the current bar’s arithmetic price return:

```pine
float priceReturn = (close - close[1]) / close[1]
```

We can evaluate the `priceReturn` variable’s calculations in another context by using it as the `expression` in a [request.security()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.security) call. The call below duplicates the variable’s calculation and evaluates it across the data from another `symbol`, returning a _separate series_ adapted to the chart’s time axis:

```pine
float requestedReturn = request.security(symbol, timeframe.period, priceReturn)
```

This example script compares the price returns of the current chart’s symbol and a user-specified symbol. It calculates the value of the `priceReturn` variable, then uses that variable as the `expression` in a [request.security()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.security) call to evaluate the calculation on the input symbol’s data. After the request, the script calculates the correlation between the `priceReturn` and `requestedReturn` series using [ta.correlation()](https://www.tradingview.com/pine-script-reference/v6/#fun_ta.correlation) and plots the result on the chart:

```pine
//@version=6
indicator("Requesting calculated variables demo", "Price return correlation")

//@variable The symbol to compare to the chart symbol.
string symbol = input.symbol("SPY", "Symbol to compare")
//@variable The number of bars in the calculation window.
int length = input.int(60, "Length", 1)

//@variable The close-to-close price return.
float priceReturn = (close - close[1]) / close[1]
//@variable The close-to-close price return calculated on another `symbol`.
float requestedReturn = request.security(symbol, timeframe.period, priceReturn)

//@variable The correlation between the `priceReturn` and `requestedReturn` over `length` bars.
float correlation = ta.correlation(priceReturn, requestedReturn, length)
//@variable The color of the correlation plot.
color plotColor = color.from_gradient(correlation, -1, 1, color.purple, color.orange)

// Plot the correlation value.
plot(correlation, "Correlation", plotColor, style = plot.style_area)
```

Note that:

-   The [request.security()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.security) call executes the same calculation used in the `priceReturn` declaration, but the request’s calculation operates on the [close](https://www.tradingview.com/pine-script-reference/v6/#var_close) values from the specified symbol’s data.
-   The script uses the [color.from\_gradient()](https://www.tradingview.com/pine-script-reference/v6/#fun_color.from_gradient) function to calculate the color for the plot of the `correlation` series on each bar. See [this section](https://www.tradingview.com/pine-script-docs/visuals/colors/#colorfrom_gradient) of the [Colors](https://www.tradingview.com/pine-script-docs/visuals/colors/) page to learn more about color gradients.

When using a variable as the `expression` argument of a `request.*()` call, it’s important to note that the function only duplicates code that affects the variable _before_ the call. It _cannot_ copy any subsequent code following the call. Consequently, if the script reassigns the variable or modifies its referenced data _after_ calling [request.security()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.security), the code evaluated on the requested data **does not** include those additional operations.

For example, the following script declares a `counter` variable and calls [request.security()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.security) to evaluate the variable from the same context as the chart. After the call, the script increments the `counter` value by one with the addition assignment operator ([+=](https://www.tradingview.com/pine-script-reference/v6/#op_+=)), then uses plots and [Pine Logs](https://www.tradingview.com/pine-script-docs/writing/debugging/#pine-logs) to display the `counter` and `requestedCounter` values for comparison.

As shown below, the plots and logs of the two variables display _different_ values. The `requestedCounter` variable has a consistent value of 0 because the [request.security()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.security) call evaluates only the initial variable declaration. The request cannot evaluate the addition assignment operation because the script includes that code _after_ the function call:

```pine
//@version=6
indicator("Modifying variables after requests demo")

//@variable A counter that starts at 0 and increments by 1 on each bar. 
var int counter = 0

//@variable Holds a consistent value of 0. 
//          `request.security()` cannot evaluate `counter += 1` in its requested context 
//          because that modification occurs *after* the call. 
int requestedCounter = request.security(syminfo.tickerid, timeframe.period, counter)

// Increment the `counter` by 1. This operation is *not* included in the `requestedCounter` calculation.
counter += 1

// Plot both variables for comparison. 
plot(counter, "Original counter", color.purple, 3)
plot(requestedCounter, "Requested counter", color.red, 3)

// Log the values of both variables in the Pine Logs pane.
if barstate.isconfirmed
    log.info("counter: {0}, requestedCounter: {1}", counter, requestedCounter)
```

#### Tuples {#tuples}

[Tuples](https://www.tradingview.com/pine-script-docs/language/type-system/#tuples) in Pine Script are comma-separated lists of expressions enclosed in square brackets. Programmers often use tuples when creating [functions](https://www.tradingview.com/pine-script-docs/language/user-defined-functions/), [conditional structures](https://www.tradingview.com/pine-script-docs/language/conditional-structures/), or [loops](https://www.tradingview.com/pine-script-docs/language/loops/) that return multiple values or references from their local scopes.

The [request.security()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.security) function can accept a tuple as its `expression` argument, allowing scripts to request multiple series of different types using a single function call. The expressions within requested tuples can be of any type outlined throughout the [Requestable data](https://www.tradingview.com/pine-script-docs/concepts/other-timeframes-and-data/#requestable-data) section of this page, excluding other tuples.

> [!NOTE]
> The combined size of all tuples returned by `request.*()` calls in a script cannot exceed 127 elements. See the [Tuple element limit](https://www.tradingview.com/pine-script-docs/writing/limitations/#tuple-element-limit) section of the [Limitations](https://www.tradingview.com/pine-script-docs/writing/limitations/) page for more information.

Tuples are particularly helpful when a script needs to retrieve more than one value from a specific context.

For example, the following script calculates the percent rank of the [close](https://www.tradingview.com/pine-script-reference/v6/#var_close) series over `length` bars and assigns the result to the `rank` variable. It then calls [request.security()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.security) to request a tuple containing the values of `rank`, `ta.crossover(rank, 50)`, and `ta.crossunder(rank, 50)` from a specified timeframe. The script plots the `requestedRank` series in a separate pane, then uses the result of a ternary expression based on the `crossOver` and `crossUnder` values within a [bgcolor()](https://www.tradingview.com/pine-script-reference/v6/#fun_bgcolor) call to conditionally highlight the pane’s background:

```pine
//@version=6
indicator("Requesting tuples demo", "Percent rank cross")

//@variable The timeframe of the request.
string timeframe = input.timeframe("240", "Timeframe")
//@variable The number of bars in the calculation.
int length = input.int(20, "Length")

//@variable The previous bar's percent rank of the `close` price over `length` bars.
float rank = ta.percentrank(close, length)[1]

// Request the `rank` value from another `timeframe`, and two "bool" values indicating the `rank` from the `timeframe`
// crossed over or under 50.
[requestedRank, crossOver, crossUnder] = request.security(
     syminfo.tickerid, timeframe, [rank, ta.crossover(rank, 50), ta.crossunder(rank, 50)],
     lookahead = barmerge.lookahead_on
 )

// Plot the `requestedRank` and create a horizontal line at 50.
plot(requestedRank, "Percent Rank", linewidth = 3)
hline(50, "Cross line", linewidth = 2)
// Highlight the background of all bars where the `timeframe`'s `crossOver` or `crossUnder` value is `true`.
bgcolor(crossOver ? color.new(color.green, 50) : crossUnder ? color.new(color.red, 50) : na)
```

Note that:

-   We’ve offset the `rank` variable’s expression by one bar using the history-referencing operator [\[\]](https://www.tradingview.com/pine-script-reference/v6/#op_[]) and included [barmerge.lookahead\_on](https://www.tradingview.com/pine-script-reference/v6/#var_barmerge.lookahead_on) in the [request.security()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.security) call to ensure the values on realtime bars do not repaint after becoming historical bars. See the [Avoiding repainting](https://www.tradingview.com/pine-script-docs/concepts/other-timeframes-and-data/#avoiding-repainting) section for more information.
-   The [request.security()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.security) call returns a tuple, so we use a _tuple declaration_ to declare the `requestedRank`, `crossOver`, and `crossUnder` variables. To learn more about using tuples, see [this section](https://www.tradingview.com/pine-script-docs/language/type-system/#tuples) of our User Manual’s [Type system](https://www.tradingview.com/pine-script-docs/language/type-system/) page.

#### User-defined functions {#user-defined-functions}

[User-defined functions](https://www.tradingview.com/pine-script-docs/language/user-defined-functions/) and [methods](https://www.tradingview.com/pine-script-docs/language/methods/#user-defined-methods) are custom functions written by users. They allow users to define sequences of operations associated with an identifier that scripts can conveniently call throughout their executions (e.g., `myUDF()`).

The [request.security()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.security) function can request the results of [user-defined functions](https://www.tradingview.com/pine-script-docs/language/user-defined-functions/) and [methods](https://www.tradingview.com/pine-script-docs/language/methods/#user-defined-methods) whose scopes consist of any types outlined throughout this page’s [Requestable data](https://www.tradingview.com/pine-script-docs/concepts/other-timeframes-and-data/#requestable-data) section.

For example, this script contains a user-defined `weightedBB()` function that calculates Bollinger Bands with the basis average weighted by a specified `weight` series. The function returns a [tuple](https://www.tradingview.com/pine-script-docs/language/type-system/#tuples) of custom band values. The script calls the `weightedBB()` as the `expression` argument in [request.security()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.security) to retrieve a [tuple](https://www.tradingview.com/pine-script-docs/concepts/other-timeframes-and-data/#tuples) of band values calculated on the specified `timeframe` and [plots](https://www.tradingview.com/pine-script-docs/visuals/plots/) the results on the chart:

```pine
//@version=6
indicator("Requesting user-defined functions demo", "Weighted Bollinger Bands", true)

//@variable The timeframe of the request.
string timeframe = input.timeframe("480", "Timeframe")

//@function     Calculates Bollinger Bands with a custom weighted basis.
//@param source The series of values to process.
//@param length The number of bars in the calculation.
//@param mult   The standard deviation multiplier.
//@param weight The series of weights corresponding to each `source` value.
//@returns      A tuple containing the basis, upper band, and lower band respectively.
weightedBB(float source, int length, float mult = 2.0, float weight = 1.0) =>
    //@variable The basis of the bands.
    float ma = math.sum(source * weight, length) / math.sum(weight, length)
    //@variable The standard deviation from the `ma`.
    float dev = 0.0
    // Loop to accumulate squared error.
    for i = 0 to length - 1
        difference = source[i] - ma
        dev += difference * difference
    // Divide `dev` by the `length`, take the square root, and multiply by the `mult`.
    dev := math.sqrt(dev / length) * mult
    // Return the bands.
    [ma, ma + dev, ma - dev]

// Request weighted bands calculated on the chart symbol's prices over 20 bars from the
// last confirmed bar on the `timeframe`.
[basis, highBand, lowBand] = request.security(
     syminfo.tickerid, timeframe, weightedBB(close[1], 20, 2.0, (high - low)[1]), lookahead = barmerge.lookahead_on
 )

// Plot the values.
basisPlot = plot(basis, "Basis", color.orange, 2)
upperPlot = plot(highBand, "Upper", color.teal, 2)
lowerPlot = plot(lowBand, "Lower", color.maroon, 2)
fill(upperPlot, lowerPlot, color.new(color.gray, 90), "Background")
```

Note that:

-   We offset the `source` and `weight` arguments in the `weightedBB()` call used as the `expression` in [request.security()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.security) and used [barmerge.lookahead\_on](https://www.tradingview.com/pine-script-reference/v6/#var_barmerge.lookahead_on) to ensure the requested results reflect the last confirmed values from the `timeframe` on realtime bars. See [this section](https://www.tradingview.com/pine-script-docs/concepts/other-timeframes-and-data/#avoiding-repainting) to learn more.

#### Chart points {#chart-points}

[Chart points](https://www.tradingview.com/pine-script-docs/language/type-system/#chart-points) are objects that represent coordinates on the chart. [Lines](https://www.tradingview.com/pine-script-docs/visuals/lines-and-boxes/#lines), [boxes](https://www.tradingview.com/pine-script-docs/visuals/lines-and-boxes/#boxes), [polylines](https://www.tradingview.com/pine-script-docs/visuals/lines-and-boxes/#polylines), and [labels](https://www.tradingview.com/pine-script-docs/visuals/text-and-shapes/#labels) use these objects to set their display locations.

The [request.security()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.security) function can use the ID of a [chart.point](https://www.tradingview.com/pine-script-reference/v6/#type_chart.point) instance in its `expression` argument, allowing scripts to retrieve chart coordinates from other contexts.

The example below requests a tuple of historical [chart points](https://www.tradingview.com/pine-script-docs/language/type-system/#chart-points) from a higher timeframe and uses them to draw [boxes](https://www.tradingview.com/pine-script-docs/visuals/lines-and-boxes/#boxes) on the chart. The script declares the `topLeft` and `bottomRight` variables that reference [chart.point](https://www.tradingview.com/pine-script-reference/v6/#type_chart.point) IDs from the last confirmed bar. It then uses [request.security()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.security) to request a [tuple](https://www.tradingview.com/pine-script-docs/concepts/other-timeframes-and-data/#tuples) containing the IDs of [chart points](https://www.tradingview.com/pine-script-docs/language/type-system/#chart-points) representing the `topLeft` and `bottomRight` from a `higherTimeframe`.

When a new bar starts on the `higherTimeframe`, the script draws a new box using the `time` and `price` coordinates from the `requestedTopLeft` and `requestedBottomRight` chart points:

```pine
//@version=6
indicator("Requesting chart points demo", "HTF Boxes", true, max_boxes_count = 500)

//@variable The timeframe to request data from.
string higherTimeframe = input.timeframe("1D", "Timeframe")

// Raise a runtime error if the `higherTimeframe` is smaller than the chart's timeframe.
if timeframe.in_seconds(higherTimeframe) < timeframe.in_seconds(timeframe.period)
    runtime.error("The selected timeframe is too small. Choose a higher timeframe.")

//@variable A `chart.point` containing top-left coordinates from the last confirmed bar.
topLeft = chart.point.now(high)[1]
//@variable A `chart.point` containing bottom-right coordinates from the last confirmed bar.
bottomRight = chart.point.from_time(time_close, low)[1]

// Request the last confirmed `topLeft` and `bottomRight` chart points from the `higherTimeframe`.
[requestedTopLeft, requestedBottomRight] = request.security(
     syminfo.tickerid, higherTimeframe, [topLeft, bottomRight], lookahead = barmerge.lookahead_on
 )

// Draw a new box when a new `higherTimeframe` bar starts.
// The box uses the `time` fields from the `requestedTopLeft` and `requestedBottomRight` as x-coordinates.
if timeframe.change(higherTimeframe)
    box.new(
         requestedTopLeft, requestedBottomRight, color.purple, 3, 
         xloc = xloc.bar_time, bgcolor = color.new(color.purple, 90)
     )
```

Note that:

-   Because we designed this example to request data from [higher timeframes](https://www.tradingview.com/pine-script-docs/concepts/other-timeframes-and-data/#higher-timeframes), we’ve included a [runtime.error()](https://www.tradingview.com/pine-script-reference/v6/#fun_runtime.error) call that the script executes if the `higherTimeframe` value represents a lower timeframe than [timeframe.period](https://www.tradingview.com/pine-script-reference/v6/#var_timeframe.period).

#### Collections {#collections}

Pine Script _collections_ ([arrays](https://www.tradingview.com/pine-script-docs/language/arrays/), [matrices](https://www.tradingview.com/pine-script-docs/language/matrices/), and [maps](https://www.tradingview.com/pine-script-docs/language/maps/)) are data structures that contain an arbitrary number of elements with specified types. The [request.security()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.security) function can retrieve the IDs of [collections](https://www.tradingview.com/pine-script-docs/language/type-system/#collections) whose elements consist of:

-   Fundamental types
-   [Chart points](https://www.tradingview.com/pine-script-docs/language/type-system/#chart-points)
-   [User-defined types](https://www.tradingview.com/pine-script-docs/language/type-system/#user-defined-types) that satisfy the criteria listed in the [section below](https://www.tradingview.com/pine-script-docs/concepts/other-timeframes-and-data/#user-defined-types)

This example below calculates the ratio of a confirmed bar’s high-low range to the range between the highest and lowest prices over 10 bars from a from a specified `symbol` and `timeframe`. It uses [maps](https://www.tradingview.com/pine-script-docs/language/maps/) to hold the values used in the calculations.

The script uses a `data` map with “string” keys and “float” values to store the current bar’s [high](https://www.tradingview.com/pine-script-reference/v6/#var_high), [low](https://www.tradingview.com/pine-script-reference/v6/#var_low), [ta.highest()](https://www.tradingview.com/pine-script-reference/v6/#fun_ta.highest), and [ta.lowest()](https://www.tradingview.com/pine-script-reference/v6/#fun_ta.lowest) results. It passes the map as the `expression` argument in a [request.security()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.security) call on each bar to retrieve another map containing the values calculated from the specified context, then assigns that map’s reference to the `otherData` variable. The script uses the “float” values associated with the “High”, “Low”, “Highest”, and “Lowest” keys of the `otherData` map to calculate the `ratio` series that it [plots](https://www.tradingview.com/pine-script-docs/visuals/plots/) in the chart pane:

```pine
//@version=6
indicator("Requesting collections demo", "Bar range ratio")

//@variable The ticker ID to request data from.
string symbol = input.symbol("", "Symbol")
//@variable The timeframe of the request.
string timeframe = input.timeframe("30", "Timeframe")

//@variable A map with "string" keys and "float" values.
var map<string, float> data = map.new<string, float>()

// Put key-value pairs into the `data` map.
map.put(data, "High", high)
map.put(data, "Low", low)
map.put(data, "Highest", ta.highest(10))
map.put(data, "Lowest", ta.lowest(10))

//@variable A new `map` whose data is calculated from the last confirmed bar of the requested context.
map<string, float> otherData = request.security(symbol, timeframe, data[1], lookahead = barmerge.lookahead_on)

//@variable The ratio of the context's bar range to the max range over 10 bars. Returns `na` if no data is available.
float ratio = na
if not na(otherData)
    ratio := (otherData.get("High") - otherData.get("Low")) / (otherData.get("Highest") - otherData.get("Lowest"))

//@variable A gradient color for the plot of the `ratio`.
color ratioColor = color.from_gradient(ratio, 0, 1, color.purple, color.orange)

// Plot the `ratio`.
plot(ratio, "Range Ratio", ratioColor, 3, plot.style_area)
```

Note that:

-   The [request.security()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.security) call in this script can return [na](https://www.tradingview.com/pine-script-reference/v6/#var_na) if no data is available from the specified context. Since one cannot call [methods](https://www.tradingview.com/pine-script-docs/language/methods/) on a [map](https://www.tradingview.com/pine-script-reference/v6/#type_map) variable when its value is [na](https://www.tradingview.com/pine-script-reference/v6/#var_na), we’ve added an [if](https://www.tradingview.com/pine-script-reference/v6/#kw_if) structure to only calculate a new `ratio` value when `otherData` references a valid [map](https://www.tradingview.com/pine-script-reference/v6/#type_map) instance.

#### User-defined types {#user-defined-types}

[User-defined types (UDTs)](https://www.tradingview.com/pine-script-docs/language/type-system/#user-defined-types) are _composite types_ containing an arbitrary number of _fields_, which can be of any available type, including other [user-defined types](https://www.tradingview.com/pine-script-docs/language/type-system/#user-defined-types).

The [request.security()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.security) function can retrieve the IDs of [objects](https://www.tradingview.com/pine-script-docs/language/objects/) produced by [UDTs](https://www.tradingview.com/pine-script-docs/language/type-system/#user-defined-types) from other contexts if their fields consist of:

-   Fundamental types
-   [Chart points](https://www.tradingview.com/pine-script-docs/language/type-system/#chart-points)
-   [Collections](https://www.tradingview.com/pine-script-docs/language/type-system/#collections) that satisfy the criteria listed in the [section above](https://www.tradingview.com/pine-script-docs/concepts/other-timeframes-and-data/#collections)
-   Other [UDTs](https://www.tradingview.com/pine-script-docs/language/type-system/#user-defined-types) whose fields consist of any of these types

The following example requests an [object](https://www.tradingview.com/pine-script-docs/language/objects/) ID using a specified `symbol` and displays its field values on a chart pane.

The script contains a `TickerInfo` UDT with “string” fields for `syminfo.*` values, an [array](https://www.tradingview.com/pine-script-reference/v6/#type_array) field to store recent “float” price data, and an “int” field to hold the requested ticker’s [bar\_index](https://www.tradingview.com/pine-script-reference/v6/#var_bar_index) value. It assigns a new `TickerInfo` ID to an `info` variable on every bar and uses the variable as the `expression` in [request.security()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.security) to retrieve the ID of an [object](https://www.tradingview.com/pine-script-docs/language/objects/) representing the calculated `info` from the specified `symbol`.

The script displays the `requestedInfo` object’s `description`, `tickerType`, `currency`, and `barIndex` values in a [label](https://www.tradingview.com/pine-script-reference/v6/#type_label) and uses [plotcandle()](https://www.tradingview.com/pine-script-reference/v6/#fun_plotcandle) to display the values from its `prices` array:

```pine
//@version=6
indicator("Requesting user-defined types demo", "Ticker info")

//@variable The symbol to request information from.
string symbol = input.symbol("NASDAQ:AAPL", "Symbol")

//@type               A custom type containing information about a ticker.
//@field description  The symbol's description.
//@field tickerType   The type of ticker.
//@field currency     The symbol's currency.
//@field prices       An array of the symbol's current prices.
//@field barIndex     The ticker's `bar_index`.
type TickerInfo
    string       description
    string       tickerType
    string       currency
    array<float> prices
    int          barIndex

//@variable A `TickerInfo` object containing current data.
info = TickerInfo.new(
     syminfo.description, syminfo.type, syminfo.currency, array.from(open, high, low, close), bar_index
 )
//@variable The `info` requested from the specified `symbol`.
TickerInfo requestedInfo = request.security(symbol, timeframe.period, info)
// Assign a new `TickerInfo` instance to `requestedInfo` if one wasn't retrieved.
if na(requestedInfo)
    requestedInfo := TickerInfo.new(prices = array.new<float>(4))

//@variable A label displaying information from the `requestedInfo` object.
var infoLabel = label.new(
     na, na, "", color = color.purple, style = label.style_label_left, textcolor = color.white, size = size.large
 )
//@variable The text to display inside the `infoLabel`.
string infoText = na(requestedInfo) ? "" : str.format(
     "{0}\nType: {1}\nCurrency: {2}\nBar Index: {3}",
     requestedInfo.description, requestedInfo.tickerType, requestedInfo.currency, requestedInfo.barIndex
 )

// Set the `point` and `text` of the `infoLabel`.
label.set_point(infoLabel, chart.point.now(array.last(requestedInfo.prices)))
label.set_text(infoLabel, infoText)
// Plot candles using the values from the `prices` array of the `requestedInfo`.
plotcandle(
     requestedInfo.prices.get(0), requestedInfo.prices.get(1), requestedInfo.prices.get(2), requestedInfo.prices.get(3),
     "Requested Prices"
 )
```

Note that:

-   The `syminfo.*` variables used in this script all return “simple string” qualified types. However, [objects](https://www.tradingview.com/pine-script-docs/language/objects/) in Pine are _always_ qualified as “series”. Consequently, all values assigned to the `info` object’s fields automatically adopt the “series” [qualifier](https://www.tradingview.com/pine-script-docs/language/type-system/#qualifiers).
-   It is possible for the [request.security()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.security) call to return [na](https://www.tradingview.com/pine-script-reference/v6/#var_na) due to differences between the data requested from the `symbol` and the main chart. This script assigns a new `TickerInfo` object to the `requestedInfo` in that case to prevent runtime errors.

## `request.security_lower_tf()` {#requestsecurity_lower_tf}

The [request.security\_lower\_tf()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.security_lower_tf) function is an alternative to [request.security()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.security) designed for reliably requesting information from lower-timeframe (LTF) contexts.

While [request.security()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.security) can retrieve data from a _single_ intrabar (LTF bar) in each chart bar, [request.security\_lower\_tf()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.security_lower_tf) retrieves data from _all_ available intrabars in each chart bar, which the script can access and use in additional calculations. Each [request.security\_lower\_tf()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.security_lower_tf) call can retrieve up to 200,000 intrabars from a lower timeframe, depending on the user’s [plan](https://www.tradingview.com/pricing/). See [this](https://www.tradingview.com/pine-script-docs/writing/limitations/#request-calls) section of our [Limitations](https://www.tradingview.com/pine-script-docs/writing/limitations/) page for more information.

> [!TIP]
> Working with the [request.security\_lower\_tf()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.security_lower_tf) function involves frequent usage of _arrays_, because the function always returns array results. Therefore, we recommend reading the [Arrays](https://www.tradingview.com/pine-script-docs/language/arrays/) page to make the most of this function and understand how to use its returned data.

Below is the function’s signature, which is similar to the signature of [request.security()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.security):

```
request.security_lower_tf(symbol, timeframe, expression, ignore_invalid_symbol, currency, ignore_invalid_timeframe, calc_bars_count) → array<type>
```

This function requests data only from timeframes that are _lower than_ or _equal to_ the chart’s timeframe ([timeframe.period](https://www.tradingview.com/pine-script-reference/v6/#var_timeframe.period)). If the `timeframe` argument of the [request.security\_lower\_tf()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.security_lower_tf) call represents a higher timeframe, the function raises a runtime error or returns [na](https://www.tradingview.com/pine-script-reference/v6/#var_na) results, depending on the `ignore_invalid_timeframe` parameter. The parameter’s default value is `false`, meaning the function raises an error and halts the script’s executions if the `timeframe` argument is invalid.

### Requesting intrabar data {#requesting-intrabar-data}

Intrabar data can provide a script with additional information that may not be obvious or accessible from solely analyzing data sampled on the chart’s timerframe. The [request.security\_lower\_tf()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.security_lower_tf) function can retrieve many data types from an intrabar context.

Before you venture further in this section, we recommend exploring the [Requestable data](https://www.tradingview.com/pine-script-docs/concepts/other-timeframes-and-data/#requestable-data) portion of the [request.security()](https://www.tradingview.com/pine-script-docs/concepts/other-timeframes-and-data/#requestsecurity) section above, which provides foundational information about the types of data one can request. The `expression` parameter in [request.security\_lower\_tf()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.security_lower_tf) accepts most of the same arguments discussed in that section, excluding direct references to [collections](https://www.tradingview.com/pine-script-docs/language/type-system/#collections) and mutable variables. Although it accepts many of the same types of arguments, this function returns [array](https://www.tradingview.com/pine-script-reference/v6/#type_array) results, which comes with some differences in interpretation and handling, as explained below.

### Intrabar data arrays {#intrabar-data-arrays}

Lower timeframes contain more data points than higher timeframes, as new values come in at a _higher frequency_. For example, when comparing a 1-minute chart to an hourly chart, the 1-minute chart will have up to 60 times the number of bars per hour, depending on the available data.

To address the fact that multiple intrabars exist within a chart bar, [request.security\_lower\_tf()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.security_lower_tf) always creates [arrays](https://www.tradingview.com/pine-script-docs/language/arrays/) to store the requested data. The elements in the arrays represent the `expression` values retrieved from the lower timeframe sorted in ascending order based on each intrabar’s timestamp.

The _type identifier_ of the constructed arrays corresponds to the data types passed in the [request.security\_lower\_tf()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.security_lower_tf) call. For example, using an “int” as the `expression` will produce an `array<int>` instance, a “bool” as the `expression` will produce an `array<bool>` instance, etc.

The following script uses intrabar information to decompose the chart’s close-to-close price changes into positive and negative parts. It calls [request.security\_lower\_tf()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.security_lower_tf) to fetch a “float” [array](https://www.tradingview.com/pine-script-reference/v6/#type_array) containing `ta.change(close)` values from a specified lower timeframe on each chart bar, then accesses all the array’s elements using a [for…in](https://www.tradingview.com/pine-script-reference/v6/#kw_for...in) loop to accumulate `positiveChange` and `negativeChange` sums. The script adds the accumulated values to calculate the `netChange` value, then [plots](https://www.tradingview.com/pine-script-docs/visuals/plots/) the results on the chart alongside the `priceChange` value for comparison:

```pine
//@version=6
indicator("Intrabar arrays demo", "Intrabar price changes")

//@variable The lower timeframe of the requested data.
string lowerTimeframe = input.timeframe("1", "Timeframe")

//@variable The close-to-close price change.
float priceChange = ta.change(close)

//@variable An array of `close` values from available intrabars on the `lowerTimeframe`.
array<float> intrabarChanges = request.security_lower_tf(syminfo.tickerid, lowerTimeframe, priceChange)

//@variable The total positive intrabar `close` movement on the chart bar.
float positiveChange = 0.0
//@variable The total negative intrabar `close` movement on the chart bar.
float negativeChange = 0.0

// Loop to calculate totals, starting from the chart bar's first available intrabar.
for change in intrabarChanges
    // Add the `change` to `positiveChange` if its sign is 1, and add to `negativeChange` if its sign is -1.
    switch math.sign(change)
        1  => positiveChange += change
        -1 => negativeChange += change

//@variable The sum of `positiveChange` and `negativeChange`. Equals the `priceChange` on bars with available intrabars.
float netChange = positiveChange + negativeChange

// Plot the `positiveChange`, `negativeChange`, and `netChange`.
plot(positiveChange, "Positive intrabar change", color.teal, style = plot.style_area)
plot(negativeChange, "Negative intrabar change", color.maroon, style = plot.style_area)
plot(netChange, "Net intrabar change", color.yellow, 5)
// Plot the `priceChange` to compare.
plot(priceChange, "Chart price change", color.orange, 2)
```

Note that:

-   The [plots](https://www.tradingview.com/pine-script-docs/visuals/plots/) based on intrabar data may not appear on all available chart bars, as [request.security\_lower\_tf()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.security_lower_tf) can only access up to the most recent 200,000 intrabars available from the requested context. When executing this function on a chart bar that doesn’t have accessible intrabar data, it will return an _empty array_.
-   The number of intrabars per chart bar may vary depending on the data available from the context and the chart the script executes on. For example, a provider’s 1-minute data feed may not include data for every minute within the 60-minute timeframe due to a lack of trading activity over some 1-minute intervals. To check the number of intrabars retrieved for a chart bar, one can use [array.size()](https://www.tradingview.com/pine-script-reference/v6/#fun_array.size) on the resulting [array](https://www.tradingview.com/pine-script-reference/v6/#type_array).
-   If the `lowerTimeframe` value is greater than the chart’s timeframe, the script will raise a _runtime error_, as we have not supplied an `ignore_invalid_timeframe` argument in the [request.security\_lower\_tf()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.security_lower_tf) call.

### Tuples of intrabar data {#tuples-of-intrabar-data}

When passing a tuple or a function call that returns a tuple as the `expression` argument in [request.security\_lower\_tf()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.security_lower_tf), the result is a tuple of [arrays](https://www.tradingview.com/pine-script-docs/language/arrays/) with [type templates](https://www.tradingview.com/pine-script-docs/language/type-system/#collections) corresponding to the types within the argument. For example, using a `[float, string, color]` tuple as the `expression` will result in `[array<float>, array<string>, array<color>]` data returned by the function. Using a tuple `expression` allows a script to fetch the IDs of several [arrays](https://www.tradingview.com/pine-script-docs/language/arrays/) containing intrabar data with a single [request.security\_lower\_tf()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.security_lower_tf) function call.

> [!NOTE]
> The combined size of all tuples returned by `request.*()` calls in a script cannot exceed 127 elements. See the [Tuple element limit](https://www.tradingview.com/pine-script-docs/writing/limitations/#tuple-element-limit) section of the [Limitations](https://www.tradingview.com/pine-script-docs/writing/limitations/) page for more information.

The following example requests OHLC data from a lower timeframe and visualizes the current bar’s intrabars on the chart using [lines and boxes](https://www.tradingview.com/pine-script-docs/visuals/lines-and-boxes/). The script calls [request.security\_lower\_tf()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.security_lower_tf) with the `[open, high, low, close]` tuple as its `expression` to retrieve a tuple of [arrays](https://www.tradingview.com/pine-script-docs/language/arrays/) representing OHLC information from a calculated `lowerTimeframe`. It then uses a [for](https://www.tradingview.com/pine-script-reference/v6/#kw_for) loop to set line coordinates with the retrieved data and current bar indices to display the results next to the current chart bar, providing a “magnified view” of the price movement within the latest candle. It also draws a [box](https://www.tradingview.com/pine-script-reference/v6/#type_box) around the [lines](https://www.tradingview.com/pine-script-docs/visuals/lines-and-boxes/#lines) to indicate the chart region occupied by intrabar drawings:

```pine
//@version=6
indicator("Tuples of intrabar data demo", "Candle magnifier", max_lines_count = 500)

//@variable The maximum number of intrabars to display.
int maxIntrabars = input.int(20, "Max intrabars", 1, 250)
//@variable The width of the drawn candle bodies.
int candleWidth = input.int(20, "Candle width", 2)

//@variable The largest valid timeframe closest to `maxIntrabars` times smaller than the chart timeframe.
string lowerTimeframe = timeframe.from_seconds(math.ceil(timeframe.in_seconds() / maxIntrabars))

//@variable An array of lines to represent intrabar wicks.
var array<line> wicks  = array.new<line>()
//@variable An array of lines to represent intrabar bodies.
var array<line> bodies = array.new<line>()
//@variable A box that surrounds the displayed intrabars.
var box magnifierBox = box.new(na, na, na, na, bgcolor = na)

// Fill the `wicks` and `bodies` arrays with blank lines on the first bar.
if barstate.isfirst
    for i = 1 to maxIntrabars
        array.push(wicks, line.new(na, na, na, na, color = color.gray))
        array.push(bodies, line.new(na, na, na, na, width = candleWidth))

//@variable A tuple of "float" arrays containing `open`, `high`, `low`, and `close` prices from the `lowerTimeframe`.
[oData, hData, lData, cData] = request.security_lower_tf(syminfo.tickerid, lowerTimeframe, [open, high, low, close])
//@variable The number of intrabars retrieved from the `lowerTimeframe` on the chart bar.
int numIntrabars = array.size(oData)

if numIntrabars > 0
    // Define the start and end bar index values for intrabar display.
    int startIndex = bar_index + 2
    int endIndex = startIndex + numIntrabars
    // Loop to update lines.
    for i = 0 to maxIntrabars - 1
        line wickLine = array.get(wicks, i)
        line bodyLine = array.get(bodies, i)
        if i < numIntrabars
            //@variable The `bar_index` of the drawing.
            int candleIndex = startIndex + i
            // Update the properties of the `wickLine` and `bodyLine`.
            line.set_xy1(wickLine, startIndex + i, array.get(hData, i))
            line.set_xy2(wickLine, startIndex + i, array.get(lData, i))
            line.set_xy1(bodyLine, startIndex + i, array.get(oData, i))
            line.set_xy2(bodyLine, startIndex + i, array.get(cData, i))
            line.set_color(bodyLine, bodyLine.get_y2() > bodyLine.get_y1() ? color.teal : color.maroon)
            continue
        // Set the coordinates of the `wickLine` and `bodyLine` to `na` if no intrabar data is available at the index.
        line.set_xy1(wickLine, na, na)
        line.set_xy2(wickLine, na, na)
        line.set_xy1(bodyLine, na, na)
        line.set_xy2(bodyLine, na, na)
    // Set the coordinates of the `magnifierBox`.
    box.set_lefttop(magnifierBox, startIndex - 1, array.max(hData))
    box.set_rightbottom(magnifierBox, endIndex, array.min(lData))
```

Note that:

-   The script draws each candle using two [lines](https://www.tradingview.com/pine-script-docs/visuals/lines-and-boxes/#lines): one to represent wicks and the other to represent the body. Since the script can display up to 500 lines on the chart, we’ve limited the `maxIntrabars` input to 250.
-   The `lowerTimeframe` value is the result of calculating the [math.ceil()](https://www.tradingview.com/pine-script-reference/v6/#fun_math.ceil) of the [timeframe.in\_seconds()](https://www.tradingview.com/pine-script-reference/v6/#fun_timeframe.in_seconds) divided by the `maxIntrabars` and converting to a [valid timeframe string](https://www.tradingview.com/pine-script-docs/concepts/timeframes/#timeframe-string-specifications) with [timeframe.from\_seconds()](https://www.tradingview.com/pine-script-reference/v6/#fun_timeframe.from_seconds).
-   The script sets the top of the box drawing using the [array.max()](https://www.tradingview.com/pine-script-reference/v6/#fun_array.max) of the requested `hData` array, and it sets the box’s bottom using the [array.min()](https://www.tradingview.com/pine-script-reference/v6/#fun_array.min) of the requested `lData` array. As we see on the chart, these values correspond to the [high](https://www.tradingview.com/pine-script-reference/v6/#var_high) and [low](https://www.tradingview.com/pine-script-reference/v6/#var_low) of the chart bar.

### Requesting collections {#requesting-collections}

In some cases, a script might need to request [collections](https://www.tradingview.com/pine-script-docs/language/type-system/#collections) from an intrabar context. However, in contrast to [request.security()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.security), scripts cannot use collection references or calls to functions that return them as the `expression` argument in a [request.security\_lower\_tf()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.security_lower_tf) call, because [arrays](https://www.tradingview.com/pine-script-docs/language/arrays/) cannot directly store references to other [collections](https://www.tradingview.com/pine-script-docs/language/type-system/#collections).

Despite these limitations, it is possible to request [collections](https://www.tradingview.com/pine-script-docs/language/type-system/#collections) from lower timeframes, if needed, with the help of _wrapper_ types.

> [!IMPORTANT]
> The technique described below is **advanced** and **not** recommended for beginners, because it requires an understanding of how [user-defined types](https://www.tradingview.com/pine-script-docs/language/type-system/#user-defined-types) with [collection](https://www.tradingview.com/pine-script-docs/language/type-system/#collections) fields work. When possible, use _simpler_ methods to manage LTF requests. Use the following technique only if others _do not_ suffice.

To make [collections](https://www.tradingview.com/pine-script-docs/language/type-system/#collections) requestable with [request.security\_lower\_tf()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.security_lower_tf), we must create a [UDT](https://www.tradingview.com/pine-script-docs/language/type-system/#user-defined-types) with a field to reference a collection ID. This step is necessary since [arrays](https://www.tradingview.com/pine-script-docs/language/arrays/) cannot reference other [collections](https://www.tradingview.com/pine-script-docs/language/type-system/#collections) directly but _can_ reference UDTs with collection fields:

```pine
//@type A "wrapper" type for storing an `array<float>` reference.
type Wrapper
    array<float> collection
```

With our `Wrapper` UDT defined, we can now pass the IDs of [objects](https://www.tradingview.com/pine-script-docs/language/objects/) of the UDT to the `expression` parameter in [request.security\_lower\_tf()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.security_lower_tf).

A straightforward approach is to use a call to the type’s built-in `*.new()` function as the `expression` argument. For example, this line of code uses a call to `Wrapper.new()` with `array.from(close)` as the `collection` argument directly within the [request.security\_lower\_tf()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.security_lower_tf) call:

```pine
//@variable An array of `Wrapper` IDs requested from the 1-minute timeframe.
array<Wrapper> wrappers = request.security_lower_tf(syminfo.tickerid, "1", Wrapper.new(array.from(close)))
```

Alternatively, we can create a [user-defined function](https://www.tradingview.com/pine-script-docs/language/user-defined-functions/) or [method](https://www.tradingview.com/pine-script-docs/language/methods/#user-defined-methods) that returns a reference to an [object](https://www.tradingview.com/pine-script-docs/language/objects/) of the [UDT](https://www.tradingview.com/pine-script-docs/language/type-system/#user-defined-types) and call that function within [request.security\_lower\_tf()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.security_lower_tf). For instance, this code calls a custom `newWrapper()` function that returns a `Wrapper` ID as the `expression` argument:

```pine
//@function Creates a new `Wrapper` instance to wrap the specified `collection`.
newWrapper(array<float> collection) =>
    Wrapper.new(collection)

//@variable An array of `Wrapper` IDs requested from the 1-minute timeframe.
array<Wrapper> wrappers = request.security_lower_tf(syminfo.tickerid, "1", newWrapper(array.from(close)))
```

The result with either of the above is an [array](https://www.tradingview.com/pine-script-reference/v6/#type_array) containing `Wrapper` IDs from all available intrabars in the chart bar, which the script can use to reference `Wrapper` instances from specific intrabars and use their `collection` fields in additional operations.

The script below utilizes this approach to collect the IDs of [arrays](https://www.tradingview.com/pine-script-docs/language/arrays/) containing intrabar data from a `lowerTimeframe`, then uses those arrays to display data from a specific lower-timeframe bar. Its custom `Prices` type contains a single `data` field to reference `array<float>` instances that hold price data, and the user-defined `newPrices()` function returns the ID of a `Prices` object.

The script calls [request.security\_lower\_tf()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.security_lower_tf) with a `newPrices()` call as its `expression` argument to retrieve the ID of an [array](https://www.tradingview.com/pine-script-reference/v6/#type_array) containing `Prices` IDs from each intrabar in the chart bar, then uses [array.get()](https://www.tradingview.com/pine-script-reference/v6/#fun_array.get) to get the ID from a specified available intrabar, if it exists. Lastly, it uses [array.get()](https://www.tradingview.com/pine-script-reference/v6/#fun_array.get) on the `data` array referenced by that instance and calls [plotcandle()](https://www.tradingview.com/pine-script-reference/v6/#fun_plotcandle) to display its values on the chart:

```pine
//@version=6
indicator("Requesting LTF collections demo", "Intrabar viewer", true)

//@variable The timeframe of the LTF data request.
string lowerTimeframe = input.timeframe("1", "Timeframe")
//@variable The index of the intrabar to show on each chart bar. 0 is the first available intrabar.
int intrabarIndex = input.int(0, "Intrabar to show", 0)

//@variable A custom type to store an `array<float>` reference.
type Prices
    array<float> data

//@function Returns the ID of a new `Prices` instance containing current `open`, `high`, `low`, and `close` prices.
newPrices() =>
    Prices.new(array.from(open, high, low, close))

//@variable An array of `Prices` IDs requested from the `lowerTimeframe`.
array<Prices> requestedPrices = request.security_lower_tf(syminfo.tickerid, lowerTimeframe, newPrices())

//@variable The `Prices` ID from the `requestedPrices` array at the `intrabarIndex`, or `na` if not available.
Prices intrabarPrices = array.size(requestedPrices) > intrabarIndex ? array.get(requestedPrices, intrabarIndex) : na
//@variable The `data` array ID from the `intrabarPrices` object, or the ID of an array of `na` values if `intrabarPrices` is `na`.
array<float> intrabarData = na(intrabarPrices) ? array.new<float>(4, na) : intrabarPrices.data

// Plot the `intrabarData` values as candles.
plotcandle(intrabarData.get(0), intrabarData.get(1), intrabarData.get(2), intrabarData.get(3))
```

Note that:

-   The `intrabarPrices` variable references a `Prices` object only if the size of the `requestedPrices` array is greater than the `intrabarIndex`, because attempting to use [array.get()](https://www.tradingview.com/pine-script-reference/v6/#fun_array.get) to retrieve an element that doesn’t exist causes an [out of bounds error](https://www.tradingview.com/pine-script-docs/language/arrays/#index-xx-is-out-of-bounds-array-size-is-yy).
-   The `intrabarData` variable references an array from the `intrabarPrices.data` field only if the `intrabarPrices` variable references a `Prices` object. If `intrabarPrices` holds [na](https://www.tradingview.com/pine-script-reference/v6/#var_na) because intrabar data is not available for a bar, the `intrabarData` variable references an array of [na](https://www.tradingview.com/pine-script-reference/v6/#var_na) values.
-   The process used in this example is _not_ necessary to achieve the intended result. Instead of using [UDTs](https://www.tradingview.com/pine-script-docs/language/type-system/#user-defined-types), we can use the tuple `[open, high, low, close]` as the `expression` argument in the request to retrieve a tuple of [arrays](https://www.tradingview.com/pine-script-docs/language/arrays/) for further operations. See the [Tuples of intrabar data](https://www.tradingview.com/pine-script-docs/concepts/other-timeframes-and-data/#tuples-of-intrabar-data) section above for more information.

## Custom contexts {#custom-contexts}

Pine Script includes multiple `ticker.*()` functions that allow scripts to construct _custom_ ticker IDs that specify additional settings for data requests when used as a `symbol` argument in [request.security()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.security) and [request.security\_lower\_tf()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.security_lower_tf):

-   [ticker.new()](https://www.tradingview.com/pine-script-reference/v6/#fun_ticker.new) constructs a custom ticker ID from a specified `prefix` and `ticker` with additional `session` and `adjustment` settings.
-   [ticker.modify()](https://www.tradingview.com/pine-script-reference/v6/#fun_ticker.modify) constructs a modified form of a specified `tickerid` with additional `session` and `adjustment` settings.
-   [ticker.heikinashi()](https://www.tradingview.com/pine-script-reference/v6/#fun_ticker.heikinashi), [ticker.renko()](https://www.tradingview.com/pine-script-reference/v6/#fun_ticker.renko), [ticker.pointfigure()](https://www.tradingview.com/pine-script-reference/v6/#fun_ticker.pointfigure), [ticker.kagi()](https://www.tradingview.com/pine-script-reference/v6/#fun_ticker.kagi), and [ticker.linebreak()](https://www.tradingview.com/pine-script-reference/v6/#fun_ticker.linebreak) construct a modified form a `symbol` with [non-standard chart](https://www.tradingview.com/pine-script-docs/concepts/non-standard-charts-data/) settings.
-   [ticker.inherit()](https://www.tradingview.com/pine-script-reference/v6/#fun_ticker.inherit) constructs a new ticker ID for a `symbol` with additional parameters inherited from the `from_tickerid` specified in the function call, allowing scripts to request the `symbol` data with the same modifiers as the `from_tickerid`, including session, dividend adjustment, currency conversion, non-standard chart type, back-adjustment, settlement-as-close, etc.
-   [ticker.standard()](https://www.tradingview.com/pine-script-reference/v6/#fun_ticker.standard) constructs a standard ticker ID representing the `symbol` _without_ additional modifiers.

Let’s explore some practical examples of applying `ticker.*()` functions to request data from custom contexts.

Suppose we want to include dividend adjustment in a stock symbol’s prices without enabling the “Adjust data for dividends” option in the “Symbol” section of the chart’s settings. We can achieve this in a script by constructing a custom ticker ID for the instrument using [ticker.new()](https://www.tradingview.com/pine-script-reference/v6/#fun_ticker.new) or [ticker.modify()](https://www.tradingview.com/pine-script-reference/v6/#fun_ticker.modify) with an `adjustment` value of [adjustment.dividends](https://www.tradingview.com/pine-script-reference/v6/#var_adjustment.dividends).

This script creates an `adjustedTickerID` using [ticker.modify()](https://www.tradingview.com/pine-script-reference/v6/#fun_ticker.modify), uses that ticker ID as the `symbol` in [request.security()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.security) to retrieve a [tuple](https://www.tradingview.com/pine-script-docs/concepts/other-timeframes-and-data/#tuples) of adjusted price values, then uses [plotcandle()](https://www.tradingview.com/pine-script-reference/v6/#fun_plotcandle) to plot the result as candles on the chart. It also highlights the background of bars where the requested prices differ from the prices without dividend adjustment.

As we see on the “NYSE:XOM” chart below, enabling dividend adjustment results in different historical values before the date of the latest dividend:

```pine
//@version=6
indicator("Custom contexts demo 1", "Adjusted prices", true)

//@variable A custom ticker ID representing the chart's symbol with the dividend adjustment modifier.
string adjustedTickerID = ticker.modify(syminfo.tickerid, adjustment = adjustment.dividends)

// Request the adjusted prices for the chart's symbol.
[o, h, l, c] = request.security(adjustedTickerID, timeframe.period, [open, high, low, close])

//@variable The color of the candles on the chart.
color candleColor = c > o ? color.teal : color.maroon

// Plot the adjusted prices.
plotcandle(o, h, l, c, "Adjusted Prices", candleColor)
// Highlight the background when `c` is different from `close`.
bgcolor(c != close ? color.new(color.orange, 80) : na)
```

Note that:

-   If a modifier included in a constructed ticker ID does not apply to the symbol, the script will _ignore_ that modifier when requesting data. For instance, this script will display the same values as the main chart on forex symbols such as “EURUSD”.

While the example above demonstrates a simple way to modify the chart’s symbol, a more frequent use case for `ticker.*()` functions is applying custom modifiers to another symbol while requesting data. If a ticker ID referenced in a script already has the modifiers one would like to apply (e.g., adjustment settings, session type, etc.), they can use [ticker.inherit()](https://www.tradingview.com/pine-script-reference/v6/#fun_ticker.inherit) to quickly and efficiently add those modifiers to another symbol.

In the example below, we’ve edited the previous script to request data for a `symbolInput` using modifiers inherited from the `adjustedTickerID`. This script calls [ticker.inherit()](https://www.tradingview.com/pine-script-reference/v6/#fun_ticker.inherit) to construct an `inheritedTickerID` and uses that ticker ID in a [request.security()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.security) call. It also requests data for the `symbolInput` without additional modifiers and [plots candles](https://www.tradingview.com/pine-script-docs/concepts/bar-plotting/#plotting-candles-with-plotcandle) for both ticker IDs in a separate chart pane to compare the difference.

As shown on the chart, the data requested using the `inheritedTickerID` includes dividend adjustment, whereas the data requested using the `symbolInput` directly does not:

```pine
//@version=6
indicator("Custom contexts demo 2", "Inherited adjustment")

//@variable The symbol to request data from.
string symbolInput = input.symbol("NYSE:PFE", "Symbol")

//@variable A custom ticker ID representing the chart's symbol with the dividend adjustment modifier.
string adjustedTickerID = ticker.modify(syminfo.tickerid, adjustment = adjustment.dividends)
//@variable A custom ticker ID representing the `symbolInput` with modifiers inherited from the `adjustedTickerID`.
string inheritedTickerID = ticker.inherit(adjustedTickerID, symbolInput)

// Request prices using the `symbolInput`.
[o1, h1, l1, c1] = request.security(symbolInput, timeframe.period, [open, high, low, close])
// Request prices using the `inheritedTickerID`.
[o2, h2, l2, c2] = request.security(inheritedTickerID, timeframe.period, [open, high, low, close])

//@variable The color of the candles that use the `inheritedTickerID` prices.
color candleColor = c2 > o2 ? color.teal : color.maroon

// Plot the `symbol` prices.
plotcandle(o1, h1, l1, c1, "Symbol", color.gray, color.gray, bordercolor = color.gray)
// Plot the `inheritedTickerID` prices.
plotcandle(o2, h2, l2, c2, "Symbol With Modifiers", candleColor)
// Highlight the background when `c1` is different from `c2`.
bgcolor(c1 != c2 ? color.new(color.orange, 80) : na)
```

Note that:

-   Since the `adjustedTickerID` represents a modified form of the [syminfo.tickerid](https://www.tradingview.com/pine-script-reference/v6/#var_syminfo.tickerid), if we modify the chart’s context in other ways, such as changing the chart type or enabling extended trading hours in the chart’s settings, those modifiers will also apply to the `adjustedTickerID` and `inheritedTickerID`. However, they will _not_ apply to the `symbolInput` since it represents a _standard_ ticker ID.

Another frequent use case for requesting custom contexts is retrieving data that uses [non-standard chart](https://www.tradingview.com/pine-script-docs/concepts/non-standard-charts-data/) calculations. For example, suppose we want to use [Renko](https://www.tradingview.com/support/solutions/43000502284-renko-charts/) price values to calculate trade signals in a [strategy()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy) script. If we simply change the chart type to “Renko” to get the prices, the [strategy](https://www.tradingview.com/pine-script-docs/concepts/strategies/) will also simulate its trades based on those synthetic prices, producing [misleading results](https://www.tradingview.com/support/solutions/43000481029/):

```pine
//@version=6
strategy(
     "Custom contexts demo 3", "Renko strategy", true, default_qty_type = strategy.percent_of_equity,
     default_qty_value = 2, initial_capital = 50000, slippage = 2,
     commission_type = strategy.commission.cash_per_contract, commission_value = 1, margin_long = 100,
     margin_short = 100
 )

//@variable When `true`, the strategy places a long market order.
bool longEntry = ta.crossover(close, open)
//@variable When `true`, the strategy places a short market order.
bool shortEntry = ta.crossunder(close, open)

if longEntry
    strategy.entry("Long Entry", strategy.long)
if shortEntry
    strategy.entry("Short Entry", strategy.short)
```

To ensure our strategy shows results based on _actual_ prices, we can create a Renko ticker ID using [ticker.renko()](https://www.tradingview.com/pine-script-reference/v6/#fun_ticker.renko) while keeping the chart on a _standard type_, allowing the script to request and use [Renko](https://www.tradingview.com/support/solutions/43000502284-renko-charts/) prices to calculate its signals without calculating the strategy results on them:

```pine
//@version=6
strategy(
     "Custom contexts demo 3", "Renko strategy", true, default_qty_type = strategy.percent_of_equity,
     default_qty_value = 2, initial_capital = 50000, slippage = 1,
     commission_type = strategy.commission.cash_per_contract, commission_value = 1, margin_long = 100,
     margin_short = 100
 )

//@variable A Renko ticker ID.
string renkoTickerID = ticker.renko(syminfo.tickerid, "ATR", 14)
// Request the `open` and `close` prices using the `renkoTickerID`.
[renkoOpen, renkoClose] = request.security(renkoTickerID, timeframe.period, [open, close])

//@variable When `true`, the strategy places a long market order.
bool longEntry = ta.crossover(renkoClose, renkoOpen)
//@variable When `true`, the strategy places a short market order.
bool shortEntry = ta.crossunder(renkoClose, renkoOpen)

if longEntry
    strategy.entry("Long Entry", strategy.long)
if shortEntry
    strategy.entry("Short Entry", strategy.short)

plot(renkoOpen)
plot(renkoClose)
```

## Historical and realtime behavior {#historical-and-realtime-behavior}

Functions in the `request.*()` namespace can behave differently on historical and realtime bars. This behavior is closely related to Pine’s [Execution model](https://www.tradingview.com/pine-script-docs/language/execution-model/).

Consider how a script behaves within the main context. Throughout the chart’s history, the script calculates its required values once and _commits_ them to that bar so their states are accessible on subsequent executions. On an unconfirmed bar, however, the script recalculates its values on _each update_ to the bar’s data to align with realtime changes. Before recalculating the values on that bar, it reverts calculated values to their last committed states, otherwise known as _rollback_, and it only commits values to that bar once the bar closes.

Now consider the behavior of data requests from other contexts with [request.security()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.security). As when evaluating historical bars in the main context, [request.security()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.security) only returns new historical values when it confirms a bar in its specified context. When executing on realtime bars, it returns recalculated values on each chart bar, similar to how a script recalculates values in the main context on the open chart bar.

However, the function only _confirms_ the requested values when a bar from its context closes. When the script restarts, what were previously _realtime_ bars become _historical_ bars. Therefore, [request.security()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.security) only returns the values it confirmed on those bars. In essence, this behavior means that requested data may _repaint_ when its values fluctuate on realtime bars without confirmation from the context.

> [!TIP]
> It is often helpful to distinguish historical bars from realtime bars when working with `request.*()` functions. Scripts can determine whether bars have historical or realtime states by using the [barstate.ishistory](https://www.tradingview.com/pine-script-reference/v6/#var_barstate.ishistory) and [barstate.isrealtime](https://www.tradingview.com/pine-script-reference/v6/#var_barstate.isrealtime) variables.

In most circumstances where a script requests data from a broader context, one will typically require confirmed, stable values that _do not_ fluctuate on realtime bars. The [section below](https://www.tradingview.com/pine-script-docs/concepts/other-timeframes-and-data/#avoiding-repainting) explains how to achieve such a result and avoid repainting data requests.

### Avoiding repainting {#avoiding-repainting}

#### Higher-timeframe data {#higher-timeframe-data}

When requesting values from a higher timeframe, they are subject to repainting since realtime bars can contain _unconfirmed_ information from developing HTF bars, and the script may adjust the times that new values come in on historical bars. To avoid repainting HTF data, one must ensure that the function only returns confirmed values with consistent timing on all bars, regardless of bar state.

The most reliable approach to achieve non-repainting results is to use an `expression` argument that only references past bars (e.g., `close[1]`) while using [barmerge.lookahead\_on](https://www.tradingview.com/pine-script-reference/v6/#var_barmerge.lookahead_on) as the `lookahead` value.

Using [barmerge.lookahead\_on](https://www.tradingview.com/pine-script-reference/v6/#var_barmerge.lookahead_on) with non-offset HTF data requests is discouraged since it prompts [request.security()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.security) to “look ahead” to the final values of an HTF bar, retrieving confirmed values _before_ they’re actually available in the script’s history. However, if the values used in the `expression` are offset by at least one bar, the “future” data the function retrieves is no longer from the future. Instead, the data represents confirmed values from established, _available_ HTF bars. In other words, applying an offset to the `expression` effectively prevents the requested data from repainting when the script restarts its executions and eliminates lookahead bias in the historical series.

The following example demonstrates a repainting HTF data request. The script uses [request.security()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.security) without offset modifications or additional arguments to retrieve the results of a [ta.wma()](https://www.tradingview.com/pine-script-reference/v6/#fun_ta.wma) call from a higher timeframe. It also highlights the background to indicate which bars were in a realtime state during its calculations.

As shown on the chart below, the [plot](https://www.tradingview.com/pine-script-reference/v6/#fun_plot) of the requested WMA only changes on historical bars when HTF bars close, whereas it fluctuates on all realtime bars since the data includes unconfirmed values from the higher timeframe:

```pine
//@version=6
indicator("Avoiding HTF repainting demo", overlay = true)

//@variable The multiplier applied to the chart's timeframe.
int tfMultiplier = input.int(10, "Timeframe multiplier", 1)
//@variable The number of bars in the moving average.
int length = input.int(5, "WMA smoothing length")

//@variable The valid timeframe string closest to `tfMultiplier` times larger than the chart timeframe.
string timeframe = timeframe.from_seconds(timeframe.in_seconds() * tfMultiplier)

//@variable The weighted MA of `close` prices over `length` bars on the `timeframe`.
//          This request repaints because it includes unconfirmed HTF data on realtime bars and it may offset the
//          times of its historical results.
float requestedWMA = request.security(syminfo.tickerid, timeframe, ta.wma(close, length))

// Plot the requested series.
plot(requestedWMA, "HTF WMA", color.purple, 3)
// Highlight the background on realtime bars.
bgcolor(barstate.isrealtime ? color.new(color.orange, 70) : na, title = "Realtime bar highlight")
```

To avoid repainting in this script, we can add `lookahead = barmerge.lookahead_on` to the [request.security()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.security) call and offset the call history of [ta.wma()](https://www.tradingview.com/pine-script-reference/v6/#fun_ta.wma) by one bar with the history-referencing operator [\[\]](https://www.tradingview.com/pine-script-reference/v6/#op_[]), ensuring the request always retrieves the last confirmed HTF bar’s WMA at the start of each new `timeframe`. Unlike the previous script, this version has consistent behavior on historical and realtime bar states, as we see below:

```pine
//@version=6
indicator("Avoiding HTF repainting demo", overlay = true)

//@variable The multiplier applied to the chart's timeframe.
int tfMultiplier = input.int(10, "Timeframe multiplier", 1)
//@variable The number of bars in the moving average.
int length = input.int(5, "WMA smoothing length")

//@variable The valid timeframe string closest to `tfMultiplier` times larger than the chart timeframe.
string timeframe = timeframe.from_seconds(timeframe.in_seconds() * tfMultiplier)

//@variable The weighted MA of `close` prices over `length` bars on the `timeframe`.
//          This request does not repaint, as it always references the last confirmed WMA value on all bars.
float requestedWMA = request.security(
     syminfo.tickerid, timeframe, ta.wma(close, length)[1], lookahead = barmerge.lookahead_on
 )

// Plot the requested value.
plot(requestedWMA, "HTF WMA", color.purple, 3)
// Highlight the background on realtime bars.
bgcolor(barstate.isrealtime ? color.new(color.orange, 70) : na, title = "Realtime bar highlight")
```

#### Lower-timeframe data {#lower-timeframe-data}

The [request.security()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.security) and [request.security\_lower\_tf()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.security_lower_tf) functions can retrieve data from lower-timeframe contexts. The [request.security()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.security) function can only retrieve data from a _single_ intrabar in each chart bar, and [request.security\_lower\_tf()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.security_lower_tf) retrieves data from _all_ available intrabars.

When using these functions to retrieve intrabar data, it’s important to note that such requests are **not** immune to repainting behavior. Historical and realtime series often rely on _separate_ data feeds. Data providers may retroactively modify realtime data, and it’s possible for races to occur in realtime data feeds, as explained in the [Data feeds](https://www.tradingview.com/pine-script-docs/concepts/other-timeframes-and-data/#data-feeds) section of this page. Either case may result in intrabar data retrieved on realtime bars repainting after the script restarts its executions.

Additionally, a particular case that _will_ cause repainting LTF requests is using [request.security()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.security) with [barmerge.lookahead\_on](https://www.tradingview.com/pine-script-reference/v6/#var_barmerge.lookahead_on) to retrieve data from the first intrabar in each chart bar. While it will generally work as expected on historical bars, it will track only the most recent intrabar on realtime bars, as [request.security()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.security) does not retain all intrabar information, and the intrabars the function retrieves on realtime bars are unsorted until restarting the script:

```pine
//@version=6
indicator("Avoiding LTF repainting demo", overlay = true)

//@variable The lower timeframe of the requested data.
string lowerTimeframe = input.timeframe("1", "Timeframe")

//@variable The first intrabar `close` requested from the `lowerTimeframe` on each bar.
//          Only works as intended on historical bars.
float requestedClose = request.security(syminfo.tickerid, lowerTimeframe, close, lookahead = barmerge.lookahead_on)

// Plot the `requestedClose`.
plot(requestedClose, "First intrabar close", linewidth = 3)
// Highlight the background on realtime bars.
bgcolor(barstate.isrealtime ? color.new(color.orange, 60) : na, title = "Realtime bar Highlight")
```

One can mitigate this behavior and track the values from the first intrabar, or any available intrabar in the chart bar, by using [request.security\_lower\_tf()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.security_lower_tf) since it maintains an [array](https://www.tradingview.com/pine-script-reference/v6/#type_array) of intrabar values ordered by the times they come in. Here, we call [array.first()](https://www.tradingview.com/pine-script-reference/v6/#fun_array.first) on a requested [array](https://www.tradingview.com/pine-script-reference/v6/#type_array) of intrabar data to retrieve the [close](https://www.tradingview.com/pine-script-reference/v6/#var_close) price from the first available intrabar in each chart bar:

```pine
//@version=6
indicator("Avoiding LTF repainting demo", overlay = true)

//@variable The lower timeframe of the requested data.
string lowerTimeframe = input.timeframe("1", "Timeframe")

//@variable An array of intrabar `close` values requested from the `lowerTimeframe` on each bar.
array<float> requestedCloses = request.security_lower_tf(syminfo.tickerid, lowerTimeframe, close)

//@variable The first intrabar `close` on each bar with available data.
float firstClose = requestedCloses.size() > 0 ? requestedCloses.first() : na

// Plot the `firstClose`.
plot(firstClose, "First intrabar close", linewidth = 3)
// Highlight the background on realtime bars.
bgcolor(barstate.isrealtime ? color.new(color.orange, 60) : na, title = "Realtime bar Highlight")
```

Note that:

-   While [request.security\_lower\_tf()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.security_lower_tf) is more optimized for handling historical and realtime intrabars, it’s still possible in some cases for minor repainting to occur due to data differences from the provider, as outlined above.
-   This code may not show intrabar data on all available chart bars, depending on how many intrabars each chart bar contains, as `request.*()` functions can retrieve up to 200,000 intrabars from an LTF context. The maximum number of requestable intrabars depends on the user’s [plan](https://www.tradingview.com/pricing/). See [this](https://www.tradingview.com/pine-script-docs/writing/limitations/#request-calls) section of the [Limitations](https://www.tradingview.com/pine-script-docs/writing/limitations/) page for more information.

## `request.currency_rate()` {#requestcurrency_rate}

When a script needs to convert values expressed in one currency to another, one can use [request.currency\_rate()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.currency_rate). This function requests a _daily rate_ for currency conversion calculations based on currency pair or [spread](https://www.tradingview.com/support/solutions/43000502298/) data from the most popular exchanges, providing a simpler alternative to fetching specific pairs or [spreads](https://www.tradingview.com/support/solutions/43000502298/) with [request.security()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.security).

While one can use [request.security()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.security) to retrieve daily currency rates, its use case is more involved than [request.currency\_rate()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.currency_rate), as one needs to supply a valid _ticker ID_ for a currency pair or spread to request the rate. Additionally, a historical offset and [barmerge.lookahead\_on](https://www.tradingview.com/pine-script-reference/v6/#var_barmerge.lookahead_on) are necessary to prevent the results from repainting, as explained in [this section](https://www.tradingview.com/pine-script-docs/concepts/other-timeframes-and-data/#avoiding-repainting).

The [request.currency\_rate()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.currency_rate) function, on the other hand, only requires _currency codes_. No ticker ID is needed when requesting rates with this function, and it ensures non-repainting results without requiring additional specification.

The function’s signature is as follows:

```
request.currency_rate(from, to, ignore_invalid_currency) → series float
```

The `from` parameter specifies the currency to convert, and the `to` parameter specifies the target currency. Both parameters accept “string” values representing valid currency codes (e.g., “USD”) or any built-in `currency.*` variable (e.g., [currency.USD](https://www.tradingview.com/pine-script-reference/v6/#var_currency.USD)).

When the function cannot calculate a valid conversion rate between the specified `from` and `to` currencies, programmers can decide whether it raises a runtime error or returns [na](https://www.tradingview.com/pine-script-reference/v6/#var_na) via the `ignore_invalid_currency` parameter. The default value is `false`, meaning the function raises a runtime error and halts the script’s executions.

The following example demonstrates a simple use case for [request.currency\_rate()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.currency_rate). Suppose we want to convert values expressed in Turkish lira ([currency.TRY](https://www.tradingview.com/pine-script-reference/v6/#var_currency.TRY)) to South Korean won ([currency.KRW](https://www.tradingview.com/pine-script-reference/v6/#var_currency.KRW)) using a daily conversion rate. If we use [request.security()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.security) to retrieve the rate, we must supply a valid ticker ID and request the last confirmed [close](https://www.tradingview.com/pine-script-reference/v6/#var_close) from the previous day.

In this case, no valid symbol exists that would allow us to retrieve a conversion rate directly with [request.security()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.security). Therefore, we first need a ticker ID for a [spread](https://www.tradingview.com/support/solutions/43000502298/) that converts TRY to an intermediate currency, such as USD, then converts the intermediate currency to KRW. We can then use that ticker ID within [request.security()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.security) with `close[1]` as the `expression` and [barmerge.lookahead\_on](https://www.tradingview.com/pine-script-reference/v6/#var_barmerge.lookahead_on) as the `lookahead` value to request a non-repainting daily rate.

Alternatively, we can achieve the same result more simply by calling [request.currency\_rate()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.currency_rate). This function does all the heavy lifting for us, only requiring `from` and `to` currency arguments to perform its calculation.

As we see below, both approaches return the same daily rate:

```pine
//@version=6
indicator("Requesting currency rates demo")

//@variable The currency to convert.
simple string fromCurrency = currency.TRY
//@variable The resulting currency.
simple string toCurrency = currency.KRW

//@variable The spread symbol to request. Required in `request.security()` because no direct symbol exists.
simple string spreadSymbol = str.format("{0}{2} * {2}{1}", fromCurrency, toCurrency, currency.USD)

//@variable The non-repainting conversion rate from `request.security()` using the `spreadSymbol`.
float securityRequestedRate = request.security(spreadSymbol, "1D", close[1], lookahead = barmerge.lookahead_on)
//@variable The non-repainting conversion rate from `request.currency_rate()`.
float nonSecurityRequestedRate = request.currency_rate(fromCurrency, toCurrency)

// Plot the requested rates. We can multiply TRY values by these rates to convert them to KRW.
plot(securityRequestedRate, "`request.security()` value", color.purple, 5)
plot(nonSecurityRequestedRate, "`request.currency_rate()` value", color.yellow, 2)
```

## `request.dividends()`, `request.splits()`, and `request.earnings()` {#requestdividends-requestsplits-and-requestearnings}

Analyzing a stock’s earnings data and corporate actions provides helpful insights into its underlying financial strength. Pine Script provides the ability to retrieve essential information about applicable stocks via [request.dividends()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.dividends), [request.splits()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.splits), and [request.earnings()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.earnings).

These are the functions’ signatures:

```
request.dividends(ticker, field, gaps, lookahead, ignore_invalid_symbol, currency) → series float

request.splits(ticker, field, gaps, lookahead, ignore_invalid_symbol) → series float

request.earnings(ticker, field, gaps, lookahead, ignore_invalid_symbol, currency) → series float
```

Each function has the same parameters in its signature, with the exception of [request.splits()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.splits), which doesn’t have a `currency` parameter.

Note that unlike the `symbol` parameter in other `request.*()` functions, the `ticker` parameter in these functions only accepts an _“Exchange:Symbol” pair_, such as “NASDAQ:AAPL”. The built-in [syminfo.ticker](https://www.tradingview.com/pine-script-reference/v6/#var_syminfo.ticker) variable does not work with these functions since it does not contain exchange information. Instead, one must use [syminfo.tickerid](https://www.tradingview.com/pine-script-reference/v6/#var_syminfo.tickerid) for such cases.

The `field` parameter determines the data the function will retrieve. Each of these functions accepts different built-in variables as the `field` argument since each requests different information about a stock:

-   The [request.dividends()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.dividends) function retrieves current dividend information for a stock, i.e., the amount per share the issuing company paid out to investors who purchased shares before the ex-dividend date. Passing the built-in [dividends.gross](https://www.tradingview.com/pine-script-reference/v6/#var_dividends.gross) or [dividends.net](https://www.tradingview.com/pine-script-reference/v6/#var_dividends.net) variables to the `field` parameter specifies whether the returned value represents dividends before or after factoring in expenses the company deducts from its payouts.
-   The [request.splits()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.splits) function retrieves current split and reverse split information for a stock. A split occurs when a company increases its outstanding shares to promote liquidity. A reverse split occurs when a company consolidates its shares and offers them at a higher price to attract specific investors or maintain their listing on a market that has a minimum per-share price. Companies express their split information as _ratios_. For example, a 5:1 split means the company issued additional shares to its shareholders so that they have five times the number of shares they had before the split, and the raw price of each share becomes one-fifth of the previous price. Passing [splits.numerator](https://www.tradingview.com/pine-script-reference/v6/#var_splits.numerator) or [splits.denominator](https://www.tradingview.com/pine-script-reference/v6/#var_splits.denominator) to the `field` parameter of [request.splits()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.splits) determines whether it returns the numerator or denominator of the split ratio.
-   The [request.earnings()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.earnings) function retrieves the earnings per share (EPS) information for a stock `ticker`’s issuing company. The EPS value is the ratio of a company’s net income to the number of outstanding stock shares, which investors consider an indicator of the company’s profitability. Passing [earnings.actual](https://www.tradingview.com/pine-script-reference/v6/#var_earnings.actual), [earnings.estimate](https://www.tradingview.com/pine-script-reference/v6/#var_earnings.estimate), or [earnings.standardized](https://www.tradingview.com/pine-script-reference/v6/#var_earnings.standardized) as the `field` argument in [request.earnings()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.earnings) respectively determines whether the function requests the actual, estimated, or standardized EPS value.

For a detailed explanation of the `gaps`, `lookahead`, and `ignore_invalid_symbol` parameters of these functions, see the [Common characteristics](https://www.tradingview.com/pine-script-docs/concepts/other-timeframes-and-data/#common-characteristics) section at the top of this page.

It’s important to note that the values returned by these functions reflect the data available as it comes in. This behavior differs from financial data originating from a [request.financial()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.financial) call in that the underlying data from such calls becomes available according to a company’s fiscal reporting period.

> [!TIP]
> Scripts can also retrieve information about upcoming earnings and dividends for an instrument via the `earnings.future_*` and `dividends.future_*` built-in variables.

Here, we’ve included an example that displays a handy [table](https://www.tradingview.com/pine-script-reference/v6/#type_table) containing the most recent dividend, split, and EPS data. The script calls the `request.*()` functions discussed in this section to retrieve the data, then converts the values to “strings” with `str.*()` functions and displays the results in the `infoTable` with [table.cell()](https://www.tradingview.com/pine-script-reference/v6/#fun_table.cell):

```pine
//@version=6
indicator("Dividends, splits, and earnings demo", overlay = true)

//@variable The size of the table's text.
string tableSize = input.string(
     size.large, "Table size", [size.auto, size.tiny, size.small, size.normal, size.large, size.huge]
 )

//@variable The color of the table's text and frame.
var color tableColor = chart.fg_color
//@variable A `table` displaying the latest dividend, split, and EPS information.
var table infoTable = table.new(position.top_right, 3, 4, frame_color = tableColor, frame_width = 1)

// Add header cells on the first bar.
if barstate.isfirst
    table.cell(infoTable, 0, 0, "Field", text_color = tableColor, text_size = tableSize)
    table.cell(infoTable, 1, 0, "Value", text_color = tableColor, text_size = tableSize)
    table.cell(infoTable, 2, 0, "Date", text_color = tableColor, text_size = tableSize)
    table.cell(infoTable, 0, 1, "Dividend", text_color = tableColor, text_size = tableSize)
    table.cell(infoTable, 0, 2, "Split", text_color = tableColor, text_size = tableSize)
    table.cell(infoTable, 0, 3, "EPS", text_color = tableColor, text_size = tableSize)

//@variable The amount of the last reported dividend as of the current bar.
float latestDividend = request.dividends(syminfo.tickerid, dividends.gross, barmerge.gaps_on)
//@variable The numerator of that last reported split ratio as of the current bar.
float latestSplitNum = request.splits(syminfo.tickerid, splits.numerator, barmerge.gaps_on)
//@variable The denominator of the last reported split ratio as of the current bar.
float latestSplitDen = request.splits(syminfo.tickerid, splits.denominator, barmerge.gaps_on)
//@variable The last reported earnings per share as of the current bar.
float latestEPS = request.earnings(syminfo.tickerid, earnings.actual, barmerge.gaps_on)

// Update the "Value" and "Date" columns when new values come in.
if not na(latestDividend)
    table.cell(
         infoTable, 1, 1, str.tostring(math.round(latestDividend, 3)), text_color = tableColor, text_size = tableSize
     )
    table.cell(infoTable, 2, 1, str.format_time(time, "yyyy-MM-dd"), text_color = tableColor, text_size = tableSize)
if not na(latestSplitNum)
    table.cell(
         infoTable, 1, 2, str.format("{0}-for-{1}", latestSplitNum, latestSplitDen), text_color = tableColor,
         text_size = tableSize
     )
    table.cell(infoTable, 2, 2, str.format_time(time, "yyyy-MM-dd"), text_color = tableColor, text_size = tableSize)
if not na(latestEPS)
    table.cell(infoTable, 1, 3, str.tostring(latestEPS), text_color = tableColor, text_size = tableSize)
    table.cell(infoTable, 2, 3, str.format_time(time, "yyyy-MM-dd"), text_color = tableColor, text_size = tableSize)
```

Note that:

-   We’ve included [barmerge.gaps\_on](https://www.tradingview.com/pine-script-reference/v6/#var_barmerge.gaps_on) in the `request.*()` calls, so they only return values when new data is available. Otherwise, they return [na](https://www.tradingview.com/pine-script-reference/v6/#var_na).
-   The script assigns a [table](https://www.tradingview.com/pine-script-reference/v6/#type_table) ID to the `infoTable` variable on the first chart bar. On subsequent bars, it updates necessary cells with new information whenever data is available.
-   If no information is available from any of the `request.*()` calls throughout the chart’s history (e.g., if the `ticker` has no dividend information), the script does not initialize the corresponding cells since it’s unnecessary.

## `request.financial()` {#requestfinancial}

Financial metrics provide investors with insights about a company’s economic and financial health that are not tangible from solely analyzing its stock prices. TradingView offers a wide variety of financial metrics from [FactSet](https://www.factset.com/) that traders can access via the “Financials” tab in the “Indicators” menu of the chart. Scripts can access available metrics for an instrument directly via the [request.financial()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.financial) function.

This is the function’s signature:

```
request.financial(symbol, financial_id, period, gaps, ignore_invalid_symbol, currency) → series float
```

As with the first parameter in [request.dividends()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.dividends), [request.splits()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.splits), and [request.earnings()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.earnings), the `symbol` parameter in [request.financial()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.financial) requires an _“Exchange:Symbol” pair_. To request financial information for the chart’s ticker ID, use [syminfo.tickerid](https://www.tradingview.com/pine-script-reference/v6/#var_syminfo.tickerid), as [syminfo.ticker](https://www.tradingview.com/pine-script-reference/v6/#var_syminfo.ticker) will not work.

The `financial_id` parameter accepts a “string” value representing the ID of the requested financial metric. TradingView has numerous financial metrics to choose from. See the [Financial IDs](https://www.tradingview.com/pine-script-docs/concepts/other-timeframes-and-data/#financial-ids) section below for an overview of all accessible metrics and their “string” identifiers.

The `period` parameter specifies the fiscal period for which new requested data comes in. It accepts one of the following “string” arguments: **“FQ” (quarterly), “FH” (semiannual), “FY” (annual), or “TTM” (trailing twelve months)**. Not all fiscal periods are available for all metrics or instruments. To confirm which periods are available for specific metrics, see the second column of the tables in the [Financial IDs](https://www.tradingview.com/pine-script-docs/concepts/other-timeframes-and-data/#financial-ids) section.

See this page’s [Common characteristics](https://www.tradingview.com/pine-script-docs/concepts/other-timeframes-and-data/#common-characteristics) section for a detailed explanation of this function’s `gaps`, `ignore_invalid_symbol`, and `currency` parameters.

It’s important to note that the data retrieved from this function comes in at a _fixed frequency_, independent of the precise date on which the data is made available within a fiscal period. For a company’s dividends, splits, and earnings per share (EPS) information, one can request data reported on exact dates via [request.dividends()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.dividends), [request.splits()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.splits), and [request.earnings()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.earnings).

This script uses [request.financial()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.financial) to retrieve information about the income and expenses of a stock’s issuing company and visualize the profitability of its typical business operations. It requests the “OPER\_INCOME”, “TOTAL\_REVENUE”, and “TOTAL\_OPER\_EXPENSE” [financial IDs](https://www.tradingview.com/pine-script-docs/concepts/other-timeframes-and-data/#financial-ids) for the [syminfo.tickerid](https://www.tradingview.com/pine-script-reference/v6/#var_syminfo.tickerid) over the latest `fiscalPeriod`, then [plots](https://www.tradingview.com/pine-script-docs/visuals/plots/) the results on the chart:

```pine
//@version=6
indicator("Requesting financial data demo", format = format.volume)

//@variable The size of the fiscal reporting period. Some options may not be available, depending on the instrument.
string fiscalPeriod = input.string("FQ", "Period", ["FQ", "FH", "FY", "TTM"])

//@variable The operating income after expenses reported for the stock's issuing company.
float operatingIncome = request.financial(syminfo.tickerid, "OPER_INCOME", fiscalPeriod)
//@variable The total revenue reported for the stock's issuing company.
float totalRevenue = request.financial(syminfo.tickerid, "TOTAL_REVENUE", fiscalPeriod)
//@variable The total operating expenses reported for the stock's issuing company.
float totalExpenses = request.financial(syminfo.tickerid, "TOTAL_OPER_EXPENSE", fiscalPeriod)

//@variable Is aqua when the `totalRevenue` exceeds the `totalExpenses`, fuchsia otherwise.
color incomeColor = operatingIncome > 0 ? color.new(color.aqua, 50) : color.new(color.fuchsia, 50)

// Display the requested data.
plot(operatingIncome, "Operating income", incomeColor, 1, plot.style_area)
plot(totalRevenue, "Total revenue", color.green, 3)
plot(totalExpenses, "Total operating expenses", color.red, 3)
```

Note that:

-   Not all `fiscalPeriod` options are available for every ticker ID. For example, companies in the US typically publish _quarterly_ reports, whereas many European companies publish _semiannual_ reports. See [this page](https://www.tradingview.com/support/solutions/43000540147) in our Help Center for more information.

### Calculating financial metrics {#calculating-financial-metrics}

The [request.financial()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.financial) function can provide scripts with numerous useful financial metrics that don’t require additional calculations. However, some commonly used financial estimates require combining an instrument’s current market price with requested financial data. Such is the case for:

-   Market Capitalization (market price \* total shares outstanding)
-   Earnings Yield (12-month EPS / market price)
-   Price-to-Book Ratio (market price / BVPS)
-   Price-to-Earnings Ratio (market price / EPS)
-   Price-to-Sales Ratio (market cap / 12-month total revenue)

The following script contains [user-defined functions](https://www.tradingview.com/pine-script-docs/language/user-defined-functions/) that calculate the above financial metrics for the [syminfo.tickerid](https://www.tradingview.com/pine-script-reference/v6/#var_syminfo.tickerid). We’ve created these functions so users can easily copy them into their scripts. This example uses them within a [str.format()](https://www.tradingview.com/pine-script-reference/v6/#fun_str.format) call to construct a `tooltipText`, which it displays in tooltips on the chart using [labels](https://www.tradingview.com/pine-script-docs/visuals/text-and-shapes/#labels). Hovering over any bar’s [label](https://www.tradingview.com/pine-script-reference/v6/#type_label) will expose the tooltip containing the metrics calculated on that bar:

```pine
//@version=6
indicator("Calculating financial metrics demo", overlay = true, max_labels_count = 500)

//@function Calculates the market capitalization (market cap) for the chart's symbol.
marketCap() =>
    //@variable The most recent number of outstanding shares reported for the symbol.
    float totalSharesOutstanding = request.financial(syminfo.tickerid, "TOTAL_SHARES_OUTSTANDING", "FQ")
    // Return the market cap value.
    totalSharesOutstanding * close

//@function Calculates the Earnings Yield for the chart's symbol.
earningsYield() =>
    //@variable The most recent 12-month earnings per share reported for the symbol.
    float eps = request.financial(syminfo.tickerid, "EARNINGS_PER_SHARE", "TTM")
    //Return the Earnings Yield percentage.
    100.0 * eps / close

//@function Calculates the Price-to-Book (P/B) ratio for the chart's symbol.
priceBookRatio() =>
    //@variable The most recent Book Value Per Share (BVPS) reported for the symbol.
    float bookValuePerShare = request.financial(syminfo.tickerid, "BOOK_VALUE_PER_SHARE", "FQ")
    // Return the P/B ratio.
    close / bookValuePerShare

//@function Calculates the Price-to-Earnings (P/E) ratio for the chart's symbol.
priceEarningsRatio() =>
    //@variable The most recent 12-month earnings per share reported for the symbol.
    float eps = request.financial(syminfo.tickerid, "EARNINGS_PER_SHARE", "TTM")
    // Return the P/E ratio.
    close / eps

//@function Calculates the Price-to-Sales (P/S) ratio for the chart's symbol.
priceSalesRatio() =>
    //@variable The most recent number of outstanding shares reported for the symbol.
    float totalSharesOutstanding = request.financial(syminfo.tickerid, "TOTAL_SHARES_OUTSTANDING", "FQ")
    //@variable The most recent 12-month total revenue reported for the symbol.
    float totalRevenue = request.financial(syminfo.tickerid, "TOTAL_REVENUE", "TTM")
    // Return the P/S ratio.
    totalSharesOutstanding * close / totalRevenue

//@variable The text to display in label tooltips.
string tooltipText = str.format(
     "Market Cap: {0} {1}\nEarnings Yield: {2}%\nP/B Ratio: {3}\nP/E Ratio: {4}\nP/S Ratio: {5}",
     str.tostring(marketCap(), format.volume), syminfo.currency, earningsYield(), priceBookRatio(),
     priceEarningsRatio(), priceSalesRatio()
 )

//@variable Displays a blank label with a tooltip containing the `tooltipText`.
label info = label.new(chart.point.now(high), tooltip = tooltipText)
```

Note that:

-   Since not all companies publish quarterly financial reports, one may need to change the “FQ” in these functions to match the minimum reporting period for a specific company, as the [request.financial()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.financial) calls will return [na](https://www.tradingview.com/pine-script-reference/v6/#var_na) when “FQ” data isn’t available.

### Financial IDs {#financial-ids}

Below is an overview of all financial metrics one can request via [request.financial()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.financial), along with the periods in which reports may be available. We’ve divided this information into four tables corresponding to the categories displayed in the “Financials” section of the “Indicators” menu:

-   [Income statements](https://www.tradingview.com/pine-script-docs/concepts/other-timeframes-and-data/#income-statements)
-   [Balance sheet](https://www.tradingview.com/pine-script-docs/concepts/other-timeframes-and-data/#balance-sheet)
-   [Cash flow](https://www.tradingview.com/pine-script-docs/concepts/other-timeframes-and-data/#cash-flow)
-   [Statistics](https://www.tradingview.com/pine-script-docs/concepts/other-timeframes-and-data/#statistics)

Each table has the following three columns:

-   The first column contains descriptions of each metric with links to Help Center pages for additional information.
-   The second column lists the possible `period` arguments allowed for the metric. Note that all available values may not be compatible with specific ticker IDs, e.g., while “FQ” may be a possible argument, it will not work if the issuing company does not publish quarterly data.
-   The third column lists the “string” IDs for the `financial_id` argument in [request.financial()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.financial).

> [!TIP]
> The tables in these sections are quite lengthy, because there are many `financial_id` arguments available. Use the **“Click to show/hide”** option above each table to toggle its visibility.

#### Income statements {#income-statements}

This table lists the available metrics that provide information about a company’s income, costs, profits and losses.

Click to show/hide

| Financial | `period` | `financial_id` |
| --- | --- | --- |
| [After tax other income/expense](https://www.tradingview.com/support/solutions/43000563497) | FQ, FH, FY, TTM | AFTER\_TAX\_OTHER\_INCOME |
| [Average basic shares outstanding](https://www.tradingview.com/support/solutions/43000670320) | FQ, FH, FY | BASIC\_SHARES\_OUTSTANDING |
| [Basic earnings per share (Basic EPS)](https://www.tradingview.com/support/solutions/43000563520) | FQ, FH, FY, TTM | EARNINGS\_PER\_SHARE\_BASIC |
| [Cost of goods sold](https://www.tradingview.com/support/solutions/43000553618) | FQ, FH, FY, TTM | COST\_OF\_GOODS |
| [Deprecation and amortization](https://www.tradingview.com/support/solutions/43000563477) | FQ, FH, FY, TTM | DEP\_AMORT\_EXP\_INCOME\_S |
| [Diluted earnings per share (Diluted EPS)](https://www.tradingview.com/support/solutions/43000553616) | FQ, FH, FY, TTM | EARNINGS\_PER\_SHARE\_DILUTED |
| [Diluted net income available to common stockholders](https://www.tradingview.com/support/solutions/43000563516) | FQ, FH, FY, TTM | DILUTED\_NET\_INCOME |
| [Diluted shares outstanding](https://www.tradingview.com/support/solutions/43000670322) | FQ, FH, FY | DILUTED\_SHARES\_OUTSTANDING |
| [Dilution adjustment](https://www.tradingview.com/support/solutions/43000563504) | FQ, FH, FY, TTM | DILUTION\_ADJUSTMENT |
| [Discontinued operations](https://www.tradingview.com/support/solutions/43000563502) | FQ, FH, FY, TTM | DISCONTINUED\_OPERATIONS |
| [EBIT](https://www.tradingview.com/support/solutions/43000670329) | FQ, FH, FY, TTM | EBIT |
| [EBITDA](https://www.tradingview.com/support/solutions/43000553610) | FQ, FH, FY, TTM | EBITDA |
| [Equity in earnings](https://www.tradingview.com/support/solutions/43000563487) | FQ, FH, FY, TTM | EQUITY\_IN\_EARNINGS |
| [Gross profit](https://www.tradingview.com/support/solutions/43000553611) | FQ, FH, FY, TTM | GROSS\_PROFIT |
| [Interest capitalized](https://www.tradingview.com/support/solutions/43000563468) | FQ, FH, FY, TTM | INTEREST\_CAPITALIZED |
| [Interest expense on debt](https://www.tradingview.com/support/solutions/43000563467) | FQ, FH, FY, TTM | INTEREST\_EXPENSE\_ON\_DEBT |
| [Interest expense, net of interest capitalized](https://www.tradingview.com/support/solutions/43000563466) | FQ, FH, FY, TTM | NON\_OPER\_INTEREST\_EXP |
| [Miscellaneous non-operating expense](https://www.tradingview.com/support/solutions/43000563479) | FQ, FH, FY, TTM | OTHER\_INCOME |
| [Net income](https://www.tradingview.com/support/solutions/43000553617) | FQ, FH, FY, TTM | NET\_INCOME |
| [Net income before discontinued operations](https://www.tradingview.com/support/solutions/43000563500) | FQ, FH, FY, TTM | NET\_INCOME\_BEF\_DISC\_OPER |
| [Non-controlling/minority interest](https://www.tradingview.com/support/solutions/43000563495) | FQ, FH, FY, TTM | MINORITY\_INTEREST\_EXP |
| [Non-operating income, excl. interest expenses](https://www.tradingview.com/support/solutions/43000563471) | FQ, FH, FY, TTM | NON\_OPER\_INCOME |
| [Non-operating income, total](https://www.tradingview.com/support/solutions/43000563465) | FQ, FH, FY, TTM | TOTAL\_NON\_OPER\_INCOME |
| [Non-operating interest income](https://www.tradingview.com/support/solutions/43000563473) | FQ, FH, FY, TTM | NON\_OPER\_INTEREST\_INCOME |
| [Operating expenses (excl. COGS)](https://www.tradingview.com/support/solutions/43000563463) | FQ, FH, FY, TTM | OPERATING\_EXPENSES |
| [Operating income](https://www.tradingview.com/support/solutions/43000563464) | FQ, FH, FY, TTM | OPER\_INCOME |
| [Other cost of goods sold](https://www.tradingview.com/support/solutions/43000563478) | FQ, FH, FY, TTM | COST\_OF\_GOODS\_EXCL\_DEP\_AMORT |
| [Other operating expenses, total](https://www.tradingview.com/support/solutions/43000563483) | FQ, FH, FY, TTM | OTHER\_OPER\_EXPENSE\_TOTAL |
| [Preferred dividends](https://www.tradingview.com/support/solutions/43000563506) | FQ, FH, FY, TTM | PREFERRED\_DIVIDENDS |
| [Pretax equity in earnings](https://www.tradingview.com/support/solutions/43000563474) | FQ, FH, FY, TTM | PRETAX\_EQUITY\_IN\_EARNINGS |
| [Pretax income](https://www.tradingview.com/support/solutions/43000563462) | FQ, FH, FY, TTM | PRETAX\_INCOME |
| [Research & development](https://www.tradingview.com/support/solutions/43000553612) | FQ, FH, FY, TTM | RESEARCH\_AND\_DEV |
| [Selling/general/admin expenses, other](https://www.tradingview.com/support/solutions/43000553614) | FQ, FH, FY, TTM | SELL\_GEN\_ADMIN\_EXP\_OTHER |
| [Selling/general/admin expenses, total](https://www.tradingview.com/support/solutions/43000553613) | FQ, FH, FY, TTM | SELL\_GEN\_ADMIN\_EXP\_TOTAL |
| [Taxes](https://www.tradingview.com/support/solutions/43000563492) | FQ, FH, FY, TTM | INCOME\_TAX |
| [Total operating expenses](https://www.tradingview.com/support/solutions/43000553615) | FQ, FH, FY, TTM | TOTAL\_OPER\_EXPENSE |
| [Total revenue](https://www.tradingview.com/support/solutions/43000553619) | FQ, FH, FY, TTM | TOTAL\_REVENUE |
| [Unusual income/expense](https://www.tradingview.com/support/solutions/43000563476) | FQ, FH, FY, TTM | UNUSUAL\_EXPENSE\_INC |

#### Balance sheet {#balance-sheet}

This table lists the metrics that provide information about a company’s capital structure.

Click to show/hide

| Financial | `period` | `financial_id` |
| --- | --- | --- |
| [Accounts payable](https://www.tradingview.com/support/solutions/43000563619) | FQ, FH, FY | ACCOUNTS\_PAYABLE |
| [Accounts receivable - trade, net](https://www.tradingview.com/support/solutions/43000563740) | FQ, FH, FY | ACCOUNTS\_RECEIVABLES\_NET |
| [Accrued payroll](https://www.tradingview.com/support/solutions/43000563628) | FQ, FH, FY | ACCRUED\_PAYROLL |
| [Accumulated depreciation, total](https://www.tradingview.com/support/solutions/43000563673) | FQ, FH, FY | ACCUM\_DEPREC\_TOTAL |
| [Additional paid-in capital/Capital surplus](https://www.tradingview.com/support/solutions/43000563874) | FQ, FH, FY | ADDITIONAL\_PAID\_IN\_CAPITAL |
| [Book value per share](https://www.tradingview.com/support/solutions/43000670330) | FQ, FH, FY | BOOK\_VALUE\_PER\_SHARE |
| [Capital and operating lease obligations](https://www.tradingview.com/support/solutions/43000563522) | FQ, FH, FY | CAPITAL\_OPERATING\_LEASE\_OBLIGATIONS |
| [Capitalized lease obligations](https://www.tradingview.com/support/solutions/43000563527) | FQ, FH, FY | CAPITAL\_LEASE\_OBLIGATIONS |
| [Cash & equivalents](https://www.tradingview.com/support/solutions/43000563709) | FQ, FH, FY | CASH\_N\_EQUIVALENTS |
| [Cash and short term investments](https://www.tradingview.com/support/solutions/43000563702) | FQ, FH, FY | CASH\_N\_SHORT\_TERM\_INVEST |
| [Common equity, total](https://www.tradingview.com/support/solutions/43000563866) | FQ, FH, FY | COMMON\_EQUITY\_TOTAL |
| [Common stock par/Carrying value](https://www.tradingview.com/support/solutions/43000563873) | FQ, FH, FY | COMMON\_STOCK\_PAR |
| [Current portion of LT debt and capital leases](https://www.tradingview.com/support/solutions/43000563557) | FQ, FH, FY | CURRENT\_PORT\_DEBT\_CAPITAL\_LEASES |
| [Deferred income, current](https://www.tradingview.com/support/solutions/43000563631) | FQ, FH, FY | DEFERRED\_INCOME\_CURRENT |
| [Deferred income, non-current](https://www.tradingview.com/support/solutions/43000563540) | FQ, FH, FY | DEFERRED\_INCOME\_NON\_CURRENT |
| [Deferred tax assets](https://www.tradingview.com/support/solutions/43000563683) | FQ, FH, FY | DEFERRED\_TAX\_ASSESTS |
| [Deferred tax liabilities](https://www.tradingview.com/support/solutions/43000563536) | FQ, FH, FY | DEFERRED\_TAX\_LIABILITIES |
| [Dividends payable](https://www.tradingview.com/support/solutions/43000563624) | FY | DIVIDENDS\_PAYABLE |
| [Goodwill, net](https://www.tradingview.com/support/solutions/43000563688) | FQ, FH, FY | GOODWILL |
| [Gross property/plant/equipment](https://www.tradingview.com/support/solutions/43000563667) | FQ, FH, FY | PPE\_TOTAL\_GROSS |
| [Income tax payable](https://www.tradingview.com/support/solutions/43000563621) | FQ, FH, FY | INCOME\_TAX\_PAYABLE |
| [Inventories - finished goods](https://www.tradingview.com/support/solutions/43000563749) | FQ, FH, FY | INVENTORY\_FINISHED\_GOODS |
| [Inventories - progress payments & other](https://www.tradingview.com/support/solutions/43000563748) | FQ, FH, FY | INVENTORY\_PROGRESS\_PAYMENTS |
| [Inventories - raw materials](https://www.tradingview.com/support/solutions/43000563753) | FQ, FH, FY | INVENTORY\_RAW\_MATERIALS |
| [Inventories - work in progress](https://www.tradingview.com/support/solutions/43000563746) | FQ, FH, FY | INVENTORY\_WORK\_IN\_PROGRESS |
| [Investments in unconsolidated subsidiaries](https://www.tradingview.com/support/solutions/43000563645) | FQ, FH, FY | INVESTMENTS\_IN\_UNCONCSOLIDATE |
| [Long term debt](https://www.tradingview.com/support/solutions/43000553621) | FQ, FH, FY | LONG\_TERM\_DEBT |
| [Long term debt excl. lease liabilities](https://www.tradingview.com/support/solutions/43000563521) | FQ, FH, FY | LONG\_TERM\_DEBT\_EXCL\_CAPITAL\_LEASE |
| [Long term investments](https://www.tradingview.com/support/solutions/43000563639) | FQ, FH, FY | LONG\_TERM\_INVESTMENTS |
| [Minority interest](https://www.tradingview.com/support/solutions/43000563884) | FQ, FH, FY | MINORITY\_INTEREST |
| [Net debt](https://www.tradingview.com/support/solutions/43000665310) | FQ, FH, FY | NET\_DEBT |
| [Net intangible assets](https://www.tradingview.com/support/solutions/43000563686) | FQ, FH, FY | INTANGIBLES\_NET |
| [Net property/plant/equipment](https://www.tradingview.com/support/solutions/43000563657) | FQ, FH, FY | PPE\_TOTAL\_NET |
| [Note receivable - long term](https://www.tradingview.com/support/solutions/43000563641) | FQ, FH, FY | LONG\_TERM\_NOTE\_RECEIVABLE |
| [Notes payable](https://www.tradingview.com/support/solutions/43000563600) | FY | NOTES\_PAYABLE\_SHORT\_TERM\_DEBT |
| [Operating lease liabilities](https://www.tradingview.com/support/solutions/43000563532) | FQ, FH, FY | OPERATING\_LEASE\_LIABILITIES |
| [Other common equity](https://www.tradingview.com/support/solutions/43000563877) | FQ, FH, FY | OTHER\_COMMON\_EQUITY |
| [Other current assets, total](https://www.tradingview.com/support/solutions/43000563761) | FQ, FH, FY | OTHER\_CURRENT\_ASSETS\_TOTAL |
| [Other current liabilities](https://www.tradingview.com/support/solutions/43000563635) | FQ, FH, FY | OTHER\_CURRENT\_LIABILITIES |
| [Other intangibles, net](https://www.tradingview.com/support/solutions/43000563689) | FQ, FH, FY | OTHER\_INTANGIBLES\_NET |
| [Other investments](https://www.tradingview.com/support/solutions/43000563649) | FQ, FH, FY | OTHER\_INVESTMENTS |
| [Other long term assets, total](https://www.tradingview.com/support/solutions/43000563693) | FQ, FH, FY | LONG\_TERM\_OTHER\_ASSETS\_TOTAL |
| [Other non-current liabilities, total](https://www.tradingview.com/support/solutions/43000563545) | FQ, FH, FY | OTHER\_LIABILITIES\_TOTAL |
| [Other receivables](https://www.tradingview.com/support/solutions/43000563741) | FQ, FH, FY | OTHER\_RECEIVABLES |
| [Other short term debt](https://www.tradingview.com/support/solutions/43000563614) | FY | OTHER\_SHORT\_TERM\_DEBT |
| [Paid in capital](https://www.tradingview.com/support/solutions/43000563871) | FQ, FH, FY | PAID\_IN\_CAPITAL |
| [Preferred stock, carrying value](https://www.tradingview.com/support/solutions/43000563879) | FQ, FH, FY | PREFERRED\_STOCK\_CARRYING\_VALUE |
| [Prepaid expenses](https://www.tradingview.com/support/solutions/43000563757) | FQ, FH, FY | PREPAID\_EXPENSES |
| [Provision for risks & charge](https://www.tradingview.com/support/solutions/43000563535) | FQ, FH, FY | PROVISION\_F\_RISKS |
| [Retained earnings](https://www.tradingview.com/support/solutions/43000563867) | FQ, FH, FY | RETAINED\_EARNINGS |
| [Shareholders’ equity](https://www.tradingview.com/support/solutions/43000557442) | FQ, FH, FY | SHRHLDRS\_EQUITY |
| [Short term debt](https://www.tradingview.com/support/solutions/43000563554) | FQ, FH, FY | SHORT\_TERM\_DEBT |
| [Short term debt excl. current portion of LT debt](https://www.tradingview.com/support/solutions/43000563563) | FQ, FH, FY | SHORT\_TERM\_DEBT\_EXCL\_CURRENT\_PORT |
| [Short term investments](https://www.tradingview.com/support/solutions/43000563716) | FQ, FH, FY | SHORT\_TERM\_INVEST |
| [Tangible book value per share](https://www.tradingview.com/support/solutions/43000597072) | FQ, FH, FY | BOOK\_TANGIBLE\_PER\_SHARE |
| [Total assets](https://www.tradingview.com/support/solutions/43000553623) | FQ, FH, FY | TOTAL\_ASSETS |
| [Total current assets](https://www.tradingview.com/support/solutions/43000557441) | FQ, FH, FY | TOTAL\_CURRENT\_ASSETS |
| [Total current liabilities](https://www.tradingview.com/support/solutions/43000557437) | FQ, FH, FY | TOTAL\_CURRENT\_LIABILITIES |
| [Total debt](https://www.tradingview.com/support/solutions/43000553622) | FQ, FH, FY | TOTAL\_DEBT |
| [Total equity](https://www.tradingview.com/support/solutions/43000553625) | FQ, FH, FY | TOTAL\_EQUITY |
| [Total inventory](https://www.tradingview.com/support/solutions/43000563745) | FQ, FH, FY | TOTAL\_INVENTORY |
| [Total liabilities](https://www.tradingview.com/support/solutions/43000553624) | FQ, FH, FY | TOTAL\_LIABILITIES |
| [Total liabilities & shareholders’ equities](https://www.tradingview.com/support/solutions/43000553626) | FQ, FH, FY | TOTAL\_LIABILITIES\_SHRHLDRS\_EQUITY |
| [Total non-current assets](https://www.tradingview.com/support/solutions/43000557440) | FQ, FH, FY | TOTAL\_NON\_CURRENT\_ASSETS |
| [Total non-current liabilities](https://www.tradingview.com/support/solutions/43000557436) | FQ, FH, FY | TOTAL\_NON\_CURRENT\_LIABILITIES |
| [Total receivables, net](https://www.tradingview.com/support/solutions/43000563738) | FQ, FH, FY | TOTAL\_RECEIVABLES\_NET |
| [Treasury stock - common](https://www.tradingview.com/support/solutions/43000563875) | FQ, FH, FY | TREASURY\_STOCK\_COMMON |

#### Cash flow {#cash-flow}

This table lists the available metrics that provide information about how cash flows through a company.

Click to show/hide

| Financial | `period` | `financial_id` |
| --- | --- | --- |
| [Amortization](https://www.tradingview.com/support/solutions/43000564143) | FQ, FH, FY, TTM | AMORTIZATION |
| [Capital expenditures](https://www.tradingview.com/support/solutions/43000564166) | FQ, FH, FY, TTM | CAPITAL\_EXPENDITURES |
| [Capital expenditures - fixed assets](https://www.tradingview.com/support/solutions/43000564167) | FQ, FH, FY, TTM | CAPITAL\_EXPENDITURES\_FIXED\_ASSETS |
| [Capital expenditures - other assets](https://www.tradingview.com/support/solutions/43000564168) | FQ, FH, FY, TTM | CAPITAL\_EXPENDITURES\_OTHER\_ASSETS |
| [Cash from financing activities](https://www.tradingview.com/support/solutions/43000553629) | FQ, FH, FY, TTM | CASH\_F\_FINANCING\_ACTIVITIES |
| [Cash from investing activities](https://www.tradingview.com/support/solutions/43000553628) | FQ, FH, FY, TTM | CASH\_F\_INVESTING\_ACTIVITIES |
| [Cash from operating activities](https://www.tradingview.com/support/solutions/43000553627) | FQ, FH, FY, TTM | CASH\_F\_OPERATING\_ACTIVITIES |
| [Change in accounts payable](https://www.tradingview.com/support/solutions/43000564150) | FQ, FH, FY, TTM | CHANGE\_IN\_ACCOUNTS\_PAYABLE |
| [Change in accounts receivable](https://www.tradingview.com/support/solutions/43000564148) | FQ, FH, FY, TTM | CHANGE\_IN\_ACCOUNTS\_RECEIVABLE |
| [Change in accrued expenses](https://www.tradingview.com/support/solutions/43000564151) | FQ, FH, FY, TTM | CHANGE\_IN\_ACCRUED\_EXPENSES |
| [Change in inventories](https://www.tradingview.com/support/solutions/43000564153) | FQ, FH, FY, TTM | CHANGE\_IN\_INVENTORIES |
| [Change in other assets/liabilities](https://www.tradingview.com/support/solutions/43000564154) | FQ, FH, FY, TTM | CHANGE\_IN\_OTHER\_ASSETS |
| [Change in taxes payable](https://www.tradingview.com/support/solutions/43000564149) | FQ, FH, FY, TTM | CHANGE\_IN\_TAXES\_PAYABLE |
| [Changes in working capital](https://www.tradingview.com/support/solutions/43000564147) | FQ, FH, FY, TTM | CHANGES\_IN\_WORKING\_CAPITAL |
| [Common dividends paid](https://www.tradingview.com/support/solutions/43000564185) | FQ, FH, FY, TTM | COMMON\_DIVIDENDS\_CASH\_FLOW |
| [Deferred taxes (cash flow)](https://www.tradingview.com/support/solutions/43000564144) | FQ, FH, FY, TTM | CASH\_FLOW\_DEFERRED\_TAXES |
| [Depreciation & amortization (cash flow)](https://www.tradingview.com/support/solutions/43000563892) | FQ, FH, FY, TTM | CASH\_FLOW\_DEPRECATION\_N\_AMORTIZATION |
| [Depreciation/depletion](https://www.tradingview.com/support/solutions/43000564142) | FQ, FH, FY, TTM | DEPRECIATION\_DEPLETION |
| [Financing activities - other sources](https://www.tradingview.com/support/solutions/43000564181) | FQ, FH, FY, TTM | OTHER\_FINANCING\_CASH\_FLOW\_SOURCES |
| [Financing activities - other uses](https://www.tradingview.com/support/solutions/43000564182) | FQ, FH, FY, TTM | OTHER\_FINANCING\_CASH\_FLOW\_USES |
| [Free cash flow](https://www.tradingview.com/support/solutions/43000553630) | FQ, FH, FY, TTM | FREE\_CASH\_FLOW |
| [Funds from operations](https://www.tradingview.com/support/solutions/43000563886) | FQ, FH, FY, TTM | FUNDS\_F\_OPERATIONS |
| [Investing activities - other sources](https://www.tradingview.com/support/solutions/43000564164) | FQ, FH, FY, TTM | OTHER\_INVESTING\_CASH\_FLOW\_SOURCES |
| [Investing activities - other uses](https://www.tradingview.com/support/solutions/43000564165) | FQ, FH, FY | OTHER\_INVESTING\_CASH\_FLOW\_USES |
| [Issuance of long term debt](https://www.tradingview.com/support/solutions/43000564176) | FQ, FH, FY, TTM | SUPPLYING\_OF\_LONG\_TERM\_DEBT |
| [Issuance/retirement of debt, net](https://www.tradingview.com/support/solutions/43000564172) | FQ, FH, FY, TTM | ISSUANCE\_OF\_DEBT\_NET |
| [Issuance/retirement of long term debt](https://www.tradingview.com/support/solutions/43000564175) | FQ, FH, FY, TTM | ISSUANCE\_OF\_LONG\_TERM\_DEBT |
| [Issuance/retirement of other debt](https://www.tradingview.com/support/solutions/43000564178) | FQ, FH, FY, TTM | ISSUANCE\_OF\_OTHER\_DEBT |
| [Issuance/retirement of short term debt](https://www.tradingview.com/support/solutions/43000564173) | FQ, FH, FY, TTM | ISSUANCE\_OF\_SHORT\_TERM\_DEBT |
| [Issuance/retirement of stock, net](https://www.tradingview.com/support/solutions/43000564169) | FQ, FH, FY, TTM | ISSUANCE\_OF\_STOCK\_NET |
| [Net income (cash flow)](https://www.tradingview.com/support/solutions/43000563888) | FQ, FH, FY, TTM | NET\_INCOME\_STARTING\_LINE |
| [Non-cash items](https://www.tradingview.com/support/solutions/43000564146) | FQ, FH, FY, TTM | NON\_CASH\_ITEMS |
| [Other financing cash flow items, total](https://www.tradingview.com/support/solutions/43000564179) | FQ, FH, FY, TTM | OTHER\_FINANCING\_CASH\_FLOW\_ITEMS\_TOTAL |
| [Other investing cash flow items, total](https://www.tradingview.com/support/solutions/43000564163) | FQ, FH, FY | OTHER\_INVESTING\_CASH\_FLOW\_ITEMS\_TOTAL |
| [Preferred dividends paid](https://www.tradingview.com/support/solutions/43000564186) | FQ, FH, FY | PREFERRED\_DIVIDENDS\_CASH\_FLOW |
| [Purchase of investments](https://www.tradingview.com/support/solutions/43000564162) | FQ, FH, FY, TTM | PURCHASE\_OF\_INVESTMENTS |
| [Purchase/acquisition of business](https://www.tradingview.com/support/solutions/43000564159) | FQ, FH, FY, TTM | PURCHASE\_OF\_BUSINESS |
| [Purchase/sale of business, net](https://www.tradingview.com/support/solutions/43000564156) | FQ, FH, FY | PURCHASE\_SALE\_BUSINESS |
| [Purchase/sale of investments, net](https://www.tradingview.com/support/solutions/43000564160) | FQ, FH, FY, TTM | PURCHASE\_SALE\_INVESTMENTS |
| [Reduction of long term debt](https://www.tradingview.com/support/solutions/43000564177) | FQ, FH, FY, TTM | REDUCTION\_OF\_LONG\_TERM\_DEBT |
| [Repurchase of common & preferred stock](https://www.tradingview.com/support/solutions/43000564171) | FQ, FH, FY, TTM | PURCHASE\_OF\_STOCK |
| [Sale of common & preferred stock](https://www.tradingview.com/support/solutions/43000564170) | FQ, FH, FY, TTM | SALE\_OF\_STOCK |
| [Sale of fixed assets & businesses](https://www.tradingview.com/support/solutions/43000564158) | FQ, FH, FY, TTM | SALES\_OF\_BUSINESS |
| [Sale/maturity of investments](https://www.tradingview.com/support/solutions/43000564161) | FQ, FH, FY | SALES\_OF\_INVESTMENTS |
| [Total cash dividends paid](https://www.tradingview.com/support/solutions/43000564183) | FQ, FH, FY, TTM | TOTAL\_CASH\_DIVIDENDS\_PAID |

#### Statistics {#statistics}

This table contains a variety of statistical metrics, including commonly used financial ratios.

Click to show/hide

| Financial | `period` | `financial_id` |
| --- | --- | --- |
| [Accruals](https://www.tradingview.com/support/solutions/43000597073) | FQ, FH, FY | ACCRUALS\_RATIO |
| [Altman Z-score](https://www.tradingview.com/support/solutions/43000597092) | FQ, FH, FY | ALTMAN\_Z\_SCORE |
| [Asset turnover](https://www.tradingview.com/support/solutions/43000597022) | FQ, FH, FY | ASSET\_TURNOVER |
| [Beneish M-score](https://www.tradingview.com/support/solutions/43000597835) | FQ, FH, FY | BENEISH\_M\_SCORE |
| [Buyback yield %](https://www.tradingview.com/support/solutions/43000597088) | FQ, FH, FY | BUYBACK\_YIELD |
| [COGS to revenue ratio](https://www.tradingview.com/support/solutions/43000597026) | FQ, FH, FY | COGS\_TO\_REVENUE |
| [Cash conversion cycle](https://www.tradingview.com/support/solutions/43000597089) | FQ, FY | CASH\_CONVERSION\_CYCLE |
| [Cash to debt ratio](https://www.tradingview.com/support/solutions/43000597023) | FQ, FH, FY | CASH\_TO\_DEBT |
| [Current ratio](https://www.tradingview.com/support/solutions/43000597051) | FQ, FH, FY | CURRENT\_RATIO |
| [Days inventory](https://www.tradingview.com/support/solutions/43000597028) | FQ, FY | DAYS\_INVENT |
| [Days payable](https://www.tradingview.com/support/solutions/43000597029) | FQ, FY | DAYS\_PAY |
| [Days sales outstanding](https://www.tradingview.com/support/solutions/43000597030) | FQ, FY | DAY\_SALES\_OUT |
| [Debt to EBITDA ratio](https://www.tradingview.com/support/solutions/43000597032) | FQ, FH, FY | DEBT\_TO\_EBITDA |
| [Debt to assets ratio](https://www.tradingview.com/support/solutions/43000597031) | FQ, FH, FY | DEBT\_TO\_ASSET |
| [Debt to equity ratio](https://www.tradingview.com/support/solutions/43000597078) | FQ, FH, FY | DEBT\_TO\_EQUITY |
| [Debt to revenue ratio](https://www.tradingview.com/support/solutions/43000597033) | FQ, FH, FY | DEBT\_TO\_REVENUE |
| [Dividend payout ratio %](https://www.tradingview.com/support/solutions/43000597738) | FQ, FH, FY, TTM | DIVIDEND\_PAYOUT\_RATIO |
| [Dividend yield %](https://www.tradingview.com/support/solutions/43000597817) | FQ, FH, FY | DIVIDENDS\_YIELD |
| [Dividends per share - common stock primary issue](https://www.tradingview.com/support/solutions/43000670334) | FQ, FH, FY, TTM | DPS\_COMMON\_STOCK\_PRIM\_ISSUE |
| [EBITDA margin %](https://www.tradingview.com/support/solutions/43000597075) | FQ, FH, FY, TTM | EBITDA\_MARGIN |
| [EPS basic one year growth](https://www.tradingview.com/support/solutions/43000597069) | FQ, FH, FY, TTM | EARNINGS\_PER\_SHARE\_BASIC\_ONE\_YEAR\_GROWTH |
| [EPS diluted one year growth](https://www.tradingview.com/support/solutions/43000597071) | FQ, FH, FY | EARNINGS\_PER\_SHARE\_DILUTED\_ONE\_YEAR\_GROWTH |
| [EPS estimates](https://www.tradingview.com/support/solutions/43000597066) | FQ, FH, FY | EARNINGS\_ESTIMATE |
| [Effective interest rate on debt %](https://www.tradingview.com/support/solutions/43000597034) | FQ, FH, FY | EFFECTIVE\_INTEREST\_RATE\_ON\_DEBT |
| [Enterprise value](https://www.tradingview.com/support/solutions/43000597077) | FQ, FH, FY | ENTERPRISE\_VALUE |
| [Enterprise value to EBIT ratio](https://www.tradingview.com/support/solutions/43000597063) | FQ, FH, FY | EV\_EBIT |
| [Enterprise value to EBITDA ratio](https://www.tradingview.com/support/solutions/43000597064) | FQ, FH, FY | ENTERPRISE\_VALUE\_EBITDA |
| [Enterprise value to revenue ratio](https://www.tradingview.com/support/solutions/43000597065) | FQ, FH, FY | EV\_REVENUE |
| [Equity to assets ratio](https://www.tradingview.com/support/solutions/43000597035) | FQ, FH, FY | EQUITY\_TO\_ASSET |
| [Float shares outstanding](https://www.tradingview.com/support/solutions/43000670341) | FY | FLOAT\_SHARES\_OUTSTANDING |
| [Free cash flow margin %](https://www.tradingview.com/support/solutions/43000597813) | FQ, FH, FY | FREE\_CASH\_FLOW\_MARGIN |
| [Fulmer H factor](https://www.tradingview.com/support/solutions/43000597847) | FQ, FY | FULMER\_H\_FACTOR |
| [Goodwill to assets ratio](https://www.tradingview.com/support/solutions/43000597036) | FQ, FH, FY | GOODWILL\_TO\_ASSET |
| [Graham’s number](https://www.tradingview.com/support/solutions/43000597084) | FQ, FY | GRAHAM\_NUMBERS |
| [Gross margin %](https://www.tradingview.com/support/solutions/43000597811) | FQ, FH, FY, TTM | GROSS\_MARGIN |
| [Gross profit to assets ratio](https://www.tradingview.com/support/solutions/43000597087) | FQ, FY | GROSS\_PROFIT\_TO\_ASSET |
| [Interest coverage](https://www.tradingview.com/support/solutions/43000597037) | FQ, FH, FY | INTERST\_COVER |
| [Inventory to revenue ratio](https://www.tradingview.com/support/solutions/43000597047) | FQ, FH, FY | INVENT\_TO\_REVENUE |
| [Inventory turnover](https://www.tradingview.com/support/solutions/43000597046) | FQ, FH, FY | INVENT\_TURNOVER |
| [KZ index](https://www.tradingview.com/support/solutions/43000597844) | FY | KZ\_INDEX |
| [Long term debt to total assets ratio](https://www.tradingview.com/support/solutions/43000597048) | FQ, FH, FY | LONG\_TERM\_DEBT\_TO\_ASSETS |
| [Net current asset value per share](https://www.tradingview.com/support/solutions/43000597085) | FQ, FY | NCAVPS\_RATIO |
| [Net income per employee](https://www.tradingview.com/support/solutions/43000597082) | FY | NET\_INCOME\_PER\_EMPLOYEE |
| [Net margin %](https://www.tradingview.com/support/solutions/43000597074) | FQ, FH, FY, TTM | NET\_MARGIN |
| [Number of employees](https://www.tradingview.com/support/solutions/43000597080) | FY | NUMBER\_OF\_EMPLOYEES |
| [Operating earnings yield %](https://www.tradingview.com/support/solutions/43000684072) | FQ, FH, FY | OPERATING\_EARNINGS\_YIELD |
| [Operating margin %](https://www.tradingview.com/support/solutions/43000597076) | FQ, FH, FY | OPERATING\_MARGIN |
| [PEG ratio](https://www.tradingview.com/support/solutions/43000597090) | FQ, FY | PEG\_RATIO |
| [Piotroski F-score](https://www.tradingview.com/support/solutions/43000597734) | FQ, FH, FY | PIOTROSKI\_F\_SCORE |
| [Price earnings ratio forward](https://www.tradingview.com/support/solutions/43000597831) | FQ, FY | PRICE\_EARNINGS\_FORWARD |
| [Price sales ratio forward](https://www.tradingview.com/support/solutions/43000597832) | FQ, FY | PRICE\_SALES\_FORWARD |
| [Quality ratio](https://www.tradingview.com/support/solutions/43000597086) | FQ, FH, FY | QUALITY\_RATIO |
| [Quick ratio](https://www.tradingview.com/support/solutions/43000597050) | FQ, FH, FY | QUICK\_RATIO |
| [Research & development to revenue ratio](https://www.tradingview.com/support/solutions/43000597739) | FQ, FH, FY | RESEARCH\_AND\_DEVELOP\_TO\_REVENUE |
| [Return on assets %](https://www.tradingview.com/support/solutions/43000597054) | FQ, FH, FY | RETURN\_ON\_ASSETS |
| [Return on common equity %](https://www.tradingview.com/support/solutions/43000656797) | FQ, FH, FY | RETURN\_ON\_COMMON\_EQUITY |
| [Return on equity %](https://www.tradingview.com/support/solutions/43000597021) | FQ, FH, FY | RETURN\_ON\_EQUITY |
| [Return on equity adjusted to book value %](https://www.tradingview.com/support/solutions/43000597055) | FQ, FH, FY | RETURN\_ON\_EQUITY\_ADJUST\_TO\_BOOK |
| [Return on invested capital %](https://www.tradingview.com/support/solutions/43000597056) | FQ, FH, FY | RETURN\_ON\_INVESTED\_CAPITAL |
| [Return on tangible assets %](https://www.tradingview.com/support/solutions/43000597052) | FQ, FH, FY | RETURN\_ON\_TANG\_ASSETS |
| [Return on tangible equity %](https://www.tradingview.com/support/solutions/43000597053) | FQ, FH, FY | RETURN\_ON\_TANG\_EQUITY |
| [Revenue estimates](https://www.tradingview.com/support/solutions/43000597067) | FQ, FH, FY | SALES\_ESTIMATES |
| [Revenue one year growth](https://www.tradingview.com/support/solutions/43000597068) | FQ, FH, FY, TTM | REVENUE\_ONE\_YEAR\_GROWTH |
| [Revenue per employee](https://www.tradingview.com/support/solutions/43000597081) | FY | REVENUE\_PER\_EMPLOYEE |
| [Shares buyback ratio %](https://www.tradingview.com/support/solutions/43000597057) | FQ, FH, FY | SHARE\_BUYBACK\_RATIO |
| [Sloan ratio %](https://www.tradingview.com/support/solutions/43000597058) | FQ, FH, FY | SLOAN\_RATIO |
| [Springate score](https://www.tradingview.com/support/solutions/43000597848) | FQ, FY | SPRINGATE\_SCORE |
| [Sustainable growth rate](https://www.tradingview.com/support/solutions/43000597736) | FQ, FY | SUSTAINABLE\_GROWTH\_RATE |
| [Tangible common equity ratio](https://www.tradingview.com/support/solutions/43000597079) | FQ, FH, FY | TANGIBLE\_COMMON\_EQUITY\_RATIO |
| [Tobin’s Q (approximate)](https://www.tradingview.com/support/solutions/43000597834) | FQ, FH, FY | TOBIN\_Q\_RATIO |
| [Total common shares outstanding](https://www.tradingview.com/support/solutions/43000670331) | FQ, FH, FY | TOTAL\_SHARES\_OUTSTANDING |
| [Zmijewski score](https://www.tradingview.com/support/solutions/43000597850) | FQ, FY | ZMIJEWSKI\_SCORE |

## `request.economic()` {#requesteconomic}

The [request.economic()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.economic) function provides scripts with the ability to retrieve economic data for a specified country or region, including information about the state of the economy (GDP, inflation rate, etc.) or of a particular industry (steel production, ICU beds, etc.).

Below is the signature for this function:

```
request.economic(country_code, field, gaps, ignore_invalid_symbol) → series float
```

The `country_code` parameter accepts a “string” value representing the identifier of the country or region to request economic data for (e.g., “US”, “EU”, etc.). See the [Country/region codes](https://www.tradingview.com/pine-script-docs/concepts/other-timeframes-and-data/#countryregion-codes) section for a complete list of codes this function supports. Note that the economic metrics available depend on the country or region specified in the function call.

The `field` parameter accepts a “string” specifying the metric that the function requests. The [Field codes](https://www.tradingview.com/pine-script-docs/concepts/other-timeframes-and-data/#field-codes) section covers all accessible metrics and the countries/regions they’re available for.

For a detailed explanation on the last two parameters of this function, see the [Common characteristics](https://www.tradingview.com/pine-script-docs/concepts/other-timeframes-and-data/#common-characteristics) section at the top of this page.

This simple example requests the growth rate of the Gross Domestic Product (“GDPQQ”) for the United States (“US”) using [request.economic()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.economic), then [plots](https://www.tradingview.com/pine-script-docs/visuals/plots/) its value on the chart with a [gradient color](concepts/colors/#colorfrom_gradient):

```pine
//@version=6
indicator("Requesting economic data demo")

//@variable The GDP growth rate for the US economy.
float gdpqq = request.economic("US", "GDPQQ")

//@variable The all-time maximum growth rate.
float maxRate = ta.max(gdpqq)
//@variable The all-time minimum growth rate.
float minRate = ta.min(gdpqq)

//@variable The color of the `gdpqq` plot.
color rateColor = switch
    gdpqq >= 0 => color.from_gradient(gdpqq, 0, maxRate, color.purple, color.blue)
    =>            color.from_gradient(gdpqq, minRate, 0, color.red, color.purple)

// Plot the results.
plot(gdpqq, "US GDP Growth Rate", rateColor, style = plot.style_area)
```

Note that:

-   This example does not include a `gaps` argument in the [request.economic()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.economic) call, so the function uses the default [barmerge.gaps\_off](https://www.tradingview.com/pine-script-reference/v6/#var_barmerge.gaps_off). In other words, it returns the last retrieved value when new data isn’t yet available.

> [!TIP]
> The tables in the sections below are rather large, because there are numerous `country_code` and `field` arguments available. Use the **“Click to show/hide”** option above each table to toggle its visibility.

### Country/region codes {#countryregion-codes}

The table in this section lists all country/region codes available for use with [request.economic()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.economic). The first column of the table contains the “string” values that represent the country or region code, and the second column contains the corresponding country/region names.

It’s important to note that the value used as the `country_code` argument determines which [field codes](https://www.tradingview.com/pine-script-docs/concepts/other-timeframes-and-data/#field-codes) are accessible to the function.

Click to show/hide

| `country_code` | Country/region name |
| --- | --- |
| AF | Afghanistan |
| AL | Albania |
| DZ | Algeria |
| AD | Andorra |
| AO | Angola |
| AG | Antigua and Barbuda |
| AR | Argentina |
| AM | Armenia |
| AW | Aruba |
| AU | Australia |
| AT | Austria |
| AZ | Azerbaijan |
| BS | Bahamas |
| BH | Bahrain |
| BD | Bangladesh |
| BB | Barbados |
| BY | Belarus |
| BE | Belgium |
| BZ | Belize |
| BJ | Benin |
| BM | Bermuda |
| BT | Bhutan |
| BO | Bolivia |
| BA | Bosnia and Herzegovina |
| BW | Botswana |
| BR | Brazil |
| BN | Brunei |
| BG | Bulgaria |
| BF | Burkina Faso |
| BI | Burundi |
| KH | Cambodia |
| CM | Cameroon |
| CA | Canada |
| CV | Cape Verde |
| KY | Cayman Islands |
| CF | Central African Republic |
| TD | Chad |
| CL | Chile |
| CN | China |
| CO | Colombia |
| KM | Comoros |
| CG | Congo |
| CR | Costa Rica |
| HR | Croatia |
| CU | Cuba |
| CY | Cyprus |
| CZ | Czech Republic |
| DK | Denmark |
| DJ | Djibouti |
| DM | Dominica |
| DO | Dominican Republic |
| TL | East Timor |
| EC | Ecuador |
| EG | Egypt |
| SV | El Salvador |
| GQ | Equatorial Guinea |
| ER | Eritrea |
| EE | Estonia |
| ET | Ethiopia |
| EU | Euro area |
| FO | Faroe Islands |
| FJ | Fiji |
| FI | Finland |
| FR | France |
| GA | Gabon |
| GM | Gambia |
| GE | Georgia |
| DE | Germany |
| GH | Ghana |
| GR | Greece |
| GL | Greenland |
| GD | Grenada |
| GT | Guatemala |
| GN | Guinea |
| GW | Guinea Bissau |
| GY | Guyana |
| HT | Haiti |
| HN | Honduras |
| HK | Hong Kong |
| HU | Hungary |
| IS | Iceland |
| IN | India |
| ID | Indonesia |
| IR | Iran |
| IQ | Iraq |
| IE | Ireland |
| IM | Isle of Man |
| IL | Israel |
| IT | Italy |
| CI | Ivory Coast |
| JM | Jamaica |
| JP | Japan |
| JO | Jordan |
| KZ | Kazakhstan |
| KE | Kenya |
| KI | Kiribati |
| XK | Kosovo |
| KW | Kuwait |
| KG | Kyrgyzstan |
| LA | Laos |
| LV | Latvia |
| LB | Lebanon |
| LS | Lesotho |
| LR | Liberia |
| LY | Libya |
| LI | Liechtenstein |
| LT | Lithuania |
| LU | Luxembourg |
| MO | Macau |
| MK | Macedonia |
| MG | Madagascar |
| MW | Malawi |
| MY | Malaysia |
| MV | Maldives |
| ML | Mali |
| MT | Malta |
| MR | Mauritania |
| MU | Mauritius |
| MX | Mexico |
| MD | Moldova |
| MC | Monaco |
| MN | Mongolia |
| ME | Montenegro |
| MA | Morocco |
| MZ | Mozambique |
| MM | Myanmar |
| NA | Namibia |
| NP | Nepal |
| NL | Netherlands |
| NC | New Caledonia |
| NZ | New Zealand |
| NI | Nicaragua |
| NE | Niger |
| NG | Nigeria |
| KP | North Korea |
| NO | Norway |
| OM | Oman |
| PK | Pakistan |
| PS | Palestine |
| PA | Panama |
| PG | Papua New Guinea |
| PY | Paraguay |
| PE | Peru |
| PH | Philippines |
| PL | Poland |
| PT | Portugal |
| PR | Puerto Rico |
| QA | Qatar |
| CD | Republic of the Congo |
| RO | Romania |
| RU | Russia |
| RW | Rwanda |
| WS | Samoa |
| SM | San Marino |
| ST | Sao Tome and Principe |
| SA | Saudi Arabia |
| SN | Senegal |
| RS | Serbia |
| SC | Seychelles |
| SL | Sierra Leone |
| SG | Singapore |
| SK | Slovakia |
| SI | Slovenia |
| SB | Solomon Islands |
| SO | Somalia |
| ZA | South Africa |
| KR | South Korea |
| SS | South Sudan |
| ES | Spain |
| LK | Sri Lanka |
| LC | St Lucia |
| VC | St Vincent and the Grenadines |
| SD | Sudan |
| SR | Suriname |
| SZ | Swaziland |
| SE | Sweden |
| CH | Switzerland |
| SY | Syria |
| TW | Taiwan |
| TJ | Tajikistan |
| TZ | Tanzania |
| TH | Thailand |
| TG | Togo |
| TO | Tonga |
| TT | Trinidad and Tobago |
| TN | Tunisia |
| TR | Turkey |
| TM | Turkmenistan |
| UG | Uganda |
| UA | Ukraine |
| AE | United Arab Emirates |
| GB | United Kingdom |
| US | United States |
| UY | Uruguay |
| UZ | Uzbekistan |
| VU | Vanuatu |
| VE | Venezuela |
| VN | Vietnam |
| YE | Yemen |
| ZM | Zambia |
| ZW | Zimbabwe |

### Field codes {#field-codes}

The table in this section lists the field codes available for use with [request.economic()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.economic). The first column contains the “string” values used as the `field` argument, and the second column contains names of each metric and links to our Help Center with additional information, including the countries/regions they’re available for.

Click to show/hide

| `field` | Metric |
| --- | --- |
| AA | [Asylum Applications](https://www.tradingview.com/support/solutions/43000650926) |
| ACR | [API Crude Runs](https://www.tradingview.com/support/solutions/43000650920) |
| AE | [Auto Exports](https://www.tradingview.com/support/solutions/43000650927) |
| AHE | [Average Hourly Earnings](https://www.tradingview.com/support/solutions/43000650928) |
| AHO | [API Heating Oil](https://www.tradingview.com/support/solutions/43000650924) |
| AWH | [Average Weekly Hours](https://www.tradingview.com/support/solutions/43000650929) |
| BBS | [Banks Balance Sheet](https://www.tradingview.com/support/solutions/43000650932) |
| BCLI | [Business Climate Indicator](https://www.tradingview.com/support/solutions/43000650935) |
| BCOI | [Business Confidence Index](https://www.tradingview.com/support/solutions/43000650936) |
| BI | [Business Inventories](https://www.tradingview.com/support/solutions/43000650937) |
| BLR | [Bank Lending Rate](https://www.tradingview.com/support/solutions/43000650933) |
| BOI | [NFIB Business Optimism Index](https://www.tradingview.com/support/solutions/43000651133) |
| BOT | [Balance Of Trade](https://www.tradingview.com/support/solutions/43000650930) |
| BP | [Building Permits](https://www.tradingview.com/support/solutions/43000650934) |
| BR | [Bankruptcies](https://www.tradingview.com/support/solutions/43000650931) |
| CA | [Current Account](https://www.tradingview.com/support/solutions/43000650988) |
| CAG | [Current Account To GDP](https://www.tradingview.com/support/solutions/43000650987) |
| CAP | [Car Production](https://www.tradingview.com/support/solutions/43000650945) |
| CAR | [Car Registrations](https://www.tradingview.com/support/solutions/43000650946) |
| CBBS | [Central Bank Balance Sheet](https://www.tradingview.com/support/solutions/43000650952) |
| CCC | [Claimant Count Change](https://www.tradingview.com/support/solutions/43000650959) |
| CCI | [Consumer Confidence Index](https://www.tradingview.com/support/solutions/43000650966) |
| CCOS | [Cushing Crude Oil Stocks](https://www.tradingview.com/support/solutions/43000650989) |
| CCP | [Core Consumer Prices](https://www.tradingview.com/support/solutions/43000650974) |
| CCPI | [Core CPI](https://www.tradingview.com/support/solutions/43000650973) |
| CCPT | [Consumer Confidence Price Trends](https://www.tradingview.com/support/solutions/43000650967) |
| CCR | [Consumer Credit](https://www.tradingview.com/support/solutions/43000650968) |
| CCS | [Credit Card Spending](https://www.tradingview.com/support/solutions/43000650982) |
| CEP | [Cement Production](https://www.tradingview.com/support/solutions/43000650951) |
| CF | [Capital Flows](https://www.tradingview.com/support/solutions/43000650944) |
| CFNAI | [Chicago Fed National Activity Index](https://www.tradingview.com/support/solutions/43000650957) |
| CI | [API Crude Imports](https://www.tradingview.com/support/solutions/43000650918) |
| CIND | [Coincident Index](https://www.tradingview.com/support/solutions/43000650960) |
| CIR | [Core Inflation Rate, YoY](https://www.tradingview.com/support/solutions/43000650975) |
| CJC | [Continuing Jobless Claims](https://www.tradingview.com/support/solutions/43000650971) |
| CN | [API Cushing Number](https://www.tradingview.com/support/solutions/43000650921) |
| COI | [Crude Oil Imports](https://www.tradingview.com/support/solutions/43000650983) |
| COIR | [Crude Oil Imports from Russia](https://www.tradingview.com/support/solutions/43000679670) |
| CONSTS | [Construction Spending](https://www.tradingview.com/support/solutions/43000650965) |
| COP | [Crude Oil Production](https://www.tradingview.com/support/solutions/43000650984) |
| COR | [Crude Oil Rigs](https://www.tradingview.com/support/solutions/43000650985) |
| CORD | [Construction Orders, YoY](https://www.tradingview.com/support/solutions/43000650963) |
| CORPI | [Corruption Index](https://www.tradingview.com/support/solutions/43000650980) |
| CORR | [Corruption Rank](https://www.tradingview.com/support/solutions/43000650981) |
| COSC | [Crude Oil Stocks Change](https://www.tradingview.com/support/solutions/43000650986) |
| COUT | [Construction Output, YoY](https://www.tradingview.com/support/solutions/43000650964) |
| CP | [Copper Production](https://www.tradingview.com/support/solutions/43000650972) |
| CPCEPI | [Core PCE Price Index](https://www.tradingview.com/support/solutions/43000650976) |
| CPI | [Consumer Price Index](https://www.tradingview.com/support/solutions/43000650969) |
| CPIHU | [CPI Housing Utilities](https://www.tradingview.com/support/solutions/43000650939) |
| CPIM | [CPI Median](https://www.tradingview.com/support/solutions/43000650940) |
| CPIT | [CPI Transportation](https://www.tradingview.com/support/solutions/43000650941) |
| CPITM | [CPI Trimmed Mean](https://www.tradingview.com/support/solutions/43000650942) |
| CPMI | [Chicago PMI](https://www.tradingview.com/support/solutions/43000650958) |
| CPPI | [Core Producer Price Index](https://www.tradingview.com/support/solutions/43000650977) |
| CPR | [Corporate Profits](https://www.tradingview.com/support/solutions/43000650978) |
| CRLPI | [Cereals Price Index](https://www.tradingview.com/support/solutions/43000679669) |
| CRR | [Cash Reserve Ratio](https://www.tradingview.com/support/solutions/43000650950) |
| CS | [Consumer Spending](https://www.tradingview.com/support/solutions/43000650970) |
| CSC | [API Crude Oil Stock Change](https://www.tradingview.com/support/solutions/43000650919) |
| CSHPI | [Case Shiller Home Price Index](https://www.tradingview.com/support/solutions/43000650947) |
| CSHPIMM | [Case Shiller Home Price Index, MoM](https://www.tradingview.com/support/solutions/43000650948) |
| CSHPIYY | [Case Shiller Home Price Index, YoY](https://www.tradingview.com/support/solutions/43000650949) |
| CSS | [Chain Store Sales](https://www.tradingview.com/support/solutions/43000650954) |
| CTR | [Corporate Tax Rate](https://www.tradingview.com/support/solutions/43000650979) |
| CU | [Capacity Utilization](https://www.tradingview.com/support/solutions/43000650943) |
| DFMI | [Dallas Fed Manufacturing Index](https://www.tradingview.com/support/solutions/43000650990) |
| DFP | [Distillate Fuel Production](https://www.tradingview.com/support/solutions/43000650996) |
| DFS | [Distillate Stocks](https://www.tradingview.com/support/solutions/43000650997) |
| DFSI | [Dallas Fed Services Index](https://www.tradingview.com/support/solutions/43000650991) |
| DFSRI | [Dallas Fed Services Revenues Index](https://www.tradingview.com/support/solutions/43000650992) |
| DG | [Deposit Growth](https://www.tradingview.com/support/solutions/43000650993) |
| DGO | [Durable Goods Orders](https://www.tradingview.com/support/solutions/43000651000) |
| DGOED | [Durable Goods Orders Excluding Defense](https://www.tradingview.com/support/solutions/43000650998) |
| DGOET | [Durable Goods Orders Excluding Transportation](https://www.tradingview.com/support/solutions/43000650999) |
| DIR | [Deposit Interest Rate](https://www.tradingview.com/support/solutions/43000650994) |
| DPI | [Disposable Personal Income](https://www.tradingview.com/support/solutions/43000650995) |
| DRPI | [Dairy Price Index](https://www.tradingview.com/support/solutions/43000679668) |
| DS | [API Distillate Stocks](https://www.tradingview.com/support/solutions/43000650922) |
| DT | [CBI Distributive Trades](https://www.tradingview.com/support/solutions/43000650938) |
| EC | [ADP Employment Change](https://www.tradingview.com/support/solutions/43000650917) |
| ED | [External Debt](https://www.tradingview.com/support/solutions/43000651012) |
| EDBR | [Ease Of Doing Business Ranking](https://www.tradingview.com/support/solutions/43000651001) |
| EHS | [Existing Home Sales](https://www.tradingview.com/support/solutions/43000651009) |
| ELP | [Electricity Production](https://www.tradingview.com/support/solutions/43000651004) |
| EMC | [Employment Change](https://www.tradingview.com/support/solutions/43000651006) |
| EMCI | [Employment Cost Index](https://www.tradingview.com/support/solutions/43000651007) |
| EMP | [Employed Persons](https://www.tradingview.com/support/solutions/43000651005) |
| EMR | [Employment Rate](https://www.tradingview.com/support/solutions/43000651008) |
| EOI | [Economic Optimism Index](https://www.tradingview.com/support/solutions/43000651002) |
| EP | [Export Prices](https://www.tradingview.com/support/solutions/43000651011) |
| ESI | [ZEW Economic Sentiment Index](https://www.tradingview.com/support/solutions/43000651213) |
| EWS | [Economy Watchers Survey](https://www.tradingview.com/support/solutions/43000651003) |
| EXP | [Exports](https://www.tradingview.com/support/solutions/43000651010) |
| EXPYY | [Exports, YoY](https://www.tradingview.com/support/solutions/43000679671) |
| FAI | [Fixed Asset Investment](https://www.tradingview.com/support/solutions/43000651016) |
| FBI | [Foreign Bond Investment](https://www.tradingview.com/support/solutions/43000651018) |
| FDI | [Foreign Direct Investment](https://www.tradingview.com/support/solutions/43000651019) |
| FE | [Fiscal Expenditure](https://www.tradingview.com/support/solutions/43000651015) |
| FER | [Foreign Exchange Reserves](https://www.tradingview.com/support/solutions/43000651020) |
| FI | [Food Inflation, YoY](https://www.tradingview.com/support/solutions/43000651017) |
| FO | [Factory Orders](https://www.tradingview.com/support/solutions/43000651014) |
| FOET | [Factory Orders Excluding Transportation](https://www.tradingview.com/support/solutions/43000651013) |
| FPI | [Food Price Index](https://www.tradingview.com/support/solutions/43000679667) |
| FSI | [Foreign Stock Investment](https://www.tradingview.com/support/solutions/43000651021) |
| FTE | [Full Time Employment](https://www.tradingview.com/support/solutions/43000651022) |
| FYGDPG | [Full Year GDP Growth](https://www.tradingview.com/support/solutions/43000679672) |
| GASP | [Gasoline Prices](https://www.tradingview.com/support/solutions/43000651040) |
| GBP | [Government Budget](https://www.tradingview.com/support/solutions/43000651050) |
| GBV | [Government Budget Value](https://www.tradingview.com/support/solutions/43000651049) |
| GCI | [Competitiveness Index](https://www.tradingview.com/support/solutions/43000650961) |
| GCR | [Competitiveness Rank](https://www.tradingview.com/support/solutions/43000650962) |
| GD | [Government Debt](https://www.tradingview.com/support/solutions/43000651052) |
| GDG | [Government Debt To GDP](https://www.tradingview.com/support/solutions/43000651051) |
| GDP | [Gross Domestic Product](https://www.tradingview.com/support/solutions/43000651038) |
| GDPA | [GDP From Agriculture](https://www.tradingview.com/support/solutions/43000651025) |
| GDPC | [GDP From Construction](https://www.tradingview.com/support/solutions/43000651026) |
| GDPCP | [GDP Constant Prices](https://www.tradingview.com/support/solutions/43000651023) |
| GDPD | [GDP Deflator](https://www.tradingview.com/support/solutions/43000651024) |
| GDPGA | [GDP Growth Annualized](https://www.tradingview.com/support/solutions/43000651033) |
| GDPMAN | [GDP From Manufacturing](https://www.tradingview.com/support/solutions/43000651027) |
| GDPMIN | [GDP From Mining](https://www.tradingview.com/support/solutions/43000651028) |
| GDPPA | [GDP From Public Administration](https://www.tradingview.com/support/solutions/43000651029) |
| GDPPC | [GDP Per Capita](https://www.tradingview.com/support/solutions/43000651035) |
| GDPPCP | [GDP Per Capita, PPP](https://www.tradingview.com/support/solutions/43000651036) |
| GDPQQ | [GDP Growth Rate](https://www.tradingview.com/support/solutions/43000651034) |
| GDPS | [GDP From Services](https://www.tradingview.com/support/solutions/43000651030) |
| GDPSA | [GDP Sales](https://www.tradingview.com/support/solutions/43000651037) |
| GDPT | [GDP From Transport](https://www.tradingview.com/support/solutions/43000651031) |
| GDPU | [GDP From Utilities](https://www.tradingview.com/support/solutions/43000651032) |
| GDPYY | [GDP, YoY](https://www.tradingview.com/support/solutions/43000651039) |
| GDTPI | [Global Dairy Trade Price Index](https://www.tradingview.com/support/solutions/43000651043) |
| GFCF | [Gross Fixed Capital Formation](https://www.tradingview.com/support/solutions/43000651060) |
| GNP | [Gross National Product](https://www.tradingview.com/support/solutions/43000651061) |
| GP | [Gold Production](https://www.tradingview.com/support/solutions/43000651044) |
| GPA | [Government Payrolls](https://www.tradingview.com/support/solutions/43000651053) |
| GPRO | [Gasoline Production](https://www.tradingview.com/support/solutions/43000651041) |
| GR | [Government Revenues](https://www.tradingview.com/support/solutions/43000651054) |
| GRES | [Gold Reserves](https://www.tradingview.com/support/solutions/43000651045) |
| GS | [API Gasoline Stocks](https://www.tradingview.com/support/solutions/43000650923) |
| GSC | [Grain Stocks Corn](https://www.tradingview.com/support/solutions/43000651057) |
| GSCH | [Gasoline Stocks Change](https://www.tradingview.com/support/solutions/43000651042) |
| GSG | [Government Spending To GDP](https://www.tradingview.com/support/solutions/43000651055) |
| GSP | [Government Spending](https://www.tradingview.com/support/solutions/43000651056) |
| GSS | [Grain Stocks Soy](https://www.tradingview.com/support/solutions/43000651058) |
| GSW | [Grain Stocks Wheat](https://www.tradingview.com/support/solutions/43000651059) |
| GTB | [Goods Trade Balance](https://www.tradingview.com/support/solutions/43000651046) |
| HB | [Hospital Beds](https://www.tradingview.com/support/solutions/43000651067) |
| HDG | [Households Debt To GDP](https://www.tradingview.com/support/solutions/43000651068) |
| HDI | [Households Debt To Income](https://www.tradingview.com/support/solutions/43000651069) |
| HICP | [Harmonised Index of Consumer Prices](https://www.tradingview.com/support/solutions/43000651062) |
| HIRMM | [Harmonised Inflation Rate, MoM](https://www.tradingview.com/support/solutions/43000679673) |
| HIRYY | [Harmonised Inflation Rate, YoY](https://www.tradingview.com/support/solutions/43000679674) |
| HMI | [NAHB Housing Market Index](https://www.tradingview.com/support/solutions/43000651132) |
| HOR | [Home Ownership Rate](https://www.tradingview.com/support/solutions/43000651065) |
| HOS | [Heating Oil Stocks](https://www.tradingview.com/support/solutions/43000651063) |
| HOSP | [Hospitals](https://www.tradingview.com/support/solutions/43000651066) |
| HPI | [House Price Index](https://www.tradingview.com/support/solutions/43000651071) |
| HPIMM | [House Price Index, MoM](https://www.tradingview.com/support/solutions/43000679678) |
| HPIYY | [House Price Index, YoY](https://www.tradingview.com/support/solutions/43000679679) |
| HS | [Home Loans](https://www.tradingview.com/support/solutions/43000651064) |
| HSP | [Household Spending](https://www.tradingview.com/support/solutions/43000651070) |
| HST | [Housing Starts](https://www.tradingview.com/support/solutions/43000651072) |
| IC | [Changes In Inventories](https://www.tradingview.com/support/solutions/43000650956) |
| ICUB | [ICU Beds](https://www.tradingview.com/support/solutions/43000651073) |
| IE | [Inflation Expectations](https://www.tradingview.com/support/solutions/43000651081) |
| IFOCC | [IFO Assessment Of The Business Situation](https://www.tradingview.com/support/solutions/43000651074) |
| IFOE | [IFO Business Developments Expectations](https://www.tradingview.com/support/solutions/43000651075) |
| IJC | [Initial Jobless Claims](https://www.tradingview.com/support/solutions/43000651084) |
| IMP | [Imports](https://www.tradingview.com/support/solutions/43000651076) |
| IMPYY | [Imports, YoY](https://www.tradingview.com/support/solutions/43000679681) |
| INBR | [Interbank Rate](https://www.tradingview.com/support/solutions/43000651085) |
| INTR | [Interest Rate](https://www.tradingview.com/support/solutions/43000651086) |
| IPA | [IP Addresses](https://www.tradingview.com/support/solutions/43000651088) |
| IPMM | [Industrial Production, MoM](https://www.tradingview.com/support/solutions/43000651078) |
| IPRI | [Import Prices](https://www.tradingview.com/support/solutions/43000651077) |
| IPYY | [Industrial Production, YoY](https://www.tradingview.com/support/solutions/43000651079) |
| IRMM | [Inflation Rate, MoM](https://www.tradingview.com/support/solutions/43000651082) |
| IRYY | [Inflation Rate, YoY](https://www.tradingview.com/support/solutions/43000651083) |
| IS | [Industrial Sentiment](https://www.tradingview.com/support/solutions/43000651080) |
| ISP | [Internet Speed](https://www.tradingview.com/support/solutions/43000651087) |
| JA | [Job Advertisements](https://www.tradingview.com/support/solutions/43000651091) |
| JAR | [Jobs To Applications Ratio](https://www.tradingview.com/support/solutions/43000651090) |
| JC | [Challenger Job Cuts](https://www.tradingview.com/support/solutions/43000650955) |
| JC4W | [Jobless Claims, 4-Week Average](https://www.tradingview.com/support/solutions/43000651089) |
| JO | [Job Offers](https://www.tradingview.com/support/solutions/43000651092) |
| JV | [Job Vacancies](https://www.tradingview.com/support/solutions/43000651093) |
| KFMI | [Kansas Fed Manufacturing Index](https://www.tradingview.com/support/solutions/43000651094) |
| LB | [Loans To Banks](https://www.tradingview.com/support/solutions/43000651104) |
| LC | [Labor Costs](https://www.tradingview.com/support/solutions/43000651101) |
| LEI | [Leading Economic Index](https://www.tradingview.com/support/solutions/43000651102) |
| LFPR | [Labor Force Participation Rate](https://www.tradingview.com/support/solutions/43000651100) |
| LG | [Loan Growth, YoY](https://www.tradingview.com/support/solutions/43000651106) |
| LIVRR | [Liquidity Injections Via Reverse Repo](https://www.tradingview.com/support/solutions/43000651103) |
| LMIC | [LMI Logistics Managers Index Current](https://www.tradingview.com/support/solutions/43000651096) |
| LMICI | [LMI Inventory Costs](https://www.tradingview.com/support/solutions/43000651095) |
| LMIF | [LMI Logistics Managers Index Future](https://www.tradingview.com/support/solutions/43000651097) |
| LMITP | [LMI Transportation Prices](https://www.tradingview.com/support/solutions/43000651098) |
| LMIWP | [LMI Warehouse Prices](https://www.tradingview.com/support/solutions/43000651099) |
| LPS | [Loans To Private Sector](https://www.tradingview.com/support/solutions/43000651105) |
| LR | [Central Bank Lending Rate](https://www.tradingview.com/support/solutions/43000650953) |
| LTUR | [Long Term Unemployment Rate](https://www.tradingview.com/support/solutions/43000651107) |
| LWF | [Living Wage Family](https://www.tradingview.com/support/solutions/43000679691) |
| LWI | [Living Wage Individual](https://www.tradingview.com/support/solutions/43000679702) |
| M0 | [Money Supply M0](https://www.tradingview.com/support/solutions/43000651125) |
| M1 | [Money Supply M1](https://www.tradingview.com/support/solutions/43000651126) |
| M2 | [Money Supply M2](https://www.tradingview.com/support/solutions/43000651127) |
| M3 | [Money Supply M3](https://www.tradingview.com/support/solutions/43000651128) |
| MA | [Mortgage Approvals](https://www.tradingview.com/support/solutions/43000651130) |
| MAPL | [Mortgage Applications](https://www.tradingview.com/support/solutions/43000651129) |
| MCE | [Michigan Consumer Expectations](https://www.tradingview.com/support/solutions/43000651119) |
| MCEC | [Michigan Current Economic Conditions](https://www.tradingview.com/support/solutions/43000651120) |
| MD | [Medical Doctors](https://www.tradingview.com/support/solutions/43000651117) |
| ME | [Military Expenditure](https://www.tradingview.com/support/solutions/43000651122) |
| MGDPYY | [Monthly GDP, YoY](https://www.tradingview.com/support/solutions/43000679714) |
| MIE1Y | [Michigan Inflation Expectations](https://www.tradingview.com/support/solutions/43000651121) |
| MIE5Y | [Michigan 5 Year Inflation Expectations](https://www.tradingview.com/support/solutions/43000651118) |
| MIP | [Mining Production, YoY](https://www.tradingview.com/support/solutions/43000651124) |
| MMI | [MBA Mortgage Market Index](https://www.tradingview.com/support/solutions/43000651108) |
| MO | [Machinery Orders](https://www.tradingview.com/support/solutions/43000651111) |
| MP | [Manufacturing Payrolls](https://www.tradingview.com/support/solutions/43000651113) |
| MPI | [Meat Price Index](https://www.tradingview.com/support/solutions/43000679666) |
| MPRMM | [Manufacturing Production, MoM](https://www.tradingview.com/support/solutions/43000651114) |
| MPRYY | [Manufacturing Production, YoY](https://www.tradingview.com/support/solutions/43000651115) |
| MR | [Mortgage Rate](https://www.tradingview.com/support/solutions/43000651131) |
| MRI | [MBA Mortgage Refinance Index](https://www.tradingview.com/support/solutions/43000651109) |
| MS | [Manufacturing Sales](https://www.tradingview.com/support/solutions/43000651116) |
| MTO | [Machine Tool Orders](https://www.tradingview.com/support/solutions/43000651112) |
| MW | [Minimum Wages](https://www.tradingview.com/support/solutions/43000651123) |
| NDCGOEA | [Orders For Non-defense Capital Goods Excluding Aircraft](https://www.tradingview.com/support/solutions/43000651148) |
| NEGTB | [Goods Trade Deficit With Non-EU Countries](https://www.tradingview.com/support/solutions/43000651047) |
| NFP | [Nonfarm Payrolls](https://www.tradingview.com/support/solutions/43000651141) |
| NGI | [Natural Gas Imports](https://www.tradingview.com/support/solutions/43000679719) |
| NGIR | [Natural Gas Imports from Russia](https://www.tradingview.com/support/solutions/43000679721) |
| NGSC | [Natural Gas Stocks Change](https://www.tradingview.com/support/solutions/43000651136) |
| NHPI | [Nationwide House Price Index](https://www.tradingview.com/support/solutions/43000651135) |
| NHS | [New Home Sales](https://www.tradingview.com/support/solutions/43000651137) |
| NHSMM | [New Home Sales, MoM](https://www.tradingview.com/support/solutions/43000651138) |
| NMPMI | [Non-Manufacturing PMI](https://www.tradingview.com/support/solutions/43000651143) |
| NO | [New Orders](https://www.tradingview.com/support/solutions/43000651139) |
| NODXMM | [Non-Oil Domestic Exports, MoM](https://www.tradingview.com/support/solutions/43000651144) |
| NODXYY | [Non-Oil Domestic Exports, YoY](https://www.tradingview.com/support/solutions/43000651145) |
| NOE | [Non-Oil Exports](https://www.tradingview.com/support/solutions/43000651142) |
| NPP | [Nonfarm Payrolls Private](https://www.tradingview.com/support/solutions/43000651140) |
| NURS | [Nurses](https://www.tradingview.com/support/solutions/43000651146) |
| NYESMI | [NY Empire State Manufacturing Index](https://www.tradingview.com/support/solutions/43000651134) |
| OE | [Oil Exports](https://www.tradingview.com/support/solutions/43000651147) |
| OPI | [Oils Price Index](https://www.tradingview.com/support/solutions/43000679665) |
| PCEPI | [PCE Price Index](https://www.tradingview.com/support/solutions/43000651149) |
| PDG | [Private Debt To GDP](https://www.tradingview.com/support/solutions/43000651160) |
| PFMI | [Philadelphia Fed Manufacturing Index](https://www.tradingview.com/support/solutions/43000651158) |
| PHSIMM | [Pending Home Sales Index, MoM](https://www.tradingview.com/support/solutions/43000651152) |
| PHSIYY | [Pending Home Sales Index, YoY](https://www.tradingview.com/support/solutions/43000651153) |
| PI | [Personal Income](https://www.tradingview.com/support/solutions/43000651155) |
| PIN | [Private Investment](https://www.tradingview.com/support/solutions/43000651161) |
| PIND | [MBA Purchase Index](https://www.tradingview.com/support/solutions/43000651110) |
| PITR | [Personal Income Tax Rate](https://www.tradingview.com/support/solutions/43000651154) |
| POP | [Population](https://www.tradingview.com/support/solutions/43000651159) |
| PPI | [Producer Price Index](https://www.tradingview.com/support/solutions/43000651165) |
| PPII | [Producer Price Index Input](https://www.tradingview.com/support/solutions/43000651164) |
| PPIMM | [Producer Price Inflation, MoM](https://www.tradingview.com/support/solutions/43000679724) |
| PPIYY | [Producer Prices Index, YoY](https://www.tradingview.com/support/solutions/43000651163) |
| PRI | [API Product Imports](https://www.tradingview.com/support/solutions/43000650925) |
| PROD | [Productivity](https://www.tradingview.com/support/solutions/43000651166) |
| PS | [Personal Savings](https://www.tradingview.com/support/solutions/43000651156) |
| PSC | [Private Sector Credit](https://www.tradingview.com/support/solutions/43000651162) |
| PSP | [Personal Spending](https://www.tradingview.com/support/solutions/43000651157) |
| PTE | [Part Time Employment](https://www.tradingview.com/support/solutions/43000651151) |
| PUAC | [Pandemic Unemployment Assistance Claims](https://www.tradingview.com/support/solutions/43000651150) |
| RAM | [Retirement Age Men](https://www.tradingview.com/support/solutions/43000651177) |
| RAW | [Retirement Age Women](https://www.tradingview.com/support/solutions/43000651178) |
| RCR | [Refinery Crude Runs](https://www.tradingview.com/support/solutions/43000651168) |
| REM | [Remittances](https://www.tradingview.com/support/solutions/43000651169) |
| RFMI | [Richmond Fed Manufacturing Index](https://www.tradingview.com/support/solutions/43000651181) |
| RFMSI | [Richmond Fed Manufacturing Shipments Index](https://www.tradingview.com/support/solutions/43000651182) |
| RFSI | [Richmond Fed Services Index](https://www.tradingview.com/support/solutions/43000651183) |
| RI | [Redbook Index](https://www.tradingview.com/support/solutions/43000651167) |
| RIEA | [Retail Inventories Excluding Autos](https://www.tradingview.com/support/solutions/43000651171) |
| RPI | [Retail Price Index](https://www.tradingview.com/support/solutions/43000651172) |
| RR | [Repo Rate](https://www.tradingview.com/support/solutions/43000651170) |
| RRR | [Reverse Repo Rate](https://www.tradingview.com/support/solutions/43000651180) |
| RSEA | [Retail Sales Excluding Autos](https://www.tradingview.com/support/solutions/43000651173) |
| RSEF | [Retail Sales Excluding Fuel](https://www.tradingview.com/support/solutions/43000651174) |
| RSMM | [Retail Sales, MoM](https://www.tradingview.com/support/solutions/43000651175) |
| RSYY | [Retail Sales, YoY](https://www.tradingview.com/support/solutions/43000651176) |
| RTI | [Reuters Tankan Index](https://www.tradingview.com/support/solutions/43000651179) |
| SBSI | [Small Business Sentiment Index](https://www.tradingview.com/support/solutions/43000651187) |
| SFHP | [Single Family Home Prices](https://www.tradingview.com/support/solutions/43000651186) |
| SP | [Steel Production](https://www.tradingview.com/support/solutions/43000651191) |
| SPI | [Sugar Price Index](https://www.tradingview.com/support/solutions/43000679563) |
| SS | [Services Sentiment](https://www.tradingview.com/support/solutions/43000651185) |
| SSR | [Social Security Rate](https://www.tradingview.com/support/solutions/43000651190) |
| SSRC | [Social Security Rate For Companies](https://www.tradingview.com/support/solutions/43000651188) |
| SSRE | [Social Security Rate For Employees](https://www.tradingview.com/support/solutions/43000651189) |
| STR | [Sales Tax Rate](https://www.tradingview.com/support/solutions/43000651184) |
| TA | [Tourist Arrivals](https://www.tradingview.com/support/solutions/43000651199) |
| TAXR | [Tax Revenue](https://www.tradingview.com/support/solutions/43000651192) |
| TCB | [Treasury Cash Balance](https://www.tradingview.com/support/solutions/43000651200) |
| TCPI | [Tokyo CPI](https://www.tradingview.com/support/solutions/43000651196) |
| TI | [Terrorism Index](https://www.tradingview.com/support/solutions/43000651194) |
| TII | [Tertiary Industry Index](https://www.tradingview.com/support/solutions/43000651195) |
| TOT | [Terms Of Trade](https://www.tradingview.com/support/solutions/43000651193) |
| TR | [Tourism Revenues](https://www.tradingview.com/support/solutions/43000651198) |
| TVS | [Total Vehicle Sales](https://www.tradingview.com/support/solutions/43000651197) |
| UC | [Unemployment Change](https://www.tradingview.com/support/solutions/43000651202) |
| UP | [Unemployed Persons](https://www.tradingview.com/support/solutions/43000651201) |
| UR | [Unemployment Rate](https://www.tradingview.com/support/solutions/43000651203) |
| WAG | [Wages](https://www.tradingview.com/support/solutions/43000651205) |
| WES | [Weapons Sales](https://www.tradingview.com/support/solutions/43000651207) |
| WG | [Wage Growth, YoY](https://www.tradingview.com/support/solutions/43000651206) |
| WHS | [Wages High Skilled](https://www.tradingview.com/support/solutions/43000679725) |
| WI | [Wholesale Inventories](https://www.tradingview.com/support/solutions/43000651208) |
| WLS | [Wages Low Skilled](https://www.tradingview.com/support/solutions/43000679727) |
| WM | [Wages In Manufacturing](https://www.tradingview.com/support/solutions/43000651204) |
| WPI | [Wholesale Price Index](https://www.tradingview.com/support/solutions/43000651209) |
| WS | [Wholesale Sales](https://www.tradingview.com/support/solutions/43000651210) |
| YUR | [Youth Unemployment Rate](https://www.tradingview.com/support/solutions/43000651211) |
| ZCC | [ZEW Current Conditions](https://www.tradingview.com/support/solutions/43000651212) |

## `request.footprint()` {#requestfootprint}

The [request.footprint()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.footprint) function enables scripts to retrieve [volume footprint](https://www.tradingview.com/support/solutions/43000726164-volume-footprint-charts-a-complete-guide/) data for the bars in the datasets on which they run. For a given bar, a volume footprint categorizes volume values from lower timeframes as “buy” (upward) or “sell” (downward) based on intrabar price action, then collects the categorized volume data into equally sized rows that cover the bar’s price range. Programmers can use retrieved footprint data to inspect the distribution of “buy”, “sell”, and total volume across the rows for a bar’s range, identify a bar’s Point of Control (POC) and other significant price levels, calculate volume delta information, detect volume imbalances, and more.

> [!NOTE]
> Volume footprints are available exclusively to users who have a Premium or Ultimate [plan](https://www.tradingview.com/pricing/). Accounts with lower-tier plans **cannot** use scripts that request volume footprint data with the [request.footprint()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.footprint) function.

The function’s signature is as follows:

```
request.footprint(ticks_per_row, va_percent, imbalance_percent) → series footprint
```

The `ticks_per_row` parameter specifies the size of each row in the calculated volume footprint, in ticks. It requires a positive “simple int” value representing a multiplier for the instrument’s minimum tick size. For example, if the argument is 100, the price range of each row equals the value of `100 * syminfo.mintick`. The specified row size affects the total number of rows in each bar’s footprint. Increase the value for fewer rows with a larger size, or decrease the value for the opposite.

The `va_percent` parameter accepts a “simple float” value specifying the percentage of the footprint’s total volume to use for calculating the bar’s _Value Area (VA)_, where a value of 100 represents 100% of the total volume. Specifying an argument is optional. The default value is 70, meaning that the footprint’s VA includes 70% of the total volume.

The `imbalance_percent` parameter accepts a “simple float” value specifying the required percentage difference between “buy” and “sell” volume in adjacent footprint rows for detecting _volume imbalances_:

-   The footprint considers a row to have a _buy imbalance_ if the row’s “buy” volume exceeds the “sell” volume of the row _below_ it by the specified percentage.
-   The footprint considers a row to have a _sell imbalance_ if the row’s “sell” volume exceeds the “buy” volume of the row _above_ it by the given percentage.

Including an `imbalance_percent` argument is optional. The default value is 300, meaning that the “buy” or “sell” volume of a footprint row must be three times (300%) larger than the opposing volume of an adjacent row to signify an imbalance.

A call to the [request.footprint()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.footprint) function returns either the _reference (ID)_ of a [footprint](https://www.tradingview.com/pine-script-reference/v6/#type_footprint) object that contains the volume footprint data for the current bar, or [na](https://www.tradingview.com/pine-script-reference/v6/#var_na) if no footprint is available for that bar.

> [!IMPORTANT]
> Scripts cannot perform more than **one** footprint request with the [request.footprint()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.footprint) function. If a script contains multiple calls to this function, it raises a _runtime error_.

Scripts can use any returned [footprint](https://www.tradingview.com/pine-script-reference/v6/#type_footprint) ID that is not [na](https://www.tradingview.com/pine-script-reference/v6/#var_na) in calls to the built-in `footprint.*()` functions to retrieve data from a bar’s volume footprint.

For example, the following script calls [request.footprint()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.footprint) on each bar to request the ID of a [footprint](https://www.tradingview.com/pine-script-reference/v6/#type_footprint) object that contains the bar’s volume footprint data. If the requested data is available, the script then uses the returned ID in calls to four `footprint.*()` functions — [footprint.total\_volume()](https://www.tradingview.com/pine-script-reference/v6/#fun_footprint.total_volume), [footprint.buy\_volume()](https://www.tradingview.com/pine-script-reference/v6/#fun_footprint.buy_volume), [footprint.sell\_volume()](https://www.tradingview.com/pine-script-reference/v6/#fun_footprint.sell_volume), and [footprint.delta()](https://www.tradingview.com/pine-script-reference/v6/#fun_footprint.delta) — to retrieve the footprint’s total volume, total “buy” and “sell” volume, and overall volume delta.

The script plots the “buy” volume, the negative “sell” volume, and the volume delta as columns for visual comparison. It also displays a color-coded [label](https://www.tradingview.com/pine-script-docs/visuals/text-and-shapes/#labels) at each bar’s high price to indicate whether the bar’s “buy” volume exceeds its “sell” volume or vice versa. Hovering over a label reveals a tooltip that shows the corresponding bar’s total volume and volume delta:

```pine
//@version=6
indicator("Requesting volume footprint data demo", max_labels_count = 500)

//@variable The number of ticks to use as the price interval for each footprint row.
int numTicksInput = input.int(100, "Ticks per footprint row", minval = 1)

//@variable References a `footprint` object for the current bar, or holds `na` if no footprint data is available.
footprint reqFootprint = request.footprint(numTicksInput)
//@variable Is `true` if the requested `footprint` ID is not `na`, and `false` otherwise.
bool hasFootprint = not na(reqFootprint)

// Retrieve the bar's total, "buy", and "sell" volume sums and overall volume delta from the `footprint` object.
float totalVolume = hasFootprint ? footprint.total_volume(reqFootprint) : na
float buyVolume   = hasFootprint ? footprint.buy_volume(reqFootprint)   : na
float sellVolume  = hasFootprint ? footprint.sell_volume(reqFootprint)  : na
float deltaVolume = hasFootprint ? footprint.delta(reqFootprint)        : na

// Plot the total "buy" and "sell" volume as bidirectional columns, where "buy" volume increases in the
// positive direction (upward plot), and "sell" volume increases in the negative direction (downward plot).
plot(buyVolume,       "Total buy volume",  #6eb21c, style = plot.style_columns)
plot(sellVolume * -1, "Total sell volume", #b21c2b, style = plot.style_columns)
// Plot bar's volume delta on top of the bidirectional columns to show the difference between "buy" and "sell" volume.
plot(deltaVolume, "Volume delta", #e8a93c, style = plot.style_columns)
hline(0, "Zero line", #d6d6d8, hline.style_solid)

// Draw a label at the bar's high. The label is green if the volume delta is positive or zero, and red otherwise.
// The label includes a tooltip that shows the bar's total volume and volume delta.
label.new(
    bar_index, high, color = deltaVolume >= 0 ? #6eb21c : #b21c2b, size = 14,
    tooltip = str.format("Total volume: \t{0}\nVolume delta: \t{1}", totalVolume, deltaVolume), force_overlay = true
)
```

Note that:

-   The `id` parameter of each `footprint.*()` function does not allow [na](https://www.tradingview.com/pine-script-reference/v6/#var_na) arguments. Therefore, this script uses [ternary operations](https://www.tradingview.com/pine-script-docs/language/operators/#-ternary-operator) that execute `footprint.*()` calls only if the retrieved [footprint](https://www.tradingview.com/pine-script-reference/v6/#type_footprint) ID is not [na](https://www.tradingview.com/pine-script-reference/v6/#var_na). If no data is available, the operations return [na](https://www.tradingview.com/pine-script-reference/v6/#var_na) directly without executing the calls.
-   On timeframes higher than or equal to “1D”, a footprint’s total volume might differ significantly from the value of the [volume](https://www.tradingview.com/pine-script-reference/v6/#var_volume) variable. Such differences occur for some instruments because _EOD_ data feeds can include data from block trades, OTC trades, and other sources, whereas requested _intraday_ data feeds do not. See the [Data feeds](https://www.tradingview.com/pine-script-docs/concepts/other-timeframes-and-data/#data-feeds) section to learn more about the types of data feeds and their differences.

While some of the `footprint.*()` functions retrieve values representing overall metrics for a requested volume footprint, as shown above, others retrieve the IDs of [volume\_row](https://www.tradingview.com/pine-script-reference/v6/#type_volume_row) objects that contain data for _individual rows_ from the footprint for more detailed analysis. For instance, the [footprint.poc()](https://www.tradingview.com/pine-script-reference/v6/#fun_footprint.poc) function retrieves the ID of the [volume\_row](https://www.tradingview.com/pine-script-reference/v6/#type_volume_row) object that contains data for a footprint’s _Point of Control_ row (i.e., the row with the highest total volume), and the [footprint.rows()](https://www.tradingview.com/pine-script-reference/v6/#fun_footprint.rows) function constructs an [array](https://www.tradingview.com/pine-script-docs/language/arrays/) containing the [volume\_row](https://www.tradingview.com/pine-script-reference/v6/#type_volume_row) IDs for _every_ row within a footprint.

Scripts can use non-na IDs of the [volume\_row](https://www.tradingview.com/pine-script-reference/v6/#type_volume_row) type in calls to the built-in `volume_row.*()` functions to retrieve data for a specific footprint row, including the row’s price levels, volume sums, volume delta, and buy or sell imbalances.

The advanced example below retrieves and displays detailed volume footprint information for visible chart bars. On each visible bar, the script requests a [footprint](https://www.tradingview.com/pine-script-reference/v6/#type_footprint) ID using [request.footprint()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.footprint). If the ID is not [na](https://www.tradingview.com/pine-script-reference/v6/#var_na), the script calls [footprint.rows()](https://www.tradingview.com/pine-script-reference/v6/#fun_footprint.rows) to create an array containing the [volume\_row](https://www.tradingview.com/pine-script-reference/v6/#type_volume_row) IDs for all rows in the footprint, and uses other `footprint.*()` calls to retrieve the individual IDs for the footprint’s POC and Value Area rows.

Afterward, the script loops through the array using a [`for...in` loop](https://www.tradingview.com/pine-script-docs/language/loops/#forin-loops). It calls multiple `volume_row.*()` functions within the loop to retrieve price levels, categorized volume values, volume delta, and imbalance states for each row. On each iteration, the script formats the retrieved “buy” and “sell” volume, volume delta, and imbalance information for the current row into a string, and then displays the text in a box drawn at the row’s price range in a separate pane. Each box uses a gradient background color based on the row’s volume delta and its total volume relative to the POC row’s total volume. The text color of each box is the chart’s foreground color if the row is the POC, purple if the row is a VA boundary, and gray otherwise.

The script also plots the retrieved POC levels and the VA boundaries as circles on the main chart pane for visual reference:

```pine
//@version=6
indicator("Retrieving footprint row data demo", max_boxes_count = 500)

//@variable The size of each footprint row, expressed in ticks.
int numTicksInput = input.int(100, "Ticks per footprint row", 1)
//@variable The percentage difference in opposing volume between rows for detecting volume imbalances.
int imbalanceInput = input.int(300, "Imbalance percentage", 1)

//@variable References a `footprint` object for the current bar, or holds `na` if no footprint data is available.
footprint reqFootprint = request.footprint(numTicksInput, imbalance_percent = imbalanceInput)

// Declare a tuple of variables to hold the values returned by the `if` structure for plotting.
// The values are not `na` only if the bar is visible and the `reqFootprint` variable does not hold `na`.
[pocHigh, pocLow, vaHigh, vaLow] = if (
    time >= chart.left_visible_bar_time and time <= chart.right_visible_bar_time and not na(reqFootprint)
)
    //@variable References an array containing a `volume_row` ID for each row in the volume footprint.
    array<volume_row> volumeRowsArray = reqFootprint.rows()

    // Retrieve `volume_row` IDs for the footprint's Point of Control, Value Area High, and Value Area Low rows.
    volume_row poc = reqFootprint.poc()
    volume_row vah = reqFootprint.vah()
    volume_row val = reqFootprint.val()

    // Loop through the array. The `row` variable holds a `volume_row` ID, starting with the one for the *lowest* row.
    for row in volumeRowsArray
        // Get the upper and lower price levels of the current row.
        float upPrice = row.up_price(), float dnPrice = row.down_price()
        // Get the row's "buy" and "sell" volume values and the volume delta.
        float buyVol = row.buy_volume(), float sellVol = row.sell_volume(), float delta = row.delta()
        // Get the row's buy and sell imbalance states.
        bool buyImbalance = row.has_buy_imbalance(), bool sellImbalance = row.has_sell_imbalance()

        //@variable A string representing the row's buy volume, sell volume, volume delta, and imbalances, respectively.
        string boxText = str.format(
            "B: {0} | S: {1} | D: {2} | I: {3}", str.tostring(buyVol, format.volume),
            str.tostring(sellVol, format.volume), str.tostring(delta, format.volume),
            buyImbalance and sellImbalance ? "Both" : buyImbalance ? "Buy" : sellImbalance ? "Sell" : "None"
        )

        // Calculate a green (for positive delta) or red (for negative delta) gradient color based on the row's volume 
        // relative to the POC volume.
        color deltaColor = delta >= 0 ? color.green : color.red
        color boxColor = color.from_gradient(
            row.total_volume() / poc.total_volume(),  0, 1, color.new(deltaColor, 100), color.new(deltaColor, 70)
        )
        // Draw a box at the price range of the row, in a separate pane, to display the `boxText` value.
        box rowBox = box.new(
            bar_index, upPrice, bar_index + 1, dnPrice, #787b8650, 1, text = boxText,
            text_color = #787b86, text_halign = text.align_left, bgcolor = boxColor
        )
        // Update the text color and formatting of the box if the current row is a Value Area boundary or the POC.
        if upPrice == vah.up_price() or upPrice == val.up_price()
            rowBox.set_text_color(color.purple)
            rowBox.set_text_formatting(text.format_bold)
        if upPrice == poc.up_price()
            rowBox.set_text_color(chart.fg_color)
            rowBox.set_text_formatting(text.format_bold)
    // Return the POC and VA levels for use in the global scope.
    [poc.up_price(), poc.down_price(), vah.up_price(), val.down_price()]

// Plot the retrieved POC and VA levels on the main chart pane.
plot(pocHigh, "POC top",    chart.fg_color,  5, plot.style_circles, force_overlay = true)
plot(pocLow,  "POC bottom", chart.fg_color,  5, plot.style_circles, force_overlay = true)
plot(vaHigh,  "VA top",     color.purple,  3, plot.style_circles, force_overlay = true)
plot(vaLow,   "VA bottom",  color.purple,  3, plot.style_circles, force_overlay = true)
```

Note that:

-   As with the built-in functions for most other [reference types](https://www.tradingview.com/pine-script-docs/language/type-system/#reference-types), scripts can call `footprint.*()` and `volume_row.*()` built-ins as functions or [methods](https://www.tradingview.com/pine-script-docs/language/methods/). This script calls the built-ins using _method syntax_.
-   The array created by [footprint.rows()](https://www.tradingview.com/pine-script-reference/v6/#fun_footprint.rows) sorts its elements in _ascending order_ by price level, where the first element refers to the [volume\_row](https://www.tradingview.com/pine-script-reference/v6/#type_volume_row) object for the row with the lowest prices, and the last refers to the one for the row with the highest prices.
-   The results of [volume\_row.has\_buy\_imbalance()](https://www.tradingview.com/pine-script-reference/v6/#fun_volume_row.has_buy_imbalance) and [volume\_row.has\_sell\_imbalance()](https://www.tradingview.com/pine-script-reference/v6/#fun_volume_row.has_sell_imbalance) calls depend on the `imbalance_percent` argument of the [request.footprint()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.footprint) call that creates the parent [footprint](https://www.tradingview.com/pine-script-reference/v6/#type_footprint) object. In this example, the “Imbalance percentage” input controls the argument, and therefore controls the behavior of the script’s `volume_row.has_*_imbalance()` calls.

To learn more about the [footprint](https://www.tradingview.com/pine-script-reference/v6/#type_footprint) and [volume\_row](https://www.tradingview.com/pine-script-reference/v6/#type_volume_row) types, and the available functions in their namespaces, refer to the [footprint and volume\_row](https://www.tradingview.com/pine-script-docs/language/type-system/#footprint-and-volume_row) section of the [Type system](https://www.tradingview.com/pine-script-docs/language/type-system/) page.

For more information about volume footprints and how they work, refer to the [Volume footprint charts](https://www.tradingview.com/support/solutions/43000726164-volume-footprint-charts-a-complete-guide/) article in our Help Center.

### Requesting footprints on other datasets {#requesting-footprints-on-other-datasets}

Scripts can request volume footprint data, or the results of calculations that use footprint data, from _other datasets_ by passing an expression that returns or uses IDs of the [footprint](https://www.tradingview.com/pine-script-reference/v6/#type_footprint) or [volume\_row](https://www.tradingview.com/pine-script-reference/v6/#type_volume_row) type as the `expression` argument in a [request.security()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.security) or [request.security\_lower\_tf()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.security_lower_tf) function call. If a call to either of these functions requests the result of an expression that relies on volume footprint data, it calculates footprints on the specified dataset rather than the script’s main dataset.

For example, the following script requests volume footprint data for the last _confirmed_ period on a specified _higher timeframe_, then uses the data to display [boxes](https://www.tradingview.com/pine-script-docs/visuals/lines-and-boxes/#boxes) across the current period for footprint rows whose total volume equals or exceeds an input percentage of the Point of Control (POC) row’s total volume.

The script uses a [request.security()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.security) call with `request.footprint(ticksInput)[1]` as the `expression` argument and [barmerge.lookahead\_on](https://www.tradingview.com/pine-script-reference/v6/#const_barmerge.lookahead_on) as the [`lookahead`](https://www.tradingview.com/pine-script-docs/concepts/other-timeframes-and-data/#lookahead) argument to fetch the last confirmed [footprint](https://www.tradingview.com/pine-script-reference/v6/#type_footprint) ID for the selected timeframe. If the result is not [na](https://www.tradingview.com/pine-script-reference/v6/#var_na) at the start of the current HTF period, the script compares each row’s volume to the volume of the POC row in a [for…in](https://www.tradingview.com/pine-script-reference/v6/#kw_for...in) loop, and draws boxes to project the levels of rows whose volume meets the specified threshold. Each box uses a volume-based gradient color, where blue hues correspond to rows with lower volume, and orange hues correspond to rows with higher volume:

```pine
//@version=6
indicator("Requesting footprints on other datasets demo", overlay = true, behind_chart = false, max_boxes_count = 500)

//@variable The higher timeframe for which to request footprint data.
string htfInput = input.timeframe("1D", "Higher timeframe")
//@variable The size of each footprint row, expressed in ticks. 
int ticksInput = input.int(100, "Ticks per row", minval = 1)
//@variable The minimum percentage of POC volume required to qualify a footprint row as a high-volume node.
float hvInput = input.float(50.0, "High-volume threshold (% of POC)", 0.0, 100.0)

// Raise an error if the specified timeframe is not higher than the chart's timeframe.
if barstate.isfirst and timeframe.in_seconds(htfInput) <= timeframe.in_seconds()
    runtime.error("The requested timeframe must be higher than the chart's timeframe.")

//@variable The ID of the *last confirmed* `footprint` object from the higher TF, or `na` when new data is not available.
//          The `request.security()` call includes `barmerge.gaps_on`, because the script requires the data only on the
//          *first bar* of each new HTF period.
footprint htfFootprint = request.security(
    syminfo.tickerid, htfInput, request.footprint(ticksInput)[1], barmerge.gaps_on, barmerge.lookahead_on
)

if not na(htfFootprint)
    //@variable References the `volume_row` object for the footprint's POC row.
    volume_row pocRow = htfFootprint.poc()
    //@variable The POC row's total volume.
    float pocVol = pocRow.total_volume()
    //@variable The minimum volume required for a high-volume row.
    float minVol = pocVol * hvInput / 100
    // Loop through all `volume_row` IDs stored in the array created by `htfFootprint.rows()`.
    for row in htfFootprint.rows()
        //@variable The current row's total volume.
        float rowVol = row.total_volume()
        // Skip the rest of the current iteration if the row's total volume is less than the required volume.
        if rowVol < minVol
            continue
        // Draw a box across the current period at the confirmed row's price levels if its volume meets the threshold,
        // and color the box using a volume-based gradient. 
        color boxColor = color.from_gradient(rowVol, minVol, pocVol, #2196f3, #ff9800)
        box.new(
            time, row.up_price(), time_close(htfInput), row.down_price(), boxColor,
            xloc = xloc.bar_time, bgcolor = color.new(boxColor, 50), 
            text = str.tostring(rowVol, format.volume), text_color = #000000
        )
```

Note that:

-   The [request.security()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.security) call includes [barmerge.gaps\_on](https://www.tradingview.com/pine-script-reference/v6/#const_barmerge.gaps_on) as the [`gaps`](https://www.tradingview.com/pine-script-docs/concepts/other-timeframes-and-data/#gaps) argument to return a non-na [footprint](https://www.tradingview.com/pine-script-reference/v6/#type_footprint) ID only on the _first_ chart bar of each new HTF period, because the script does not require new footprint data on _every_ bar. 
-   The script calls the [time()](https://www.tradingview.com/pine-script-reference/v6/#fun_time) and [time\_close()](https://www.tradingview.com/pine-script-reference/v6/#fun_time_close) functions to retrieve the expected opening and closing timestamps for the current HTF period. The box drawings use these timestamps as their left and right coordinates.

Any [request.security()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.security) or [request.security\_lower\_tf()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.security_lower_tf) call whose expression depends on [footprint](https://www.tradingview.com/pine-script-reference/v6/#type_footprint) objects counts toward the total number of [request.footprint()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.footprint) calls, because such requests _copy_ all code required to calculate the result on the retrieved dataset. A script raises a _runtime error_ if its outputs depend on data from more than **one** unique [request.footprint()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.footprint) call, regardless of whether those requests execute on the script’s main dataset or on the datasets fetched by other `request.*()` calls.

Consider the following script, which defines a calculation that requests volume footprint data for the current dataset, computes a moving average of the POC row’s median price, then declares a `pocMA` variable to store the average. The script uses the variable as the `expression` argument in a [request.security()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.security) call to calculate the average POC level on the “1D” timeframe, then assigns the result to the `requestedMA` variable. Lastly, the script attempts to plot both the `pocMA` and `requestedMA` series on the chart, but fails and raises a _runtime error_:

```pine
//@version=6
indicator("Too many footprint requests demo")

//@variable The size of each footprint row, expressed in ticks.
int ticksInput = input.int(100, "Ticks per row", 1)

//#region
// The code in this region is copied into the `request.security()` call's context, because all of it affects 
// the `pocMA` variable used as the call's `expression` argument.

// Request the ID of a `footprint` object.
footprint fp = request.footprint(ticksInput)
// Get the `volume_row` ID for the POC row if the requested `footprint` ID is not `na`.
volume_row poc = na(fp) ? na : fp.poc()
// Calculate a moving average of the POC row's midpoint.
float pocMA = ta.sma(na(poc) ? na : math.avg(poc.up_price(), poc.down_price()), 5)
//#endregion

// Retrieve the `pocMA` value calculated on the "1D" timeframe.
// This request copies all code in the region above and evaluates it on the requested dataset.
// Therefore, it executes its own separate version of the `request.footprint()` call. 
float requestedMA = request.security(syminfo.tickerid, "1D", pocMA)

// Using *both* the `pocMA` and `requestedMA` variables in the script's outputs causes a *runtime error*, 
// because the variables' values depend on *two* different footprint requests: one for the chart's timeframe, 
// and the other for the requested timeframe.
plot(requestedMA, "Daily POC MA", color.purple, 3)
plot(pocMA,       "Chart POC MA", color.blue,   3)
```

The above script raises an error because its _outputs_ (in this case, its [plots](https://www.tradingview.com/pine-script-docs/visuals/plots/)) depend on both the `pocMA` and `requestedMA` variables, and the values of those variables depend on **two** separate footprint requests — one for the script’s main dataset and the other for the dataset retrieved by the [request.security()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.security) call. _Both_ of these requests count toward the total number of [request.footprint()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.footprint) calls, even though one of those calls is not defined _explicitly_ in the code.

A simple way to resolve this error is to _remove_ one of the script’s two [plot()](https://www.tradingview.com/pine-script-reference/v6/#fun_plot) calls. As explained in the [Compiled tokens](https://www.tradingview.com/pine-script-docs/writing/limitations/#compiled-tokens) section of the [Limitations](https://www.tradingview.com/pine-script-docs/writing/limitations/) page, the Pine Script compiler automatically _discards_ code that a script’s _outputs_ do not depend on, including `request.*()` calls. Therefore, if we remove the plot of the `pocMA` series, for example, the [request.footprint()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.footprint) call defined explicitly in the global scope _does not_ execute. Instead, only the [request.footprint()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.footprint) call copied into the [request.security()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.security) call’s context executes, because that is the only one that the script’s outputs now require:

```pine
//@version=6
indicator("Removed footprint request demo")

//@variable The size of each footprint row, expressed in ticks.
int ticksInput = input.int(100, "Ticks per row", 1)

//#region
// Although this code is still defined in the global scope, it does *not* execute in this scope, because the script's 
// outputs no longer depend on the data assigned to the `fp`, `poc`, or `pocMA` variables now that we removed the 
// `plot(pocMA)` call.

footprint  fp    = request.footprint(ticksInput)
volume_row poc   = na(fp) ? na : fp.poc()
float      pocMA = ta.sma(na(poc) ? na : math.avg(poc.up_price(), poc.down_price()), 5)
//#endregion

// This request still copies all code in the region above to calculate a POC average on the "1D" timeframe. 
// The compiler does not discard this request, including the `request.footprint()` call that executes in its context, 
// because the script still uses the `requestedMA` variable in its outputs.
float requestedMA = request.security(syminfo.tickerid, "1D", pocMA)

// If we plot only the `requestedMA` series, and *not* the `pocMA` series, the "Too many `request.footprint()` calls" 
// error no longer occurs, because the script no longer requires two separate footprint requests to determine its outputs.
// The only footprint request that it requires now is the one copied into the `request.security()` call's context.
plot(requestedMA, "Daily POC MA", color.purple, 3)
```

Note that:

-   Although the `pocMA` variable declaration and its dependencies no longer execute directly, the [Pine Profiler](https://www.tradingview.com/pine-script-docs/writing/profiling-and-optimization/#pine-profiler) displays performance details next to that code in the Pine Editor. Those details represent the performance of the _copied_ calculations that execute within the [request.security()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.security) call’s context in this case, as the profiler cannot display that information elsewhere. See the [When requesting other contexts](https://www.tradingview.com/pine-script-docs/writing/profiling-and-optimization/#when-requesting-other-contexts) section of the [Profiling and optimization](https://www.tradingview.com/pine-script-docs/writing/profiling-and-optimization/) page to learn more about this behavior.

## `request.seed()` {#requestseed}

TradingView aggregates a vast amount of data from its many providers, including price and volume information on tradable instruments, financials, economic data, and more, which users can retrieve in Pine Script using the functions discussed in the sections above, as well as multiple built-in variables.

To further expand the horizons of possible data one can analyze on TradingView, we have [Pine Seeds](https://github.com/tradingview-pine-seeds/docs), which allows users to supply custom _user-maintained_ EOD data feeds via GitHub for use on TradingView charts and within Pine Script code.

> [!IMPORTANT]
> The creation of _new_ Pine Seeds repositories is currently **unavailable**. However, the data feeds from _existing_ repositories are still accessible to charts and scripts. The [Pine Seeds documentation](https://github.com/tradingview-pine-seeds/docs/blob/main/README.md) on GitHub provides in-depth information about Pine Seeds functionality and instructions for requesting the return of full Pine Seeds support.

To retrieve data from a Pine Seeds data feed within a script, use the [request.seed()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.seed) function. Below is the function’s signature:

```
request.seed(source, symbol, expression, ignore_invalid_symbol, calc_bars_count) → series <type>
```

The `source` parameter specifies the unique name of the user-maintained GitHub repository that contains the data feed.

The `symbol` parameter represents the file name from the “data/” directory of the `source` repository, excluding the “.csv” file extension. See [this page](https://github.com/tradingview-pine-seeds/docs/blob/main/data.md) for information about the structure of the data stored in repositories.

The `expression` parameter is the series to evaluate using data extracted from the requested context. It is similar to the equivalent in [request.security()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.security) and [request.security\_lower\_tf()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.security_lower_tf). Data feeds stored in user-maintained repos contain [time](https://www.tradingview.com/pine-script-reference/v6/#var_time), [open](https://www.tradingview.com/pine-script-reference/v6/#var_open), [high](https://www.tradingview.com/pine-script-reference/v6/#var_high), [low](https://www.tradingview.com/pine-script-reference/v6/#var_low), [close](https://www.tradingview.com/pine-script-reference/v6/#var_close), and [volume](https://www.tradingview.com/pine-script-reference/v6/#var_volume) information, meaning the `expression` argument can use the corresponding built-in variables, including variables derived from them (e.g., [bar\_index](https://www.tradingview.com/pine-script-reference/v6/#var_bar_index), [ohlc4](https://www.tradingview.com/pine-script-reference/v6/#var_ohlc4), etc.) to request their values from the context of the custom data.

The script below visualizes sample data from the [seed\_crypto\_santiment](https://github.com/tradingview-pine-seeds/seed_crypto_santiment) demo repository. It uses two calls to [request.seed()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.seed) to retrieve the [close](https://www.tradingview.com/pine-script-reference/v6/#var_close) values from the repository’s [BTC\_SENTIMENT\_POSITIVE\_TOTAL](https://github.com/tradingview-pine-seeds/seed_crypto_santiment/blob/master/data/BTC_SENTIMENT_POSITIVE_TOTAL.csv) and [BTC\_SENTIMENT\_NEGATIVE\_TOTAL](https://github.com/tradingview-pine-seeds/seed_crypto_santiment/blob/master/data/BTC_SENTIMENT_NEGATIVE_TOTAL.csv) data feeds and [plots](https://www.tradingview.com/pine-script-docs/visuals/plots/) the results on the chart as step lines:

```pine
//@version=6
indicator("Pine Seeds demo", format=format.volume)

//@variable The total positive sentiment for BTC extracted from the "seed_crypto_santiment" repository.
float positiveTotal = request.seed("seed_crypto_santiment", "BTC_SENTIMENT_POSITIVE_TOTAL", close)
//@variable The total negative sentiment for BTC extracted from the "seed_crypto_santiment" repository.
float negativeTotal = request.seed("seed_crypto_santiment", "BTC_SENTIMENT_NEGATIVE_TOTAL", close)

// Plot the data.
plot(positiveTotal, "Positive sentiment", color.teal, 2, plot.style_stepline)
plot(negativeTotal, "Negative sentiment", color.maroon, 2, plot.style_stepline)
```

Note that:

-   This example requests data from the repository highlighted in the [Pine Seeds documentation](https://github.com/tradingview-pine-seeds/docs/blob/main/README.md). It exists solely for example purposes, and its data _does not_ update on a regular basis.
-   Unlike most other `request.*()` functions, [request.seed()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.seed) does not have a `gaps` parameter. It always returns [na](https://www.tradingview.com/pine-script-reference/v6/#var_na) values when no new data exists.
-   Pine Seeds data is searchable from the chart’s symbol search bar. To load a data feed on the chart, enter the _“Repo:File” pair_, similar to searching for an “Exchange:Symbol” pair.
