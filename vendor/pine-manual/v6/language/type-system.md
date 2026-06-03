---
title: Type system
source: https://www.tradingview.com/pine-script-docs/language/type-system/
section: language
---

# Type system {#type-system}

## Introduction {#introduction}

Pine Script® uses a system of _types_ and _type qualifiers_ to categorize the data in a script and indicate where and how the script can use it. This system applies to all values and references in a script, and to the variables, function parameters, and fields that store them.

[Types](https://www.tradingview.com/pine-script-docs/language/type-system/#types) in Pine Script indicate the kinds of information that a script’s data represents. Some types directly represent _values_, such as numbers, logical conditions, colors, or text, while others define _structures_ for special tasks, such as displaying [visuals](https://www.tradingview.com/pine-script-docs/visuals/overview/) on the chart. [Qualifiers](https://www.tradingview.com/pine-script-docs/language/type-system/#qualifiers) indicate when the values of any given type are accessible, and whether those values can change across script executions.

The combination of a type and a qualifier forms a _qualified type_, which determines the operations and functions with which a value or reference is compatible.

> [!NOTE]
> For the sake of brevity, we often use the term “type” when referring to a qualified type.

The type system closely connects to the [execution model](https://www.tradingview.com/pine-script-docs/language/execution-model/) and its [time series](https://www.tradingview.com/pine-script-docs/language/execution-model/#time-series) structure — together, they determine how a script behaves as it runs on a dataset. Although it’s possible to write simple scripts without understanding these foundational topics, learning about them and their nuances is key to mastering Pine Script.

## Qualifiers {#qualifiers}

Pine’s type qualifiers ([const](https://www.tradingview.com/pine-script-docs/language/type-system/#const), [input](https://www.tradingview.com/pine-script-docs/language/type-system/#input), [simple](https://www.tradingview.com/pine-script-docs/language/type-system/#simple), and [series](https://www.tradingview.com/pine-script-docs/language/type-system/#series)) indicate _when_ values in a script are accessible — either at compile time, input time, or runtime — and whether those values can change across script executions:

`"const"`

Established at _compile time_, when the user saves the script in the Pine Editor or applies the script to a dataset. Values qualified as “const” remain _constant_ during every script execution.

`"input"`

Established at _input time_, when the system confirms input data from the script’s “Settings/Inputs” tab or the chart. Similar to “const” values, “input” values _do not change_ as the script runs on the dataset.

`"simple"`

Established by the script at runtime, on the _first_ available bar. On all subsequent bars, values qualified as “simple” do not change.

`"series"`

_Dynamic_. Values qualified as “series” are available at runtime, and they are the **only** values that can change across bars.

> [!NOTE]
> The “const”, “input”, and “simple” qualifiers apply exclusively to [value types](https://www.tradingview.com/pine-script-docs/language/type-system/#value-types). Pine’s [reference types](https://www.tradingview.com/pine-script-docs/language/type-system/#reference-types), such as [label](https://www.tradingview.com/pine-script-reference/v6/#type_label) and [array](https://www.tradingview.com/pine-script-reference/v6/#type_array) types, automatically inherit the “series” qualifier.

Pine Script uses the following _qualifier hierarchy_ to determine the compatibility of values in a script’s calculations:

```
const < input < simple < series
```

In this hierarchy, “const” is the _lowest_ (_weakest_) qualifier, and “series” is the _highest_ (_strongest_). Any variable, parameter, or operation that accepts a value with a specific qualifier also allows a value of the same type with a _weaker_ qualifier, but **not** one that is stronger.

For instance, a function parameter that accepts a value of the “simple int” qualified type also allows a value of the “input int” or “const int” type, because “const” and “input” are _lower_ than “simple” in the qualifier hierarchy. However, the parameter _cannot_ accept a “series int” value, because “series” is higher in the hierarchy than “simple”.

Pine also uses this hierarchy to determine the qualifiers assigned to the results of expressions, i.e., function calls and operations. The returned types of an expression always inherit the _strongest_ qualifier in the calculation. For example, an expression that performs a calculation using “input” and “simple” values returns “simple” results, because “simple” is a _stronger_ qualifier than “input”.

Note that a script **cannot** change the qualifier of a returned value to one that is lower in the hierarchy to make it compatible with specific operations or functions. For instance, if a calculation returns a value qualified as “series”, the script cannot modify that value’s qualifier later to enable using it in code that requires “simple” or “const” values.

The following sections explain the behavior of each type qualifier, as well as the built-in keywords that programmers can use to specify qualifiers in their code.

### const {#const}

Values qualified as “const” are available at _compile time_, before the script starts its first execution. Compilation occurs when the user saves the script in the Pine Editor, and immediately before a script starts to run on the chart or in another location. Values with the “const” qualifier remain constant after compilation; they do not change during any script execution. All _literal values_ and the results of expressions that use only values qualified as “const” automatically inherit the “const” qualifier.

The following list shows a few values of each [fundamental type](https://www.tradingview.com/pine-script-docs/language/type-system/#types). All of these represent literal values if a script includes them directly in its source code:

-   _literal int_: `1`, `-1`, `42`
-   _literal float_: `1.`, `1.0`, `3.14`, `6.02E-23`, `3e8`
-   _literal bool_: `true`, `false`
-   _literal color_: `#FF55C6`, `#FF55C6ff`
-   _literal string_: `"A text literal"`, `"Embedded single quotes 'text'"`, `'Embedded double quotes "text"'`

Scripts can [declare variables](https://www.tradingview.com/pine-script-docs/language/variable-declarations/) that hold “const” values, and use those variables to calculate other constants. In the example below, we use “const” variables to set the title of a script and its plots. The script compiles successfully, because the [indicator()](https://www.tradingview.com/pine-script-reference/v6/#fun_indicator) and [plot()](https://www.tradingview.com/pine-script-reference/v6/#fun_plot) calls used in the code both require a `title` argument of the _“const string”_ qualified type:

```pine
//@version=6

// All of the following global variables automatically inherit the "const" qualifier,
// because their "string" values are constants that are available at *compile time*.

//@variable The indicator's title.
INDICATOR_TITLE = "const demo"
//@variable The title of the first plot.
var PLOT1_TITLE = "High"
//@variable The title of the second plot.
PLOT2_TITLE = "Low"
//@variable The title of the third plot.
PLOT3_TITLE = "Midpoint between " + PLOT1_TITLE + " and " + PLOT2_TITLE

// Set the title of the indicator using `INDICATOR_TITLE`.
indicator(title = INDICATOR_TITLE, overlay = true)

// Set the title of each plot using the `PLOT*_TITLE` variables.
plot(high, PLOT1_TITLE)
plot(low, PLOT2_TITLE)
plot(hl2, PLOT3_TITLE)
```

Note that:

-   All the variables above the [indicator()](https://www.tradingview.com/pine-script-reference/v6/#fun_indicator) call in this script have the “const” qualifier, because they hold a literal value or the result of [operations](https://www.tradingview.com/pine-script-docs/language/operators/) that use only “const” values.
-   All our “const” variables in this example have names in _uppercase snake case_ so that they are easy to distinguish in the code, as recommended by our [Style guide](https://www.tradingview.com/pine-script-docs/writing/style-guide/).
-   Although the “const” variables in this script hold constant values, the script initializes them on _every bar_. The only exception is `PLOT1_TITLE`, which the script initializes only on the _first_ bar, because its declaration includes the [var](https://www.tradingview.com/pine-script-reference/v6/#kw_var) keyword. See the [Declaration modes](https://www.tradingview.com/pine-script-docs/language/variable-declarations/#declaration-modes) section of the [Variable declarations](https://www.tradingview.com/pine-script-docs/language/variable-declarations/) page to learn more.

Any variable or function parameter that requires a “const” value _cannot_ accept a value with the “input”, “simple”, or “series” qualifier, because “const” is the _lowest_ qualifier in the [qualifier hierarchy](https://www.tradingview.com/pine-script-docs/language/type-system/#qualifiers).

For example, the following script combines a literal string with the value of [syminfo.ticker](https://www.tradingview.com/pine-script-reference/v6/#var_syminfo.ticker) to set the value of a `scriptTitle` variable. Then, it attempts to use the variable as the `title` argument of the [indicator()](https://www.tradingview.com/pine-script-reference/v6/#fun_indicator) declaration statement, causing a _compilation error_. The `title` parameter requires a “const string” argument, but `scriptTitle` holds a value of the type _“simple string”_:

```pine
//@version=6

//@variable Holds a value intended for use as the `title` argument in the `indicator()` call.
//          However, this variable's type is "simple string", not "const string", because 
//          the value of `syminfo.ticker` is not available until *after* compilation.
var scriptTitle = "My indicator for " + syminfo.ticker

// This statement causes an error. The `indicator()` statement cannot use a "simple string"
// value as its `title` argument. It requires a "const string" value.
indicator(title = scriptTitle)

plot(close - open)
```

Note that:

-   The [syminfo.ticker](https://www.tradingview.com/pine-script-reference/v6/#var_syminfo.ticker) variable holds a “simple string” value because it depends on data that is available only at _runtime_. Combining this value with a literal string produces another “simple string” value, because “simple” is a stronger qualifier than “const”.
-   We did not name the `scriptTitle` variable using snake case, because our [Style guide](https://www.tradingview.com/pine-script-docs/writing/style-guide/) recommends using _lower camel case_ to name variables that do not hold “const” values.

Programmers can restrict the behavior of a variable and force _constant assignments_ on each execution by prefixing its declaration with the [const](https://www.tradingview.com/pine-script-reference/v6/#type_const) keyword, followed by a [type keyword](https://www.tradingview.com/pine-script-docs/language/type-system/#types) or type identifier. If a variable includes [const](https://www.tradingview.com/pine-script-reference/v6/#type_const) in its declaration, the script cannot _change_ its value with the reassignment or compound assignment [operators](https://www.tradingview.com/pine-script-docs/language/operators/). This restriction applies even if the new assigned value is still a constant.

For example, the script below declares a `myVar` variable using the [const](https://www.tradingview.com/pine-script-reference/v6/#type_const) keyword. Then, it attempts to change the variable’s value with the [+=](https://www.tradingview.com/pine-script-reference/v6/#op_+=) operator, causing a compilation error:

```pine
//@version=6
indicator("Cannot reassign `const` variable demo")

//@variable A "float" variable declared using the `const` keyword.
const float myVar = 0.0

// Using the `+=` operator on `myVar` causes an error, because scripts *cannot* modify variables declared using `const`.
myVar += 1.0

plot(myVar)
```

For a variable of any [value type](https://www.tradingview.com/pine-script-docs/language/type-system/#value-types), applying the [const](https://www.tradingview.com/pine-script-reference/v6/#type_const) keyword to the declaration prevents the script from assigning a value qualified as “input”, “simple”, or “series”. Likewise, if a [user-defined function](https://www.tradingview.com/pine-script-docs/language/user-defined-functions/) parameter of these types includes the keyword in its declaration, it accepts only “const” values.

> [!NOTE]
> Using the [const](https://www.tradingview.com/pine-script-reference/v6/#type_const) keyword in a script is usually optional. However, the keyword is required when defining _exported variables_ in [libraries](https://www.tradingview.com/pine-script-docs/concepts/libraries/).

The following script attempts to use the value of the [close](https://www.tradingview.com/pine-script-reference/v6/#var_close) variable as the initial value of a `myVar` variable declared using the [const](https://www.tradingview.com/pine-script-reference/v6/#type_const) keyword. However, [close](https://www.tradingview.com/pine-script-reference/v6/#var_close) is _not compatible_ with the variable, so a compilation error occurs. The value of [close](https://www.tradingview.com/pine-script-reference/v6/#var_close) is of the type _“series float”_, because it updates from bar to bar, but the `myVar` variable requires a “const float” value:

```pine
//@version=6
indicator("Cannot assign values with stronger qualifiers demo")

// This declaration causes an error. The value of `close` is of the type "series float", 
// but `myVar` accepts only a "const float" value. 
const float myVar = close

plot(myVar)
```

Note that:

-   If we remove the [const](https://www.tradingview.com/pine-script-reference/v6/#type_const) keyword from the variable declaration, the `myVar` variable automatically inherits the “series” qualifier, and no error occurs.

> [!NOTE]
> Scripts can also use the [const](https://www.tradingview.com/pine-script-reference/v6/#type_const) keyword when declaring variables of most [special types](https://www.tradingview.com/pine-script-docs/language/type-system/#types), such as [line](https://www.tradingview.com/pine-script-reference/v6/#type_line), [label](https://www.tradingview.com/pine-script-reference/v6/#type_label), and [array](https://www.tradingview.com/pine-script-reference/v6/#type_array) types. However, this keyword **does not** set the _qualifier_ of these variables; it only prevents the script from changing the variable’s assigned _reference (ID)_ during each execution. Special types and other reference types _always_ inherit the “series” qualifier. See the [Value vs. reference types](https://www.tradingview.com/pine-script-docs/language/type-system/#value-vs-reference-types) section for advanced details.

### input {#input}

Values qualified as “input” are established at _input time_. They are similar to “const” values, because they are available before the first script execution and never change during runtime. However, unlike “const” values, “input” values depend on user input.

All function parameters that have the “input” qualifier can accept only “input” or “const” values; they do not allow values qualified as “simple” or “series”.

Most of the built-in `input.*()` functions return values qualified as “input”. These functions create adjustable [inputs](https://www.tradingview.com/pine-script-docs/concepts/inputs/) in the script’s “Settings/Inputs” tab, enabling users to change specific values in a script without altering its source code. Each time the user changes the value of an input, the script _reloads_ across all bars on the chart — from the first available bar to the most recent bar — to update its results using the specified value, as explained in the [Execution model](https://www.tradingview.com/pine-script-docs/language/execution-model/) page.

> [!NOTE]
> The only `input*()` function that does _not_ return a value qualified as “input” is [input.source()](https://www.tradingview.com/pine-script-reference/v6/#fun_input.source). That function returns the value of a built-in price series, such as [close](https://www.tradingview.com/pine-script-reference/v6/#var_close), or a value from another script’s [plots](https://www.tradingview.com/pine-script-docs/visuals/plots/). Therefore, its return type is _“series float”_, which is not compatible with code that requires “input float” values. See the [Source input](https://www.tradingview.com/pine-script-docs/concepts/inputs/#source-input) section of the [Inputs](https://www.tradingview.com/pine-script-docs/concepts/inputs/) page to learn more.

The following script requests the value of an [RSI](https://www.tradingview.com/support/solutions/43000502338-relative-strength-index-rsi/) calculated on the dataset for a specific symbol and timeframe, and then plots the result on the chart as columns. The script includes two [string inputs](https://www.tradingview.com/pine-script-docs/concepts/inputs/#string-input) that specify the context of the request, and it uses a [float input](https://www.tradingview.com/pine-script-docs/concepts/inputs/#float-input) value to set the base of the plotted columns. If the user changes any of these inputs in the “Settings/Inputs” tab, the script reloads to update its results for every bar:

```pine
//@version=6
indicator("'input' values demo")

//@variable An "input string" value representing the requested symbol.
symbolInput = input.string("AAPL", "Symbol")
//@variable An "input string" value representing the timeframe of the requested data.
timeframeInput = input.string("1D", "Timeframe")
//@variable An "input float" value for specifying the base of the plotted columns.
colBaseInput = input.float(50.0, "Column base", 0.0, 100.0)

//@variable The value of an RSI calculated on the `symbolInput` symbol and `timeframeInput` timeframe.
//          The `request.security()` function's `symbol` and `timeframe` parameters accept "series string" values, 
//          so they also allow weaker qualified types such as "input string".
requestedRSI = request.security(symbol = symbolInput, timeframe = timeframeInput, expression = ta.rsi(close, 14))

// Plot the `requestedRSI` value as columns with the base defined by `colBaseInput`. 
// This call works, because `histbase` requires an "int" or "float" value with the "const" or "input" qualifier.
plot(requestedRSI, "RSI", color.purple, 1, plot.style_columns, histbase = colBaseInput)
```

Note that:

-   The [plot()](https://www.tradingview.com/pine-script-reference/v6/#fun_plot) function’s `histbase` parameter, which sets the base of the plotted columns, has the expected type “input int” or “input float”. As such, it cannot accept “simple int/float” or “series int/float” values, because “simple” and “series” are stronger qualifiers than “input”.
-   The [request.security()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.security) function requests data from a specified dataset. Its `symbol` and `timeframe` parameters, which define the context of the request, accept “series string” values by default. Therefore, these parameters also accept “input string” values. See the [Other timeframes and data](https://www.tradingview.com/pine-script-docs/concepts/other-timeframes-and-data/) page to learn more about `request.*()` functions.

Some built-in `chart.*` variables also hold “input” values, because these variables update at input time based on changes to the _chart_. Scripts that use these variables reload, executing across the entire dataset again, if any chart changes affect their values.

The example below uses some of these variables to display a gradient background color that incrementally changes over the chart’s visible bars. It uses [chart.left\_visible\_bar\_time](https://www.tradingview.com/pine-script-reference/v6/#var_chart.left_visible_bar_time) and [chart.right\_visible\_bar\_time](https://www.tradingview.com/pine-script-reference/v6/#var_chart.right_visible_bar_time) to get the timestamps of the leftmost and rightmost visible bars for its calculation, and it uses [chart.bg\_color](https://www.tradingview.com/pine-script-reference/v6/#var_chart.bg_color) and [chart.fg\_color](https://www.tradingview.com/pine-script-reference/v6/#var_chart.fg_color) to define the start and end colors of the gradient. If the user scrolls or zooms on the chart, or changes the chart’s background color, the script reloads to generate new results:

```pine
//@version=6
indicator("Built-in 'input' variables demo")

//@variable The difference between `time` and the leftmost visible bar's time, relative to the visible range.
//          The `chart.*` variables in this calculation depend on input data from the visible chart, so their type is
//          "input int".
gradientValue = (time - chart.left_visible_bar_time) / (chart.right_visible_bar_time - chart.left_visible_bar_time)

//@variable The gradient color. The `chart.*` variables in this calculation are of the type "input color", because 
//          they depend on the "Background" inputs in the "Canvas" tab of the chart's settings. 
gradientColor = color.from_gradient(gradientValue, 0, 1, chart.bg_color, chart.fg_color)

// Color the background using the `gradientColor` value.
bgcolor(gradientColor)
```

### simple {#simple}

Values qualified as “simple” are established at runtime, while the script executes on the _first_ available bar. Similar to values qualified as “input” or “const”, “simple” values _do not change_ across bars.

All variables and function parameters that have the “simple” qualifier can accept only “simple”, “input”, or “const” values; they do not allow values qualified as “series”.

Many [built-in variables](https://www.tradingview.com/pine-script-docs/language/built-ins/#built-in-variables), such as most `syminfo.*` and `timeframe.*` variables, hold “simple” values instead of “const” or “input” because they depend on information that a script can obtain only _after_ it starts running on a dataset. Likewise, various [built-in function](https://www.tradingview.com/pine-script-docs/language/built-ins/#built-in-functions) parameters require values with the “simple” qualifier or a weaker one.

The following script uses [request.security()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.security) with a `calc_bars_count` argument to retrieve a limited history of daily [close](https://www.tradingview.com/pine-script-reference/v6/#var_close) values. It determines the number of historical days in the request based on the “simple string” value of [syminfo.type](https://www.tradingview.com/pine-script-reference/v6/#var_syminfo.type). For cryptocurrency symbols, the call requests 14 days of historical data. For other symbols, it requests 10 days of data. The script compiles successfully because the `reqDays` variable holds the type “simple int”, which matches the expected type for the `calc_bars_count` parameter:

```pine
//@version=6
indicator("'simple' values demo")

//@variable The number of historical days in the data request. This variable's type is "simple int",
//          because the strongest qualified type in the calculation is "simple string".
reqDays = syminfo.type == "crypto" ? 14 : 10

//@variable The `close` value from the "1D" timeframe. 
//          This call works because `calc_bars_count` expects a "simple int" argument.
requestedClose = request.security(syminfo.tickerid, "1D", close, calc_bars_count = reqDays)

plot(requestedClose)
```

Programmers can explicitly define variables and parameters that require “simple” values, or values with a weaker qualifier, by prefixing their declaration with the [simple](https://www.tradingview.com/pine-script-reference/v6/#type_simple) keyword, followed by a [type keyword](https://www.tradingview.com/pine-script-docs/language/type-system/#types). Variables declared with this keyword can hold runtime-calculated values that do not change across bars. These variables cannot accept values qualified as “series”, even if those values remain consistent on every bar.

The script below attempts to assign the result of a [math.random()](https://www.tradingview.com/pine-script-reference/v6/#fun_math.random) call to a `rand` variable declared with the [simple](https://www.tradingview.com/pine-script-reference/v6/#type_simple) keyword, causing a compilation error. The [math.random()](https://www.tradingview.com/pine-script-reference/v6/#fun_math.random) function returns a _different value_ on each call, meaning its return type is “series float”. However, the [simple](https://www.tradingview.com/pine-script-reference/v6/#type_simple) keyword forces the `rand` variable to require a “simple float”, “input float”, or “const float” value:

```pine
//@version=6
indicator("Cannot assign a 'series' value demo")

// This declaration causes an error. `math.random()` returns a "series float" value, but the `rand` variable 
// requires a "float" value with the "simple" qualifier or a weaker one.
simple float rand = math.random()

plot(rand)
```

> [!NOTE]
> Using the [simple](https://www.tradingview.com/pine-script-reference/v6/#type_simple) keyword is optional in most cases. However, the keyword is required to define exported _library functions_ that accept only arguments with “simple” or weaker qualifiers and return “simple” results. See the [Libraries](https://www.tradingview.com/pine-script-docs/concepts/libraries/) page to learn more.

### series {#series}

Values qualified as “series” provide the most flexibility in a script’s calculations. These values are available at runtime, and they are the **only** values that can _change_ from bar to bar.

All variables and function parameters that accept a “series” value also allow values with any other qualifier, because “series” is the _highest_ qualifier in the [qualifier hierarchy](https://www.tradingview.com/pine-script-docs/language/type-system/#qualifiers).

All built-in variables that store bar information — such as [open](https://www.tradingview.com/pine-script-reference/v6/#var_open), [high](https://www.tradingview.com/pine-script-reference/v6/#var_high), [low](https://www.tradingview.com/pine-script-reference/v6/#var_low), [close](https://www.tradingview.com/pine-script-reference/v6/#var_close), [volume](https://www.tradingview.com/pine-script-reference/v6/#var_volume), [time](https://www.tradingview.com/pine-script-reference/v6/#var_time), [bar\_index](https://www.tradingview.com/pine-script-reference/v6/#var_bar_index), and [barstate.isconfirmed](https://www.tradingview.com/pine-script-reference/v6/#var_barstate.isconfirmed) — always hold “series” values. The same applies to variables that store data from internal calculations that update from bar to bar, such as [ta.vwap](https://www.tradingview.com/pine-script-reference/v6/#var_ta.vwap) and [ta.pvi](https://www.tradingview.com/pine-script-reference/v6/#var_ta.pvi).

If an expression’s result _can vary_ on any execution, it automatically inherits the “series” qualifier. Similarly, even if an expression returns an unchanging result on every bar, that result is still qualified as “series” if the calculation depends on at least one “series” value.

> [!NOTE]
> [Special types](https://www.tradingview.com/pine-script-docs/language/type-system/#types) and [user-defined types](https://www.tradingview.com/pine-script-docs/language/type-system/#user-defined-types) automatically inherit the “series” qualifier, meaning any calculation involving these types returns “series” results. Scripts **cannot** create instances of these types with weaker qualifiers such as “simple” or “const”. See the [reference types](https://www.tradingview.com/pine-script-docs/language/type-system/#reference-types) section for more information.

The following script calculates highest and lowest values from a `sourceInput` series and a “const float” value over `lengthInput` bars. The `highest` and `lowest` variables automatically inherit the “series” qualifier because the [ta.highest()](https://www.tradingview.com/pine-script-reference/v6/#fun_ta.highest) and [ta.lowest()](https://www.tradingview.com/pine-script-reference/v6/#fun_ta.lowest) functions always return “series” results. These functions never return a value with a weaker qualifier, even if they calculate on a constant, because their `source` parameter is of the type “series float”:

```pine
//@version=6
indicator("'series' values demo", overlay = true)

//@variable The source series to process in the `ta.highest()` call. 
//          This variable's type is "series float", because `input.source()` returns "series" values.
sourceInput = input.source(close, "Source")
//@variable The number of bars to analyze. This variable's type is "input int".
lengthInput = input.int(20, "Length")

//@variable The highest `sourceInput` value over `lengthInput` bars. This variable's type is "series float".
highest = ta.highest(source = sourceInput, length = lengthInput)
//@variable The result of calculating the lowest value from a constant. This variable's type is also "series float". 
//          The `source` parameter's type is "series float", so the function returns a "series float" value, regardless 
//          of the argument's type qualifier. 
lowest = ta.lowest(source = 100.0, length = lengthInput)

plot(highest, "Highest", color.green)
plot(lowest,  "Lowest",  color.red)
```

Programmers can use the [series](https://www.tradingview.com/pine-script-reference/v6/#type_series) keyword to explicitly define variables and parameters that accept “series” values. A script cannot use a variable declared using this keyword in any part of the code that requires “simple” or weaker qualifiers, even if the variable’s assigned value never changes.

For example, the script below declares a `lengthInput` variable with the [series](https://www.tradingview.com/pine-script-reference/v6/#type_series) keyword. Then, it attempts to use the variable as the `length` argument of a [ta.ema()](https://www.tradingview.com/pine-script-reference/v6/#fun_ta.ema) function call, causing a compilation error. Although the variable’s value comes from an [integer input](https://www.tradingview.com/pine-script-docs/concepts/inputs/#integer-input), the [series](https://www.tradingview.com/pine-script-reference/v6/#type_series) keyword causes its type to become “series int” instead of “input int”. This type is not compatible with the [ta.ema()](https://www.tradingview.com/pine-script-reference/v6/#fun_ta.ema) function’s `length` parameter, because the strongest qualified type that the parameter accepts is “simple int”:

```pine
//@version=6
indicator("`series` keyword demo", overlay = true)

//@variable Holds a value intended for use as the `length` argument in `ta.ema()`.
//          Although the variable's value is from an input, its type is "series int" because the declaration uses the 
//          `series` keyword.
series int lengthInput = input.int(20, "Length")

// This function call causes an error. The `length` parameter requires a "simple int", "input int", or "const int"
// argument; it cannot accept a "series int" value. 
ema = ta.ema(close, length = lengthInput)

plot(ema)
```

## Types {#types}

Types define the _categories_ of values in a script and determine the kinds of functions and operations with which those values are compatible. Each type represents a different kind of data. The primary types available in Pine Script consist of the following:

-   Fundamental types: [int](https://www.tradingview.com/pine-script-docs/language/type-system/#int), [float](https://www.tradingview.com/pine-script-docs/language/type-system/#float), [bool](https://www.tradingview.com/pine-script-docs/language/type-system/#bool), [color](https://www.tradingview.com/pine-script-docs/language/type-system/#color), and [string](https://www.tradingview.com/pine-script-docs/language/type-system/#string)
-   [Enum types (enums)](https://www.tradingview.com/pine-script-docs/language/type-system/#enum-types)
-   Special types: [plot](https://www.tradingview.com/pine-script-docs/language/type-system/#plot-and-hline), [hline](https://www.tradingview.com/pine-script-docs/language/type-system/#plot-and-hline), [line](https://www.tradingview.com/pine-script-docs/language/type-system/#drawing-types), [linefill](https://www.tradingview.com/pine-script-docs/language/type-system/#drawing-types), [box](https://www.tradingview.com/pine-script-docs/language/type-system/#drawing-types), [polyline](https://www.tradingview.com/pine-script-docs/language/type-system/#drawing-types), [label](https://www.tradingview.com/pine-script-docs/language/type-system/#drawing-types), [table](https://www.tradingview.com/pine-script-docs/language/type-system/#drawing-types), [chart.point](https://www.tradingview.com/pine-script-docs/language/type-system/#chart-points), [footprint](https://www.tradingview.com/pine-script-docs/language/type-system/#footprint-and-volume_row), [volume\_row](https://www.tradingview.com/pine-script-docs/language/type-system/#footprint-and-volume_row), [array](https://www.tradingview.com/pine-script-docs/language/type-system/#collections), [matrix](https://www.tradingview.com/pine-script-docs/language/type-system/#collections), and [map](https://www.tradingview.com/pine-script-docs/language/type-system/#collections)
-   [User-defined types (UDTs)](https://www.tradingview.com/pine-script-docs/language/type-system/#user-defined-types)
-   [void](https://www.tradingview.com/pine-script-docs/language/type-system/#void)

Fundamental types and enum types are also known as [value types](https://www.tradingview.com/pine-script-docs/language/type-system/#value-types). Variables of these types directly hold values. Additionally, value types can inherit any [type qualifier](https://www.tradingview.com/pine-script-docs/language/type-system/#qualifiers), depending on their use in the code. By contrast, special types and UDTs are [reference types](https://www.tradingview.com/pine-script-docs/language/type-system/#reference-types). Variables of these types do not store direct values; they hold _references_ (sometimes referred to as _IDs_) that provide access to data stored _elsewhere_ in memory. Instances of these types always inherit the “series” qualifier, regardless of how the script uses them.

> [!NOTE]
> Pine Script also includes a set of _unique value types_. These types are compatible only with specific built-in parameters and operators. For example, [plot.style\_line](https://www.tradingview.com/pine-script-reference/v6/#const_plot.style_line) and other `plot.style_*` constants are of the _“plot\_style”_ type. A value of this type is required only by the `style` parameter of the [plot()](https://www.tradingview.com/pine-script-reference/v6/#fun_plot) function; other built-ins _cannot_ use it. The only other way scripts can use `plot.style_*` constants is by assigning their values to separate variables or comparing them with the [\==](https://www.tradingview.com/pine-script-reference/v6/#op_==) or [!=](https://www.tradingview.com/pine-script-reference/v6/#op_!=) operators. See the “Constants” section of the [Reference Manual](https://www.tradingview.com/pine-script-reference/v6/) to learn about other unique types and their uses.

Programmers can explicitly define the type of a variable, function parameter, or field by prefixing its declaration with a _type keyword_ (e.g., [int](https://www.tradingview.com/pine-script-reference/v6/#type_int)) or a _type identifier_ (e.g., `array<int>`). Specifying types in code is usually optional, because the compiler can automatically determine type information in most cases. However, type specification is _required_ when:

-   Declaring [variables](https://www.tradingview.com/pine-script-docs/language/variable-declarations/), [user-defined function](https://www.tradingview.com/pine-script-docs/language/user-defined-functions/) parameters, or [UDT](https://www.tradingview.com/pine-script-docs/language/type-system/#user-defined-types) fields with initial [`na` values](https://www.tradingview.com/pine-script-docs/language/type-system/#na-value).
-   Defining the parameters of exported [library functions](https://www.tradingview.com/pine-script-docs/concepts/libraries/#library-functions), or declaring exported constants.
-   Using [qualifier keywords](https://www.tradingview.com/pine-script-docs/language/type-system/#qualifiers) in a variable or parameter declaration.
-   Declaring the first parameter of a [user-defined method](https://www.tradingview.com/pine-script-docs/language/methods/#user-defined-methods).

> [!TIP]
> Even when specifying a type in the code is not mandatory, doing so helps to promote code readability. Additionally, using type keywords helps the Pine Editor provide relevant code suggestions.

The example below calculates a moving average and detects when the [close](https://www.tradingview.com/pine-script-reference/v6/#var_close) series crosses over the value. The script uses values of different fundamental types in its calculations. It includes the [int](https://www.tradingview.com/pine-script-reference/v6/#type_int), [float](https://www.tradingview.com/pine-script-reference/v6/#type_float), [bool](https://www.tradingview.com/pine-script-reference/v6/#type_bool), [color](https://www.tradingview.com/pine-script-reference/v6/#type_color), and [string](https://www.tradingview.com/pine-script-reference/v6/#type_string) keywords in its variable declarations to specify which type each variable accepts:

```pine
//@version=6
indicator("Type keywords demo", overlay = true)

// The `string`, `int`, `float`, `bool` and `color` keywords are *optional* in the following variable declarations:
string MA_TITLE    = "MA"
int    lengthInput = input.int(100, "Length", minval = 2)
float  ma          = ta.sma(close, lengthInput)
bool   crossUp     = ta.crossover(close, ma)
color  maColor     = close > ma ? color.lime : color.fuchsia

// Specifying a type is required in this declaration, because the variable's initial value is `na`.
// The `float` keyword tells the compiler that the variable accepts "float" values.
var float crossValue = na

// Update the `crossValue` variable based on the `crossUp` condition.
if crossUp
    crossValue := close

plot(ma, MA_TITLE, maColor)
plot(crossValue, "Cross value", style = plot.style_circles)
plotchar(crossUp, "Cross Up", "▲", location.belowbar, size = size.small)
```

Note that:

-   The first five variables in this script _do not_ require type keywords in their declarations, but including them helps promote readability. However, the `crossValue` variable _does_ require a specified type in its declaration because its initial value is [na](https://www.tradingview.com/pine-script-reference/v6/#var_na).

> [!TIP]
> To confirm a variable’s type, hover over its identifier in the Pine Editor. The editor displays a pop-up window containing the name of the variable’s type and additional information about the variable.

The sections below explain the different types available in Pine Script and how they work.

### Value types {#value-types}

The types covered in the following sections are _value types_. These types directly represent values, such as numbers, logical conditions, colors, or text sequences. Value types are compatible with any [type qualifier](https://www.tradingview.com/pine-script-docs/language/type-system/#qualifiers), depending on their use in the code. Additionally, value types, unlike reference types, are compatible with arithmetic and logical [operators](https://www.tradingview.com/pine-script-docs/language/operators/).

#### int {#int}

Values of the “int” type represent _integers_: whole numbers _without_ fractional parts.

Literal integers in a script are sequences of decimal digits without a decimal point (`.`). These literals can also include the _unary_ [+](https://www.tradingview.com/pine-script-reference/v6/#op_+) or [\-](https://www.tradingview.com/pine-script-reference/v6/#op_-) operators at the beginning of the sequence to specify their sign (positive or negative).

Below are a few examples of literal integers:

```pine
1
-1
750
```

Many built-in variables hold “int” values, including [bar\_index](https://www.tradingview.com/pine-script-reference/v6/#var_bar_index), [time](https://www.tradingview.com/pine-script-reference/v6/#var_time), [timenow](https://www.tradingview.com/pine-script-reference/v6/#var_timenow), [dayofmonth](https://www.tradingview.com/pine-script-reference/v6/#var_dayofmonth), and [strategy.wintrades](https://www.tradingview.com/pine-script-reference/v6/#var_strategy.wintrades).

#### float {#float}

Values of the “float” type represent _floating-point_ numbers. In contrast to “int” values, “float” values represent the whole _and_ fractional parts of a number.

Literal floating-point values in Pine have two different formats:

-   A sequence of decimal digits that contains a decimal point (`.`) to separate the number’s whole and fractional parts. This format can include a unary [+](https://www.tradingview.com/pine-script-reference/v6/#op_+) or [\-](https://www.tradingview.com/pine-script-reference/v6/#op_-) operator at the beginning to specify the number’s sign.
-   A number, with an _optional_ decimal point, followed by `e` or `E` and an additional _whole number_. The number before _and_ after `e` or `E` can include the unary [+](https://www.tradingview.com/pine-script-reference/v6/#op_+) or [\-](https://www.tradingview.com/pine-script-reference/v6/#op_-) operator. This format represents a floating-point number in [E notation](https://en.wikipedia.org/wiki/Scientific_notation#E_notation). It translates to _“X multiplied by 10 raised to the power of Y”_, where “X” is the number before `e` or `E`, and “Y” is the number that follows. This format provides a compact way to represent very large or very small values.

Below are a few examples of floating-point literals:

```pine
3.14159    // Rounded value of Pi (π)
-3.0
6.02e23    // 6.02 * 10^23 (a very large number)
1.6e-19    // 1.6 * 10^-19 (a very small number)
```

The internal precision of “float” values in Pine Script is 1e-16. Floating-point values in Pine cannot precisely represent numbers with more than 16 fractional digits. However, note that [comparison operators](https://www.tradingview.com/pine-script-docs/language/operators/#comparison-operators) automatically round “float” operands to _nine_ fractional digits.

Many built-in variables store “float” values, including [close](https://www.tradingview.com/pine-script-reference/v6/#var_close), [hlcc4](https://www.tradingview.com/pine-script-reference/v6/#var_hlcc4), [volume](https://www.tradingview.com/pine-script-reference/v6/#var_volume), [ta.vwap](https://www.tradingview.com/pine-script-reference/v6/#var_ta.vwap), and [strategy.position\_size](https://www.tradingview.com/pine-script-reference/v6/#var_strategy.position_size).

> [!NOTE]
> Pine Script automatically converts “int” values to the “float” type if a script passes those values to variables or function parameters that require “float” values. Likewise, Pine converts “int” values to “float” in arithmetic or comparison [operations](https://www.tradingview.com/pine-script-docs/language/operators/) that include a “float” operand. See the [Type casting](https://www.tradingview.com/pine-script-docs/language/type-system/#type-casting) section to learn more.

#### bool {#bool}

Values of the “bool” type represent the [Boolean](https://en.wikipedia.org/wiki/Boolean_data_type) truth values of conditions (_true_ or _false_). Scripts use these values in [conditional structures](https://www.tradingview.com/pine-script-docs/language/conditional-structures/) and expressions to trigger specific calculations in the code. All _comparison_ and _logical_ [operators](https://www.tradingview.com/pine-script-docs/language/operators/) return “bool” values.

There are only two possible “bool” literals in Pine Script:

```pine
true    // true value
false   // false value
```

In contrast to most other types, values of the “bool” type are _never_ [na](https://www.tradingview.com/pine-script-reference/v6/#var_na). Any expression or structure with the “bool” return type returns `false` instead of [na](https://www.tradingview.com/pine-script-reference/v6/#var_na) if data is _not available_.

For example, if a script uses the [history-referencing operator](https://www.tradingview.com/pine-script-docs/language/operators/#-history-referencing-operator) to retrieve the value of a “bool” variable from a previous bar that does not exist, that operation returns `false`. Likewise, an [if](https://www.tradingview.com/pine-script-reference/v6/#kw_if) statement with a return expression of the “bool” type returns `false` if none of its _local blocks_ activate. By contrast, expressions and structures with other return types, excluding [void](https://www.tradingview.com/pine-script-docs/language/type-system/#void), return [na](https://www.tradingview.com/pine-script-reference/v6/#var_na) if there is no available data.

All built-in variables that represent conditions store “bool” values, including [barstate.isfirst](https://www.tradingview.com/pine-script-reference/v6/#var_barstate.isfirst), [chart.is\_heikinashi](https://www.tradingview.com/pine-script-reference/v6/#var_chart.is_heikinashi), [session.ismarket](https://www.tradingview.com/pine-script-reference/v6/#var_session.ismarket), and [timeframe.isdaily](https://www.tradingview.com/pine-script-reference/v6/#var_timeframe.isdaily).

> [!NOTE]
> In contrast to some other languages, Pine Script does _not_ automatically convert other types to the “bool” type in logical expressions. Scripts can explicitly convert “int” or “float” values to the “bool” type by using the [bool()](https://www.tradingview.com/pine-script-reference/v6/#fun_bool) function. See the [Type casting](https://www.tradingview.com/pine-script-docs/language/type-system/#type-casting) section to learn more about type conversions.

#### color {#color}

Values of the “color” type represent _RGB colors_, which scripts use to define the colors of chart [visuals](https://www.tradingview.com/pine-script-docs/visuals/overview/). Color literals in Pine have the format `#RRGGBB` or `#RRGGBBAA`, where:

-   Each symbol after the number sign (`#`) represents a [hexadecimal digit](https://en.wikipedia.org/wiki/Hexadecimal), which is a _numeral_ from `0` to `9` or a _letter_ from `A` (for 10) to `F` (for 15). Each set of _two digits_ represents one of the color’s _component values_, ranging from 0 (`00`) to 255 (`FF`).
-   The `RR`, `GG`, and `BB` parts represent the color’s _red_, _green_, and _blue_ components, respectively. The last pair of digits, `AA`, is optional; it specifies the color’s opacity (_alpha_). If the pair is `00`, the color is _transparent_. If `FF` or not specified, the color is _fully opaque_.
-   All letters in the literal value can be uppercase or lowercase.

Below are several examples of literal “color” values:

```pine
#000000      // Black
#FF0000      // Red
#00FF00      // Green
#0000FF      // Blue
#FFFFFF      // White
#808080      // A shade of gray
#3ff7a0      // A custom green-cyan color
#FF000080    // 50% transparent red
#FF0000ff    // Equivalent to #FF0000; fully opaque red
#FF000000    // Completely transparent (invisible) red
```

Pine Script also includes several built-in [color constants](https://www.tradingview.com/pine-script-docs/visuals/colors/#constant-colors), such as [color.green](https://www.tradingview.com/pine-script-reference/v6/#const_color.green), [color.orange](https://www.tradingview.com/pine-script-reference/v6/#const_color.orange), [color.red](https://www.tradingview.com/pine-script-reference/v6/#const_color.red), and [color.blue](https://www.tradingview.com/pine-script-reference/v6/#const_color.blue). Note that [color.blue](https://www.tradingview.com/pine-script-reference/v6/#const_color.blue) is the default color for [plots](https://www.tradingview.com/pine-script-docs/visuals/plots/), and it is the default value for several _color properties_ of [drawing types](https://www.tradingview.com/pine-script-docs/language/type-system/#drawing-types).

The `color` namespace contains functions for retrieving color components, modifying colors, and creating new colors. For instance, scripts can use [color.new()](https://www.tradingview.com/pine-script-reference/v6/#fun_color.new) to define a copy of a built-in color with different transparency, or use [color.rgb()](https://www.tradingview.com/pine-script-reference/v6/#fun_color.rgb) to create a new color with specific red, green, blue, and transparency components.

Note that the `red`, `green`, and `blue` parameters of the [color.rgb()](https://www.tradingview.com/pine-script-reference/v6/#fun_color.rgb) function expect a number from _0 to 255_, where 0 means no intensity and 255 means maximum intensity. The `transp` parameter of [color.rgb()](https://www.tradingview.com/pine-script-reference/v6/#fun_color.rgb) and [color.new()](https://www.tradingview.com/pine-script-reference/v6/#fun_color.new) expects a value from _0 to 100_, where 0 means fully opaque and 100 means completely transparent. Both functions automatically _clamp_ arguments to these ranges, and they round the specified values to _whole numbers_.

The example below creates a new “color” value with [color.rgb()](https://www.tradingview.com/pine-script-reference/v6/#fun_color.rgb), modifies the color’s transparency based on the current day of the week with [color.new()](https://www.tradingview.com/pine-script-reference/v6/#fun_color.new), and then displays the resulting color in the chart’s background:

```pine
//@version=6
indicator("`color.*()` functions demo")

//@variable A color with custom red, green, and blue components. The variable's type is "const color".
color BASE_COLOR = color.rgb(0, 99, 165)

//@variable A calculated transparency value based on the current day of the week. This variable's type is "series int".
int transparency = 50 + int(40 * dayofweek / 7)

//@variable A modified copy of `BASE_COLOR` with dynamic transparency. 
//          This variable's type is "series color", because its calculation depends on a "series int" value.
color modifiedColor = color.new(BASE_COLOR, transparency)

// Color the background using the `modifiedColor` value.
bgcolor(modifiedColor)
```

Note that:

-   The value stored by `BASE_COLOR` is of the type “const color” because it depends on only “const” values. However, the modified color returned by [color.new()](https://www.tradingview.com/pine-script-reference/v6/#fun_color.new) is of the type “series color”, because the [dayofweek](https://www.tradingview.com/pine-script-reference/v6/#var_dayofweek) variable used in the calculation has the “series” [qualifier](https://www.tradingview.com/pine-script-docs/language/type-system/#qualifiers).

To learn more about working with colors in Pine, see the [Colors](https://www.tradingview.com/pine-script-docs/visuals/colors/) page.

#### string {#string}

Values of the “string” type contain sequences of encoded characters representing text, including letters, digits, symbols, spaces, or other Unicode characters. Scripts use strings in many ways, such as to define titles, express symbols and timeframes, create alerts and debug messages, and display text on the chart.

Literal strings in Pine Script are sequences of characters enclosed by two ASCII quotation marks (`"`) or apostrophes (`'`). For example:

```pine
"This is a literal string enclosed in quotation marks."

'This is a literal string enclosed in apostrophes.'
```

Quotation marks and apostrophes are functionally similar when used as the enclosing delimiters of literal strings. A string enclosed in quotation marks can contain any number of apostrophes. Likewise, a string enclosed in apostrophes can contain any number of quotation marks. For example:

```pine
"It's an example"

'The "Star" indicator'
```

A literal string can prefix some characters with the backslash character (`\`) to _change_ their meaning. For example, applying a backslash to a quotation mark or apostrophe adds that character directly into a literal string’s sequence instead of treating the character as the _end_ of the string:

```pine
'It\'s an example'

"The \"Star\" indicator"
```

Applying a backslash to the `n` or `t` characters in a literal string creates _escape sequences_ for multiline text or indentation respectively, which scripts can render using `plot*()` functions, [Pine Logs](https://www.tradingview.com/pine-script-docs/writing/debugging/#pine-logs), or some [drawing types](https://www.tradingview.com/pine-script-docs/language/type-system/#drawing-types). For example, this string represents multiline text with a single word per line:

```pine
"This\nString\nContains\nOne\nWord\nPer\nLine"
```

Scripts can use two operators, [+](https://www.tradingview.com/pine-script-reference/v6/#op_+) and [+=](https://www.tradingview.com/pine-script-reference/v6/#op_+=), to _concatenate_ (combine) two separate strings. These operators create a new string containing the first operand’s character sequence followed by the second operand’s sequence. For example:

```pine
"This creates a " + "concatenated string."
```

The `str` namespace contains several built-in functions that perform string-based calculations or create new strings. For example, the script below calls [str.format()](https://www.tradingview.com/pine-script-reference/v6/#fun_str.format) on each bar to create a _formatted string_ containing representations of “float” price values, and it displays the result as multiline text in a label positioned at the bar’s [high](https://www.tradingview.com/pine-script-reference/v6/#var_high) value:

```pine
//@version=6
indicator("Formatted string demo", overlay = true)

//@variable A "series string" value containing representations of the bar's OHLC prices.
string ohlcString = str.format("Open: {0}\nHigh: {1}\nLow: {2}\nClose: {3}", open, high, low, close)

// Draw a label to display the `ohlcString` value as multiline text at the bar's `high` value.
label.new(bar_index, high, ohlcString, textcolor = color.white)
```

Several built-in variables that contain symbol and timeframe information store “string” values, e.g., [syminfo.tickerid](https://www.tradingview.com/pine-script-reference/v6/#var_syminfo.tickerid), [syminfo.currency](https://www.tradingview.com/pine-script-reference/v6/#var_syminfo.currency), and [timeframe.period](https://www.tradingview.com/pine-script-reference/v6/#var_timeframe.period).

For detailed information about Pine strings and the built-in `str.*()` functions, refer to the [Strings](https://www.tradingview.com/pine-script-docs/concepts/strings/) page. To learn more about displaying text from strings, see the [Text and shapes](https://www.tradingview.com/pine-script-docs/visuals/text-and-shapes/) and [Debugging](https://www.tradingview.com/pine-script-docs/writing/debugging/) pages.

#### Enum types {#enum-types}

The [enum](https://www.tradingview.com/pine-script-reference/v6/#kw_enum) keyword enables the creation of an _enum_, otherwise known as an _enumeration_, _enumerated type_, or _enum type_. An enum is a unique type that contains distinct _named fields_. These fields represent the _members_ (i.e., possible values) of the enum type. Programmers can use enums to maintain strict control over the values accepted by variables, parameters, conditional expressions, [collections](https://www.tradingview.com/pine-script-docs/language/type-system/#collections), and the fields of [UDT](https://www.tradingview.com/pine-script-docs/language/type-system/#user-defined-types) objects. Additionally, scripts can use the [input.enum()](https://www.tradingview.com/pine-script-reference/v6/#fun_input.enum) function to create enum-based dropdown [inputs](https://www.tradingview.com/pine-script-docs/concepts/inputs/) in the “Settings/Inputs” tab.

The syntax to declare an enum is as follows:

```
[export ]enum <enumName>
    <field_1>[ = <title_1>]
    <field_2>[ = <title_2>]
    ...
    <field_N>[ = <title_N>]
```

Where:

-   [export](https://www.tradingview.com/pine-script-reference/v6/#kw_export) is the optional keyword for exporting the enum from a library, enabling its use in other scripts. See the [Enum types](https://www.tradingview.com/pine-script-docs/concepts/libraries/#enum-types) section of the [Libraries](https://www.tradingview.com/pine-script-docs/concepts/libraries/) page to learn more about exporting enums.
-   `enumName` is the name of the enum type. Scripts can use the enum’s name as the _type keyword_ in [variable declarations](https://www.tradingview.com/pine-script-docs/language/variable-declarations/), parameter and field declarations, and the _type templates_ of collections.
-   `field_*` is the name of an enum field. The field represents a _named member_ (value) of the `enumName` type. Each field must have a _unique_ name that does not match the name or title of any other member in the enum. To retrieve an enum member, use _dot notation_ syntax on the enum’s name (e.g., `enumName.field_1`).
-   `title_*` is a “const string” value representing the _title_ of an enum member. If the enum declaration does not specify a member’s title, its title is the “string” representation of its name. The [input.enum()](https://www.tradingview.com/pine-script-reference/v6/#fun_input.enum) function displays enum member titles within a dropdown input in the “Settings/Inputs” tab. To retrieve the “string” title of an enum member, use the [str.tostring()](https://www.tradingview.com/pine-script-reference/v6/#fun_str.tostring) function on that member (e.g., `str.tostring(enumName.field_1)`). As with member names, each enum member’s title must be _unique_; it cannot match the name or title of another member in the same enum.

The following code block declares an enum named `maChoice`. Each field within the declaration represents a unique, constant member of the `maChoice` enum type with a distinct title:

```pine
//@enum       An enumeration of named values for moving average selection.
//@field sma  Specifies a Simple Moving Average.
//@field ema  Specifies an Exponential Moving Average.
//@field wma  Specifies a Weighted Moving Average.
//@field hma  Specifies a Hull Moving Average.
enum maChoice
    sma = "Simple Moving Average"
    ema = "Exponential Moving Average"
    wma = "Weighted Moving Average"
    hma = "Hull Moving Average"
```

The following script uses the [input.enum()](https://www.tradingview.com/pine-script-reference/v6/#fun_input.enum) function to create a dropdown input from our `maChoice` enum in the “Settings/Inputs” tab. The dropdown displays each field’s _title_ as a possible choice. The value of `maInput` is the `maChoice` _member_ corresponding to the selected title. The script compares the `maChoice` value inside a [switch](https://www.tradingview.com/pine-script-reference/v6/#kw_switch) structure to determine which `ta.*()` function it uses to calculate a moving average:

```pine
//@version=6
indicator("Enum types demo", overlay = true)

//@enum       An enumeration of named values for moving average selection.
//@field sma  Specifies a Simple Moving Average.
//@field ema  Specifies an Exponential Moving Average.
//@field wma  Specifies a Weighted Moving Average.
//@field hma  Specifies a Hull Moving Average.
enum maChoice
    sma = "Simple Moving Average"
    ema = "Exponential Moving Average"
    wma = "Weighted Moving Average"
    hma = "Hull Moving Average"

//@variable The `maChoice` member representing a selected moving average name.
//          This variable's type is "input maChoice".
maChoice maInput = input.enum(maChoice.sma, "Moving average type")
//@variable The length of the moving average.
int lengthInput = input.int(20, "Length", 1, 4999)

//@variable The moving average corresponding to the selected enum member.
float selectedMA = switch maInput
    maChoice.sma => ta.sma(close, lengthInput)
    maChoice.ema => ta.ema(close, lengthInput)
    maChoice.wma => ta.wma(close, lengthInput)
    maChoice.hma => ta.hma(close, lengthInput)

// Plot the `selectedMA` value.
plot(selectedMA, "Selected moving average", color.teal, 3)
```

See the [Enums](https://www.tradingview.com/pine-script-docs/language/enums/) page and the [Enum input](https://www.tradingview.com/pine-script-docs/concepts/inputs/#enum-input) section of the [Inputs](https://www.tradingview.com/pine-script-docs/concepts/inputs/) page to learn more about using enums and enum inputs.

### Reference types {#reference-types}

All the types covered in the following sections are _reference types_. These types _do not_ directly represent values. Instead, scripts use them to create _objects_: logical entities that store data in a distinct location. Variables of reference types hold _references_, also known as _IDs_, that identify objects in memory and enable access to their data.

In contrast to [value types](https://www.tradingview.com/pine-script-docs/language/type-system/#value-types), which support _any_ [type qualifier](https://www.tradingview.com/pine-script-docs/language/type-system/#qualifiers), instances of a reference type automatically inherit the “series” qualifier, because each instance is _unique_. Additionally, because reference types do not represent values, they are _not_ compatible with any arithmetic or logical [operators](https://www.tradingview.com/pine-script-docs/language/operators/).

For advanced information about how these types differ from value types, see the [Value vs. reference types](https://www.tradingview.com/pine-script-docs/language/type-system/#value-vs-reference-types) section at the bottom of the page.

#### plot and hline {#plot-and-hline}

Pine Script uses the “plot” and “hline” types to display [plots](https://www.tradingview.com/pine-script-docs/visuals/plots/) and horizontal [levels](https://www.tradingview.com/pine-script-docs/visuals/levels/) on the chart. The [plot()](https://www.tradingview.com/pine-script-reference/v6/#fun_plot) and [hline()](https://www.tradingview.com/pine-script-reference/v6/#fun_hline) functions create instances of these types. Each call to these functions returns a _reference (ID)_ to a specific “plot” or “hline” instance. Scripts can assign the references returned by these functions to variables for use with the [fill()](https://www.tradingview.com/pine-script-reference/v6/#fun_fill) function, which colors the space between two displayed plots or levels.

> [!NOTE]
> Only the [plot()](https://www.tradingview.com/pine-script-reference/v6/#fun_plot) and [hline()](https://www.tradingview.com/pine-script-reference/v6/#fun_hline) functions return usable IDs. All other plot-related functions — including [plotchar()](https://www.tradingview.com/pine-script-reference/v6/#fun_plotchar), [plotshape()](https://www.tradingview.com/pine-script-reference/v6/#fun_plotshape), [plotarrow()](https://www.tradingview.com/pine-script-reference/v6/#fun_plotarrow), [plotbar()](https://www.tradingview.com/pine-script-reference/v6/#fun_plotbar), [plotcandle()](https://www.tradingview.com/pine-script-reference/v6/#fun_plotcandle), [barcolor()](https://www.tradingview.com/pine-script-reference/v6/#fun_barcolor), and [bgcolor()](https://www.tradingview.com/pine-script-reference/v6/#fun_bgcolor) — return [void](https://www.tradingview.com/pine-script-docs/language/type-system/#void), because they produce _only_ visual outputs. Scripts _cannot_ use data from these functions in other parts of the code.

The following example calculates two [EMAs](https://www.tradingview.com/support/solutions/43000592270-exponential-moving-average/), and then uses two [plot()](https://www.tradingview.com/pine-script-reference/v6/#fun_plot) calls to display their values on the chart. It assigns the “plot” IDs returned by the function calls to variables, then uses those variables in a call to [fill()](https://www.tradingview.com/pine-script-reference/v6/#fun_fill) to color the visual space between the displayed plots:

```pine
//@version=6
indicator("plot fill demo", overlay = true)

//@variable A "series float" value representing a 10-bar EMA of `close`.
float emaFast = ta.ema(close, 10)
//@variable A "series float" value representing a 20-bar EMA of `close`.
float emaSlow = ta.ema(close, 20)

//@variable Holds the ID of the plot that displays the `emaFast` series.
emaFastPlot = plot(emaFast, "Fast EMA", color.orange, 3)
//@variable Holds the ID of the plot that displays the `emaSlow` series.
emaSlowPlot = plot(emaSlow, "Slow EMA", color.gray, 3)

// Color the space between the outputs from the "plot" objects referenced by `emaFastPlot` and `emaSlowPlot`.
fill(emaFastPlot, emaSlowPlot, color.new(color.purple, 50), "EMA Fill")
```

Note that:

-   Pine does _not_ include type keywords for specifying variables of the “plot” or “hline” type. Variables of these types never hold [na](https://www.tradingview.com/pine-script-reference/v6/#var_na), so Pine can always determine their type information automatically.
-   A single [fill()](https://www.tradingview.com/pine-script-reference/v6/#fun_fill) function call cannot use both a “plot” and “hline” ID. The function requires two IDs of the _same type_.

In addition to displaying the complete history of “series” values on the chart, “plot” objects enable _indicator-on-indicator_ functionality. Scripts can access values from _another_ script’s plots for their calculations by using the [input.source()](https://www.tradingview.com/pine-script-reference/v6/#fun_input.source) function. See the [Source input](https://www.tradingview.com/pine-script-docs/concepts/inputs/#source-input) section of the [Inputs](https://www.tradingview.com/pine-script-docs/concepts/inputs/) page to learn more.

> [!NOTE]
> In contrast to variables of all other reference types, variables of the “plot” or “hline” type cannot refer to different plots or levels across bars. All variables of these types must consistently hold the references returned by the _same_ [plot()](https://www.tradingview.com/pine-script-reference/v6/#fun_plot) or [hline()](https://www.tradingview.com/pine-script-reference/v6/#fun_hline) calls on _every execution_. Additionally, the functions that create “plot” and “hline” objects work only in the _global scope_; scripts cannot use them in the _local scopes_ of [user-defined functions](https://www.tradingview.com/pine-script-docs/language/user-defined-functions/), [conditional structures](https://www.tradingview.com/pine-script-docs/language/conditional-structures/), or [loops](https://www.tradingview.com/pine-script-docs/language/loops/).

#### Drawing types {#drawing-types}

Pine’s drawing types serve as structures for creating _drawing objects_, which scripts use to display custom chart [visuals](https://www.tradingview.com/pine-script-docs/visuals/overview/). The available drawing types are [line](https://www.tradingview.com/pine-script-reference/v6/#type_line), [linefill](https://www.tradingview.com/pine-script-reference/v6/#type_linefill), [box](https://www.tradingview.com/pine-script-reference/v6/#type_box), [polyline](https://www.tradingview.com/pine-script-reference/v6/#type_polyline), [label](https://www.tradingview.com/pine-script-reference/v6/#type_label), and [table](https://www.tradingview.com/pine-script-reference/v6/#type_table).

Each drawing type has an associated _namespace_ with the _same name_. This namespace contains all the available built-ins for creating and managing drawing objects. For example, the `label` namespace contains all the built-in functions and variables for creating and managing [labels](https://www.tradingview.com/pine-script-docs/visuals/text-and-shapes/#labels). To create new instances of any drawing type, scripts can use the following `*.new()` functions from each type’s namespace: [line.new()](https://www.tradingview.com/pine-script-reference/v6/#fun_line.new), [linefill.new()](https://www.tradingview.com/pine-script-reference/v6/#fun_linefill.new), [box.new()](https://www.tradingview.com/pine-script-reference/v6/#fun_box.new), [polyline.new()](https://www.tradingview.com/pine-script-reference/v6/#fun_polyline.new), [label.new()](https://www.tradingview.com/pine-script-reference/v6/#fun_label.new), and [table.new()](https://www.tradingview.com/pine-script-reference/v6/#fun_table.new).

Each of these `*.new()` functions creates a new drawing object on every call, and it returns the _ID (reference)_ of that specific object. The other functions in the type’s namespace require this ID to access and delete, copy, or modify the drawing. For example, a script can use the ID returned by [line.new()](https://www.tradingview.com/pine-script-reference/v6/#fun_line.new) later to delete the underlying [line](https://www.tradingview.com/pine-script-reference/v6/#type_line) object with [line.delete()](https://www.tradingview.com/pine-script-reference/v6/#fun_line.delete), copy the object with [line.copy()](https://www.tradingview.com/pine-script-reference/v6/#fun_line.copy), or update the drawing’s color with [line.set\_color()](https://www.tradingview.com/pine-script-reference/v6/#fun_line.set_color).

For detailed information about lines, boxes, and polylines, see the [Lines and boxes](https://www.tradingview.com/pine-script-docs/visuals/lines-and-boxes/) page. To learn more about tables and labels, see the [Tables](https://www.tradingview.com/pine-script-docs/visuals/tables/) page and the [Labels](https://www.tradingview.com/pine-script-docs/visuals/text-and-shapes/#labels) section of the [Text and shapes](https://www.tradingview.com/pine-script-docs/visuals/text-and-shapes/) page.

#### Chart points {#chart-points}

The [chart.point](https://www.tradingview.com/pine-script-reference/v6/#type_chart.point) type is a special type that scripts use to generate _chart points_. Chart points are objects that contain _chart coordinates_. Scripts use information from these objects to position [lines](https://www.tradingview.com/pine-script-docs/visuals/lines-and-boxes/#lines), [boxes](https://www.tradingview.com/pine-script-docs/visuals/lines-and-boxes/#boxes), [polylines](https://www.tradingview.com/pine-script-docs/visuals/lines-and-boxes/#polylines), and [labels](https://www.tradingview.com/pine-script-docs/visuals/text-and-shapes/#labels) on the chart.

Objects of the [chart.point](https://www.tradingview.com/pine-script-reference/v6/#type_chart.point) type contain three _fields_: `time`, `index`, and `price`. The `time` and `index` fields both represent horizontal locations (_x-coordinates_). The `price` field represents the vertical location (_y-coordinate_). Whether a drawing instance uses the `time` or `index` field from a chart point as an x-coordinate depends on the drawing’s `xloc` property. By default, drawings use the `index` field from a chart point and _ignore_ the `time` field.

Multiple functions in the `chart.point` _namespace_ create chart points:

-   The [chart.point.new()](https://www.tradingview.com/pine-script-reference/v6/#fun_chart.point.new) function creates a new chart point containing specified `time`, `index`, and `price` values.
-   The [chart.point.now()](https://www.tradingview.com/pine-script-reference/v6/#fun_chart.point.now) function creates a chart point with a specified `price` value. The object’s `time` and `index` field automatically contain the [time](https://www.tradingview.com/pine-script-reference/v6/#var_time) and [bar\_index](https://www.tradingview.com/pine-script-reference/v6/#var_bar_index) values from the bar on which the function call occurs.
-   The [chart.point.from\_index()](https://www.tradingview.com/pine-script-reference/v6/#fun_chart.point.from_index) function creates a chart point with only specified `price` and `index` values. The `time` field of the created object is [na](https://www.tradingview.com/pine-script-reference/v6/#var_na). Therefore, all chart points from this function are intended for use with drawings whose `xloc` property is [xloc.bar\_index](https://www.tradingview.com/pine-script-reference/v6/#const_xloc.bar_index).
-   The [chart.point.from\_time()](https://www.tradingview.com/pine-script-reference/v6/#fun_chart.point.from_time) function creates a chart point with only specified `price` and `time` values. The `index` field of the created object is [na](https://www.tradingview.com/pine-script-reference/v6/#var_na). Therefore, all chart points from this function are intended for use with drawings whose `xloc` property is [xloc.bar\_time](https://www.tradingview.com/pine-script-reference/v6/#const_xloc.bar_time).
-   The [chart.point.copy()](https://www.tradingview.com/pine-script-reference/v6/#fun_chart.point.copy) function creates a new chart point with the _same_ `time`, `index`, and `price` values as the one referenced by the specified `id` argument.

The following script draws a new line from the previous bar’s [high](https://www.tradingview.com/pine-script-reference/v6/#var_high) value to the current bar’s [low](https://www.tradingview.com/pine-script-reference/v6/#var_low) value on each execution. It also displays labels at both points of the line. The script sets the coordinates of the [line](https://www.tradingview.com/pine-script-reference/v6/#type_line) and [label](https://www.tradingview.com/pine-script-reference/v6/#type_label) drawings using data from chart points created by the [chart.point.from\_index()](https://www.tradingview.com/pine-script-reference/v6/#fun_chart.point.from_index) and [chart.point.now()](https://www.tradingview.com/pine-script-reference/v6/#fun_chart.point.now) functions:

```pine
//@version=6
indicator("Chart points demo", overlay = true)

//@variable References a chart point containing the previous bar's `bar_index` and `high` values.
firstPoint = chart.point.from_index(bar_index - 1, high[1])
//@variable References a chart point containing the current bar's `bar_index`, `time`, and `low` values.
chart.point secondPoint = chart.point.now(low)

//@variable References a line connecting the coordinates from the objects referenced by `firstPoint` and `secondPoint`. 
line myLine = line.new(firstPoint, secondPoint, color = color.purple, width = 3)

// Draw a label at the `index` and `price` coordinates of the chart point referenced by `firstPoint`.
// The label displays a string representing the first chart point's `price` value.  
label.new(
     firstPoint, str.tostring(firstPoint.price), color = color.green,
     style = label.style_label_down, textcolor = color.white
 )

// Draw a label at the `index` and `price` coordinates of the chart point referenced by `secondPoint`.
// The label displays a string representing the second chart point's `price` value. 
label.new(
     secondPoint, str.tostring(secondPoint.price), color = color.red,
     style = label.style_label_up, textcolor = color.white
 )
```

Refer to the [Lines and boxes](https://www.tradingview.com/pine-script-docs/visuals/lines-and-boxes/) page for additional examples of using chart points.

#### footprint and volume\_row {#footprint-and-volume_row}

The [footprint](https://www.tradingview.com/pine-script-reference/v6/#type_footprint) and [volume\_row](https://www.tradingview.com/pine-script-reference/v6/#type_volume_row) types are special data types that scripts use when requesting [volume footprint](https://www.tradingview.com/support/solutions/43000726164-volume-footprint-charts-a-complete-guide/) information with the [request.footprint()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.footprint) function. An object of the [footprint](https://www.tradingview.com/pine-script-reference/v6/#type_footprint) type stores the available volume footprint data for a specific bar. A [volume\_row](https://www.tradingview.com/pine-script-reference/v6/#type_volume_row) object stores the data for an _individual row_ within a bar’s volume footprint.

The only way to create objects of the [footprint](https://www.tradingview.com/pine-script-reference/v6/#type_footprint) type is by calling the [request.footprint()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.footprint) function. A call to the function returns either the _reference (ID)_ of a [footprint](https://www.tradingview.com/pine-script-reference/v6/#type_footprint) object that contains the retrieved volume footprint data for the current bar, or [na](https://www.tradingview.com/pine-script-reference/v6/#var_na) if no footprint data is available.

Scripts can use [footprint](https://www.tradingview.com/pine-script-reference/v6/#type_footprint) IDs in calls to the functions from the `footprint` _namespace_ to retrieve the calculated volume footprint data. Each function has an `id` parameter that requires a non-na ID of the [footprint](https://www.tradingview.com/pine-script-reference/v6/#type_footprint) type.

Some of the available `footprint.*()` functions return values representing overall metrics from a specific bar’s volume footprint:

-   The [footprint.buy\_volume()](https://www.tradingview.com/pine-script-reference/v6/#fun_footprint.buy_volume) function calculates the total “buy” volume for the volume footprint.
-   The [footprint.sell\_volume()](https://www.tradingview.com/pine-script-reference/v6/#fun_footprint.sell_volume) function calculates the total “sell” volume for the volume footprint.
-   The [footprint.total\_volume()](https://www.tradingview.com/pine-script-reference/v6/#fun_footprint.total_volume) function calculates the sum of the footprint’s total “buy” volume and total “sell” volume.
-   The [footprint.delta()](https://www.tradingview.com/pine-script-reference/v6/#fun_footprint.delta) function calculates the volume footprint’s overall volume delta. The value represents the difference between the footprint’s total “buy” volume and total “sell” volume. A positive value indicates that the total “buy” volume is greater than the total “sell” volume, and a negative value indicates the opposite.

The other `footprint.*()` functions retrieve the _IDs_ of [volume\_row](https://www.tradingview.com/pine-script-reference/v6/#type_volume_row) objects that contain data for individual rows in the volume footprint represented by a [footprint](https://www.tradingview.com/pine-script-reference/v6/#type_footprint) object:

-   The [footprint.poc()](https://www.tradingview.com/pine-script-reference/v6/#fun_footprint.poc) function finds the _Point of Control (POC)_ row of the volume footprint and returns the ID of a [volume\_row](https://www.tradingview.com/pine-script-reference/v6/#type_volume_row) object containing data for that row. The POC is the footprint row that has the largest total volume.
-   The [footprint.vah()](https://www.tradingview.com/pine-script-reference/v6/#fun_footprint.vah) function finds the _Value Area High (VAH)_ row of the volume footprint and returns a [volume\_row](https://www.tradingview.com/pine-script-reference/v6/#type_volume_row) ID for that row. The VAH row is the highest one in the footprint’s _Value Area_.
-   The [footprint.val()](https://www.tradingview.com/pine-script-reference/v6/#fun_footprint.val) function finds the _Value Area Low (VAL)_ row of the volume footprint and returns a [volume\_row](https://www.tradingview.com/pine-script-reference/v6/#type_volume_row) ID for that row. The VAL row is the lowest one in the footprint’s Value Area.
-   The [footprint.get\_row\_by\_price()](https://www.tradingview.com/pine-script-reference/v6/#fun_footprint.get_row_by_price) function searches the volume footprint to find the row whose price range includes a specified price level. If the price belongs to one of the footprint’s rows, the function returns the ID of the [volume\_row](https://www.tradingview.com/pine-script-reference/v6/#type_volume_row) object that contains the data for that row. If the price level does _not_ belong to any row in the footprint, the function returns [na](https://www.tradingview.com/pine-script-reference/v6/#var_na).
-   The [footprint.rows()](https://www.tradingview.com/pine-script-reference/v6/#fun_footprint.rows) function creates an [array](https://www.tradingview.com/pine-script-docs/language/arrays/) that contains the [volume\_row](https://www.tradingview.com/pine-script-reference/v6/#type_volume_row) IDs for _every row_ within the volume footprint, sorted in _ascending order_ by the rows’ price levels. The first element refers to the [volume\_row](https://www.tradingview.com/pine-script-reference/v6/#type_volume_row) object for the _lowest_ row, and the last refers to the one for the _highest_ row. The array’s _type identifier_ is `array<volume_row>`. See the [Collections](https://www.tradingview.com/pine-script-docs/language/type-system/#collections) section below to learn more about collection type identifiers.

> [!NOTE]
> The `va_percent` argument of a [request.footprint()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.footprint) call specifies the percentage of the total volume that the resulting [footprint](https://www.tradingview.com/pine-script-reference/v6/#type_footprint) object uses for the volume footprint’s Value Area. Therefore, changes to the argument directly affect the results of [footprint.vah()](https://www.tradingview.com/pine-script-reference/v6/#fun_footprint.vah) and [footprint.val()](https://www.tradingview.com/pine-script-reference/v6/#fun_footprint.val) calls that use the returned ID.

The only way to access objects of the [volume\_row](https://www.tradingview.com/pine-script-reference/v6/#type_volume_row) type is by calling any of the above functions using a valid [footprint](https://www.tradingview.com/pine-script-reference/v6/#type_footprint) ID. Scripts can retrieve data from objects of this type for detailed footprint analysis by using their IDs in calls to the functions in the `volume_row` _namespace_. Each function has an `id` parameter that requires a non-na ID of the [volume\_row](https://www.tradingview.com/pine-script-reference/v6/#type_volume_row) type:

-   The [volume\_row.up\_price()](https://www.tradingview.com/pine-script-reference/v6/#fun_volume_row.up_price) function returns the upper price level of the footprint row.
-   The [volume\_row.down\_price()](https://www.tradingview.com/pine-script-reference/v6/#fun_volume_row.down_price) function returns the lower price level of the footprint row.
-   The [volume\_row.buy\_volume()](https://www.tradingview.com/pine-script-reference/v6/#fun_volume_row.buy_volume) function calculates the total “buy” volume for the footprint row.
-   The [volume\_row.sell\_volume()](https://www.tradingview.com/pine-script-reference/v6/#fun_volume_row.sell_volume) function calculates the total “sell” volume for the footprint row.
-   The [volume\_row.total\_volume()](https://www.tradingview.com/pine-script-reference/v6/#fun_volume_row.total_volume) function calculates the sum of the footprint row’s total “buy” volume and total “sell” volume.
-   The [volume\_row.delta()](https://www.tradingview.com/pine-script-reference/v6/#fun_volume_row.delta) function calculates the volume delta for the footprint row. The value represents the difference between the row’s “buy” volume and “sell” volume. A positive value indicates that the row’s “buy” volume exceeds its “sell” volume, and a negative value indicates the opposite.
-   The [volume\_row.has\_buy\_imbalance()](https://www.tradingview.com/pine-script-reference/v6/#fun_volume_row.has_buy_imbalance) function checks whether the footprint row has a _buy imbalance_, based on the `imbalance_percent` argument of the original [request.footprint()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.footprint) call. It returns `true` if the row’s “buy” volume exceeds the “sell” volume of the row _below_ it by the specified percentage, and `false` otherwise.
-   The [volume\_row.has\_sell\_imbalance()](https://www.tradingview.com/pine-script-reference/v6/#fun_volume_row.has_sell_imbalance) function checks whether the footprint row has a _sell imbalance_, based on the `imbalance_percent` argument of the original [request.footprint()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.footprint) call. It returns `true` if the row’s “sell” volume exceeds the “buy” volume of the row _above_ it by the specified percentage, and `false` otherwise.

> [!NOTE]
> The `imbalance_percent` argument of a [request.footprint()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.footprint) call determines the percentage difference that the resulting footprint uses for detecting volume imbalances. Changing the argument directly affects the results of the [volume\_row.has\_buy\_imbalance()](https://www.tradingview.com/pine-script-reference/v6/#fun_volume_row.has_buy_imbalance) and [volume\_row.has\_sell\_imbalance()](https://www.tradingview.com/pine-script-reference/v6/#fun_volume_row.has_sell_imbalance) calls that use [volume\_row](https://www.tradingview.com/pine-script-reference/v6/#type_volume_row) IDs from the [footprint](https://www.tradingview.com/pine-script-reference/v6/#type_footprint) object created by the request.

See the [`request.footprint()`](https://www.tradingview.com/pine-script-docs/concepts/other-timeframes-and-data/#requestfootprint) section of the [Other timeframes and data](https://www.tradingview.com/pine-script-docs/concepts/other-timeframes-and-data/) page for more information about footprint requests, and for examples that demonstrate how to use the `footprint.*()` and `volume_row.*()` functions to retrieve footprint data.

To learn more about volume footprints and how they work, refer to the [Volume footprint charts](https://www.tradingview.com/support/solutions/43000726164-volume-footprint-charts-a-complete-guide/) article in our Help Center.

#### Collections {#collections}

Pine Script _collections_ ([arrays](https://www.tradingview.com/pine-script-docs/language/arrays/), [matrices](https://www.tradingview.com/pine-script-docs/language/matrices/), and [maps](https://www.tradingview.com/pine-script-docs/language/maps/)) are objects that store values or the _IDs (references)_ of other objects as _elements_. Collection types enable scripts to group multiple values or IDs in a single location and perform advanced calculations. Arrays and matrices contain elements of _one_ specific type. Maps can contain data of _two_ types: one type for the _keys_, and another for the corresponding _value elements_. The `array`, `matrix`, and `map` _namespaces_ include all the built-in functions for creating and managing collections.

A collection’s _type identifier_ consists of two parts: a _keyword_ defining the collection’s _category_ ([array](https://www.tradingview.com/pine-script-reference/v6/#type_array), [matrix](https://www.tradingview.com/pine-script-reference/v6/#type_matrix), or [map](https://www.tradingview.com/pine-script-reference/v6/#type_map)), and a _type template_ specifying the _types of elements_ that the collection stores. The type template for array or matrix types consists of a single type keyword enclosed in angle brackets (e.g., `<int>` for a collection of “int” values). The type template for a map type consists of _two_ comma-separated keywords surrounded by angle brackets (e.g., `<string, int>` for a map of “string” keys and “int” values).

Below, we list some examples of collection type identifiers and the types that they represent:

-   `array<int>` — an array type for storing “int” values.
-   `array<label>` — an array type for storing [label](https://www.tradingview.com/pine-script-reference/v6/#type_label) IDs.
-   `array<myUDT>` — an array type for storing references to objects of a `myUDT` [user-defined type](https://www.tradingview.com/pine-script-docs/language/type-system/#user-defined-types).
-   `matrix<float>` — a matrix type for storing “float” values.
-   `matrix<line>` — a matrix type for storing [line](https://www.tradingview.com/pine-script-reference/v6/#type_line) IDs.
-   `map<string, float>` — a map type for storing key-value pairs with “string” keys and “float” value elements.
-   `map<int, myUDT>` — a map type for storing “int” values as keys, and references to `myUDT` objects as value elements.

Scripts also use type templates in the `*.new*()` functions that create new collections. For example, a call to `array.new<int>()` creates an array that stores “int” values, and a call to `map.new<int, color>()` creates a map that stores “int” keys and corresponding “color” values.

Programmers can explicitly define variables, parameters, and fields that accept references to objects of specific collection types by using the type identifier as the _type keyword_ in the declaration. The following code snippet declares variables that hold references to collections of the type `array<int>`, `array<float>`, and `matrix<float>`:

```pine
//@variable References an "int" array with a single element.
array<int> myIntArray = array.new<int>(1, 10)

//@variable Holds an initial reference of `na`. Can reference an array of "float" values.
array<float> myFloatArray = na

//@variable References a "float" matrix with two rows and three columns.
matrix<float> myFloatMatrix = matrix.new<float>(2, 3, 0.0)
```

> [!IMPORTANT]
> The `array` namespace also includes _legacy functions_ for creating arrays of specific built-in types. For example, [array.new\_float()](https://www.tradingview.com/pine-script-reference/v6/#fun_array.new_float) creates a “float” array, just like `array.new<float>()`. However, we recommend using the general-purpose [array.new<type>()](https://www.tradingview.com/pine-script-reference/v6/#fun_array.new<type>) function, because it can create arrays of _any_ supported type.
>
> An alternative way to specify an array variable’s type is to prefix its declaration with the _element_ type keyword, followed by empty _square brackets_ (`[]`). For example, a variable whose declaration includes `int[]` as the type keyword accepts a reference to a collection of the type `array<int>`. However, this legacy format is _deprecated_; future versions of Pine Script might not support it. Therefore, we recommend using the `array<type>` format to define type identifiers for readability and consistency.
>
> Note that there are no alternative `*.new*()` functions or type declaration formats for matrices or maps.

Scripts can construct collections and type templates for most available types, including:

-   All [value types](https://www.tradingview.com/pine-script-docs/language/type-system/#value-types): [int](https://www.tradingview.com/pine-script-reference/v6/#type_int), [float](https://www.tradingview.com/pine-script-reference/v6/#type_float), [bool](https://www.tradingview.com/pine-script-reference/v6/#type_bool), [color](https://www.tradingview.com/pine-script-reference/v6/#type_color), [string](https://www.tradingview.com/pine-script-reference/v6/#type_string), and [enum types](https://www.tradingview.com/pine-script-docs/language/type-system/#enum-types).
-   The following _special types_: [line](https://www.tradingview.com/pine-script-reference/v6/#type_line), [linefill](https://www.tradingview.com/pine-script-reference/v6/#type_linefill), [box](https://www.tradingview.com/pine-script-reference/v6/#type_box), [polyline](https://www.tradingview.com/pine-script-reference/v6/#type_polyline), [label](https://www.tradingview.com/pine-script-reference/v6/#type_label), [table](https://www.tradingview.com/pine-script-reference/v6/#type_table), [chart.point](https://www.tradingview.com/pine-script-reference/v6/#type_chart.point), [footprint](https://www.tradingview.com/pine-script-reference/v6/#type_footprint), and [volume\_row](https://www.tradingview.com/pine-script-reference/v6/#type_volume_row).
-   [User-defined types (UDTs)](https://www.tradingview.com/pine-script-docs/language/type-system/#user-defined-types).

Note that maps can use any of these types as value elements, but they can store only _value types_ as _keys_. See the [Maps](https://www.tradingview.com/pine-script-docs/language/maps/) page to learn more.

Collections _cannot_ store elements of any of the following types:

-   The _unique types_ for specific built-ins, such as “plot\_style”, “plot\_display”, and “barmerge\_gaps”.
-   The “plot” or “hline” type.
-   Any collection type.

> [!TIP]
> Although collections cannot directly store the IDs of other collections, they _can_ store references to _user-defined type_ instances that contain collection IDs in their fields. See the next section to learn more about UDTs.

#### User-defined types {#user-defined-types}

The [type](https://www.tradingview.com/pine-script-reference/v6/#kw_type) keyword enables the creation of _user-defined types (UDTs)_. UDTs are composite types; they can contain an arbitrary number of _fields_ that can be of _any_ supported type, including [collection types](https://www.tradingview.com/pine-script-docs/language/type-system/#collections) and other user-defined types. Scripts use UDTs to create [custom objects](https://www.tradingview.com/pine-script-docs/language/objects/) that can store multiple types of data in a single location.

The syntax to declare a user-defined type is as follows:

```
[export ]type <UDT_identifier>
    [varip ][field_type ]<field_name>[ = <value>]
    ...
```

Where:

-   [export](https://www.tradingview.com/pine-script-reference/v6/#kw_export) is the optional keyword for exporting the UDT from a _library_, enabling its use in other scripts. See the [User-defined types and objects](https://www.tradingview.com/pine-script-docs/concepts/libraries/#user-defined-types-and-objects) section of the [Libraries](https://www.tradingview.com/pine-script-docs/concepts/libraries/) page to learn more.
-   `UDT_identifier` is the _name_ of the user-defined type.
-   [varip](https://www.tradingview.com/pine-script-reference/v6/#kw_varip) is an optional keyword that enables the field’s assigned data to _persist_ across all ticks within a bar, similar to a [`varip` variable](https://www.tradingview.com/pine-script-docs/language/variable-declarations/#varip).
-   `field_type` is a type keyword or identifier, which specifies the field’s type.
-   `field_name` is the name of the field.
-   `value` is an optional _default value_ for the field. Each time that the script creates a new instance of the UDT, it initializes the field with the specified value. If not specified, the field’s default value is [na](https://www.tradingview.com/pine-script-reference/v6/#var_na), or `false` if the field’s type is “bool”. Note that the default value _cannot_ be the result of a function call or any other expression; only a _literal value_ or a compatible _built-in variable_ is allowed.

The following example declares a UDT named `pivotPoint`. The type contains two fields for storing pivot data: `pivotTime` and `priceLevel`. The `pivotTime` field is of the type “int”, and `priceLevel` is of the type “float”:

```pine
//@type             A custom type for creating objects that store pivot information.
//@field pivotTime  Stores the pivot's timestamp.
//@field priceLevel Stores the pivot's price.
type pivotPoint
    int   pivotTime
    float priceLevel
```

User-defined types can contain fields for referencing other UDT objects. Additionally, UDTs support _type recursion_, meaning a UDT can include fields for referencing objects of the _same_ UDT. Below, we added a `nextPivot` field to our `pivotPoint` type. Objects of this version of the UDT can store a _reference (ID)_ to a separate object of the same `pivotPoint` type in this field:

```pine
//@type             A custom type for creating objects that store pivot information.
//@field pivotTime  Stores the pivot's timestamp.
//@field priceLevel Stores the pivot's price.
//@field nextPivot  Stores the reference to *another* instance of the `pivotPoint` type.
type pivotPoint
    int        pivotTime
    float      priceLevel
    pivotPoint nextPivot
```

Every user-defined type includes built-in `*.new()` and `*.copy()` functions for creating objects or copying existing ones. Both functions construct a new object on every call and return that object’s ID. For example, `pivotPoint.new()` creates a new instance of our `pivotPoint` type and returns its ID for use in other parts of the script.

To learn more about objects of UDTs and how to use them, see the [Objects](https://www.tradingview.com/pine-script-docs/language/objects/) page.

### void {#void}

Pine Script includes some [built-in functions](https://www.tradingview.com/pine-script-docs/language/built-ins/#built-in-functions) that produce _side effects_ — such as creating triggers for [alerts](https://www.tradingview.com/pine-script-docs/concepts/alerts/), generating chart [visuals](https://www.tradingview.com/pine-script-docs/visuals/overview/), or modifying [collections](https://www.tradingview.com/pine-script-docs/language/type-system/#collections) — _without_ returning any value or reference. The return type of these functions is **“void”**, which represents the _absence_ of usable data. The “void” type applies to every function that performs actions without returning anything that the script can use elsewhere in the code.

For example, [plotshape()](https://www.tradingview.com/pine-script-reference/v6/#fun_plotshape) performs an action (plots shapes on the chart), but it does _not_ return a usable ID like the [plot()](https://www.tradingview.com/pine-script-reference/v6/#fun_plot) function does. Therefore, its return type is “void”. Another example is the [alert()](https://www.tradingview.com/pine-script-reference/v6/#fun_alert) function. The function creates an alert trigger without returning any data that the script can use elsewhere, so it also has the “void” return type.

Because “void” represents the absence of usable data, scripts _cannot_ call functions that return “void” in other calculations or assign their results to variables. Additionally, there is no available keyword to specify that an expression returns the “void” type.

## `na` value {#na-value}

Pine Script includes a special value called [na](https://www.tradingview.com/pine-script-reference/v6/#var_na), which is an abbreviation for _“not available”_. Scripts use [na](https://www.tradingview.com/pine-script-reference/v6/#var_na) to represent an _undefined_ value or reference. It is similar to `null` in Java or `NONE` in Python.

Pine can automatically _cast_ [na](https://www.tradingview.com/pine-script-reference/v6/#var_na) values to almost any type. The type assigned to an [na](https://www.tradingview.com/pine-script-reference/v6/#var_na) value depends on how the code uses it. However, in some cases, more than one type might be valid for a piece of code that includes [na](https://www.tradingview.com/pine-script-reference/v6/#var_na), and the compiler cannot determine which type to assign in those cases.

For example, this line of code declares a `myVar` variable with an initial value of [na](https://www.tradingview.com/pine-script-reference/v6/#var_na). This line causes a _compilation error_, because the type of data the variable might hold later is _uncertain_. It might store a “float” value for plotting, a “string” value for setting text in a label, or maybe even a _reference_ to a [drawing object](https://www.tradingview.com/pine-script-docs/language/type-system/#drawing-types):

```pine
// This declaration causes an error, because the type that `myVar` accepts is *uncertain*.
myVar = na
```

To resolve this error, we must explicitly define the variable’s type in the code. For instance, if the `myVar` variable will store “float” values, we can prefix the variable with the [float](https://www.tradingview.com/pine-script-reference/v6/#type_float) keyword to specify its type as “float”:

```pine
// It is clear to the compiler that this variable accepts "float" values, so this declaration does not cause an error.
float myVar = na
```

Alternatively, we can use the [float()](https://www.tradingview.com/pine-script-reference/v6/#fun_float) function to explicitly cast the [na](https://www.tradingview.com/pine-script-reference/v6/#var_na) value’s type to “float”, causing the variable to automatically inherit the “float” type:

```pine
// This declaration does not cause an error, because `na` is cast to "float", and `myVar` inherits the type.
myVar = float(na)
```

Scripts can test whether the result from a variable or expression is [na](https://www.tradingview.com/pine-script-reference/v6/#var_na) by using the [na()](https://www.tradingview.com/pine-script-reference/v6/#fun_na) function. The function returns `true` if the value or reference is _undefined_. Otherwise, it returns `false`. For example, the following [ternary operation](https://www.tradingview.com/pine-script-docs/language/operators/#-ternary-operator) returns 0 if the value of `myVar` is [na](https://www.tradingview.com/pine-script-reference/v6/#var_na), or [close](https://www.tradingview.com/pine-script-reference/v6/#var_close) if the value is defined:

```pine
//@variable Holds 0 if the the value of `myVar` is `na`; `close` otherwise.
float myClose = na(myVar) ? 0 : close
```

It is crucial to note that scripts **cannot** directly _compare_ values to [na](https://www.tradingview.com/pine-script-reference/v6/#var_na), because by definition, [na](https://www.tradingview.com/pine-script-reference/v6/#var_na) values are undefined. The [\==](https://www.tradingview.com/pine-script-reference/v6/#op_==), [!=](https://www.tradingview.com/pine-script-reference/v6/#op_!=) operators, and all other [comparison operators](https://www.tradingview.com/pine-script-docs/language/operators/#comparison-operators) always return `false` if at least one of the operands is a variable with an [na](https://www.tradingview.com/pine-script-reference/v6/#var_na) value. Therefore, [na](https://www.tradingview.com/pine-script-reference/v6/#var_na) comparisons can cause _unexpected results_. Additionally, if a script attempts to use [na](https://www.tradingview.com/pine-script-reference/v6/#var_na) directly as an operand in any comparison operation, it causes a _compilation error_. For example:

```pine
// This line causes an error, because using `na` directly as an operand for the `==` operator is *not allowed*.
float myClose = myVar == na ? 0 : close
```

Best practices often involve _replacing_ occurrences of undefined values in the code to prevent them from propagating in a script’s calculations. There are three ways to replace [na](https://www.tradingview.com/pine-script-reference/v6/#var_na) values with defined values in a script’s calculations, depending on the type:

-   For the “int”, “float”, and “color” types, scripts can use the [nz()](https://www.tradingview.com/pine-script-reference/v6/#fun_nz) function to replace [na](https://www.tradingview.com/pine-script-reference/v6/#var_na) values with a type-specific default value (`0` for “int”, `0.0` for “float”, or `#00000000` for “color”) or a specified replacement.
-   Alternatively, scripts can use the [fixnan()](https://www.tradingview.com/pine-script-reference/v6/#fun_fixnan) function to replace [na](https://www.tradingview.com/pine-script-reference/v6/#var_na) values of the above types in a series with the latest non-na value from that series’ history.
-   For other types such as “string”, scripts can test for an undefined value using the [na()](https://www.tradingview.com/pine-script-reference/v6/#fun_na) function and replace it if the function returns `true`.

The following line of code uses the [nz()](https://www.tradingview.com/pine-script-reference/v6/#fun_nz) function to replace the value of `close[1]` with the current bar’s [open](https://www.tradingview.com/pine-script-reference/v6/#var_open) value if the expression’s result is [na](https://www.tradingview.com/pine-script-reference/v6/#var_na). This logic prevents the code from returning [na](https://www.tradingview.com/pine-script-reference/v6/#var_na) on the first bar, where there is _no_ previous [close](https://www.tradingview.com/pine-script-reference/v6/#var_close) value for the [\[\]](https://www.tradingview.com/pine-script-reference/v6/#op_[]) operator to access:

```pine
//@variable Holds `true` if the current `close` value is above the previous `close` (or the current `open` if the previous `close` is `na`).
bool risingClose = close > nz(close[1], open)
```

Replacing [na](https://www.tradingview.com/pine-script-reference/v6/#var_na) values to avoid unintended results is especially helpful when a calculation involves data that can _persist_ across bars.

For example, the script below declares a global `allTimeHigh` variable using the [var](https://www.tradingview.com/pine-script-reference/v6/#kw_var) keyword, meaning the variable is initialized only on the first bar and persists on all subsequent bars. On each bar, the script updates the variable with the result of a [math.max()](https://www.tradingview.com/pine-script-reference/v6/#fun_math.max) call that returns the maximum between the current `allTimeHigh` and [high](https://www.tradingview.com/pine-script-reference/v6/#var_high) values, then plots the result.

This script plots [na](https://www.tradingview.com/pine-script-reference/v6/#var_na) instead of the chart’s all-time high on every bar. The `allTimeHigh` variable has an initial value of [na](https://www.tradingview.com/pine-script-reference/v6/#var_na), and the [math.max()](https://www.tradingview.com/pine-script-reference/v6/#fun_math.max) function _cannot_ compare [na](https://www.tradingview.com/pine-script-reference/v6/#var_na) to the current value of [high](https://www.tradingview.com/pine-script-reference/v6/#var_high). Therefore, the function call consistently returns [na](https://www.tradingview.com/pine-script-reference/v6/#var_na):

```pine
//@version=6
indicator("Persistent `na` result demo", overlay = true)

//@variable A variable intended to store the chart's all-time high as of the current bar, with an initial value of `na`. 
var float allTimeHigh = na

// Compare the current `allTimeHigh` and `high` values, and update the `allTimeHigh` with the maximum.
// This line does not assign the current all-time high to the variable; the value remains `na` on *every bar*.
// The `math.max()` function cannot compare undefined values, so it returns `na` if at least one argument is `na`. 
allTimeHigh := math.max(allTimeHigh, high)

plot(allTimeHigh)
```

To fix this script’s behavior and enable it to calculate the chart’s all-time high as intended, we must stop the script from passing an [na](https://www.tradingview.com/pine-script-reference/v6/#var_na) value to the [math.max()](https://www.tradingview.com/pine-script-reference/v6/#fun_math.max) call. In the version below, we included the expression `nz(allTimeHigh, high)` as the first argument in the function call. Now, on any execution where the `allTimeHigh` value is [na](https://www.tradingview.com/pine-script-reference/v6/#var_na), the script replaces it with the value of [high](https://www.tradingview.com/pine-script-reference/v6/#var_high), preventing [na](https://www.tradingview.com/pine-script-reference/v6/#var_na) values from persisting in the calculation:

```pine
//@version=6
indicator("Replaced `na` demo", overlay = true)

//@variable Stores the chart's all-time high value as of the current bar. 
var float allTimeHigh = na

// The `nz()` call in this line replaces `allTimeHigh` with `high` when the variable's value is `na`. Now, the 
// `math.max()` function never receives an `na` argument, so the `na` result no longer persists. 
allTimeHigh := math.max(nz(allTimeHigh, high), high)

plot(allTimeHigh)
```

Note that:

-   An alternative way to fix this script’s behavior is to initialize the `allTimeHigh` variable using the value of [high](https://www.tradingview.com/pine-script-reference/v6/#var_high). The fix works in this case because the script does not use [na](https://www.tradingview.com/pine-script-reference/v6/#var_na) later in its calculations.

> [!NOTE]
> Some built-in functions automatically _ignore_ [na](https://www.tradingview.com/pine-script-reference/v6/#var_na) values in their calculations, preventing them from continuously returning [na](https://www.tradingview.com/pine-script-reference/v6/#var_na) results in most cases. For example, [ta.max()](https://www.tradingview.com/pine-script-reference/v6/#fun_ta.max) calculates the all-time high of a series _without_ considering [na](https://www.tradingview.com/pine-script-reference/v6/#var_na) values. Check the “Remarks” section of a function’s entry in the [Reference Manual](https://www.tradingview.com/pine-script-reference/v6/) to confirm whether it ignores [na](https://www.tradingview.com/pine-script-reference/v6/#var_na) in its calculations.

## Type casting {#type-casting}

Pine Script can convert (cast) values of one type to another type either by using specific functions, or automatically.

The _automatic_ type-casting process can cast “int” values to the “float” type when necessary. All variables, function parameters, fields, and expressions that allow the “float” type can also accept “int” values, because any integer is equivalent to a floating-point number with its fractional part set to 0. If a script assigns an “int” value to a variable, function parameter, or field declared with the [float](https://www.tradingview.com/pine-script-reference/v6/#type_float) keyword, the assigned value’s type automatically changes to “float”. Likewise, Pine converts “int” values to “float” in arithmetic or comparison [operations](https://www.tradingview.com/pine-script-docs/language/operators/) that include a “float” operand.

For example, the following line of code uses the addition operator [+](https://www.tradingview.com/pine-script-reference/v6/#op_+) with “int” and “float” operands. Pine automatically casts the “int” value to the “float” type before performing the addition operation, and thus the expression returns a “float” result:

```pine
// This variable holds a "float" value, because any arithmetic operation with "int" and "float" operands
// returns a "float" result.
myVar = bar_index + close
```

Sometimes, a script must cast data of one type to another. Scripts can cast [`na` values](https://www.tradingview.com/pine-script-docs/language/type-system/#na-value), or numeric values, to specific types by using the following _type-casting functions_: [int()](https://www.tradingview.com/pine-script-reference/v6/#fun_int), [float()](https://www.tradingview.com/pine-script-reference/v6/#fun_float), [bool()](https://www.tradingview.com/pine-script-reference/v6/#fun_bool), [color()](https://www.tradingview.com/pine-script-reference/v6/#fun_color), [string()](https://www.tradingview.com/pine-script-reference/v6/#fun_string), [line()](https://www.tradingview.com/pine-script-reference/v6/#fun_line), [linefill()](https://www.tradingview.com/pine-script-reference/v6/#fun_linefill), [label()](https://www.tradingview.com/pine-script-reference/v6/#fun_label), [box()](https://www.tradingview.com/pine-script-reference/v6/#fun_box), and [table()](https://www.tradingview.com/pine-script-reference/v6/#fun_table).

For example, the script below declares a `LENGTH` variable of the “const float” type, then attempts to use that variable as the `length` argument of a [ta.sma()](https://www.tradingview.com/pine-script-reference/v6/#fun_ta.sma) function call:

```pine
//@version=6
indicator("Invalid type demo", overlay = true)

//@variable Holds a "const float" value intended for use in the `length` argument of `ta.sma()`.
float LENGTH = 10.0

// This line causes an error, because the `length` parameter has the expected type "series int".
float sma = ta.sma(close, length = LENGTH)

plot(sma)
```

The above code causes the following compilation error:

```
Cannot call `ta.sma()` with the argument `length = LENGTH`. An argument of "const float" type was used but a "series int" is expected.
```

This error tells us that the code uses a “float” value where an “int” value is required. There is no automatic rule to cast “float” to “int”, so we must resolve the error manually. In this version of the code, we used the [int()](https://www.tradingview.com/pine-script-reference/v6/#fun_int) function to cast the “float” value of the `LENGTH` variable to the “int” type in the [ta.sma()](https://www.tradingview.com/pine-script-reference/v6/#fun_ta.sma) call. Now, the script compiles successfully:

```pine
//@version=6
indicator("Explicit casting demo")

//@variable Holds a "const float" value intended for use in the `length` argument of `ta.sma()`.
float LENGTH = 10.0

// This line does not cause an error, because the `int()` function converts the `length` argument's type to "const int".
float sma = ta.sma(close, length = int(LENGTH))

plot(sma)
```

Note that:

-   The [int()](https://www.tradingview.com/pine-script-reference/v6/#fun_int) function removes all fractional information from a “float” value _without_ rounding. For instance, a call such as `int(10.5)` returns a value of 10, not 11. To round a “float” value to the nearest whole number before converting it to “int”, use [math.round()](https://www.tradingview.com/pine-script-reference/v6/#fun_math.round).

For most available types, explicit type casting is useful when defining variables that have initial [na](https://www.tradingview.com/pine-script-reference/v6/#var_na) values or references, as explained in the previous section, [`na` value](https://www.tradingview.com/pine-script-docs/language/type-system/#na-value).

For example, a script can declare a variable that holds an [na](https://www.tradingview.com/pine-script-reference/v6/#var_na) reference of the [label](https://www.tradingview.com/pine-script-reference/v6/#type_label) type in either of the following, equivalent ways:

```pine
// Explicitly specify that the variable can reference `label` objects.
label myLabel = na

// Explicitly cast the `na` instance to the `label` type, causing `myLabel` to inherit the type.
myLabel = label(na)
```

## Tuples {#tuples}

A _tuple_ is a _comma-separated list_ of expressions or identifiers enclosed in square brackets (e.g., `[expr1, expr2, expr3]`). If a structure that creates a local scope, such as a function, [method](https://www.tradingview.com/pine-script-docs/language/methods/), [conditional structure](https://www.tradingview.com/pine-script-docs/language/conditional-structures/), or [loop](https://www.tradingview.com/pine-script-docs/language/loops/), returns more than one result, the code lists the expressions for all the results in the form of a tuple at the end of the structure’s local block.

For example, the following [user-defined function](https://www.tradingview.com/pine-script-docs/language/user-defined-functions/) returns a tuple containing two values. The first item in the tuple is the sum of the function’s `a` and `b` arguments, and the second is the product of those two values:

```pine
//@function Calculates the sum and product of two "float" values.
calcSumAndProduct(float a, float b) =>
    //@variable The sum of `a` and `b`.
    float sum = a + b
    //@variable The product of `a` and `b`.
    float product = a * b
    // Return a tuple containing the `sum` and `product` values.
    [sum, product]
```

When calling this function later in the code, the script must use a [tuple declaration](https://www.tradingview.com/pine-script-docs/language/variable-declarations/#tuple-declarations) to declare one new variable for each value returned by the function to use its data. For example, the `hlSum` and `hlProduct` variables in the following tuple declaration hold the `sum` and `product` values returned by a `calcSumAndProduct()` call:

```pine
// Declare a tuple containing a variable for each value returned by the `calcSumAndProduct()` call.
[hlSum, hlProduct] = calcSumAndProduct(high, low)
```

Note that:

-   In contrast to individual [variable declarations](https://www.tradingview.com/pine-script-docs/language/variable-declarations/), tuple declarations _do not_ support [type keywords](https://www.tradingview.com/pine-script-docs/language/type-system/#types). The compiler automatically determines the type of each variable in a declared tuple.

If a script’s calculations do not require _all_ the values returned by a function or structure, programmers can [use an underscore](https://www.tradingview.com/pine-script-docs/language/variable-declarations/#using-an-underscore-as-an-identifier) as the _identifier_ for one or more returned items in the tuple declaration. If a variable’s identifier is an underscore, that variable is not usable elsewhere in the code, such as in comparisons or arithmetic expressions.

For example, if we do not require the `product` value returned by our `calcSumAndProduct()` function, we can replace the `hlProduct` variable with `_` in our declared tuple:

```pine
// Declare a tuple with `_` as the second identifier, signifying that the script does not use the second returned value.
// The `_` identifier in this tuple is *not* usable elsewhere in the code.
[hlSum, _] = calcSumAndProduct(high, low)
```

In the above examples, the resulting tuple contains two items of the same type (“float”). However, Pine does not restrict tuples to only one type; a single tuple can contain multiple items of _different types_. For example, the custom `chartInfo()` function below returns a five-item tuple containing “int”, “float”, “bool”, “color”, and “string” values:

```pine
//@function Returns information about the current chart.
chartInfo() =>
    //@variable The first visible bar's UNIX time value.
    int firstVisibleTime = chart.left_visible_bar_time
    //@variable The `close` value at the `firstVisibleTime`.
    float firstVisibleClose = ta.valuewhen(ta.cross(time, firstVisibleTime), close, 0)
    //@variable Is `true` if the chart has a standard chart type; `false` otherwise.
    bool isStandard = chart.is_standard
    //@variable The foreground color of the chart.
    color fgColor = chart.fg_color
    //@variable The ticker ID of the current chart.
    string symbol = syminfo.tickerid
    // Return a tuple containing the values.
    [firstVisibleTime, firstVisibleClose, isStandard, fgColor, symbol]
```

Scripts can also pass tuples to the `expression` parameter of `request.*()` functions, enabling them to retrieve _multiple_ series from a single function call. A single call to [request.security()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.security) or another `request.*()` function that requests a tuple of data still counts as _one_ data request, not multiple. See the [Other timeframes and data](https://www.tradingview.com/pine-script-docs/concepts/other-timeframes-and-data/) page to learn more about `request.*()` functions and the data that they can retrieve.

The following code snippet defines a `roundedOHLC()` function that returns a tuple of OHLC prices rounded to the nearest values that are divisible by the symbol’s minimum tick size. We use this function as the `expression` argument in a [request.security()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.security) call to retrieve a tuple containing the symbol’s rounded price values on the “1D” timeframe:

```pine
//@function Returns a tuple of OHLC values, rounded to the nearest tick.
roundedOHLC() =>
    [math.round_to_mintick(open), math.round_to_mintick(high), math.round_to_mintick(low), math.round_to_mintick(close)]

[op, hi, lo, cl] = request.security(syminfo.tickerid, "1D", roundedOHLC())
```

An alternative way to perform the same request is to pass the tuple of rounded values _directly_ to the `expression` parameter of the [request.security()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.security) call. For example:

```pine
[op, hi, lo, cl] = request.security(
     syminfo.tickerid, "1D",
     [math.round_to_mintick(open), math.round_to_mintick(high), math.round_to_mintick(low), math.round_to_mintick(close)]
 )
```

Note that:

-   Only the `request.*()` functions that have an `expression` parameter and the `input.*()` functions that include an `options` parameter support this argument format. No other functions can use tuples as arguments.

Conditional structures and loops can use tuples as their return expressions, enabling them to return multiple values at once after the script exits their scopes. For example, the [if](https://www.tradingview.com/pine-script-reference/v6/#kw_if) statement below returns a two-item tuple from one of its local blocks:

```pine
[v1, v2] = if close > open
    [high, close]
else
    [close, low]
```

The following [switch](https://www.tradingview.com/pine-script-reference/v6/#kw_switch) statement is equivalent to the above [if](https://www.tradingview.com/pine-script-reference/v6/#kw_if) statement:

```pine
[v1, v2] = switch
    close > open => [high, close]
    =>              [close, low]
```

It’s crucial to emphasize that only the _local scopes_ of functions, conditional structures, or loops can return tuples. In contrast to [if](https://www.tradingview.com/pine-script-reference/v6/#kw_if) and [switch](https://www.tradingview.com/pine-script-reference/v6/#kw_switch) statements, [ternary operations](https://www.tradingview.com/pine-script-docs/language/operators/#-ternary-operator) are **not** conditional structures; they are _expressions_ that _do not_ create local scopes. Therefore, they cannot return tuples.

For example, this line of code attempts to return a tuple from a ternary operation, causing a _compilation error_:

```pine
// Causes an error. Only local scopes can return tuples, and the `?:` operator does not create new scopes.
[v1, v2] = close > open ? [high, close] : [close, low]
```

Although all items in a tuple do not have to be of the same _type_, it’s important to note that every item inherits the **same** [type qualifier](https://www.tradingview.com/pine-script-docs/language/type-system/#qualifiers). All items within a tuple _returned_ by a local scope inherit either the “simple” or “series” qualifier, depending on the structure and the items’ types. Therefore, because “series” is the stronger qualifier, all other items in the returned tuple automatically inherit the “series” qualifier if at least one item is qualified as “series”.

For example, the script below defines a `getParameters()` function that returns a two-item tuple. The script attempts to use the values returned by the function as arguments in a [ta.ema()](https://www.tradingview.com/pine-script-reference/v6/#fun_ta.ema) function call, causing a compilation error. The [ta.ema()](https://www.tradingview.com/pine-script-reference/v6/#fun_ta.ema) function requires a `length` argument of the type “simple int”, but the `len` variable’s type is _“series int”_. The value assigned to `len` automatically inherits the “series” qualifier because the `source` argument of the `getParameters()` call is of the type “series float”:

```pine
//@version=6
indicator("Qualified types in tuples demo")

getParameters(float source, simple int length) =>
    // Although the expected type of the `length` parameter is "simple int", the
    // `length` value in the returned tuple inherits the "series" qualifier if the
    // `source` value has that qualifier, because all items in a tuple inherit the *same* qualifier.
    [source, length]

// Declare a tuple containing the values returned by a `getParameters()` call.
// Both variables in this tuple have the "series" qualifier, because `close` is of the type "series float".
[src, len] = getParameters(source = close, length = 20)

// This line causes an error. `ta.ema()` expects a "simple int" `length` argument, but `len` has the type "series int".
plot(ta.ema(source = src, length = len))
```

## Value vs. reference types {#value-vs-reference-types}

> [!TIP]
> This section contains advanced details about the differences between value and reference types. To make the most of this information, we recommend that newcomers to Pine Script start by reading about the available [types](https://www.tradingview.com/pine-script-docs/language/type-system/#types), and then come back to this section to learn more about their differences.

Every type in Pine Script, excluding [void](https://www.tradingview.com/pine-script-docs/language/type-system/#void), is either a [value type](https://www.tradingview.com/pine-script-docs/language/type-system/#value-types) or a [reference type](https://www.tradingview.com/pine-script-docs/language/type-system/#types).

All [fundamental types](https://www.tradingview.com/pine-script-docs/language/type-system/#types), [enum types](https://www.tradingview.com/pine-script-docs/language/type-system/#enum-types), and the _unique types_ for specific function parameters are _value types_. These types directly _represent_ values, which scripts can use in arithmetic, comparison, or logical [operations](https://www.tradingview.com/pine-script-docs/language/operators/). Variables of these types store values. Likewise, expressions that return these types return values. Values can become available at compile time, input time, or runtime. Therefore, they can inherit _any_ [type qualifier](https://www.tradingview.com/pine-script-docs/language/type-system/#qualifiers), depending on their use in the code.

By contrast, [user-defined types (UDTs)](https://www.tradingview.com/pine-script-docs/language/type-system/#user-defined-types) and _special types_ — including [label](https://www.tradingview.com/pine-script-reference/v6/#type_label), [line](https://www.tradingview.com/pine-script-reference/v6/#type_line), [linefill](https://www.tradingview.com/pine-script-reference/v6/#type_linefill), [polyline](https://www.tradingview.com/pine-script-reference/v6/#type_polyline), [box](https://www.tradingview.com/pine-script-reference/v6/#type_box), [table](https://www.tradingview.com/pine-script-reference/v6/#type_table), [chart.point](https://www.tradingview.com/pine-script-reference/v6/#type_chart.point), and [collection types](https://www.tradingview.com/pine-script-docs/language/type-system/#collections) — are _reference types_. These types serve as structures for creating _objects_. An object is **not** a value; it is a logical entity that stores data in a distinct memory location. Each separate object has a unique associated _reference_, similar to a pointer, which identifies the object in memory and enables the script to access its data. Variables of reference types hold these object references; they **do not** store objects directly.

> [!NOTE]
> For simplicity and ease of discussion, we sometimes use the term _“ID”_ as a substitute for _“object reference”_.

Scripts create objects exclusively at _runtime_, using the available constructor functions from the type’s namespace (e.g., [label.new()](https://www.tradingview.com/pine-script-reference/v6/#fun_label.new)). Every call to these functions creates a _new object_ with a _unique reference_. Therefore, unlike value types, reference types automatically inherit the “series” qualifier; they never inherit _weaker_ qualifiers such as “simple” or “const”.

For example, the following script declares a `myLabel` variable and assigns it the result of a [label.new()](https://www.tradingview.com/pine-script-reference/v6/#fun_label.new) function call with constant `x` and `y` arguments on the first bar. Although the script calls [label.new()](https://www.tradingview.com/pine-script-reference/v6/#fun_label.new) only _once_, with “const” arguments, the variable’s type is _“series label”_. The type is **not** “const label”, because every call to the function returns a new, unique [label](https://www.tradingview.com/pine-script-reference/v6/#type_label) reference, which no other call can reproduce:

```pine
//@version=6
indicator("'series' reference demo")

//@variable References a label created on the first bar using "const" arguments.
//          Although the script creates only one label, using constant values, this variable's type is "series label" 
//          because the assigned `label` reference is *unique*. No additional function calls can create the same label 
//          instance or return the same reference.
var label myLabel = label.new(0, 0, "A new 'series' label")
```

Note that:

-   The script creates a label only on the first bar because the variable that stores its reference is declared in the _global scope_ using the [var](https://www.tradingview.com/pine-script-reference/v6/#kw_var) keyword. See the [Declaration modes](https://www.tradingview.com/pine-script-docs/language/variable-declarations/#declaration-modes) section of the [Variable declarations](https://www.tradingview.com/pine-script-docs/language/variable-declarations/) page to learn more.

### Modifying variables vs. objects {#modifying-variables-vs-objects}

Each variable of a [value type](https://www.tradingview.com/pine-script-docs/language/type-system/#value-types) holds an independent value, and the only way to modify that variable’s data is by using the reassignment or compound assignment [operators](https://www.tradingview.com/pine-script-docs/language/operators/). Each use of these operators directly overwrites the stored value, thus removing it from the current execution.

Scripts can also modify variables of [reference types](https://www.tradingview.com/pine-script-docs/language/type-system/#reference-types) with the [:=](https://www.tradingview.com/pine-script-reference/v6/#op_:=) operator, but not a compound assignment operator such as [+=](https://www.tradingview.com/pine-script-reference/v6/#op_+=), because object references are _not compatible_ with arithmetic or logical operations. However, it’s crucial to note that reassigning a variable of a reference type _does not_ directly affect any object; it only assigns _another reference_ to that variable. The object referenced before the operation _can_ remain in memory and affect the script’s results, depending on the type and the script’s logic.

To understand this distinction, consider the following script, which uses a variable to store [label](https://www.tradingview.com/pine-script-reference/v6/#type_label) references on the last historical bar. First, the script initializes the `myLabel` variable with the result of one [label.new()](https://www.tradingview.com/pine-script-reference/v6/#fun_label.new) call. Then, it uses the [:=](https://www.tradingview.com/pine-script-reference/v6/#op_:=) operator to assign the variable the result of a _second_ [label.new()](https://www.tradingview.com/pine-script-reference/v6/#fun_label.new) call. Reassigning the `myLabel` variable only changes the variable’s stored reference; it _does not_ overwrite the _first_ [label](https://www.tradingview.com/pine-script-reference/v6/#type_label) object. The first label _still exists_ in memory. Consequently, this script displays _two_ separate drawings:

```pine
//@version=6
indicator("Reassigning reference-type variables demo")

if barstate.islastconfirmedhistory
    // Create a new `label` object and assign its reference to `myLabel`. 
    label myLabel = label.new(bar_index, 0, "First label")

    // Create another `label` object and assign that object's reference to the variable.
    // This reassignment operation does not overwrite the first label. It only changes the variable's assigned
    // reference. The first object still exists and produces an output on the chart. 
    myLabel := label.new(bar_index, 20, "Second label")
```

Note that:

-   Objects remain in memory until a script no longer uses them. For [drawing types](https://www.tradingview.com/pine-script-docs/language/type-system/#drawing-types), the runtime system automatically maintains a limited number of active objects. It begins deleting those objects, starting with the oldest ones, only if a script reaches its _drawing limit_ (e.g., ~50 labels by default).
-   A script can also explicitly delete objects of drawing types by using the built-in `*.delete()` functions, such as [label.delete()](https://www.tradingview.com/pine-script-reference/v6/#fun_label.delete). For example, if we add the call `label.delete(myLabel)` before the final line in the code above, the script removes the first label before assigning the second label’s reference to the `myLabel` variable.

Because objects are not values, but entities that store data separately, scripts do not modify their data by reassigning the variables that reference them. To access or modify an object’s data, programmers must do either of the following, depending on the type:

-   Use the built-in _getter_ and _setter_ functions available for most special types. For example, [label.get\_x()](https://www.tradingview.com/pine-script-reference/v6/#fun_label.get_x) retrieves the `x` value from a [label](https://www.tradingview.com/pine-script-reference/v6/#type_label) object, and [label.set\_x()](https://www.tradingview.com/pine-script-reference/v6/#fun_label.set_x) updates a label’s `x` value.
-   Use _dot notation_ syntax on a variable of a [UDT](https://www.tradingview.com/pine-script-docs/language/type-system/#user-defined-types) or the [chart.point](https://www.tradingview.com/pine-script-reference/v6/#type_chart.point) type to access the object’s _field_. Then, to change the field’s assigned data, use a reassignment or compound assignment operator after the syntax. For example, `myObj.price` retrieves the `price` field of the object referenced by the `myObj` variable, and `myObj.price := 10` sets that field’s value to 10.

> [!NOTE]
> Because reference types always inherit the “series” qualifier, all data retrieved from an object also inherits the qualifier. Values stored by an object never qualify as “simple”, “input”, or “const”, even if the script constructs the object using values with those weaker [qualifiers](https://www.tradingview.com/pine-script-docs/language/type-system/#qualifiers).

The example below creates a [chart point](https://www.tradingview.com/pine-script-docs/language/type-system/#chart-points) and a [label](https://www.tradingview.com/pine-script-reference/v6/#type_label) instance on the first bar, and then modifies the two objects on every bar. With each execution, the script updates the `price` (“float”) and `index` (“int”) fields of the chart point, then uses its reference in a [label.set\_point()](https://www.tradingview.com/pine-script-reference/v6/#fun_label.set_point) call to change the label’s coordinates. Lastly, the script uses [label.get\_y()](https://www.tradingview.com/pine-script-reference/v6/#fun_label.get_y) to get the label’s `y` value (“float”), then uses a plot to display the value:

```pine
//@version=6
indicator("Modifying objects demo", overlay = true)

//@variable Maintains a persistent reference to one `chart.point` object with an initial `price` field of `na`. 
var chart.point myPoint = chart.point.now(na)

//@variable Maintains a persistent reference to one `label` object initialized using the `myPoint` chart point.
var label myLabel = label.new(myPoint, "Persistent label")

// Update the chart point referenced by `myPoint` on each bar by reassigning the object's *fields*.
myPoint.index := bar_index
myPoint.price := close

// Update the label referenced by `myLabel` using a call to `label.set_point()`. The call uses the `index` field of 
// the chart point for the label's x-coordinate, and the `price` field for the y-coordinate.
label.set_point(myLabel, myPoint)

// Retrieve the y-coordinate from the `myLabel` label, confirming that both persistent objects were modified.
plot(label.get_y(myLabel), "Label y-coordinate")
```

Note that:

-   The [label.set\_point()](https://www.tradingview.com/pine-script-reference/v6/#fun_label.set_point) call in this example uses the `index` field of the chart point to set the label’s `x` value, and it uses the `price` field to set the `y` value. It does not use the `time` field from the chart point for the `x` value, because the default `xloc` property for labels is [xloc.bar\_index](https://www.tradingview.com/pine-script-reference/v6/#const_xloc.bar_index).

#### Modifying global data in local scopes {#modifying-global-data-in-local-scopes}

Every script has one _global_ [scope](https://www.tradingview.com/pine-script-docs/faq/programming/#what-does-scope-mean), and it includes zero or more _local_ scopes from any [conditional structures](https://www.tradingview.com/pine-script-docs/language/conditional-structures/), [loops](https://www.tradingview.com/pine-script-docs/language/loops/), [user-defined functions](https://www.tradingview.com/pine-script-docs/language/user-defined-functions/) or [methods](https://www.tradingview.com/pine-script-docs/language/methods/#user-defined-methods), or other structures. Most structures that create local scopes can access and use any global variables declared above them in the source code, because a script’s local scopes _embed_ into the global scope.

Conditional structures and loops defined in the global scope, as well as the nested structures within them, can also contain [reassignment](https://www.tradingview.com/pine-script-docs/language/operators/#-reassignment-operator) or [compound assignment](https://www.tradingview.com/pine-script-docs/language/operators/#compound-assignment-operators) operations that modify global variables. In other words, either of these structures can directly change the data associated with global variables of value types and reference types.

For example, the script below declares a persistent “int” variable named `counter` in the global scope. Then, it uses the [+=](https://www.tradingview.com/pine-script-reference/v6/#op_+=) and [:=](https://www.tradingview.com/pine-script-reference/v6/#op_:=) operators inside nested [if](https://www.tradingview.com/pine-script-reference/v6/#kw_if) statements to update the variable’s assigned value based on cyclic occurrences of a pseudorandom condition:

```pine
//@version=6
indicator("Modifying global variables in conditional structures demo")

//@variable The number of conditions that occur before the counter value resets.
int cycleSizeInput = input.int(10, "Cycle size", 1)

//@variable A persistent global variable for counting occurrences of a condition in cycles. 
var int counter = 0

// Logic to update `counter` based on a pseudorandom condition.
if math.random() < 0.5
    // Increase the `counter` value by one when the condition occurs.
    counter += 1
    // Reset the `counter` value to 1 if it exceeds the value of `cycleSizeInput`. 
    if counter > cycleSizeInput
        counter := 1

// Plot the `counter` value.
plot(counter, "Counter value")
```

By contrast, user-defined functions and methods _cannot_ use the reassignment or compound assignment operators on global variables, because variables declared outside a [function scope](https://www.tradingview.com/pine-script-docs/language/user-defined-functions/#function-scopes) cannot accept _different_ values _or_ references during the execution of a _function call_. Consequently, functions and methods _cannot modify_ the data associated with global variables of [value types](https://www.tradingview.com/pine-script-docs/language/type-system/#value-types).

> [!NOTE]
> The same limitation applies to function _parameters_. Function definitions cannot contain reassignment or compound assignment operations that modify declared parameters, because the _arguments_ for those parameters cannot change while a function call executes.

Below, we edited the previous script to demonstrate this limitation. The following script version defines an `updateCounter()` function that attempts to modify the global `counter` variable from inside its scope using the same [+=](https://www.tradingview.com/pine-script-reference/v6/#op_+=) and [:=](https://www.tradingview.com/pine-script-reference/v6/#op_:=) operations as the example above. However, because the variable exists _outside_ the function’s definition, the function _cannot_ change its value. As such, a _compilation error_ occurs:

```pine
//@version=6
indicator("Cannot modify global variables in functions demo")

//@variable The number of conditions that occur before the counter value resets.
int cycleSizeInput = input.int(10, "Cycle size", 1)

//@variable A persistent global variable for counting occurrences of a condition in cycles. 
var int counter = 0

//@function Attempts to increment and cyclically reset the `counter` variable based on a pseudorandom condition.
//          This function *does not* compile, because modifying global variables in function scopes is *not allowed*.
updateCounter() =>
    if math.random() < 0.5
        // Attempting to increment `counter` causes a compilation error.
        // The variable's value *cannot change* during the execution of an `updateCounter()` call. 
        counter += 1
        if counter > cycleSizeInput
            // Reassigning the `counter` variable causes the same error. 
            counter := 1

updateCounter() // This call does not work. 

plot(counter, "Counter value")
```

To modify global data from within the scope of a function call, programmers can use global variables of [reference types](https://www.tradingview.com/pine-script-docs/language/type-system/#reference-types) instead of value types in the function’s definition. As explained in the [previous section](https://www.tradingview.com/pine-script-docs/language/type-system/#modifying-variables-vs-objects), scripts do _not_ modify objects of these types by reassigning the variables that reference them. Instead, they _reassign fields_ or use _setter functions_, depending on the type, to update the data that an object stores _elsewhere_ in memory. Therefore, because a variable’s assigned reference _does not change_ after a script modifies an object, functions _can_ change the data associated with global variables of reference types, unlike those of value types.

For example, the script version below declares a [user-defined type](https://www.tradingview.com/pine-script-docs/language/type-system/#user-defined-types) named `Counter` with an “int” field named `value`. Then, it creates a new object of that type with a call to `Counter.new()`, and assigns the returned reference to a persistent global variable named `myCounter`. The `updateCounter()` function in this script uses the [+=](https://www.tradingview.com/pine-script-reference/v6/#op_+=) and [:=](https://www.tradingview.com/pine-script-reference/v6/#op_:=) operators on the `value` _field_ of the `Counter` object referenced by the `myCounter` variable rather than directly reassigning the variable. Although the `value` field’s assigned value can change during the execution of an `updateCounter()` call, the global variable itself remains unchanged; it still holds the reference to the _same_ `Counter` object while the call executes. As a result, the script compiles successfully:

```pine
//@version=6
indicator("Modifying globally referenced objects in functions demo")

//@variable The number of conditions that occur before the counter value resets.
int cycleSizeInput = input.int(10, "Cycle size", 1)

//@type         A custom type for creating objects that store counter data.
//@field value  The counter value, initialized to 0 by default. 
type Counter
    int value = 0

//@variable A persistent global variable that holds the reference of a `Counter` object.
var Counter myCounter = Counter.new()

//@function Increments and cyclically resets the `value` field of the object referenced by `myCounter` based on a 
//          pseudorandom condition.
//          This function does *not* cause an error, because it does not modify the global variable. 
updateCounter() =>
    if math.random() < 0.5
        // Increase the `value` *field* of the `Counter` object referenced by `myCounter` when the condition occurs.
        myCounter.value += 1
        // Reset the `value` field to 1 if it exceeds the value of `cycleSizeInput`. 
        if myCounter.value > cycleSizeInput
            myCounter.value := 1

// Modify the object referenced by `myCounter`. This function call works without issue.
updateCounter()

// Plot the value of the object's `value` field, i.e., the condition counter.
plot(myCounter.value, "Counter value")
```

### Copies vs. shared references {#copies-vs-shared-references}

Variables of value types hold values that act as _independent copies_, because the only way to modify their data is through reassignment. If a script directly assigns one variable’s value to another variable, it can change either variable’s data later without affecting the other variable’s data in any way.

For example, the following script initializes a `myVar1` variable with a value of 10, and then initializes a `myVar2` variable using `myVar1`. Afterward, the script adds 10 to `myVar1` with the [+=](https://www.tradingview.com/pine-script-reference/v6/#op_+=) operator, and plots the values of both variables on the chart. The script plots two different values (20 and 10), because changes to the value of `myVar1` do not affect the data accessed by `myVar2`:

```pine
//@version=6
indicator("Value type independence demo")

// Initialize the first variable with a value of 10.
int myVar1 = 10
// Initialize the second variable using the first. This variable's value is now 10.
int myVar2 = myVar1

// Increase the first variable's value by 10. Now, the value of `myVar1` is 20, but the value of `myVar2` is still 10.
myVar1 += 10

// Plot both values for comparison.
plot(myVar1, "First variable", color.blue, 3)
plot(myVar2, "Second variable", color.purple, 3)
```

The same behavior does not apply to variables of reference types. Assigning the reference stored by one variable to another **does not** create a new _copy_ of an object. Instead, both variables refer to the **same** object in memory. As a result, the script can access or change that object’s data through _either_ variable and produce the same results.

The following example demonstrates this behavior. On the last historical bar, the script creates a new label with [label.new()](https://www.tradingview.com/pine-script-reference/v6/#fun_label.new) and assigns the returned reference to the `myLabel1` variable. Then, it initializes the `myLabel2` variable using `myLabel1`. The script calls [label.set\_color()](https://www.tradingview.com/pine-script-reference/v6/#fun_label.set_color) to modify the label referenced by `myLabel1`, and then calls [label.set\_style()](https://www.tradingview.com/pine-script-reference/v6/#fun_label.set_style) and [label.set\_text()](https://www.tradingview.com/pine-script-reference/v6/#fun_label.set_text) to modify the one referenced by `myLabel2`.

A newcomer to reference types might expect this script to display _two_ separate labels, with different colors, orientation, and text. However, the script shows only **one** label on the chart, and that label includes the changes from all `label.set_*()` calls. Modifying the label referenced by `myLabel2` directly affects the one referenced by `myLabel1`, and vice versa, because both variables refer to the **same** [label](https://www.tradingview.com/pine-script-reference/v6/#type_label) object:

```pine
//@version=6
indicator("Shared object references demo")

if barstate.islastconfirmedhistory
    // Create a new label and assign its reference to a variable.
    label myLabel1 = label.new(bar_index, 0, "First label", color = color.green, size = size.large)
    // Initialize a second variable using the `myLabel1` variable.
    // This variable declaration *does not* copy the label referenced by `myLabel1`; it only copies that variable's 
    // *reference* to the new variable. 
    label myLabel2 = myLabel1

    // Change the color of the label referenced by `myLabel1`.
    label.set_color(myLabel1, color.red)
    // Update the style and text of the label referenced by `myLabel2`.
    // Because both variables refer to the *same object*, all the label changes affect that one object, 
    // regardless of which variable the script uses in the `label.set_*()` calls. 
    label.set_style(myLabel2, label.style_label_up)
    label.set_text(myLabel2, "Second label")
```

Most reference types, including [user-defined types](https://www.tradingview.com/pine-script-docs/language/type-system/#user-defined-types), feature a built-in `*.copy()` function. This function creates a new, _independent_ object that contains the same data as the original object, and that new object has a _unique reference_. The script can modify the copied object’s data without directly affecting the original.

In the following example, we changed the previous script to initialize `myLabel2` using the expression `label.copy(myLabel1)`, which creates an independent copy of the label referenced by `myLabel1` and returns a new reference. Now, `myLabel1` and `myLabel2` refer to two _separate_ labels, and changes to the label referenced by one of the variables do not affect the other:

```pine
//@version=6
indicator("Copied objects demo")

if barstate.islastconfirmedhistory
    // Create a new label and assign its reference to a variable.
    label myLabel1 = label.new(bar_index, 0, "First label", color = color.green, size = size.large)
    // Initialize a second variable using `label.copy(myLabel1)`. This variable now references an independent copy 
    // of the initial label instead of pointing to the same object as `myLabel1`, and the script now displays two labels 
    // on the chart.
    label myLabel2 = label.copy(myLabel1)

    // Now that `myLabel2` refers to a different `label` object than `myLabel1`, this call does not affect that object.
    label.set_color(myLabel1, color.red)
    // Likewise, these two calls do not affect the label referenced by `myLabel1`.
    label.set_style(myLabel2, label.style_label_up)
    label.set_text(myLabel2, "Second label")
```

> [!NOTE]
> The `*.copy()` function creates a _shallow copy_ of an object, not a _deep copy_. If a script uses this function to copy a [collection](https://www.tradingview.com/pine-script-docs/language/type-system/#collections) or [UDT](https://www.tradingview.com/pine-script-docs/language/type-system/#user-defined-types) instance that stores _other object references_, the contents of the copied instance refer to the **same objects** as the original instance. See the [Copying objects](https://www.tradingview.com/pine-script-docs/language/objects/#copying-objects) section of the [Objects](language/objects/) page for more information.

### Using `const` with reference types {#using-const-with-reference-types}

Scripts can use the [const](https://www.tradingview.com/pine-script-reference/v6/#type_const) keyword when declaring variables of most [reference types](https://www.tradingview.com/pine-script-docs/language/type-system/#reference-types), except for [plot](https://www.tradingview.com/pine-script-docs/language/type-system/#plot-and-hline), [hline](https://www.tradingview.com/pine-script-docs/language/type-system/#plot-and-hline), and [user-defined types](https://www.tradingview.com/pine-script-docs/language/type-system/#user-defined-types). However, with reference types, the keyword behaves _differently_ than it does with [value types](https://www.tradingview.com/pine-script-docs/language/type-system/#value-types).

Recall that for a variable of a value type, the [const](https://www.tradingview.com/pine-script-reference/v6/#type_const) keyword directly _restricts_ the [qualifier](https://www.tradingview.com/pine-script-docs/language/type-system/#qualifiers) of that variable to “const”, _and_ it prevents the script from using the reassignment or compound assignment [operators](https://www.tradingview.com/pine-script-docs/language/operators/) to modify that variable — even if the assigned value from those operations is otherwise a constant.

For variables of reference types, using the [const](https://www.tradingview.com/pine-script-reference/v6/#type_const) keyword to declare them also prevents a script from reassigning those variables. However, in contrast to its behavior with value types, the keyword **does not** set the _qualifier_ of a reference-type variable to “const”. As explained in previous sections, reference types automatically inherit the “series” qualifier, because each call to a function that creates objects produces a _new_ object with a _unique_ reference — any call to the function in the code never returns the same object reference more than once.

For example, the script below creates an [array](https://www.tradingview.com/pine-script-docs/language/arrays/) of pseudorandom “float” values using [array.from()](https://www.tradingview.com/pine-script-reference/v6/#fun_array.from), and then assigns the returned reference to a variable declared using the [const](https://www.tradingview.com/pine-script-reference/v6/#type_const) keyword on each bar. During each execution, the [array.from()](https://www.tradingview.com/pine-script-reference/v6/#fun_array.from) call creates a _new array_ and returns a unique “series” reference. However, this script does _not_ cause an error, even though the variable’s _qualifier_ is “series”, because the variable’s assigned reference remains _consistent_ for the rest of each execution:

```pine
//@version=6
indicator("Using `const` with reference types demo")

//@variable Holds a reference to an array of three pseudorandom "float" values.
//          Although the variable is declared using `const`, the reference returned by `array.from()` has the "series" 
//          qualifier, because each execution creates a new, unique array object. Additionally, all elements in the 
//          array are of the type "series float".
//          This *does not* cause an error, because the script does not *reassign* the variable during any execution.
const array<float> randArray = array.from(math.random(), math.random(), math.random())

// Plot the sum of the `randArray` elements.
plot(randArray.sum())
```

However, if we use the [:=](https://www.tradingview.com/pine-script-reference/v6/#op_:=) operator to reassign the `randArray` variable, a compilation error occurs, because the [const](https://www.tradingview.com/pine-script-reference/v6/#type_const) keyword prevents the script from assigning _another_ array reference to the variable during each execution. For example:

```pine
//@version=6
indicator("Invalid reassignment demo")

//@variable Holds a reference to an array of three pseudorandom "float" values.
//          Although the variable is declared using `const`, the reference returned by `array.from()` has the "series" 
//          qualifier, because each execution creates a new, unique array object. Additionally, all elements in the 
//          array are of the type "series float".
const array<float> randArray = array.from(math.random(), math.random(), math.random())

// This line causes an error, because the `const` keyword prevents reassignment operations on the `randArray` variable.
randArray := array.new<float>(3, 0.0)

// Plot the sum of the `randArray` elements.
plot(randArray.sum())
```

It’s important to note that the [const](https://www.tradingview.com/pine-script-reference/v6/#type_const) keyword _does not_ directly prevent a script from modifying a [collection](https://www.tradingview.com/pine-script-docs/language/type-system/#collections) or [drawing object](https://www.tradingview.com/pine-script-docs/language/type-system/#drawing-types) referenced by a variable or function parameter. Scripts can still use the available _setter functions_ to change that object’s data, because those functions _do not_ affect the identifier’s associated reference.

Below, we edited our script by including a call to [array.set()](https://www.tradingview.com/pine-script-reference/v6/#fun_array.set). The call sets the first element of the array referenced by `randArray` to 0. Although the contents of the array change _after_ each `randArray` declaration, the variable’s assigned reference does not, so no error occurs:

```pine
//@version=6
indicator("Valid modification demo")

//@variable Holds a reference to an array of three pseudorandom "float" values.
//          Although the variable is declared using `const`, the reference returned by `array.from()` has the "series" 
//          qualifier, because each execution creates a new, unique array object. Additionally, all elements in the 
//          array are of the type "series float".
const array<float> randArray = array.from(math.random(), math.random(), math.random())

// This line *does not* cause an error, even though it changes the array's contents, because `randArray` still refers 
// to the *same* array instance for the rest of the execution.
array.set(randArray, 0, 0.0)

// Plot the sum of the `randArray` elements.
plot(randArray.sum())
```
