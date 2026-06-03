---
title: Built-ins
source: https://www.tradingview.com/pine-script-docs/language/built-ins/
section: language
---

# Built-ins {#built-ins}

## Introduction {#introduction}

Pine Script® has hundreds of _built-in_ variables and functions. They provide your scripts with valuable information and make calculations for you, dispensing you from coding them. The better you know the built-ins, the more you will be able to do with your Pine scripts.

On this page, we present an overview of some of Pine’s built-in variables and functions. They will be covered in more detail in the pages of this manual covering specific themes.

All built-in variables and functions are defined in the Pine Script [v6 Reference Manual](https://www.tradingview.com/pine-script-reference/v6/). It is called a “Reference Manual” because it is the definitive reference on the Pine Script language. It is an essential tool that will accompany you anytime you code in Pine, whether you are a beginner or an expert. If you are learning your first programming language, make the [Reference Manual](https://www.tradingview.com/pine-script-reference/v6/) your friend. Ignoring it will make your programming experience with Pine Script difficult and frustrating — as it would with any other programming language.

Variables and functions in the same family share the same _namespace_, which is a prefix to the function’s name. The [ta.sma()](https://www.tradingview.com/pine-script-reference/v6/#fun_ta.sma) function, for example, is in the `ta` namespace, which stands for “technical analysis”. A namespace can contain both variables and functions.

Some variables have function versions as well, e.g.:

-   The [ta.tr](https://www.tradingview.com/pine-script-reference/v6/#fun_ta.tr) variable returns the “True Range” of the current bar. The [ta.tr(true)](https://www.tradingview.com/pine-script-reference/v6/#fun_ta.tr) function call also returns the “True Range”, but when the previous [close](https://www.tradingview.com/pine-script-reference/v6/#var_close) value which is normally needed to calculate it is [na](https://www.tradingview.com/pine-script-reference/v6/#var_na), it calculates using `high - low` instead.
-   The [time](https://www.tradingview.com/pine-script-reference/v6/#var_time) variable gives the time at the [open](https://www.tradingview.com/pine-script-reference/v6/#var_open) of the current bar. The [time(timeframe)](https://www.tradingview.com/pine-script-reference/v6/#fun_time) function returns the time of the bar’s [open](https://www.tradingview.com/pine-script-reference/v6/#var_open) from the `timeframe` specified, even if the chart’s timeframe is different. The [time(timeframe, session)](https://www.tradingview.com/pine-script-reference/v6/#fun_time) function returns the time of the bar’s [open](https://www.tradingview.com/pine-script-reference/v6/#var_open) from the `timeframe` specified, but only if it is within the `session` time. The [time(timeframe, session, timezone)](https://www.tradingview.com/pine-script-reference/v6/#fun_time) function returns the time of the bar’s [open](https://www.tradingview.com/pine-script-reference/v6/#var_open) from the `timeframe` specified, but only if it is within the `session` time in the specified `timezone`.

## Built-in variables {#built-in-variables}

Built-in variables exist for different purposes. These are a few examples:

-   Price- and volume-related variables: [open](https://www.tradingview.com/pine-script-reference/v6/#var_open), [high](https://www.tradingview.com/pine-script-reference/v6/#var_high), [low](https://www.tradingview.com/pine-script-reference/v6/#var_low), [close](https://www.tradingview.com/pine-script-reference/v6/#var_close), [hl2](https://www.tradingview.com/pine-script-reference/v6/#var_hl2), [hlc3](https://www.tradingview.com/pine-script-reference/v6/#var_hlc3), [ohlc4](https://www.tradingview.com/pine-script-reference/v6/#var_ohlc4), and [volume](https://www.tradingview.com/pine-script-reference/v6/#var_volume).
-   Symbol-related information in the `syminfo` namespace: [syminfo.basecurrency](https://www.tradingview.com/pine-script-reference/v6/#var_syminfo.basecurrency), [syminfo.currency](https://www.tradingview.com/pine-script-reference/v6/#var_syminfo.currency), [syminfo.description](https://www.tradingview.com/pine-script-reference/v6/#var_syminfo.description), [syminfo.main\_tickerid](https://www.tradingview.com/pine-script-reference/v6/#var_syminfo.main_tickerid), [syminfo.mincontract](https://www.tradingview.com/pine-script-reference/v6/#var_syminfo.mincontract), [syminfo.mintick](https://www.tradingview.com/pine-script-reference/v6/#var_syminfo.mintick), [syminfo.pointvalue](https://www.tradingview.com/pine-script-reference/v6/#var_syminfo.pointvalue), [syminfo.prefix](https://www.tradingview.com/pine-script-reference/v6/#var_syminfo.prefix), [syminfo.root](https://www.tradingview.com/pine-script-reference/v6/#var_syminfo.root), [syminfo.session](https://www.tradingview.com/pine-script-reference/v6/#var_syminfo.session), [syminfo.ticker](https://www.tradingview.com/pine-script-reference/v6/#var_syminfo.ticker), [syminfo.tickerid](https://www.tradingview.com/pine-script-reference/v6/#var_syminfo.tickerid), [syminfo.timezone](https://www.tradingview.com/pine-script-reference/v6/#var_syminfo.timezone), and [syminfo.type](https://www.tradingview.com/pine-script-reference/v6/#var_syminfo.type).
-   Timeframe (a.k.a. “interval” or “resolution”, e.g., 15sec, 30min, 60min, 1D, 3M) variables in the `timeframe` namespace: [timeframe.isseconds](https://www.tradingview.com/pine-script-reference/v6/#var_timeframe.isseconds), [timeframe.isminutes](https://www.tradingview.com/pine-script-reference/v6/#var_timeframe.isminutes), [timeframe.isintraday](https://www.tradingview.com/pine-script-reference/v6/#var_timeframe.isintraday), [timeframe.isdaily](https://www.tradingview.com/pine-script-reference/v6/#var_timeframe.isdaily), [timeframe.isweekly](https://www.tradingview.com/pine-script-reference/v6/#var_timeframe.isweekly), [timeframe.ismonthly](https://www.tradingview.com/pine-script-reference/v6/#var_timeframe.ismonthly), [timeframe.isdwm](https://www.tradingview.com/pine-script-reference/v6/#var_timeframe.isdwm), [timeframe.multiplier](https://www.tradingview.com/pine-script-reference/v6/#var_timeframe.multiplier), [timeframe.main\_period](https://www.tradingview.com/pine-script-reference/v6/#var_timeframe.main_period), and [timeframe.period](https://www.tradingview.com/pine-script-reference/v6/#var_timeframe.period).
-   Bar states in the `barstate` namespace (see the [Bar states](https://www.tradingview.com/pine-script-docs/concepts/bar-states/) page): [barstate.isconfirmed](https://www.tradingview.com/pine-script-reference/v6/#var_barstate.isconfirmed), [barstate.isfirst](https://www.tradingview.com/pine-script-reference/v6/#var_barstate.isfirst), [barstate.ishistory](https://www.tradingview.com/pine-script-reference/v6/#var_barstate.ishistory), [barstate.islast](https://www.tradingview.com/pine-script-reference/v6/#var_barstate.islast), [barstate.islastconfirmedhistory](https://www.tradingview.com/pine-script-reference/v6/#var_barstate.islastconfirmedhistory), [barstate.isnew](https://www.tradingview.com/pine-script-reference/v6/#var_barstate.isnew), and [barstate.isrealtime](https://www.tradingview.com/pine-script-reference/v6/#var_barstate.isrealtime).
-   Strategy-related information in the `strategy` namespace: [strategy.equity](https://www.tradingview.com/pine-script-reference/v6/#var_strategy.equity), [strategy.initial\_capital](https://www.tradingview.com/pine-script-reference/v6/#var_strategy.initial_capital), [strategy.grossloss](https://www.tradingview.com/pine-script-reference/v6/#var_strategy.grossloss), [strategy.grossprofit](https://www.tradingview.com/pine-script-reference/v6/#var_strategy.grossprofit), [strategy.wintrades](https://www.tradingview.com/pine-script-reference/v6/#var_strategy.wintrades), [strategy.losstrades](https://www.tradingview.com/pine-script-reference/v6/#var_strategy.losstrades), [strategy.position\_size](https://www.tradingview.com/pine-script-reference/v6/#var_strategy.position_size), [strategy.position\_avg\_price](https://www.tradingview.com/pine-script-reference/v6/#var_strategy.position_avg_price), [strategy.wintrades](https://www.tradingview.com/pine-script-reference/v6/#var_strategy.wintrades), etc.

## Built-in functions {#built-in-functions}

Many functions are used for the result(s) they return. These are a few examples:

-   Math-related functions in the `math` namespace: [math.abs()](https://www.tradingview.com/pine-script-reference/v6/#fun_math.abs), [math.log()](https://www.tradingview.com/pine-script-reference/v6/#fun_math.log), [math.max()](https://www.tradingview.com/pine-script-reference/v6/#fun_math.max), [math.random()](https://www.tradingview.com/pine-script-reference/v6/#fun_math.random), [math.round\_to\_mintick()](https://www.tradingview.com/pine-script-reference/v6/#fun_math.round_to_mintick), etc.
-   Technical indicators in the `ta` namespace: [ta.sma()](https://www.tradingview.com/pine-script-reference/v6/#fun_ta.sma), [ta.ema()](https://www.tradingview.com/pine-script-reference/v6/#fun_ta.ema), [ta.macd()](https://www.tradingview.com/pine-script-reference/v6/#fun_ta.macd), [ta.rsi()](https://www.tradingview.com/pine-script-reference/v6/#fun_ta.rsi), [ta.supertrend()](https://www.tradingview.com/pine-script-reference/v6/#fun_ta.supertrend), etc.
-   Support functions often used to calculate technical indicators in the `ta` namespace: [ta.barssince()](https://www.tradingview.com/pine-script-reference/v6/#fun_ta.barssince), [ta.crossover()](https://www.tradingview.com/pine-script-reference/v6/#fun_ta.crossover), [ta.highest()](https://www.tradingview.com/pine-script-reference/v6/#fun_ta.highest), etc.
-   Functions to request data from other symbols or timeframes in the `request` namespace: [request.dividends()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.dividends), [request.earnings()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.earnings), [request.financial()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.financial), [request.quandl()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.quandl), [request.security()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.security), [request.splits()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.splits).
-   Functions to manipulate strings in the `str` namespace: [str.format()](https://www.tradingview.com/pine-script-reference/v6/#fun_str.format), [str.length()](https://www.tradingview.com/pine-script-reference/v6/#fun_str.length), [str.tonumber()](https://www.tradingview.com/pine-script-reference/v6/#fun_str.tonumber), [str.tostring()](https://www.tradingview.com/pine-script-reference/v6/#fun_str.tostring), etc.
-   Functions used to define the input values that script users can modify in the script’s “Settings/Inputs” tab, in the `input` namespace: [input()](https://www.tradingview.com/pine-script-reference/v6/#fun_input), [input.color()](https://www.tradingview.com/pine-script-reference/v6/#fun_input.color), [input.int()](https://www.tradingview.com/pine-script-reference/v6/#fun_input.int), [input.session()](https://www.tradingview.com/pine-script-reference/v6/#fun_input.session), [input.symbol()](https://www.tradingview.com/pine-script-reference/v6/#fun_input.symbol), etc.
-   Functions used to manipulate colors in the `color` namespace: [color.from\_gradient()](https://www.tradingview.com/pine-script-reference/v6/#fun_color.from_gradient), [color.rgb()](https://www.tradingview.com/pine-script-reference/v6/#fun_color.rgb), [color.new()](https://www.tradingview.com/pine-script-reference/v6/#fun_color.new), etc.

Some functions do not return a result but are used for their side effects, which means they do something, even if they don’t return a result:

-   Functions used as a declaration statement defining one of three types of Pine scripts, and its properties. Each script must begin with a call to one of these functions: [indicator()](https://www.tradingview.com/pine-script-reference/v6/#fun_indicator), [strategy()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy) or [library()](https://www.tradingview.com/pine-script-reference/v6/#fun_library).
-   Plotting or coloring functions: [bgcolor()](https://www.tradingview.com/pine-script-reference/v6/#fun_bgcolor), [plotbar()](https://www.tradingview.com/pine-script-reference/v6/#fun_plotbar), [plotcandle()](https://www.tradingview.com/pine-script-reference/v6/#fun_plotcandle), [plotchar()](https://www.tradingview.com/pine-script-reference/v6/#fun_plotchar), [plotshape()](https://www.tradingview.com/pine-script-reference/v6/#fun_plotshape), [fill()](https://www.tradingview.com/pine-script-reference/v6/#fun_fill).
-   Strategy functions placing orders, in the `strategy` namespace: [strategy.cancel()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.cancel), [strategy.close()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.close), [strategy.entry()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.entry), [strategy.exit()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.exit), [strategy.order()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.order), etc.
-   Strategy functions returning information on indivdual past trades, in the `strategy` namespace: [strategy.closedtrades.entry\_bar\_index()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.closedtrades.entry_bar_index), [strategy.closedtrades.entry\_price()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.closedtrades.entry_price), [strategy.closedtrades.entry\_time()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.closedtrades.entry_time), [strategy.closedtrades.exit\_bar\_index()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.closedtrades.exit_bar_index), [strategy.closedtrades.max\_drawdown()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.closedtrades.max_drawdown), [strategy.closedtrades.max\_runup()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.closedtrades.max_runup), [strategy.closedtrades.profit()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.closedtrades.profit), etc.
-   Functions to generate alert events: [alert()](https://www.tradingview.com/pine-script-reference/v6/#fun_alert) and [alertcondition()](https://www.tradingview.com/pine-script-reference/v6/#fun_alertcondition).

Other functions return a result, but we don’t always use it, e.g.: [hline()](https://www.tradingview.com/pine-script-reference/v6/#fun_hline), [plot()](https://www.tradingview.com/pine-script-reference/v6/#fun_plot), [array.pop()](https://www.tradingview.com/pine-script-reference/v6/#fun_array.pop), [label.new()](https://www.tradingview.com/pine-script-reference/v6/#fun_label.new), etc.

All built-in functions are defined in the Pine Script [v6 Reference Manual](https://www.tradingview.com/pine-script-reference/v6/). You can click on any of the function names listed here to go to its entry in the Reference Manual, which documents the function’s signature, i.e., the list of _parameters_ it accepts and the qualified type of the value(s) it returns (a function can return more than one result). The Reference Manual entry will also list, for each parameter:

-   Its name.
-   The qualified type of the value it requires (we use _argument_ to name the values passed to a function when calling it).
-   If the parameter is required or not.

All built-in functions have one or more parameters defined in their signature. Not all parameters are required for every function.

Let’s look at the [ta.vwma()](https://www.tradingview.com/pine-script-reference/v6/#fun_ta.vwma) function, which returns the volume-weighted moving average of a source value. This is its entry in the Reference Manual:

The entry gives us the information we need to use it:

-   What the function does.
-   Its signature (or definition):

```
ta.vwma(source, length) → series float
```

-   The parameters it includes: `source` and `length`
-   The qualified type of the result it returns: “series float”.
-   An example showing it in use: `plot(ta.vwma(close, 15))`.
-   An example showing what it does, but in long form, so you can better understand its calculations. Note that this is meant to explain --- not as usable code, because it is more complicated and takes longer to execute. There are only disadvantages to using the long form.
-   The “RETURNS” section explains exacty what value the function returns.
-   The “ARGUMENTS” section lists each parameter and gives the critical information concerning what qualified type is required for arguments used when calling the function.
-   The “SEE ALSO” section refers you to related Reference Manual entries.

This is a call to the function in a line of code that declares a `myVwma` variable and assigns the result of `ta.vwma(close, 20)` to it:

```pine
myVwma = ta.vwma(close, 20)
```

Note that:

-   We use the built-in variable [close](https://www.tradingview.com/pine-script-reference/v6/#var_close) as the argument for the `source` parameter.
-   We use `20` as the argument for the `length` parameter.
-   If placed in the global scope (i.e., starting in a line’s first position), it will be executed by the Pine Script runtime on each bar of the chart.

We can also use the parameter names when calling the function. Parameter names are called _keyword arguments_ when used in a function call:

```pine
myVwma = ta.vwma(source = close, length = 20)
```

You can change the position of arguments when using keyword arguments, but only if you use them for all your arguments. When calling functions with many parameters such as [indicator()](https://www.tradingview.com/pine-script-reference/v6/#fun_indicator), you can also forego keyword arguments for the first arguments, as long as you don’t skip any. If you skip some, you must then use keyword arguments so the Pine Script compiler can figure out which parameter they correspond to, e.g.:

```pine
indicator("Example", "Ex", true, max_bars_back = 100)
```

Mixing things up this way is not allowed:

```pine
indicator(precision = 3, "Example") // Compilation error!
```

**When calling built-ins, it is critical to ensure that the arguments you use are of the required qualified type, which will vary for each parameter.**

To learn how to do this, one needs to understand Pine Script’s [type system](https://www.tradingview.com/pine-script-docs/language/type-system/). The Reference Manual entry for each built-in function includes an “ARGUMENTS” section which lists the qualified type required for the argument supplied to each of the function’s parameters.
