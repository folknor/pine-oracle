---
title: Non-standard charts data
source: https://www.tradingview.com/pine-script-docs/concepts/non-standard-charts-data/
section: concepts
---

# Non-standard charts data {#non-standard-charts-data}

## Introduction {#introduction}

Pine Script® features several `ticker.*()` functions that generate _ticker identifiers_ for requesting data from _non-standard_ chart feeds. The available functions that create these ticker IDs are [ticker.heikinashi()](https://www.tradingview.com/pine-script-reference/v6/#fun_ticker.heikinashi), [ticker.renko()](https://www.tradingview.com/pine-script-reference/v6/#fun_ticker.renko), [ticker.linebreak()](https://www.tradingview.com/pine-script-reference/v6/#fun_ticker.linebreak), [ticker.kagi()](https://www.tradingview.com/pine-script-reference/v6/#fun_ticker.kagi), and [ticker.pointfigure()](https://www.tradingview.com/pine-script-reference/v6/#fun_ticker.pointfigure). Scripts can use these functions’ returned values as the `symbol` argument in [request.security()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.security) calls to access non-standard chart data while running on _any_ chart type.

NoteThe [Renko](https://www.tradingview.com/support/solutions/43000502284), [Line Break](https://www.tradingview.com/support/solutions/43000502273), [Kagi](https://www.tradingview.com/support/solutions/43000502272), and [Point & Figure](https://www.tradingview.com/support/solutions/43000502276) chart types construct bars using price data from _lower timeframes_. Therefore, the bars on these chart types only _approximate_ the values that correspond to calculations using tick data.

## `ticker.heikinashi()` {#tickerheikinashi}

_Heikin-Ashi_ means _average bar_ in Japanese. The open/high/low/close values of Heikin-Ashi candlesticks are synthetic; they are not actual market prices. They are calculated by averaging combinations of real OHLC values from the current and previous bar. The calculations used make Heikin-Ashi bars less noisy than normal candlesticks. They can be useful to make visual assessments, but are unsuited to backtesting or automated trading, as orders execute on market prices — not Heikin-Ashi prices.

The [ticker.heikinashi()](https://www.tradingview.com/pine-script-reference/v6/#fun_ticker.heikinashi) function creates a special ticker identifier for requesting Heikin-Ashi data with the [request.security()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.security) function.

This script requests the close value of Heikin-Ashi bars and plots them on top of the normal candlesticks:

![image](https://www.tradingview.com/pine-script-docs/_astro/NonStandardCharts-TickerHeikinAshi-01.Cf57zbCt_5nxwn.webp)

```pine
//@version=6
indicator("HA Close", "", true)
haTicker = ticker.heikinashi(syminfo.tickerid)
haClose = request.security(haTicker, timeframe.period, close)
plot(haClose, "HA Close", color.black, 3)
```

Note that:

-   The close values for Heikin-Ashi bars plotted as the black line are very different from those of real candles using market prices. They act more like a moving average.
-   The black line appears over the chart bars because we have selected “Visual Order/Bring to Front” from the script’s “More” menu.

If you wanted to omit values for extended hours in the last example, an intermediary ticker without extended session information would need to be created first:

![image](https://www.tradingview.com/pine-script-docs/_astro/NonStandardCharts-TickerHeikinAshi-02.B9TEp43B_Z1JChjK.webp)

```pine
//@version=6
indicator("HA Close", "", true)
regularSessionTicker = ticker.new(syminfo.prefix, syminfo.ticker, session.regular)
haTicker = ticker.heikinashi(regularSessionTicker)
haClose = request.security(haTicker, timeframe.period, close, gaps = barmerge.gaps_on)
plot(haClose, "HA Close", color.black, 3, plot.style_linebr)
```

Note that:

-   We use the [ticker.new()](https://www.tradingview.com/pine-script-reference/v6/#fun_ticker.new) function first, to create a ticker without extended session information.
-   We use that ticker instead of [syminfo.tickerid](https://www.tradingview.com/pine-script-reference/v6/#var_syminfo.tickerid) in our [ticker.heikinashi()](https://www.tradingview.com/pine-script-reference/v6/#fun_ticker.heikinashi) call.
-   In our [request.security()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.security) call, we set the `gaps` parameter’s value to `barmerge.gaps_on`. This instructs the function not to use previous values to fill slots where data is absent. This makes it possible for it to return [na](https://www.tradingview.com/pine-script-reference/v6/#var_na) values outside of regular sessions.
-   To be able to see this on the chart, we also need to use a special `plot.style_linebr` style, which breaks the plots on [na](https://www.tradingview.com/pine-script-reference/v6/#var_na) values.

This script plots Heikin-Ashi candles under the chart:

![image](https://www.tradingview.com/pine-script-docs/_astro/NonStandardCharts-TickerHeikinAshi-03.DDXVmouO_IPPYC.webp)

```pine
//@version=6
indicator("Heikin-Ashi candles")
CANDLE_GREEN = #26A69A
CANDLE_RED   = #EF5350

haTicker = ticker.heikinashi(syminfo.tickerid)
[haO, haH, haL, haC] = request.security(haTicker, timeframe.period, [open, high, low, close])
candleColor = haC >= haO ? CANDLE_GREEN : CANDLE_RED
plotcandle(haO, haH, haL, haC, color = candleColor)
```

Note that:

-   We use a [tuple](https://www.tradingview.com/pine-script-docs/language/variable-declarations/#tuple-declarations) with [request.security()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.security) to fetch four values with the same call.
-   We use [plotcandle()](https://www.tradingview.com/pine-script-reference/v6/#fun_plotcandle) to plot our candles. See the [Bar plotting](https://www.tradingview.com/pine-script-docs/visuals/bar-plotting/) page for more information.

## `ticker.renko()` {#tickerrenko}

_Renko_ bars only plot price movements, without taking time or volume into consideration. They look like bricks stacked in adjacent columns. A new brick is only drawn after the price passes the top or bottom by a predetermined amount. The [ticker.renko()](https://www.tradingview.com/pine-script-reference/v6/#fun_ticker.renko) function creates a ticker id which can be used with [request.security()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.security) to fetch Renko values, but there is no Pine Script function to draw Renko bars on the chart:

```pine
//@version=6
indicator("", "", true)
renkoTicker = ticker.renko(syminfo.tickerid, "ATR", 10)
renkoLow = request.security(renkoTicker, timeframe.period, low)
plot(renkoLow)
```

## `ticker.linebreak()` {#tickerlinebreak}

The _Line Break_ chart type displays a series of vertical boxes that are based on price changes. The [ticker.linebreak()](https://www.tradingview.com/pine-script-reference/v6/#fun_ticker.linebreak) function creates a ticker id which can be used with [request.security()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.security) to fetch “Line Break” values, but there is no Pine Script function to draw such bars on the chart:

```pine
//@version=6
indicator("", "", true)
lineBreakTicker = ticker.linebreak(syminfo.tickerid, 3)
lineBreakClose = request.security(lineBreakTicker, timeframe.period, close)
plot(lineBreakClose)
```

## `ticker.kagi()` {#tickerkagi}

_Kagi_ charts are made of a continuous line that changes directions. The direction changes when the price changes beyond a predetermined amount. The [ticker.kagi()](https://www.tradingview.com/pine-script-reference/v6/#fun_ticker.kagi) function creates a ticker id which can be used with [request.security()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.security) to fetch “Kagi” values, but there is no Pine Script function to draw such bars on the chart:

```pine
//@version=6
indicator("", "", true)
kagiBreakTicker = ticker.linebreak(syminfo.tickerid, 3)
kagiBreakClose = request.security(kagiBreakTicker, timeframe.period, close)
plot(kagiBreakClose)
```

## `ticker.pointfigure()` {#tickerpointfigure}

_Point and Figure_ (PnF) charts only plot price movements, without taking time into consideration. A column of X’s is plotted as the price rises, and O’s are plotted when price drops. The [ticker.pointfigure()](https://www.tradingview.com/pine-script-reference/v6/#fun_ticker.pointfigure) function creates a ticker id which can be used with [request.security()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.security) to fetch “PnF” values, but there is no Pine Script function to draw such bars on the chart. Every column of X’s or O’s is represented with four numbers. You may think of them as synthetic OHLC PnF values:

```pine
//@version=6
indicator("", "", true)
pnfTicker = ticker.pointfigure(syminfo.tickerid, "hl", "ATR", 14, 3)
[pnfO, pnfC] = request.security(pnfTicker, timeframe.period, [open, close], barmerge.gaps_on)
plot(pnfO, "PnF Open", color.green, 4, plot.style_linebr)
plot(pnfC, "PnF Close", color.red, 4, plot.style_linebr)
```
