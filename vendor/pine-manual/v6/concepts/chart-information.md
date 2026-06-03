---
title: Chart information
source: https://www.tradingview.com/pine-script-docs/concepts/chart-information/
section: concepts
---

# Chart information {#chart-information}

## Introduction {#introduction}

The way scripts can obtain information about the chart and symbol they are currently running on is through a subset of Pine Script®‘s [built-in variables](https://www.tradingview.com/pine-script-docs/language/built-ins/#built-in-variables). The ones we cover here allow scripts to access information relating to:

-   The chart’s prices and volume
-   The chart’s symbol
-   The chart’s timeframe
-   The session (or time period) the symbol trades on

## Prices and volume {#prices-and-volume}

The built-in variables for OHLCV values are:

-   [open](https://www.tradingview.com/pine-script-reference/v6/#var_open): the bar’s opening price.
-   [high](https://www.tradingview.com/pine-script-reference/v6/#var_high): the bar’s highest price, or the highest price reached during the realtime bar’s elapsed time.
-   [low](https://www.tradingview.com/pine-script-reference/v6/#var_low): the bar’s lowest price, or the lowest price reached during the realtime bar’s elapsed time.
-   [close](https://www.tradingview.com/pine-script-reference/v6/#var_close): the bar’s closing price, or the **current price** in the realtime bar.
-   [volume](https://www.tradingview.com/pine-script-reference/v6/#var_volume): the volume traded during the bar, or the volume traded during the realtime bar’s elapsed time. The unit of volume information varies with the instrument. It is in shares for stocks, in lots for forex, in contracts for futures, in the base currency for crypto, etc.

Other values are available through:

-   [hl2](https://www.tradingview.com/pine-script-reference/v6/#var_hl2): the average of the bar’s [high](https://www.tradingview.com/pine-script-reference/v6/#var_high) and [low](https://www.tradingview.com/pine-script-reference/v6/#var_low) values.
-   [hlc3](https://www.tradingview.com/pine-script-reference/v6/#var_hlc3): the average of the bar’s [high](https://www.tradingview.com/pine-script-reference/v6/#var_high), [low](https://www.tradingview.com/pine-script-reference/v6/#var_low) and [close](https://www.tradingview.com/pine-script-reference/v6/#var_close) values.
-   [ohlc4](https://www.tradingview.com/pine-script-reference/v6/#var_ohlc4): the average of the bar’s [open](https://www.tradingview.com/pine-script-reference/v6/#var_open), [high](https://www.tradingview.com/pine-script-reference/v6/#var_high), [low](https://www.tradingview.com/pine-script-reference/v6/#var_low) and [close](https://www.tradingview.com/pine-script-reference/v6/#var_close) values.

On historical bars, the values of the above variables do not vary during the bar because only OHLCV information is available on them. When running on historical bars, scripts execute on the bar’s [close](https://www.tradingview.com/pine-script-reference/v6/#var_close), when all the bar’s information is known and cannot change during the script’s execution on the bar.

Realtime bars are another story altogether. When indicators (or strategies using `calc_on_every_tick = true`) run in realtime, the values of the above variables (except [open](https://www.tradingview.com/pine-script-reference/v6/#var_open)) will vary between successive iterations of the script on the realtime bar, because they represent their **current** value at one point in time during the progress of the realtime bar. This may lead to one form of [repainting](https://www.tradingview.com/pine-script-docs/concepts/repainting/). See the page on Pine Script’s [execution model](https://www.tradingview.com/pine-script-docs/language/execution-model/) for more details.

The [\[\]](https://www.tradingview.com/pine-script-reference/v6/#op_[]) [history-referencing operator](https://www.tradingview.com/pine-script-docs/language/operators/#-history-referencing-operator) can be used to refer to past values of the built-in variables, e.g., `close[1]` refers to the value of [close](https://www.tradingview.com/pine-script-reference/v6/#var_close) on the previous bar, relative to the particular bar the script is executing on.

## Symbol information {#symbol-information}

Built-in variables in the `syminfo` namespace provide scripts with information on the symbol of the chart the script is running on. This information changes every time a script user changes the chart’s symbol. The script then re-executes on all the chart’s bars using the new values of the built-in variables:

-   [syminfo.basecurrency](https://www.tradingview.com/pine-script-reference/v6/#var_syminfo.basecurrency): the base currency, e.g., “BTC” in “BTCUSD”, or “EUR” in “EURUSD”.
-   [syminfo.currency](https://www.tradingview.com/pine-script-reference/v6/#var_syminfo.currency): the quote currency, e.g., “USD” in “BTCUSD”, or “CAD” in “USDCAD”.
-   [syminfo.description](https://www.tradingview.com/pine-script-reference/v6/#var_syminfo.description): The long description of the symbol.
-   [syminfo.main\_tickerid](https://www.tradingview.com/pine-script-reference/v6/#var_syminfo.main_tickerid): The symbol’s _main_ ticker identifier. It behaves almost identically to [syminfo.tickerid](https://www.tradingview.com/pine-script-reference/v6/#var_syminfo.tickerid), referencing the symbol’s exchange prefix, name, and additional ticker data. However, this variable _always_ represents the _current_ chart’s ticker ID, even within requested contexts.
-   [syminfo.mincontract](https://www.tradingview.com/pine-script-reference/v6/#var_syminfo.mincontract): The symbol’s smallest tradable amount, which is set by its exchange. For example, the minimum for NASDAQ asset “AAPL” is 1 token, while the minimum for BITSTAMP cryptocurrency “ETHUSD” is 0.0001 tokens.
-   [syminfo.mintick](https://www.tradingview.com/pine-script-reference/v6/#var_syminfo.mintick): The symbol’s tick value, or the minimum increment price can move in. Not to be confused with _pips_ or _points_. On “ES1!” (“S&P 500 E-Mini”) the tick size is 0.25 because that is the minimal increment the price moves in.
-   [syminfo.pointvalue](https://www.tradingview.com/pine-script-reference/v6/#var_syminfo.pointvalue): The point value is the multiple of the underlying asset determining a contract’s value. On “ES1!” (“S&P 500 E-Mini”) the point value is 50, so a contract is worth 50 times the price of the instrument.
-   [syminfo.prefix](https://www.tradingview.com/pine-script-reference/v6/#var_syminfo.prefix): The prefix is the exchange or broker’s identifier: “NASDAQ” or “BATS” for “AAPL”, “CME\_MINI\_DL” for “ES1!”.
-   [syminfo.root](https://www.tradingview.com/pine-script-reference/v6/#var_syminfo.root): It is the ticker’s prefix for structured tickers like those of futures. It is “ES” for “ES1!”, “ZW” for “ZW1!”.
-   [syminfo.session](https://www.tradingview.com/pine-script-reference/v6/#var_syminfo.session): It reflects the session setting on the chart for that symbol. If the “Chart settings/Symbol/Session” field is set to “Extended”, it will only return “extended” if the symbol and the user’s feed allow for extended sessions. It is rarely displayed and used mostly as an argument to the `session` parameter in [ticker.new()](https://www.tradingview.com/pine-script-reference/v6/#fun_ticker.new).
-   [syminfo.ticker](https://www.tradingview.com/pine-script-reference/v6/#var_syminfo.ticker): It is the symbol’s name, without the exchange part ([syminfo.prefix](https://www.tradingview.com/pine-script-reference/v6/#var_syminfo.prefix)): “BTCUSD”, “AAPL”, “ES1!”, “USDCAD”.
-   [syminfo.tickerid](https://www.tradingview.com/pine-script-reference/v6/#var_syminfo.tickerid): The symbol’s ticker identifier, consisting of its exchange prefix and symbol name, e.g., “NASDAQ:MSFT”. It can also include ticker information beyond the “prefix:ticker” form, such as extended hours, dividend adjustments, currency conversion, etc. To retrieve the standard “prefix:ticker” form only, pass the variable to [ticker.standard()](https://www.tradingview.com/pine-script-reference/v6/#fun_ticker.standard). When used in a `request.*()` call’s `expression` argument, this variable references the _requested_ context’s ticker ID. Otherwise, it references the current chart’s ticker ID.
-   [syminfo.timezone](https://www.tradingview.com/pine-script-reference/v6/#var_syminfo.timezone): The timezone the symbol is traded in. The string is an [IANA time zone database name](https://en.wikipedia.org/wiki/List_of_tz_database_time_zones) (e.g., “America/New\_York”).
-   [syminfo.type](https://www.tradingview.com/pine-script-reference/v6/#var_syminfo.type): The type of market the symbol belongs to. The values are “stock”, “futures”, “index”, “forex”, “crypto”, “fund”, “dr”, “cfd”, “bond”, “warrant”, “structured” and “right”.

This script displays these built-in variables and their values for the current symbol in a [table](https://www.tradingview.com/pine-script-docs/concepts/tables/) on the chart:

```pine
//@version=6
indicator("`syminfo.*` built-ins demo", overlay = true)

//@variable The `syminfo.*` built-ins, displayed in the left column of the table.
string txtLeft =
  "syminfo.basecurrency: "  + "\n" +
  "syminfo.currency: "      + "\n" +
  "syminfo.description: "   + "\n" +
  "syminfo.main_tickerid: " + "\n" +
  "syminfo.mincontract: "   + "\n" +
  "syminfo.mintick: "       + "\n" +
  "syminfo.pointvalue: "    + "\n" +
  "syminfo.prefix: "        + "\n" +
  "syminfo.root: "          + "\n" +
  "syminfo.session: "       + "\n" +
  "syminfo.ticker: "        + "\n" +
  "syminfo.tickerid: "      + "\n" +
  "syminfo.timezone: "      + "\n" +
  "syminfo.type: "

//@variable The values of the `syminfo.*` built-ins, displayed in the right column of the table.
string txtRight =
  syminfo.basecurrency              + "\n" +
  syminfo.currency                  + "\n" +
  syminfo.description               + "\n" +
  syminfo.main_tickerid             + "\n" +
  str.tostring(syminfo.mincontract) + "\n" +
  str.tostring(syminfo.mintick)     + "\n" +
  str.tostring(syminfo.pointvalue)  + "\n" +
  syminfo.prefix                    + "\n" +
  syminfo.root                      + "\n" +
  syminfo.session                   + "\n" +
  syminfo.ticker                    + "\n" +
  syminfo.tickerid                  + "\n" +
  syminfo.timezone                  + "\n" +
  syminfo.type

if barstate.islast
    var table t = table.new(position.middle_right, 2, 1)
    table.cell(t, 0, 0, txtLeft, bgcolor = color.yellow, text_halign = text.align_right)
    table.cell(t, 1, 0, txtRight, bgcolor = color.yellow, text_halign = text.align_left)
```

## Chart timeframe {#chart-timeframe}

A script can obtain information on the type of timeframe used on the chart using these built-ins, which all return a “simple bool” result:

-   [timeframe.isseconds](https://www.tradingview.com/pine-script-reference/v6/#var_timeframe.isseconds)
-   [timeframe.isminutes](https://www.tradingview.com/pine-script-reference/v6/#var_timeframe.isminutes)
-   [timeframe.isintraday](https://www.tradingview.com/pine-script-reference/v6/#var_timeframe.isintraday)
-   [timeframe.isdaily](https://www.tradingview.com/pine-script-reference/v6/#var_timeframe.isdaily)
-   [timeframe.isweekly](https://www.tradingview.com/pine-script-reference/v6/#var_timeframe.isweekly)
-   [timeframe.ismonthly](https://www.tradingview.com/pine-script-reference/v6/#var_timeframe.ismonthly)
-   [timeframe.isdwm](https://www.tradingview.com/pine-script-reference/v6/#var_timeframe.isdwm)

Additional built-ins return more specific timeframe information:

-   [timeframe.multiplier](https://www.tradingview.com/pine-script-reference/v6/#var_timeframe.multiplier) returns a “simple int” containing the multiplier of the timeframe unit. A chart timeframe of one hour will return `60` because intraday timeframes are expressed in minutes. A 30sec timeframe will return `30` (seconds), a daily chart will return `1` (day), a quarterly chart will return `3` (months), and a yearly chart will return `12` (months). The value of this variable cannot be used as an argument to `timeframe` parameters in built-in functions, as they expect a string in timeframe specifications format.
-   [timeframe.period](https://www.tradingview.com/pine-script-reference/v6/#var_timeframe.period) holds a “string” representing the script’s timeframe. It follows Pine’s [timeframe string specifications](https://www.tradingview.com/pine-script-docs/concepts/timeframes/#timeframe-string-specifications), where the string consists of a quantity (multiplier) and unit, e.g., “1D”, “2W”, “3M”. When used in a `request.*()` call’s `expression` argument, this variable references the _requested_ context’s timeframe. Otherwise, it references the script’s main timeframe.
-   [timeframe.main\_period](https://www.tradingview.com/pine-script-reference/v6/#var_timeframe.main_period) holds a “string” representing the _main_ timeframe, which is either the `timeframe` argument specified in the [indicator()](https://www.tradingview.com/pine-script-reference/v6/#fun_indicator) declaration, or the current chart’s timeframe. It behaves almost identically to [timeframe.period](https://www.tradingview.com/pine-script-reference/v6/#var_timeframe.period). However, this variable _always_ represents the script’s _main_ timeframe, even within requested contexts.

See the page on [Timeframes](https://www.tradingview.com/pine-script-docs/concepts/timeframes/) for more information.

## Session information {#session-information}

Session information is available in different forms:

-   The [syminfo.session](https://www.tradingview.com/pine-script-reference/v6/#var_syminfo.session) built-in variable returns a value that is either [session.regular](https://www.tradingview.com/pine-script-reference/v6/#const_session.regular) or [session.extended](https://www.tradingview.com/pine-script-reference/v6/#const_session.extended). It reflects the session setting on the chart for that symbol. If the “Chart settings/Symbol/Session” field is set to “Extended”, it will only return “extended” if the symbol and the user’s feed allow for extended sessions. It is used when a session type is expected, for example as the argument for the `session` parameter in [ticker.new()](https://www.tradingview.com/pine-script-reference/v6/#fun_ticker.new).
-   [Session state built-ins](https://www.tradingview.com/pine-script-docs/concepts/sessions/#session-variables-reference) provide information on the trading session a bar belongs to.
