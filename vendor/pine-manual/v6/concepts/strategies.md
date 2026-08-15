---
title: Strategies
source: https://www.tradingview.com/pine-script-docs/concepts/strategies/
section: concepts
---

# Strategies {#strategies}

## Introduction {#introduction}

Pine Script® strategies are specialized scripts that simulate trades across historical and realtime bars, allowing users to backtest and forward test their trading systems. Strategy scripts have many of the same capabilities as [indicator](https://www.tradingview.com/pine-script-reference/v6/#fun_indicator) scripts, and they provide the ability to place, modify, and cancel hypothetical orders and analyze performance results.

When a script uses the [strategy()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy) function as its [declaration statement](https://www.tradingview.com/pine-script-docs/language/declaration-statements/), it gains access to the `strategy.*` namespace, which features numerous functions and variables for simulating orders and retrieving essential strategy information. Additionally, the script generates a detailed [strategy report](https://www.tradingview.com/pine-script-docs/concepts/strategies/#strategy-report) in a dedicated tab below the chart.

## A simple strategy example {#a-simple-strategy-example}

The following script is a simple strategy that simulates entering a long or short position when two moving averages (MAs) cross. If the fast MA crosses over the slow MA, the strategy places a [market order](https://www.tradingview.com/pine-script-docs/concepts/strategies/#market-orders) named “Buy” to enter a long position. If the fast MA crosses under the slow MA, it places a market order named “Sell” to open a short position instead:

```pine
//@version=6
strategy("Simple strategy demo", overlay = true)

//@variable Defines the lengths of the fast and slow moving averages.
int lengthInput = input.int(14, "Base length", minval = 2)

// Calculate a fast MA using the `lengthInput` value and a slow MA using twice that value.
float fastMA = ta.sma(close, lengthInput)
float slowMA = ta.sma(close, lengthInput * 2)

// If the fast MA crosses over the slow MA, place an order to close any short position and enter a long position.
if ta.crossover(fastMA, slowMA)
    strategy.entry("Buy", direction = strategy.long)

// If the fast MA crosses under the slow MA, place an order to close any long position and enter a short position.
if ta.crossunder(fastMA, slowMA)
    strategy.entry("Sell", direction = strategy.short)

// Plot both moving averages for reference. 
plot(fastMA, "Fast MA", color.aqua)
plot(slowMA, "Slow MA", color.orange)
```

Note that:

-   The [strategy()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy) function call declares that the script is a strategy named “Simple strategy demo” that displays visuals on the main chart pane.
-   The [strategy.entry()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.entry) function is the command that the script uses to create _entry orders_ and [reverse positions](https://www.tradingview.com/pine-script-docs/concepts/strategies/#reversing-positions). The “Buy” entry order closes any short position and opens a new long position. The “Sell” entry order closes any long position and opens a new short position.

## Applying a strategy to a chart {#applying-a-strategy-to-a-chart}

To test a strategy, add it to the chart. Select a built-in, published, or personal strategy from the “Indicators, metrics, and strategies” menu, or write a custom strategy in the Pine Editor and select the “Add to chart” button from the editor’s options:

The script plots trade markers on the bars in the main chart pane and displays a detailed _strategy report_ within a separate tab in the chart’s bottom panel:

See the [Strategy report](https://www.tradingview.com/pine-script-docs/concepts/strategies/#strategy-report) section below to learn how to read and interpret the performance data displayed by this tab.

> [!IMPORTANT]
> The performance results from a strategy applied to _non-standard charts_ ([Heikin Ashi](https://www.tradingview.com/support/solutions/43000619436), [Renko](https://www.tradingview.com/support/solutions/43000502284), [line break](https://www.tradingview.com/support/solutions/43000502273), [Kagi](https://www.tradingview.com/support/solutions/43000502272), [point & figure](https://www.tradingview.com/support/solutions/43000502276), and [range](https://www.tradingview.com/support/solutions/43000474007)) **do not** reflect actual market conditions by default. The strategy simulates trades using the chart’s **synthetic** prices, which do not represent real-world market prices. Consequently, running a strategy on a non-standard chart typically produces **unrealistic** results.
>
> Therefore, we strongly recommend using **standard** chart types when testing strategies. Alternatively, on Heikin Ashi charts, users can simulate order fills using actual prices by selecting the “Standard bars” option from the “Heikin Ashi mode” input in the script’s “Settings/Properties” tab. Programmers can specify that a strategy uses this behavior by default by including `fill_orders_on_standard_ohlc = true` in the [strategy()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy) declaration statement.

## Strategy report {#strategy-report}

The _strategy report_ visualizes the hypothetical trading performance of a simulated strategy. The report automatically appears within a tab in the chart’s bottom pannel if at least one strategy script is active on the chart. The report displays performance results for only _one_ strategy at a time. If two or more strategies are active on the chart, users can specify which strategy to analyze by selecting its name from the _context menu_ opened by the dropdown arrow in the tab’s header:

After the selected script executes across the chart’s data, the strategy report populates two main sub-tabs with relevant data from the strategy simulation:

-   The [“Metrics” tab](https://www.tradingview.com/pine-script-docs/concepts/strategies/#metrics-tab) provdes a detailed summary of the strategy’s performance. The sections in the tab include a time-series chart for analyzing trade activity and equity growth, multiple graphs for analyzing essential performance data, and several useful metrics for assessing the strategy’s long, short, and overall trading performance.
-   The [“Trades” tab](https://www.tradingview.com/pine-script-docs/concepts/strategies/#trades-tab) displays a list of the strategy’s simulated trades in chronological order. The items in the list show essential information about each trade, including the trade’s number and direction, the entry and exit prices, and multiple trade-wise performance metrics.

Users can switch between these sub-tabs by selecting the icons in the top-left corner of the strategy report.

> [!TIP]
> Users can also _download_ all applicable data that populates these sub-tabs by selecting the “Download data as XLSX” option in the context menu.

The additional items next to the “Metrics” and “Trades” icons at the top of the strategy report provide quick options where users can customize the testing period and enable [Deep Backtesting](https://www.tradingview.com/support/solutions/43000666199-what-is-deep-backtesting/) mode, set the strategy’s initial capital and [account currency](https://www.tradingview.com/pine-script-docs/concepts/strategies/#currency), control the level of [historical bar detail](https://www.tradingview.com/pine-script-docs/concepts/strategies/#adjusting-historical-bar-detail) in the simulation, and adjust the script’s [execution settings](https://www.tradingview.com/support/solutions/43000786178/), respectively:

When a script author [publishes](https://www.tradingview.com/pine-script-docs/writing/publishing/) a strategy, the publication’s script page includes a _compact_ version of the strategy report that displays hypothetical performance results from running the script with specific properties on the published chart. The publication’s report contains similar information to the report displayed below a user’s chart. It also contains an additional [“Properties” tab](https://www.tradingview.com/pine-script-docs/concepts/strategies/#properties-tab), which displays the input values and properties that the author used while preparing the publication.

### “Metrics” tab {#metrics-tab}

The [“Metrics”](https://www.tradingview.com/support/solutions/43000681735/) tab of the strategy report provides a detailed summary of a strategy’s performance over a sequence of simulated trades. It organizes the performance details into five main sections:

-   [Key stats](https://www.tradingview.com/pine-script-docs/concepts/strategies/#key-stats): Displays key overall performance metrics, as well as a chart that plots the series of cumulative equity changes and other overall performance values.
-   [Return details](https://www.tradingview.com/pine-script-docs/concepts/strategies/#return-details): Displays graphs and tables for inspecting the strategy’s overall returns and costs, the strategy’s profitability and potential relative to a buy-and-hold system, and the strategy’s risk-adjusted performance ratios.
-   [Trades analysis](https://www.tradingview.com/pine-script-docs/concepts/strategies/#trades-analysis): Displays graphs to represent the distribution of trade outcomes, as well as tables containing key trade-wise performance metrics.
-   [Equity run-ups and drawdowns](https://www.tradingview.com/pine-script-docs/concepts/strategies/#equity-run-ups-and-drawdowns): Displays graphs and tables with essential information about the strategy’s positive and negative equity fluctuations.
-   [Capital efficiency](https://www.tradingview.com/pine-script-docs/concepts/strategies/#capital-efficiency): Displays overall metrics relating to the strategy’s capital and margin at the top, as well as a chart and tables containing details about the strategy’s capital usage, margin usage, overall returns, and margin calls.

Some of the tables in this tab contain “All”, “Long”, and “Short” columns. The “All” column shows the performance metrics for all simulated trades. The “Long” and “Short” columns show relevant metrics separately for long and short trades.

The following sections explain the types of information that each part of the “Metrics” tab contains and how it displays the data.

> [!TIP]
> Most of the tables in the “Metrics” tab reveal a “Show description” icon next to a metric’s name when the user hovers over the metric’s row. Clicking that icon opens a Help Center article containing details about the metric’s calculations and meaning. Refer to the [Strategy report metrics](https://www.tradingview.com/support/folders/43000599093-strategy-report-metrics/) page in our Help Center for a list of these metrics and their corresponding articles.

#### Key stats {#key-stats}

The “Key stats” section of the strategy report’s “Metrics” tab provides a quick overview of the strategy’s overall performance. The top of the section displays a few essential performance metrics, including the strategy’s total profit or loss, the [maximum drawdown](https://www.tradingview.com/support/solutions/43000681690/), the total number of profitable trades compared to the total number of closed trades, and the strategy’s [profit factor](https://www.tradingview.com/support/solutions/43000681698/).

The [“Performance” chart](https://www.tradingview.com/support/solutions/43000681735/) below these metrics optionally displays up to four plots to visualize equity growth and trade volatility:

-   The “Cumulative PnL” baseline plot displays the cumulative change in the strategy’s equity across all closed trades, experessed as a currency amount or a percentage of the strategy’s initial capital.
-   The “Buy and hold” plot displays the cumulative capital or percentage change in equity for a strategy that opens a single long trade and holds it throughout the entire trading period.
-   The “Trades excursions” plot displays green and red columns representing the maximum _unrealized_ profit and loss for each trade, expressed as a currency amount or a percentage of the trade’s size.
-   The “Run-ups and drawdowns” plot displays green and red horizontal bars to highlight periods of run-up and drawdown in the strategy’s equity across the trading range.

Users can specify the chart’s scale type, download or share an image of the chart, and expand or collapse the chart view by selecting the icons above the chart’s top-right edge.

Hovering over the points on the “Performance” chart anywhere above the “Run-ups and drawdowns” plot reveals a tooltip containing details for a specific trade, including the trade’s number, direction, and closing time. The tooltip also displays the values from the “Cumulative PnL”, “Buy and hold”, and “Trades excursions” plots, depending on which plots are active. When a user clicks the point highlighted by the tooltip, the main price chart automatically scrolls to the chart bar on which the corresponding trade closed, then displays that bar’s time in a temporary tooltip:

Note that:

-   The main price chart automatically scrolls to a trade’s closing bar only if the strategy uses the _default_ testing period. Selecting a different period from the “Testing period” menu at the top of the tab activates _Deep Backtesting_ mode, which does _not_ support scrolling the main chart from the report’s “Performance” chart.

Users can also hover over the horizontal bars in the “Run-ups and drawdowns” plot to view a separate tooltip containing the total run-up or drawdown for each displayed period:

#### Return details {#return-details}

The “Return details” section of the “Metrics” tab analyzes the strategy’s cumulative returns and compares them to a buy-and-hold benchmark. The “Overview” sub-tab in this section displays two graphs:

-   The “Profit structure” graph on the left displays bars representing the strategy’s total profit, total loss, total commissions, open profit or loss, and the total net profit or loss.
-   The “Benchmarking” graph displays the maximum, minimum, and current total return of the strategy compared to the buy-and-hold return values over the same testing range.

The sub-tab also displays the strategy’s open profit or loss, expected payoff per trade, outperformance, and Sharpe ratio above the graphs for quick reference:

The other sub-tabs within this section display detailed information about the strategy’s overall returns:

-   The “Returns” tab displays the strategy’s initial capital, open profit or loss, net profit or loss, gross profit and loss, profit factor, commissions paid, and expected payoff per trade. It displays applicable metrics separately for all trades, all long trades, and all short trades.
-   The “Benchmarking” tab displays the overall returns of the buy-and-hold benchmark and the strategy’s outperformance.
-   The “Risk-adjusted performance” tab displays the strategy’s Sharpe and Sortino ratios, which gauge a strategy’s risk-adjusted return relative to the risk-free rate specified by the [strategy()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy) declaration statement.

#### Trades analysis {#trades-analysis}

The “Trades analysis” section of the “Metrics” tab analyzes the overall performance of the strategy’s individual trades. The “Overview” sub-tab in this section displays two graphs for analyzing the distribution of trade outcomes:

-   The “Returns distribution” histogram graph displays the distribution of trade returns. Each column in the histogram shows the number of trades that closed with returns within a specific range. Red columns represent negative returns (losses), and green columns represent positive returns (profits). The graph also displays dashed vertical lines at the overall average profit and loss values.
-   The “Trades distribution” donut graph displays the quantity of winning, losing, and break-even trades relative to the strategy’s total number of trades.

The “Overview” sub-tab also displays the strategy’s average profit or loss, the average number of bars per trade, and the larget profit and loss from a single trade above the two graphs:

The “Trades analysis details” sub-tab displays multiple useful statistics for all trades, all long trades, and all short trades. It includes metrics such as the total number of trades, the total winning and losing trades, the percentage of profitable trades, the average return amounts, the maximum profit and loss, and the number of outliers in the returns distribution:

#### Equity run-ups and drawdowns {#equity-run-ups-and-drawdowns}

The “Equity run-ups and drawdowns” section of the “Metrics” tab analyzes the strategy’s periods of equity growth (_run-up_) and decline (_drawdown_). The “Overview” sub-tab in this section includes two graphs to simplify run-up and drawdown inspection:

-   The “Alternating growth and decline” graph displays vertical bars for periods of run-up and drawdown in chronological order. Green bars represent run-up periods, and red bars represent drawdown periods.
-   The “Comparison of growth and decline periods” graph displays horizontal bars representing the maximum, average, and current run-up and drawdown percentages for a quick visual comparison.

The sub-tab also displays the average run-up and drawdown durations, the maximum drawdown as a percentage of the initial capital, and the ratio of the total profit or loss to the maximum drawdown above the graphs:

The “Run-ups” and “Drawdowns” sub-tabs display tables containing detailed metrics for the strategy’s run-up and drawdown periods, including:

-   The average duration of each run-up/drawdown period.
-   The average run-up/drawdown amount per period.
-   The maximum run-up/drawdown across the testing range on both an intrabar and close-to-close basis.
-   The maximum intrabar run-up/drawdown as a percentage of the strategy’s initial capital.
-   The ratio of the strategy’s total profit or loss relative to the maximum drawdown.

#### Capital efficiency {#capital-efficiency}

The “Capital efficiency” section of the “Metrics” tab evaluates how the strategy uses its simulated funds and available [margin](https://www.tradingview.com/pine-script-docs/concepts/strategies/#margin-and-leverage). The “Overview” sub-tab contains a “Margin usage” chart that displays the amount of margin used on each trade in order, expressed as a currency amount or a percentage of the strategy’s available funds. The sub-tab also shows the strategy’s compound annual growth rate (CAGR), the minimum account size required to avoid a margin call, the overall return on the initial capital, and the total number of _margin call events_:

The other sub-tabs in this section contain detailed information about the strategy’s capital and margin usage for all trades, all long trades, and all short trades:

-   The “Capital usage” tab displays the strategy’s CAGR, the overall return based on the initial capital, the minimum account size, the overall return based on the minimum account size, and the net profit or loss as a percentage of the largest loss.
-   The “Margin usage” tab displays the average margin per trade, the maximum margin used on a trade, the average profit per unit of margin, the total number of margin calls, and the total trade volume liquidated by all margin calls.

### “Trades” tab {#trades-tab}

The [“Trades”](https://www.tradingview.com/support/solutions/43000681737) tab of the strategy report lists the strategy’s simulated trades in ascending or descending order by time. The list includes two columns that are always active by default: “Trade number” and “Type”. The “Trade number” column lists each trade’s number and direction. Users can change the sorting order of the list’s items by clicking on the “Trade number” column heading. The list is sorted in descending order by default. The “Type” column contains fields for each trade’s entry and exit order. If the strategy is not running in [Deep Backtesting](https://www.tradingview.com/support/solutions/43000666199-what-is-deep-backtesting/) mode, hovering the mouse over either field in a listed item reveals a “Show on chart” icon, which users can click to scroll the main chart to the trade’s entry or exit bar.

The list optionally includes any of the following additional columns, depending on the user’s selections:

-   “Date and time”: Shows the entry and exit date and time for each trade.
-   “Signal”: Shows the name or comment assigned to each trade’s entry and exit orders.
-   “Price”: Shows each trade’s entry and exit price in the instrument’s quoted currency.
-   “Size”: Shows each trade’s size, expressed as a number of contracts/lots/shares/units and a quantity of the strategy’s account currency.
-   “Return”: Shows each trade’s net return as a percentage of the trade’s size.
-   “Favorable excursion”: Shows each trade’s maximum unrealized profit as a currency amount and a percentage of the trade’s size.
-   “Adverse excursion”: Shows each trade’s maximum unrealized loss as a currency amount and a percentage of the trade’s size.
-   “Cumulative PnL”: Shows the strategy’s total profit or loss at the time of each trade, expressed as a currency amount and a percentage of the strategy’s initial capital.
-   “Duration (bars)”: Shows the number of bars for which each trade remained open.

Users can specify which columns to display by selecting the “Column setup” icon above the list’s top-right edge. The adjacent “Download” icon enables users to download all available data for the list as a _CSV_ file.

Note that:

-   The downloaded CSV file for the list of trades contains data for _all_ available columns, regardless of the display columns selected in the “Column setup” menu.
-   If a strategy uses the default testing range, it preserves individual data for up to the latest _9000 trades_. If the strategy uses a different testing range, enabling Deep Backtesting mode, it maintains individual trade data for _all_ trades. This behavior affects the results available from the list of trades, any downloaded CSV files, and the built-in `strategy.*()` functions that retrieve [individual trade information](https://www.tradingview.com/pine-script-docs/concepts/strategies/#individual-trade-information). However, it does _not_ affect the data displayed in the [“Metrics” tab](https://www.tradingview.com/pine-script-docs/concepts/strategies/#metrics-tab) or accessed by other `strategy.*` built-ins. See the [trade limit](https://www.tradingview.com/pine-script-docs/concepts/strategies/#trade-limit) section to learn more.

### “Properties” tab {#properties-tab}

When a script author [publishes](https://www.tradingview.com/pine-script-docs/writing/publishing/) a strategy, the publication’s script page displays a _compact_ version of the strategy report to demonstrate the script’s performance results for a specific dataset. The published report also includes an extra _“Properties”_ tab, which displays details about the dataset, script inputs, and strategy properties that the author used while [preparing](https://www.tradingview.com/pine-script-docs/writing/publishing/#preparing-a-publication) the publication. The tab organizes this information into four collapsible sections:

-   The “Date range” section shows the selected testing range and the overall available backtesting range.
-   The “Symbol info” section displays the chart’s symbol, timeframe, type, point value, currency, and tick size. It also includes the chart’s specified precision setting.
-   The “Strategy inputs” section lists the names and values of all the [inputs](https://www.tradingview.com/pine-script-docs/pine-script-docs/concepts/inputs/) available in the strategy’s “Settings/Inputs” tab. This section appears in the tab only if the script includes `input*()` calls or specifies a nonzero [`calc_bars_count`](https://www.tradingview.com/pine-script-docs/language/declaration-statements/#calc_bars_count) argument in the [strategy()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy) declaration statement.
-   The “Strategy properties” section provides an overview of the properties that the author specified in the script’s “Settings/Properties” tab, including the strategy’s initial capital, account currency, order size, leverage, pyramiding, commission, slippage, and other settings.

Note that:

-   If users download performance data as an XLSX file from the strategy report below their charts, the exported data from that report also includes “Properties” tab data for reference.

## Broker emulator {#broker-emulator}

TradingView uses a [broker emulator](https://www.tradingview.com/support/solutions/43000786181/) to simulate trades while running a strategy script. Unlike brokers in real-world trading, the broker emulator fills a strategy’s orders using only the available _chart data_ by default. Consequently, it executes orders on historical bars _after a bar closes_. Similarly, depending on the selected [calculation behavior](https://www.tradingview.com/pine-script-docs/concepts/strategies/#altering-calculation-behavior), the earliest point at which it can fill orders on realtime bars is _after a new tick_. For detailed information about this behavior, refer to the [Execution model](https://www.tradingview.com/pine-script-docs/language/execution-model/) page.

Because the broker emulator uses only price data from the chart by default, it makes default _assumptions_ about intrabar price movement when filling orders on historical bars. The emulator analyzes the opening, high, low, and closing prices of chart bars to infer historical intrabar activity using the following logic:

-   If the opening price of a bar is closer to the high than it is to the low, the emulator assumes that the market price moved in this order: **open → high → low → close**.
-   If the opening price of a bar is closer to the low than it is to the high, the emulator assumes that the market price moved in this order: **open → low → high → close**.
-   When filling _price-based orders_ (all orders except [market orders](https://www.tradingview.com/pine-script-docs/concepts/strategies/#market-orders)), the emulator assumes that _no gaps_ exist inside each chart bar; it considers _any_ price within the bar’s range as a valid level for filling pending orders.
-   If the market price crosses a price-based order’s level during the gap between one bar’s closing time and the next bar’s opening time, the emulator assumes that intrabar data _does not exist_ within the gap. Rather than filling the order at the specified price in that case, the emulator fills the order at the _opening price_ of the bar following the gap.

The following image labels the OHLC values of a few historical bars using numbers 1-4 to demonstrate the broker emulator’s default assumptions about intrabar price movement. The “1” label represents the bar’s first tick, and the “4” label represents the last tick:

### Adjusting historical bar detail {#adjusting-historical-bar-detail}

Users with Premium and Ultimate [plans](https://www.tradingview.com/pricing/) can override the broker emulator’s chart-based assumptions and increase the level of intrabar detail on historical bars, allowing for more precise order fills in the strategy’s backtest. To enable high historical bar detail, select the “High” option from the strategy’s [Bar detalization](https://www.tradingview.com/support/solutions/43000786180/) settings, which are available in the “Settings/Properties” tab and at the top of the [strategy report](https://www.tradingview.com/pine-script-docs/concepts/strategies/#strategy-report) below the chart. Programmers can also configure a strategy to use high historical detail by default by including `use_bar_magnifier = true` in the [strategy()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy) declaration statement.

If a strategy enables high historical detail, the broker emulator retrieves open, high, low, and close prices from the bars on a suitable _lower timeframe_, when possible, to increase the number of ticks available for estimating price action and filling orders on historical bars. This setting also allows the script to perform multiple _additional executions_ on each historical bar, depending on the selected [Script execution](https://www.tradingview.com/support/solutions/43000786178/) settings. See the [Altering calculation behavior](https://www.tradingview.com/pine-script-docs/concepts/strategies/#altering-calculation-behavior) section below to learn more.

The following example illustrates how changing the level of historical bar detail can enhance the behavior of [limit orders](https://www.tradingview.com/pine-script-docs/concepts/strategies/#limit-orders). The script below creates entry and exit limit orders, named “Buy” and “Exit”, on the first bar whose opening time equals or exceeds an input timestamp. For visual reference, the script highlights the chart’s background in orange when it places the orders, and it draws two horizontal [lines](https://www.tradingview.com/pine-script-docs/visuals/lines-and-boxes/#lines) at the order prices. The [strategy()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy) statement does not include `use_bar_magnifier = true`. Therefore, the broker emulator uses only chart data to determine when it can fill both orders by default:

```pine
//@version=6
strategy("Default historical detail demo", overlay = true, behind_chart = false)

//@variable The minimum opening timestamp for the bar on which to place the orders.
int orderTime = input.time(timestamp("08 April 2024 00:00"), "Order time")

// Declare variables to reference the lines for the entry and exit levels.
var line buyLine = na
var line exitLine  = na

//@variable Translucent orange for the bar on which the script creates the orders, and `na` on all other bars. 
color orderColor = na

// Logic to place and visualize the orders
if time[1] < orderTime and time >= orderTime
    // Calculate the order price levels.
    float buyPrice  = hl2 - (high - low)
    float exitPrice = buyPrice + (high - low) * 0.25 

    // Place the "Buy" limit order at the `buyPrice` level.
    strategy.entry("Buy", strategy.long, limit = buyPrice)
    // Place the "Exit" limit order at the `exitPrice` level.
    strategy.exit("Exit", "Buy", limit = exitPrice)

    // Set the `orderColor` value to a translucent orange for the chart's background.
    orderColor := #ff980033
    // Initialize horizontal lines to visualize the order prices.
    buyLine  := line.new(bar_index, buyPrice,  bar_index + 1, buyPrice,  color = #2962ff, width = 2)
    exitLine := line.new(bar_index, exitPrice, bar_index + 1, exitPrice, color = #d500f9, width = 2)

// Extend the horizontal lines to the current bar when the position closes. 
if ta.change(strategy.closedtrades) > 0
    buyLine.set_x2(bar_index)
    exitLine.set_x2(bar_index)

// Highlight the chart's background on the order bar. 
bgcolor(orderColor)
```

If we apply this script to a weekly “NASDAQ:MSFT” chart, the broker emulator fills the “Buy” order one bar after the script creates the orders, then fills the “Exit” order several bars later. On the bar where the “Buy” order fills, the open is closer to the high than it is to the low, so the emulator assumes that the price moved from open to high, high to low, then low to close. Consequently, the emulator infers that after the market price crossed below the blue line, triggering the “Buy” order, it did not move back up and touch the fuchsia line to trigger the “Exit” order on the same bar. In other words, the strategy could not enter and exit the position in the _same_ week, according to the broker emulator’s assumptions:

If we include `use_bar_magnifier = true` in the [strategy()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy) statement, the strategy enables high historical detail by default. When this setting is active on a weekly chart, the broker emulator retrieves open, high, low, and close prices from the _daily timeframe_. The bars on the daily timeframe show that, contrary to the broker emulator’s default assumption, the market price _did_ move to the “Exit” order’s price after triggering the “Entry” order in the same week. Therefore, the emulator can fill both orders on the _same weekly bar_ in this case. Below, we show the strategy’s result on the weekly chart after enabling high historical detail, alongside the bars on the daily chart with the entry and exit points annotated:

```pine
//@version=6
strategy("High historical detail demo", overlay = true, behind_chart = false, use_bar_magnifier = true)

//@variable The minimum opening timestamp for the bar on which to place the orders.
int orderTime = input.time(timestamp("08 April 2024 00:00"), "Order time")

// Declare variables to reference the lines for the entry and exit levels.
var line buyLine = na
var line exitLine  = na

//@variable Translucent orange for the bar on which the script creates the orders, and `na` on all other bars. 
color orderColor = na

// Logic to place and visualize the orders
if time[1] < orderTime and time >= orderTime
    // Calculate the order price levels.
    float buyPrice  = hl2 - (high - low)
    float exitPrice = buyPrice + (high - low) * 0.25 

    // Place the "Buy" limit order at the `buyPrice` level.
    strategy.entry("Buy", strategy.long, limit = buyPrice)
    // Place the "Exit" limit order at the `exitPrice` level.
    strategy.exit("Exit", "Buy", limit = exitPrice)

    // Set the `orderColor` value to a translucent orange for the chart's background.
    orderColor := #ff980033
    // Initialize horizontal lines to visualize the order prices.
    buyLine  := line.new(bar_index, buyPrice,  bar_index + 1, buyPrice,  color = #2962ff, width = 2)
    exitLine := line.new(bar_index, exitPrice, bar_index + 1, exitPrice, color = #d500f9, width = 2)

// Extend the horizontal lines to the current bar when the position closes. 
if ta.change(strategy.closedtrades) > 0
    buyLine.set_x2(bar_index)
    exitLine.set_x2(bar_index)

// Highlight the chart's background on the order bar. 
bgcolor(orderColor)
```

Note that:

-   We used the [Horizontal line](https://www.tradingview.com/support/solutions/43000518124/), [Vertical line](https://www.tradingview.com/support/solutions/43000518093/), and [Callout](https://www.tradingview.com/support/solutions/43000516978/) drawing tools to annotate the daily chart on the right.

> [!NOTE]
> A strategy’s “Bar detalization” setting _does not_ affect the level of bar detail on the “1S” or “1T” timeframes. The broker emulator always analyzes _one_ tick per bar on a “1T” chart, and _four_ ticks per bar on a “1S” chart.
>
> Scripts can request a maximum of _200,000_ bars from any lower timeframe. Due to this limitation, the requested datasets for some symbols and timeframes might not include intrabar coverage for early bars on the chart. The broker emulator uses its [default assumptions](https://www.tradingview.com/pine-script-docs/concepts/strategies/#broker-emulator) when filling orders on bars that do not have intrabar data on the requested timeframe.

## Orders and trades {#orders-and-trades}

Pine Script strategies use orders to initiate trades and manage positions, similar to real-world trading. In this context, an _order_ is an instruction that a strategy sends to the [broker emulator](https://www.tradingview.com/pine-script-docs/concepts/strategies/#broker-emulator) to perform a market action, and a _trade_ is the resulting transaction that opens after the emulator fills an entry order. A market position is total of all open trades.

Let’s take a closer look at how strategy orders work and how they become trades. Every 20 bars, the following script creates a long [market order](https://www.tradingview.com/pine-script-docs/concepts/strategies/#market-orders) with [strategy.entry()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.entry) and draws a [label](https://www.tradingview.com/pine-script-reference/v6/#type_label). It calls [strategy.close\_all()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.close_all) on each bar from the global scope to generate a market order to close any open position:

```pine
//@version=6
strategy("Order execution demo", overlay = true, behind_chart = false)

//@variable Is `true` on every 20th bar, and `false` otherwise.
bool longCondition = bar_index % 20 == 0

// Place a long market order and draw a label when the `longCondition` value is `true`.
if longCondition
    strategy.entry("My Long Entry ID", direction = strategy.long)
    label.new(
        bar_index, high, text = "Long entry order created", 
        color = color.lime, style = label.style_label_lower_right, 
        textcolor = color.black, size = size.large
    )

// Place a market order to close any open position.
strategy.close_all()
```

Note that:

-   Although the script calls [strategy.close\_all()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.close_all) on every bar, the function creates a new exit order only if the strategy has an _open position_. If there is no open position, the function call has no effect.

The blue arrows on the above chart show where the strategy entered a long position, and the purple arrows mark the bars where the strategy closed the position. Notice that the label drawings appear one bar _before_ the entry markers, and the entry markers appear one bar _before_ the closing markers. This sequence illustrates order creation and execution in action.

By default, the earliest point at which the [broker emulator](https://www.tradingview.com/pine-script-docs/concepts/strategies/#broker-emulator) fills an order is on the next available tick, because creating and filling an order on the _same_ tick is _unrealistic_. If a strategy uses the default [calculation behavior](https://www.tradingview.com/pine-script-docs/concepts/strategies/#altering-calculation-behavior), it updates its calculations only after a bar closes, meaning the next tick on which an order can fill is at the _open_ of the _following bar_. For example, when the above script’s `longCondition` value is `true` on bar 20, the script places an entry order that fills on the next available tick, which is at the open of bar 21. When the script updates its calculations at the close of bar 21, it then places an exit order to close the current position on the following tick, at the open of bar 22.

## Order types {#order-types}

Pine Script strategies can simulate different order types to suit specific trading system needs. The main order types include [market](https://www.tradingview.com/pine-script-docs/concepts/strategies/#market-orders), [limit](https://www.tradingview.com/pine-script-docs/concepts/strategies/#limit-orders), [stop](https://www.tradingview.com/pine-script-docs/concepts/strategies/#stop-and-stop-limit-orders), and [stop-limit](https://www.tradingview.com/pine-script-docs/concepts/strategies/#stop-and-stop-limit-orders).

### Market orders {#market-orders}

A _market order_ is the simplest type of order. A market order is an instruction to buy or sell an instrument as soon as possible, irrespective of the price. Therefore, the [broker emulator](https://www.tradingview.com/pine-script-docs/concepts/strategies/#broker-emulator) always executes it on the next available tick. Most [order placement commands](https://www.tradingview.com/pine-script-docs/concepts/strategies/#order-placement-and-cancellation) generate market orders by default, but also include parameters for creating other types of orders.

The example script below alternates between placing long and short market orders in cycles of a specified length. When the [bar\_index](https://www.tradingview.com/pine-script-reference/v6/#var_bar_index) value is divisible by twice the input length, the script generates a long market order. If the [bar\_index](https://www.tradingview.com/pine-script-reference/v6/#var_bar_index) value is divisible by the input length but not by twice the length, the script places a short market order instead. The script also draws [labels](https://www.tradingview.com/pine-script-docs/visuals/text-and-shapes/#labels) to indicate bars on which it creates the orders:

```pine
//@version=6
strategy("Market order demo", overlay = true)

//@variable The number of bars between long and short entries.
int lengthInput = input.int(10, "Cycle length", 1)

//@function Displays a specified string in a label at the current bar's high.
debugLabel(string txt, color lblColor) => label.new(
     bar_index, high, text = txt, color = lblColor, textcolor = color.white, 
     style = label.style_label_lower_right, size = size.large
 )

//@variable Is `true` every `2 * lengthInput` bars, and `false` otherwise.
bool longCondition = bar_index % (2 * lengthInput) == 0
//@variable Is `true` every `lengthInput` bars, and `false` otherwise.
bool shortCondition = bar_index % lengthInput == 0

// Generate a long market order with a green label when the long condition occurs.
if longCondition
    debugLabel("Long market order created", color.green)
    strategy.entry("My Long Entry Id", strategy.long)
// Otherwise, generate a short market order with a red label when the short condition occurs.
else if shortCondition
    debugLabel("Short market order created", color.red)
    strategy.entry("My Short Entry Id", strategy.short)
```

Note that:

-   This script uses the default [calculation behavior](https://www.tradingview.com/pine-script-docs/concepts/strategies/#altering-calculation-behavior): it executes once on each bar’s closing tick. Therefore, as indicated by the labels and trade markers, the broker emulator fills each new order at the open of the following bar.
-   The [strategy.entry()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.entry) command can automatically _reverse_ an open position in the opposite direction. See the [Reversing positions](https://www.tradingview.com/pine-script-docs/concepts/strategies/#reversing-positions) section below for more information.

### Limit orders {#limit-orders}

A _limit order_ is an instruction to buy or sell an instrument at a specific price or better (lower than specified for long orders, and higher than specified for short orders), irrespective of the time. To simulate a limit order in a strategy script, pass a _price_ value to the `limit` parameter of an applicable [order placement command](https://www.tradingview.com/pine-script-docs/concepts/strategies/#order-placement-and-cancellation).

When the market price reaches a limit order’s value, or crosses it in the favorable direction, the [broker emulator](https://www.tradingview.com/pine-script-docs/concepts/strategies/#broker-emulator) fills the order at that value or a better price. When a strategy generates a limit order at a _worse_ value than the current market price (higher for long orders and lower for short orders), the emulator fills the order on the next available tick rather than waiting for the market price to reach that value.

For example, when the following script executes on the bar that is 100 bars before the latest bar, it calls the [strategy.entry()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.entry) command with a `limit` argument to generate a long limit order 800 ticks above the bar’s [close](https://www.tradingview.com/pine-script-reference/v6/#var_close) value. The script also draws a [label](https://www.tradingview.com/pine-script-reference/v6/#type_label) on the bar where it creates the order, and it draws a horizontal [line](https://www.tradingview.com/pine-script-reference/v6/#type_line) to visualize the order’s price:

```pine
//@version=6
strategy("Limit order demo", overlay = true)

//@function Draws a line and a label with specified text at a given price level.
debugLabel(float price, string txt) =>
    label.new(
         bar_index, price, text = txt, color = color.teal, textcolor = color.white, 
         style = label.style_label_lower_right, size = size.large
     )
    line.new(
         bar_index, price, bar_index + 1, price, color = color.teal, extend = extend.right, 
         style = line.style_dashed
     )

// Place a long limit order and draw a label and line 100 bars before the last bar.
if last_bar_index - bar_index == 100
    float limitPrice = close - syminfo.mintick * 800
    debugLabel(limitPrice, "Long Limit order created")
    strategy.entry("Long", strategy.long, limit = limitPrice)
```

Notice that the label and the start of the line in the chart above occur several bars before the “Long” entry marker. The [broker emulator](https://www.tradingview.com/pine-script-docs/concepts/strategies/#broker-emulator) cannot fill the limit order while the market price remains _above_ the `limitPrice` value, because that value is a _worse_ price for the long trade. After the price subsequently drops and reaches the order’s price, the emulator fills the order mid-bar at that price.

If we change the previous example to place a long limit order _above_ the bar’s [close](https://www.tradingview.com/pine-script-reference/v6/#var_close) value rather than _below_, the broker emulator fills the order on the next available tick — similar to a [market order](https://www.tradingview.com/pine-script-docs/concepts/strategies/#market-orders) — because the closing price is already a more _favorable_ value for the long trade. In the script version below, we set the limit order’s price to 800 ticks above the bar’s close to demonstrate this effect:

```pine
//@version=6
strategy("Limit order demo", overlay = true)

//@function Draws a line and a label with specified text at a given price level.
debugLabel(float price, string txt) =>
    label.new(
         bar_index, price, text = txt, color = color.teal, textcolor = color.white, 
         style = label.style_label_lower_right, size = size.large
     )
    line.new(
         bar_index, price, bar_index + 1, price, color = color.teal, extend = extend.right, 
         style = line.style_dashed
     )

// Place a long limit order and draw a label and line 100 bars before the last bar.
if last_bar_index - bar_index == 100
    float limitPrice = close + syminfo.mintick * 800
    debugLabel(limitPrice, "Long Limit order created")
    strategy.entry("Long", strategy.long, limit = limitPrice)
```

### Stop and stop-limit orders {#stop-and-stop-limit-orders}

A _stop order_ is an instruction to activate a new [market](https://www.tradingview.com/pine-script-docs/concepts/strategies/#market-orders) or [limit](https://www.tradingview.com/pine-script-docs/concepts/strategies/#limit-orders) order when the market price reaches a specific price or a _worse_ value (higher than specified for long orders and lower than specified for short orders). To simulate a stop order, pass a price value to the `stop` parameter of an applicable [order placement command](https://www.tradingview.com/pine-script-docs/concepts/strategies/#order-placement-and-cancellation).

If a strategy generates a stop order at a _better_ value than the current market price, it activates the subsequent order without waiting for the market price to reach that value.

The following example script calls the [strategy.entry()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.entry) with a `stop` argument to place a stop order 800 ticks above the current [close](https://www.tradingview.com/pine-script-reference/v6/#var_close) value while executing on the bar that is 100 bars before the last bar. It also draws a [label](https://www.tradingview.com/pine-script-reference/v6/#type_label) to indicate the bar on which it created the order, and it draws a horizontal [line](https://www.tradingview.com/pine-script-reference/v6/#type_line) at the stop price.

The following example calls [strategy.entry()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.entry) to place a stop order 800 ticks above the [close](https://www.tradingview.com/pine-script-reference/v6/#var_close) 100 bars before the last historical chart bar. It also draws a [label](https://www.tradingview.com/pine-script-reference/v6/#type_label) on the bar where it creates the order and a [line](https://www.tradingview.com/pine-script-reference/v6/#type_line) to display the stop price. As we see in the chart below, the strategy enters a long position immediately after the price crosses the stop level:

```pine
//@version=6
strategy("Stop order demo", overlay = true)

//@function Draws a line and a label with specified text at a given price level.
debugLabel(float price, string txt) =>
    label.new(
         bar_index, high, text = txt, color = color.teal, textcolor = color.white, 
         style = label.style_label_lower_right, size = size.large
     )
    line.new(bar_index, high, bar_index, price, style = line.style_dotted, color = color.teal)
    line.new(
         bar_index, price, bar_index + 1, price, color = color.teal, extend = extend.right, 
         style = line.style_dashed
     )

// Place a long stop order and draw a label and line 100 bars before the last bar.
if last_bar_index - bar_index == 100
    float stopPrice = close + syminfo.mintick * 800
    debugLabel(stopPrice, "Long Stop order created")
    strategy.entry("Long", strategy.long, stop = stopPrice)
```

Note that:

-   A basic stop order is essentially the _opposite_ of a limit order in terms of its execution based on the market price. If we use a limit order instead of a stop order in this scenario, the order executes immediately on the next bar. See the [Limit orders](https://www.tradingview.com/pine-script-docs/concepts/strategies/#limit-orders) section above for an example.

If a [strategy.entry()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.entry) or [strategy.order()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.order) call includes a `stop` _and_ `limit` argument, it creates a _stop-limit order_. Unlike a basic stop order, which triggers a market order when the current price is at the `stop` level or a worse value, a stop-limit order creates a subsequent _limit order_ to fill at the specified `limit` price.

Below, we modified the previous script to simulate and visualize a stop-limit order. This script version includes the bar’s [low](https://www.tradingview.com/pine-script-reference/v6/#var_low) as the `limit` price in the [strategy.entry()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.entry) command. It also creates additional drawings to show where the strategy activates the subsequent limit order and to visualize the limit price.

In example chart below, notice how the market price reaches the limit level on the next bar after the script creates the stop-limit order, but the strategy does not enter a position because the limit order is not yet active. After the market price reaches the stop level, the strategy places the subsequent limit order, and then the [broker emulator](https://www.tradingview.com/pine-script-docs/concepts/strategies/#broker-emulator) fills that order after the market price reverses to the limit level:

```pine
//@version=6
strategy("Stop-Limit order demo", overlay = true)

//@function Draws a line and a label with specified text at a given price level.
debugLabel(price, txt, lblColor, lineWidth = 1) =>
    label.new(
         bar_index, high, text = txt, color = lblColor, textcolor = color.white, 
         style = label.style_label_lower_right, size = size.large
     )
    line.new(bar_index, close, bar_index, price, style = line.style_dotted, color = lblColor, width = lineWidth)
    line.new(
         bar_index, price, bar_index + 1, price, color = lblColor, extend = extend.right, 
         style = line.style_dashed, width = lineWidth
     )

// Declare persistent variables to store the stop and limit prices.
var float stopPrice  = na
var float limitPrice = na

// Place a long stop-limit order and draw a label and lines 100 bars before the last bar.
if last_bar_index - bar_index == 100
    stopPrice  := close + syminfo.mintick * 800
    limitPrice := low
    debugLabel(limitPrice, "", color.gray)
    debugLabel(stopPrice, "Long Stop-Limit order created", color.teal)
    strategy.entry("Long", strategy.long, stop = stopPrice, limit = limitPrice)

// Draw another line and label when the strategy activates the limit order.
if high >= stopPrice
    debugLabel(limitPrice, "Limit order activated", color.green, 2)
    stopPrice := na
```

## Order placement and cancellation {#order-placement-and-cancellation}

The `strategy.*` namespace features the following five functions that simulate the placement of orders, known as _order placement commands_: [strategy.entry()](https://www.tradingview.com/pine-script-docs/concepts/strategies/#strategyentry), [strategy.order()](https://www.tradingview.com/pine-script-docs/concepts/strategies/#strategyorder), [strategy.exit()](https://www.tradingview.com/pine-script-docs/concepts/strategies/#strategyexit), [strategy.close()](https://www.tradingview.com/pine-script-docs/concepts/strategies/#strategyclose-and-strategyclose_all), and [strategy.close\_all()](https://www.tradingview.com/pine-script-docs/concepts/strategies/#strategyclose-and-strategyclose_all).

Additionally, the namespace includes the following two functions that cancel pending orders, known as _order cancellation commands_: [strategy.cancel()](https://www.tradingview.com/pine-script-docs/concepts/strategies/#strategycancel-and-strategycancel_all) and [strategy.cancel\_all()](https://www.tradingview.com/pine-script-docs/concepts/strategies/#strategycancel-and-strategycancel_all).

The sections below explain these commands, their unique characteristics, and how to use them.

### `strategy.entry()` {#strategyentry}

The [strategy.entry()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.entry) command generates _entry orders_. Its unique features help simplify opening and managing positions. This order placement command generates [market orders](https://www.tradingview.com/pine-script-docs/concepts/strategies/#market-orders) by default. It can also create [limit](https://www.tradingview.com/pine-script-docs/concepts/strategies/#limit-orders), [stop](https://www.tradingview.com/pine-script-docs/concepts/strategies/#stop-and-stop-limit-orders), and [stop-limit](https://www.tradingview.com/pine-script-docs/concepts/strategies/#stop-and-stop-limit-orders) orders by using the `limit` and `stop` parameters, as explained in the [Order types](https://www.tradingview.com/pine-script-docs/concepts/strategies/#order-types) section above.

#### Reversing positions {#reversing-positions}

One of the [strategy.entry()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.entry) command’s unique features is its ability to _reverse_ an open position automatically. By default, when an order from a [strategy.entry()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.entry) call executes while a position is open in the opposite direction, the command automatically _adds_ the position’s size to the new order’s size. The added quantity allows the order to close the current position and open a new position for the specified number of contracts/lots/shares/units in the new direction.

For instance, if a strategy has an open position of 15 shares in the [strategy.long](https://www.tradingview.com/pine-script-reference/v6/#const_strategy.long) direction, then calls [strategy.entry()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.entry) to place a new [market order](https://www.tradingview.com/pine-script-docs/concepts/strategies/#market-orders) in the [strategy.short](https://www.tradingview.com/pine-script-reference/v6/#const_strategy.short) direction, the size of the resulting transaction is the specified entry size **plus** 15 shares.

The example below demonstrates this behavior in action. On each 100th bar, the script calls [strategy.entry()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.entry) with the argument `qty = 15` to enter a long position of 15 shares. Then, 50 bars later, it executes another call with a `qty` argument of 5, to enter a short position of five shares. The script also highlights the chart’s background in blue when the long condition occurs, and in red when the short condition occurs, to indicate the bars where it places each order:

```pine
//@version=6
strategy("Reversing positions demo", overlay = true)

//@variable Is `true` on every 100th bar, and `false` otherwise.
bool buyCondition = bar_index % 100 == 0
//@variable Is `true` on every 50th bar, and `false` otherwise.
bool sellCondition = bar_index % 50 == 0
 
if buyCondition
    // Place a "buy" market order to close any short position and enter a long position of 15 shares.
    strategy.entry("buy", strategy.long, qty = 15)
else if sellCondition
    // Place a "sell" market order to close any long position and enter a short position of 5 shares.
    strategy.entry("sell", strategy.short, qty = 5)

// Highlight the background when the `buyCondition` or `sellCondition` value is `true`.
bgcolor(buyCondition  ? color.new(color.blue, 90) : sellCondition ? color.new(color.red, 90) : na)
```

Although the long and short orders open trades with different sizes, each trade marker on the chart above shows _20 shares_ as the traded quantity. These markers display the total size of each _transaction_, not the size of each resulting _position_. Each execution of one of the [strategy.entry()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.entry) calls automatically reverses the current position by adding the position’s size (e.g., 15 for a long trade) to the new entry size (e.g., 5 for a short entry), resulting in a total transaction size of 20 shares. However, the resulting positions are 15 shares for long entries and 5 shares for short entries.

> [!NOTE]
> The [strategy.risk.allow\_entry\_in()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.risk.allow_entry_in) function _overrides_ the allowed direction for the [strategy.entry()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.entry) command. If a script specifies a trade direction with this [risk management](https://www.tradingview.com/pine-script-docs/concepts/strategies/#risk-management) command, orders from [strategy.entry()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.entry) calls in the opposite direction _close_ an open position _without_ reversing it.

#### Pyramiding {#pyramiding}

Another unique characteristic of the [strategy.entry()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.entry) command is its connection to a strategy’s _pyramiding_ property. Pyramiding specifies the maximum number of open trades, from the orders created by [strategy.entry()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.entry) calls, that a strategy allows for a single position. After the number of open trades from [strategy.entry()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.entry) calls reaches the pyramiding limit, the strategy does not execute new orders from subsequent calls to the command until at least one of those trades closes.

Programmers can specify the default pyramiding limit for a strategy by including a `pyramiding` argument in the [strategy()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy) declaration statement. The default argument is 1, meaning the strategy can open new positions but cannot add to them using orders from [strategy.entry()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.entry) calls. Script users can adjust a strategy’s pyramiding limit via the “Pyramiding” input in the “Settings/Properties” tab.

The following example calls the [strategy.entry()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.entry) function to place a [market order](https://www.tradingview.com/pine-script-docs/concepts/strategies/#market-orders) once on every 25th bar. The order direction changes once every 100 bars. Therefore, each 100-bar cycle executes the command using the same direction four times. The script highlights the chart’s background for each bar on which the entry condition occurs.

As shown below, although the script calls the [strategy.entry()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.entry) command using the same `direction` argument four times per 100-bar cycle, only the _first_ call in each cycle results in a new trade if we run the script with the default settings. The others do not contribute to the open position because the script’s default `pyramiding` value is 1:

```pine
//@version=6
strategy("Pyramiding demo", overlay = true)

//@variable Represents the direction of the entry orders. A value of 1 means long, and -1 means short.
var int direction = 1
//@variable Is `true` once every 25 bars, and `false` otherwise.
bool entryCondition = bar_index % 25 == 0

// Change the `direction` value on every 100th bar.
if bar_index % 100 == 0
    direction *= -1

// Place a market order based on the current `direction` value when the `entryCondition` value is `true`.
if entryCondition
    strategy.entry("Entry", direction == 1 ? strategy.long : strategy.short)

//@variable When the entry condition occurs, is a blue color if the `direction` is 1 and a red color otherwise.
color bgColor = entryCondition ? (direction == 1 ? color.new(color.blue, 80) : color.new(color.red, 80)) : na
// Highlight the chart's background using the `bgColor` value. 
bgcolor(bgColor, title = "Order highlight")
```

If we include `pyramiding = 4` in the script’s [strategy()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy) declaration statement, the script can use [strategy.entry()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.entry) calls to open up to four trades in the same direction by default. After applying this change to our example script, all the script’s entry orders now result in new trades:

```pine
//@version=6
strategy("Pyramiding demo", overlay = true, pyramiding = 4)

//@variable Represents the direction of the entry orders. A value of 1 means long, and -1 means short.
var int direction = 1
//@variable Is `true` once every 25 bars, and `false` otherwise.
bool entryCondition = bar_index % 25 == 0

// Change the `direction` value on every 100th bar.
if bar_index % 100 == 0
    direction *= -1

// Place a market order based on the current `direction` value when the `entryCondition` value is `true`.
if entryCondition
    strategy.entry("Entry", direction == 1 ? strategy.long : strategy.short)

//@variable When the entry condition occurs, is a blue color if the `direction` is 1 and a red color otherwise.
color bgColor = entryCondition ? (direction == 1 ? color.new(color.blue, 80) : color.new(color.red, 80)) : na
// Highlight the chart's background using the `bgColor` value. 
bgcolor(bgColor, title = "Order highlight")
```

> [!IMPORTANT]
> In some cases, _price-based_ orders from the [strategy.entry()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.entry) command can cause a strategy’s entry count for a position to exceed the specified pyramiding limit. If multiple calls to this command generate [limit](https://www.tradingview.com/pine-script-docs/concepts/strategies/#limit-orders), [stop](https://www.tradingview.com/pine-script-docs/concepts/strategies/#stop-and-stop-limit-orders), or [stop-limit](https://www.tradingview.com/pine-script-docs/concepts/strategies/#stop-and-stop-limit-orders) orders on the _same tick_, the broker emulator fills each one that the price action triggers, regardless of the specified limit.

### `strategy.order()` {#strategyorder}

The [strategy.order()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.order) command generates a _basic order_. Unlike other order placement commands, which can behave differently based on a strategy’s properties and open trades, this command _ignores_ most properties, such as [pyramiding](https://www.tradingview.com/pine-script-docs/concepts/strategies/#pyramiding), and simply creates orders with the specified parameters. This command generates [market orders](https://www.tradingview.com/pine-script-docs/concepts/strategies/#market-orders) by default. It can also create [limit](https://www.tradingview.com/pine-script-docs/concepts/strategies/#limit-orders), [stop](https://www.tradingview.com/pine-script-docs/concepts/strategies/#stop-and-stop-limit-orders), and [stop-limit](https://www.tradingview.com/pine-script-docs/concepts/strategies/#stop-and-stop-limit-orders) orders by using the `limit` and `stop` parameters. Orders from [strategy.order()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.order) calls can open new positions and modify or close existing ones. When a strategy executes an order from this command, the resulting market position is the _net sum_ of the open position size and the filled order quantity.

The following script uses [strategy.order()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.order) calls to enter and exit positions. The strategy places a long [market order](https://www.tradingview.com/pine-script-docs/concepts/strategies/#market-orders) for 15 units once every 100 bars. On every 25th bar that is not a multiple of 100, it places a short market order for five units. The script also highlights the background to signify where the strategy places each “buy” and “sell” order:

```pine
//@version=6
strategy("`strategy.order()` demo", overlay = true)

//@variable Is `true` on every 100th bar, and `false` otherwise.
bool buyCondition = bar_index % 100 == 0
//@variable Is `true` on every 25th bar, and `false` otherwise.
bool sellCondition = bar_index % 25 == 0

if buyCondition
    // Place a "buy" market order to trade 15 units in the long direction. 
    strategy.order("buy", strategy.long, qty = 15)
else if sellCondition
    // Place a "sell" market order to trade 5 units in the short direction.
    strategy.order("sell", strategy.short, qty = 5)

// Highlight the background when the `buyCondition` or `sellCondition` value is `true`.
bgcolor(buyCondition ? color.new(color.blue, 90) : sellCondition ? color.new(color.red, 90) : na)
```

The strategy above never opens a _short position_. Unlike the [strategy.entry()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.entry) command, the [strategy.order()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.order) command _does not_ automatically [reverse](https://www.tradingview.com/pine-script-docs/concepts/strategies/#reversing-positions) open positions. After filling a “buy” order, the strategy has an open long position of 15 units. The three subsequent “sell” orders _reduce_ the position by _five_ units each, and 15 - 5 \* 3 = 0. In other words, the strategy opens a long position on every 100th bar and gradually reduces the size to 0 using three successive short orders. If we used the [strategy.entry()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.entry) command instead of [strategy.order()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.order) in this example, the strategy would alternate between entering long and short trades of 15 and five units, respectively.

### `strategy.exit()` {#strategyexit}

The [strategy.exit()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.exit) command generates _exit orders_. It features several unique behaviors that link to open trades, helping to simplify closing market positions and creating multi-level exits with _take-profit_, _stop-loss_, and _trailing stop_ orders.

Unlike other order placement commands, which can generate a _single order_ per call, each call to [strategy.exit()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.exit) can produce _more than one_ type of exit order, depending on its arguments. Additionally, a single call to this command can generate exit orders for _multiple entries_, depending on the specified `from_entry` argument and the strategy’s open trades.

#### Take-profit and stop-loss {#take-profit-and-stop-loss}

The most basic use of the [strategy.exit()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.exit) command is the placement of [limit orders](https://www.tradingview.com/pine-script-docs/concepts/strategies/#limit-orders) to trigger exits after earning enough money (take-profit), [stop orders](https://www.tradingview.com/pine-script-docs/concepts/strategies/#stop-and-stop-limit-orders) to trigger exits after losing too much money (stop-loss), or both (bracket).

Four parameters determine the prices of the command’s take-profit and stop-loss orders:

-   The `profit` and `loss` parameters accept _relative_ values representing the number of _ticks_ by which the market price must move away from the entry price to trigger an exit.
-   The `limit` and `stop` parameters accept _absolute_ values representing the specific _prices_ that trigger an exit when the market price reaches them.

When a [strategy.exit()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.exit) call includes arguments for the relative _and_ absolute parameters defining take-profit or stop-loss levels (`profit` and `limit` or `loss` and `stop`), it creates orders only at the levels expected to trigger exits _first_.

For instance, if the `profit` distance is 19 ticks and the `limit` level is 20 ticks past the entry price in the favorable direction, the [strategy.exit()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.exit) command places a take-profit order based on the `profit` argument, because the market price will move 19 ticks past the entry price before it reaches the `limit` value. In contrast, if the `profit` distance is 20 ticks and the `limit` level is 19 ticks past the entry price in the favorable direction, the command places a take-profit order at the `limit` value because the price will reach that value first.

> [!IMPORTANT]
> The [strategy.exit()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.exit) command’s `limit` and `stop` parameters **do not** behave the same as the `limit` and `stop` parameters of the [strategy.entry()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.entry) and [strategy.order()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.order) commands. Calling [strategy.entry()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.entry) or [strategy.order()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.order) with `limit` and `stop` arguments creates a single [stop-limit order](https://www.tradingview.com/pine-script-docs/concepts/strategies/#stop-and-stop-limit-orders). In contrast, calling [strategy.exit()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.exit) with both arguments creates **two exit orders**: a take-profit order at the `limit` price and a stop-loss order at the `stop` price.

The following example creates exit bracket (take-profit and stop-loss) orders with the [strategy.exit()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.exit) command. When the `buyCondition` value is `true`, the script calls the [strategy.entry()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.entry) command to place a “buy” [market order](https://www.tradingview.com/pine-script-docs/concepts/strategies/#market-orders). Then, it calls the [strategy.exit()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.exit) command with `limit` and `stop` arguments to create a take-profit order at the `limitPrice` value and a stop-loss order at the `stopPrice` value. The script plots the `limitPrice` and `stopPrice` values on the chart to visualize the exit order prices:

```pine
//@version=6
strategy("Take-profit and stop-loss demo", overlay = true)

//@variable Is `true` on every 100th bar, and `false` otherwise.
bool buyCondition = bar_index % 100 == 0

//@variable Stores the current take-profit order price. 
var float takeProfit = na
//@variable Stores the current stop-loss order price.
var float stopLoss = na

if buyCondition
    // Update the `takeProfit` and `stopLoss` values.
    if strategy.opentrades == 0
        takeProfit := close * 1.01
        stopLoss   := close * 0.99
    // Place a long market order. 
    strategy.entry("buy", strategy.long)
    // Place a take-profit order at the `takeProfit` value and a stop-loss order at the `stopLoss` value.
    strategy.exit("exit", "buy", limit = takeProfit, stop = stopLoss)

// Set the `takeProfit` and `stopLoss` values to `na` when the position closes.
if ta.change(strategy.closedtrades) > 0
    takeProfit := na
    stopLoss   := na

// Plot the `takeProfit` and `stopLoss` series to visualize the exit levels.
plot(takeProfit, "TP", color.green, style = plot.style_circles)
plot(stopLoss,   "SL", color.red,   style = plot.style_circles)
```

Note that:

-   We did not specify a `qty` or `qty_percent` argument in the [strategy.exit()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.exit) call, meaning it creates orders to exit 100% of the “buy” order’s size.
-   The [strategy.exit()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.exit) command’s exit orders _do not_ necessarily execute at the specified prices. Strategies can fill [limit orders](https://www.tradingview.com/pine-script-docs/concepts/strategies/#limit-orders) at _better_ prices and [stop orders](https://www.tradingview.com/pine-script-docs/concepts/strategies/#stop-and-stop-limit-orders) at _worse_ prices, depending on the range of values available to the [broker emulator](https://www.tradingview.com/pine-script-docs/concepts/strategies/#broker-emulator).

If a [strategy.exit()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.exit) call includes a `from_entry` argument, the resulting exit orders apply to only the existing entry orders that have a matching ID. If the specified `from_entry` value does not match the entry ID of any trade in the current position, the command _does not_ create any exit orders.

Below, we changed the `from_entry` argument of the [strategy.exit()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.exit) call in our previous script to `"buy2"`, instructing the command to create exit orders only for open trades that have the “buy2” entry ID. This version does not place _any_ exit orders, because the strategy does not create any entry order with the “buy2” ID:

```pine
//@version=6
strategy("Invalid `from_entry` ID demo", overlay = true)

//@variable Is `true` on every 100th bar, and `false` otherwise.
bool buyCondition = bar_index % 100 == 0

//@variable Stores the current take-profit order price. 
var float takeProfit = na
//@variable Stores the current stop-loss order price.
var float stopLoss = na

if buyCondition
    // Update the `takeProfit` and `stopLoss` values.
    if strategy.opentrades == 0
        takeProfit := close * 1.01
        stopLoss   := close * 0.99
    // Place a long market order. 
    strategy.entry("buy", strategy.long)
    // Attempt to place an exit bracket for "buy2" entries.
    // This call has no effect because the strategy does not create entry orders with the "buy2" ID.
    strategy.exit("exit", "buy2", limit = takeProfit, stop = stopLoss)

// Set the `takeProfit` and `stopLoss` values to `na` when the position closes.
if ta.change(strategy.closedtrades) > 0
    takeProfit := na
    stopLoss   := na

// Plot the `takeProfit` and `stopLoss` series to visualize the exit levels.
plot(takeProfit, "TP", color.green, style = plot.style_circles)
plot(stopLoss,   "SL", color.red,   style = plot.style_circles)
```

Note that:

-   If a [strategy.exit()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.exit) call _does not_ include a `from_entry` argument, it creates exit orders for _all_ open trades in the current position, regardless of their entry IDs. See the [Exits for multiple entries](https://www.tradingview.com/pine-script-docs/concepts/strategies/#exits-for-multiple-entries) section below to learn more.

#### Partial and multi-level exits {#partial-and-multi-level-exits}

Strategies can use more than one call to the [strategy.exit()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.exit) command to create successive _partial_ exit orders for the same entry ID. This behavior helps simplify the formation of multi-level exit strategies. To exit from a position using multiple [strategy.exit()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.exit) calls, include a `qty` or `qty_percent` argument in each call to specify the portion of the traded quantity to close. If the sum of the exit order sizes exceeds the open position, the strategy automatically _reduces_ their sizes to match the total size of the position.

> [!NOTE]
> If a [strategy.exit()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.exit) call includes _both_ `qty` and `qty_percent` arguments, the command uses the `qty` value to size the order and ignores the `qty_percent` value.

The following example demonstrates a simple strategy that creates two partial exit order brackets for an entry ID. When the `buyCondition` value is `true`, the script places a “buy” [market order](https://www.tradingview.com/pine-script-docs/concepts/strategies/#market-orders) for two shares with a [strategy.entry()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.entry) call, and it creates “exit1” and “exit2” bracket orders using two calls to [strategy.exit()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.exit). The first call uses a `qty` value of 1, and the second uses a `qty` value of 3:

```pine
//@version=6
strategy("Multi-level exit demo", "test", overlay = true, behind_chart = false)

//@variable Is `true` on every 100th bar, and `false` otherwise.
bool buyCondition = bar_index % 100 == 0

//@variable The take-profit price for "exit1" orders.
var float takeProfit1 = na
//@variable The take-profit price for "exit2" orders.
var float takeProfit2 = na
//@variable The stop-loss price for "exit1" orders.
var float stopLoss1 = na
//@variable The stop-loss price for "exit2" orders.
var float stopLoss2 = na

if buyCondition
    // Update the `takeProfit*` and `stopLoss*` values.
    if strategy.opentrades == 0
        takeProfit1 := close * 1.01
        takeProfit2 := close * 1.02
        stopLoss1   := close * 0.99
        stopLoss2   := close * 0.98
    // Place a long market order with a `qty` value of 2.
    strategy.entry("buy", strategy.long, qty = 2)
    // Place an "exit1" bracket with a `qty` value of 1 at the `takeProfit1` and `stopLoss1` levels.
    strategy.exit("exit1", "buy", limit = takeProfit1, stop = stopLoss1, qty = 1)
    // Place an "exit2" bracket with a `qty` value of 3 at the `takeProfit2` and `stopLoss2` levels.
    // The size of the resulting orders *decreases* to match the open position,
    // because the "exit1" bracket always fills first. 
    strategy.exit("exit2", "buy", limit = takeProfit2, stop = stopLoss2, qty = 3)

// Set the `takeProfit1` and `stopLoss1` values to `na` when the price reaches either value. 
if high >= takeProfit1 or low <= stopLoss1
    takeProfit1 := na
    stopLoss1   := na
// Set the `takeProfit2` and `stopLoss2` values to `na` when the price reaches either value. 
if high >= takeProfit2 or low <= stopLoss2
    takeProfit2 := na
    stopLoss2   := na

// Plot the `takeProfit*` and `stopLoss*` series to visualize the levels.
plot(takeProfit1, "TP1", color.green, style = plot.style_circles)
plot(takeProfit2, "TP2", color.green, style = plot.style_circles)
plot(stopLoss1,   "SL1", color.red,   style = plot.style_circles)
plot(stopLoss2,   "SL2", color.red,   style = plot.style_circles)
```

As we can see from the trade markers on the chart above, the strategy first executes the “exit1” take-profit or stop-loss order to reduce the open position by one share, leaving only one remaining share in the position. However, the “exit2” order bracket has a specified size of _three shares_, which exceeds the remaining position. Rather than using this specified quantity, the strategy automatically _reduces_ the “exit2” orders to _one_ share, allowing it to close the position successfully.

Note that:

-   The broker emulator fills only **one** exit order from the “exit1” bracket, **not both**. If a [strategy.exit()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.exit) call generates more than one exit order type for an entry ID, the strategy fills the only the _first_ triggered one and automatically _cancels_ the others.
-   The strategy reduces the “exit2” orders because all orders from the [strategy.exit()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.exit) calls automatically belong to the same [strategy.oca.reduce](https://www.tradingview.com/pine-script-reference/v6/#const_strategy.oca.reduce) _OCA group_ by default. See the [OCA groups](https://www.tradingview.com/pine-script-docs/concepts/strategies/#oca-groups) section below to learn more.

When creating multiple exit orders with _different_ [strategy.exit()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.exit) calls, it’s crucial to note that the orders from each call automatically _reserve_ a portion of the open position. The orders from one [strategy.exit()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.exit) call _cannot_ exit the portion of a position that a previous call already reserved.

For example, the script below generates a “buy” entry order for 20 shares with a [strategy.entry()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.entry) call, then creates “limit” and “stop” exit orders with two separate calls to [strategy.exit()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.exit), while executing on the bar that is 100 bars before the last bar. The exit commands specify a quantity of 19 shares for the “limit” order and 20 for the “stop” order:

```pine
//@version=6
strategy("Reserved exit demo", "test", overlay = true, behind_chart = false)

//@variable The price of the "limit" exit order.
var float limitPrice = na
//@variable The price of the "stop" exit order.
var float stopPrice = na
//@variable Is `true` 100 bars before the last chart bar, and `false` otherwise.
bool longCondition = last_bar_index - bar_index == 100

if longCondition
    // Update the `limitPrice` and `stopPrice` values.
    limitPrice := close * 1.01
    stopPrice  := close * 0.99
    // Place a long market order for 20 shares.
    strategy.entry("buy", strategy.long, 20)
    // Create a take-profit order for 19 shares at the `limitPrice` value.
    strategy.exit("limit", limit = limitPrice, qty = 19)
    // Create a stop-loss order at the `stopPrice` value. Although this call specifies a `qty` value of 20, the 
    // previous `strategy.exit()` call reserved 19 shares. 
    // Consequently, this call creates an exit order for only **one share**. 
    strategy.exit("stop", stop = stopPrice, qty = 20)

//@variable Is `true` if the strategy has an open position, and `false` otherwise.
bool showPlot = strategy.opentrades > 0

// Plot the `limitPrice` and `stopPrice` values when the `showPlot` value is `true`.
plot(showPlot ? limitPrice : na, "Limit (take-profit) price", color.green, 2, plot.style_linebr)
plot(showPlot ? stopPrice : na, "Stop (stop-loss) price", color.red, 2, plot.style_linebr)
```

Users who are unfamiliar with the [strategy.exit()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.exit) command’s unique behaviors might expect the above strategy to close the entire market position if it fills the “stop” order before the “limit” order. However, the trade markers on the chart below show that the “stop” order _reduces_ the position by only **one share**. The [strategy.exit()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.exit) call for the “limit” order executes _first_ in the code. Consequently, the “limit” exit order reserves _19 shares_ of the open position. This reservation leaves only one share available for the “stop” order to close, regardless of when the strategy fills it:

#### Trailing stops {#trailing-stops}

One of the [strategy.exit()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.exit) command’s key features is its ability to create _trailing stops_, i.e., stop-loss orders that trail behind the market price by a specified amount whenever it moves to a more favorable value (higher for long positions and lower for short positions).

This type of exit order has two components: an _activation level_ and a _trail offset_. The activation level is the value that the market price must cross to activate the trailing stop calculation. The trail offset is the distance by which the activated stop follows behind the market price as it reaches successively better values.

Three [strategy.exit()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.exit) parameters determine the activation level and the trail offset of a trailing stop order:

-   The `trail_price` parameter accepts an _absolute price value_ for the trailing stop’s activation level.
-   The `trail_points` parameter is an alternative way to specify the activation level. Its value represents the _tick distance_ from the entry price required to activate the trailing stop.
-   The `trail_offset` parameter accepts a value representing the order’s trail offset as a specified number of ticks.

To create and activate a trailing stop order, a [strategy.exit()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.exit) call must specify a `trail_offset` argument and either a `trail_price` or `trail_points` argument. If the call contains both `trail_price` and `trail_points` arguments, the command uses the level expected to activate the stop _first_. For instance, if the `trail_points` distance is 50 ticks and the `trail_price` value is 51 ticks past the entry price in the favorable direction, the [strategy.exit()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.exit) command uses the `trail_points` value to set the activation level because the market price will move that distance _before_ reaching the `trail_price` level.

The example below demonstrates how a trailing stop order works in detail. The strategy places a “Long” [market order](https://www.tradingview.com/pine-script-docs/concepts/strategies/#market-orders) with the [strategy.entry()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.entry) command 100 bars before the last chart bar. Then, it calls the [strategy.exit()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.exit) command with `trail_price` and `trail_offset` arguments on the following bar to create a trailing stop. The script uses [lines](https://www.tradingview.com/pine-script-docs/visuals/lines-and-boxes/#lines), [labels](https://www.tradingview.com/pine-script-docs/visuals/text-and-shapes/#labels), and a [plot](https://www.tradingview.com/pine-script-reference/v6/#fun_plot) to visualize the trailing stop’s behavior.

The green line on the chart shows the level that the market price must reach to activate the trailing stop order. After the price reaches that level from below, the script uses a blue plot to display the trailing stop’s price. Each time the market price reaches a new high after activating the trailing stop, the stop’s price _increases_ to maintain a maximum distance of `trailOffsetInput` ticks from the best value. The exit order _does not_ change its price level when the price decreases or does not reach a new high. Eventually, the market price crosses below the trailing stop, triggering an exit:

```pine
//@version=6
strategy("Trailing stop order demo", overlay = true, behind_chart = false)

//@variable The distance from the entry price required to activate the trailing stop.
int activationOffsetInput = input.int(1000, "Activation level offset (in ticks)", 0)
//@variable The distance the stop follows behind the highest `high` value after activation.
int trailOffsetInput = input.int(2000, "Trailing stop offset (in ticks)", 0)

//@variable Draws a label and an optional line at the specified price level.
debugDrawings(float price, string txt, color drawingColor, bool drawLine = false) =>
    label.new(
        bar_index, price, text = txt, color = drawingColor, textcolor = color.white,
        style = label.style_label_lower_right, size = size.large
    )
    if drawLine
        line.new(
            bar_index, price, bar_index + 1, price, color = drawingColor, extend = extend.right,
            style = line.style_dashed
        )

//@variable The level required to activate the trailing stop.
var float activationLevel = na
//@variable The price of the trailing stop.
var float trailingStop = na
//@variable The value that the trailing stop would have if it was currently active. 
float theoreticalStopPrice = high - trailOffsetInput * syminfo.mintick

// Place a long market order 100 bars before the last historical bar.
if last_bar_index - bar_index == 100
    strategy.entry("Long", strategy.long)

// Create and visualize the exit order on the next bar.
if last_bar_index - bar_index == 99
    // Update the `activationLevel` value.
    activationLevel := open + syminfo.mintick * activationOffsetInput
    // Create the trailing stop order that activates at the `activationLevel` value and trails behind the high by 
    // `trailOffsetInput` ticks. 
    strategy.exit(
         "Trailing Stop", from_entry = "Long", trail_price = activationLevel, 
         trail_offset = trailOffsetInput
     )
    // Create drawings to indicate the activation level.
    debugDrawings(activationLevel, "Trailing Stop Activation Level", color.green, true)

// Visualize the trailing stop's levels while the position is open.
if strategy.opentrades == 1
    // Create drawings when the high is above the `activationLevel` value for the first time to indicate when the 
    // stop activates. 
    if na(trailingStop) and high >= activationLevel
        debugDrawings(activationLevel, "Activation level crossed", color.green)
        trailingStop := theoreticalStopPrice
        debugDrawings(trailingStop, "Trailing Stop Activated", color.blue)
    // Otherwise, update the `trailingStop` value when the `theoreticalStopPrice` value reaches a new high.
    else if theoreticalStopPrice > trailingStop
        trailingStop := theoreticalStopPrice

// Plot the `trailingStop` series to visualize the trailing stop's price movement. 
plot(trailingStop, "Trailing Stop")
```

#### Exits for multiple entries {#exits-for-multiple-entries}

A single call to the [strategy.exit()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.exit) command can generate exit orders for _more than one_ entry in an open position, depending on the call’s `from_entry` value.

If an open position consists of two or more entries with the same ID, a single [strategy.exit()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.exit) call that uses the ID as the `from_entry` argument places exit orders for _every_ corresponding entry created _before_ or _on_ the bar where the call occurs.

For example, the following script periodically calls the [strategy.entry()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.entry) command on two consecutive bars to enter and add to a long position. Both calls use `"buy"` as the `id` argument. After creating the second entry order, the script calls the [strategy.exit()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.exit) command once with `"buy"` as its `from_entry` argument to generate _separate_ exit orders for each trade with that entry ID. When the market price reaches the `takeProfit` or `stopLoss` value, the [broker emulator](https://www.tradingview.com/pine-script-docs/concepts/strategies/#broker-emulator) fills _two_ exit orders and closes the position:

```pine
//@version=6
strategy("Exits for entries with the same ID demo", overlay = true, behind_chart = false, pyramiding = 2)

//@variable The take-profit price for exit commands.
var float takeProfit = na
//@variable The stop-loss price for exit commands.
var float stopLoss   = na

//@variable Is `true` on two consecutive bars in 100-bar cycles, and `false` otherwise. 
bool buyCondition = math.min(bar_index % 100, math.max(bar_index - 1, 0) % 100) == 0

if buyCondition
    // Place a "buy" market order to enter a trade. 
    strategy.entry("buy", strategy.long)
    // Calculate exits after a trade is open.
    if strategy.opentrades == 1
        // Update the `takeProfit` and `stopLoss` values.
        takeProfit := close * 1.01
        stopLoss   := close * 0.99
        // Place exit orders to close both "buy" trades.
        strategy.exit("exit", "buy", limit = takeProfit, stop = stopLoss)

// Set the `takeProfit` and `stopLoss` values to `na` after both trades close.
if ta.change(strategy.closedtrades) == 2
    takeProfit := na
    stopLoss   := na

// Plot the `takeProfit` and `stopLoss` values.
plot(takeProfit, "TP", color.green, style = plot.style_circles)
plot(stopLoss,   "SL", color.red,   style = plot.style_circles)
```

A single [strategy.exit()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.exit) call can also generate exit orders for _all_ entries in an open position, irrespective of entry ID, if it does _not_ include a `from_entry` argument.

Below, we changed the [strategy.entry()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.entry) instance in the above script to create an entry order with a distinct ID on each call, and we removed the `from_entry` argument from the [strategy.exit()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.exit) call. Because this script version does not specify the entries to which the exit orders apply, the [strategy.exit()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.exit) call creates orders for _every_ trade in the position:

```pine
//@version=6
strategy("Exits for entries with different IDs demo", overlay = true, behind_chart = false, pyramiding = 2)

//@variable The take-profit price for exit commands.
var float takeProfit = na
//@variable The stop-loss price for exit commands.
var float stopLoss   = na

//@variable Is `true` on two consecutive bars in 100-bar cycles, and `false` otherwise.
bool buyCondition = math.min(bar_index % 100, math.max(bar_index - 1, 0) % 100) == 0

if buyCondition
    // Place a long market order with a *unique ID*. 
    strategy.entry("buy" + str.tostring(strategy.opentrades + strategy.closedtrades), strategy.long)
    // Calculate exits after a trade is open.
    if strategy.opentrades == 1
        // Update the `takeProfit` and `stopLoss` values.
        takeProfit := close * 1.01
        stopLoss   := close * 0.99
        // Place exit orders for *all* entries in the position, irrespective of ID.
        strategy.exit("exit", limit = takeProfit, stop = stopLoss)

// Set the `takeProfit` and `stopLoss` values to `na` after both trades close.
if ta.change(strategy.closedtrades) == 2
    takeProfit := na
    stopLoss   := na

// Plot the `takeProfit` and `stopLoss` values.
plot(takeProfit, "TP", color.green, style = plot.style_circles)
plot(stopLoss,   "SL", color.red,   style = plot.style_circles)
```

It’s crucial to note that a call to [strategy.exit()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.exit) without a `from_entry` argument _persists_ and creates exit orders for all open trades in a position, regardless of _when_ the entries occur. This behavior can affect strategies that manage positions with multiple entries or exits. If a strategy has an open position and calls [strategy.exit()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.exit) on any bar without specifying a `from_entry` ID, it generates exit orders for each entry created _before_ or on that bar, and it _continues_ to generate exit orders for subsequent entries _after_ that bar until the position closes.

Let’s explore this behavior and how it works. The script below calls the [strategy.entry()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.entry) command to create a long entry order on each bar within a user-specified time range. It also calls the [strategy.exit()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.exit) command without a `from_entry` argument on only _one bar_ within that range to generate exit orders for _every_ entry in the open position. The exit command uses a `loss` value of 0, which means that an exit order fills each time the market price is not above one of the entry prices.

The script prompts the user to select three points before it starts its calculations. The first point specifies when order creation begins, the second determines when the single [strategy.exit()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.exit) call occurs, and the third specifies when order creation stops:

```pine
//@version=6
strategy("Exit persist demo", overlay = true, behind_chart = false, pyramiding = 100)

//@variable The time when order creation starts. 
int entryStartTime = input.time(0, "Start time for entries", confirm = true)
//@variable The time when the `strategy.exit()` call occurs.
int exitCallTime = input.time(0, "Exit call time", confirm = true)
//@variable The time when order creation stops.
int entryEndTime = input.time(0, "End time for entries", confirm = true)

// Raise a runtime error if incorrect timestamps are chosen.
if exitCallTime <= entryStartTime or entryEndTime <= exitCallTime or entryEndTime <= entryStartTime
    runtime.error("The input timestamps must follow this condition: entryStartTime < exitCallTime < entryEndTime.")

// Create variables to track entry and exit conditions. 
bool entriesStart = time == entryStartTime
bool callExit     = time == exitCallTime
bool entriesEnd   = time == entryEndTime
bool callEntry    = time >= entryStartTime and time < entryEndTime

// Place a long entry order when `callEntry` is `true`.
if callEntry
    strategy.entry("Entry", strategy.long)

// Call `strategy.exit()` when `callExit` is `true`, which occurs only once.
// This single call persists and creates exit orders for EVERY entry in the position because it does not 
// specify a `from_entry` ID.
if callExit
    strategy.exit("Exit", loss = 0)

// Draw labels to signify when entries start, when the `strategy.exit()` call occurs, and when order placement stops.
switch 
    entriesStart => label.new(
         bar_index, high, "Start placing entry orders.", color = color.green, textcolor = color.white, 
         style = label.style_label_lower_right, size = size.large
     )
    callExit => label.new(
         bar_index, high, "Call `strategy.exit()` once.", color = color.blue, textcolor = color.white, 
         style = label.style_label_lower_right, size = size.large
     )
    entriesEnd => label.new(
         bar_index, high, "Stop placing orders.", color = color.red, textcolor = color.white, 
         style = label.style_label_lower_left, size = size.large
     )

// Create a line and label to visualize the lowest entry price, i.e., the price required to close the position.
var line lowestLine = line.new(
     entryStartTime + 1000, na, entryEndTime, na, xloc.bar_time, extend.right, color.orange, width = 2
 )
var lowestLabel = label.new(
     entryStartTime + 1000, na, "Lowest entry price", color = color.orange, 
     style = label.style_label_upper_right, xloc = xloc.bar_time
 )

// Update the price values of the `lowestLine` and `lowestLabel` after each new entry.
if callEntry[1]
    var float lowestPrice = strategy.opentrades.entry_price(0)
    float entryPrice = strategy.opentrades.entry_price(strategy.opentrades - 1)
    if not na(entryPrice)
        lowestPrice := math.min(lowestPrice, entryPrice)
        lowestLine.set_y1(lowestPrice)
        lowestLine.set_y2(lowestPrice)
        lowestLabel.set_y(lowestPrice)

// Highlight the background when `entriesStart`, `callExit`, and `entriesEnd` occurs.
bgcolor(entriesStart ? color.new(color.green, 80) : na, title = "Entries start highlight")
bgcolor(callExit ? color.new(color.blue, 80) : na, title = "Exit call highlight")
bgcolor(entriesEnd ? color.new(color.red, 80) : na, title = "Entries end highlight")
```

Note that:

-   We included `pyramiding = 100` in the [strategy()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy) declaration statement, allowing the position to include up to 100 open trades from [strategy.entry()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.entry) orders.
-   The script uses [labels](https://www.tradingview.com/pine-script-docs/visuals/text-and-shapes/#labels) and a [bgcolor()](https://www.tradingview.com/pine-script-reference/v6/#fun_bgcolor) call to signify when order placement starts and stops and when the [strategy.exit()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.exit) call occurs.
-   The script draws a [line](https://www.tradingview.com/pine-script-docs/visuals/lines-and-boxes/#lines) and a label at the lowest entry price to show the value the market price must reach to close the entire position.

We can observe the unique exit behavior of the [strategy.exit()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.exit) command in this example by comparing the code itself with the script’s visuals. The script calls the [strategy.exit()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.exit) command _one time_, only on the bar with the blue label. However, that single call places exit orders for _every_ entry that occurs **before** or on that bar, then continues placing exit orders for each new entry **after** that bar. This behavior occurs because the [strategy.exit()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.exit) function cannot determine when to stop placing orders if it does not link to entries with a specific ID. In this case, the command only ceases to create new exit orders only after the position fully _closes_.

The above script exhibits different behaviors if we include a `from_entry` argument in the [strategy.exit()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.exit) call. If a call to this command specifies a `from_entry` ID, the exit orders apply to entries that the strategy created _before_ or _on_ the bar of the call using that ID. The command does not place exit orders for subsequent entries created _after_ the bar in that case, even if those entries have the same ID.

In the script version below, we added `from_entry = "Entry"` to our script’s [strategy.exit()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.exit) call to specify that it produces exit orders only for entries that have the “Entry” ID. Only 17 exits occur across the same range this time, each corresponding to an entry order created _before_ or _on_ the bar with the blue label. The call does not generate new orders for any trade that the strategy opens _after_ that bar, regardless of their entry ID:

```pine
//@version=6
strategy("Exit persist demo", overlay = true, behind_chart = false, pyramiding = 100)

//@variable The time when order creation starts. 
int entryStartTime = input.time(0, "Start time for entries", confirm = true)
//@variable The time when the `strategy.exit()` call occurs.
int exitCallTime = input.time(0, "Exit call time", confirm = true)
//@variable The time when order creation stops.
int entryEndTime = input.time(0, "End time for entries", confirm = true)

// Raise a runtime error if incorrect timestamps are chosen.
if exitCallTime <= entryStartTime or entryEndTime <= exitCallTime or entryEndTime <= entryStartTime
    runtime.error("The input timestamps must follow this condition: entryStartTime < exitCallTime < entryEndTime.")

// Create variables to track entry and exit conditions. 
bool entriesStart = time == entryStartTime
bool callExit     = time == exitCallTime
bool entriesEnd   = time == entryEndTime
bool callEntry    = time >= entryStartTime and time < entryEndTime

// Place a long entry order when `callEntry` is `true`.
if callEntry
    strategy.entry("Entry", strategy.long)

// Call `strategy.exit()` when `callExit` is `true`, which occurs only once.
// This single call only places exit orders for all entries with the "Entry" ID created before or on the bar where 
// `callExit` occurs. It DOES NOT affect any subsequent entries created after that bar.
if callExit
    strategy.exit("Exit", from_entry = "Entry", loss = 0)

// Draw labels to signify when entries start, when the `strategy.exit()` call occurs, and when order placement stops.
switch 
    entriesStart => label.new(
         bar_index, high, "Start placing entry orders.", color = color.green, textcolor = color.white, 
         style = label.style_label_lower_right, size = size.large
     )
    callExit => label.new(
         bar_index, high, "Call `strategy.exit()` once.", color = color.blue, textcolor = color.white, 
         style = label.style_label_lower_right, size = size.large
     )
    entriesEnd => label.new(
         bar_index, high, "Stop placing orders.", color = color.red, textcolor = color.white, 
         style = label.style_label_lower_left, size = size.large
     )

// Create a line and label to visualize the lowest entry price, i.e., the price required to close the position.
var line lowestLine = line.new(
     entryStartTime + 1000, na, entryEndTime, na, xloc.bar_time, extend.right, color.orange, width = 2
 )
var lowestLabel = label.new(
     entryStartTime + 1000, na, "Lowest entry price", color = color.orange, 
     style = label.style_label_upper_right, xloc = xloc.bar_time
 )

// Update the price values of the `lowestLine` and `lowestLabel` after each new entry.
if callEntry[1]
    var float lowestPrice = strategy.opentrades.entry_price(0)
    float entryPrice = strategy.opentrades.entry_price(strategy.opentrades - 1)
    if not na(entryPrice)
        lowestPrice := math.min(lowestPrice, entryPrice)
        lowestLine.set_y1(lowestPrice)
        lowestLine.set_y2(lowestPrice)
        lowestLabel.set_y(lowestPrice)

// Highlight the background when `entriesStart`, `callExit`, and `entriesEnd` occurs.
bgcolor(entriesStart ? color.new(color.green, 80) : na, title = "Entries start highlight")
bgcolor(callExit ? color.new(color.blue, 80) : na, title = "Exit call highlight")
bgcolor(entriesEnd ? color.new(color.red, 80) : na, title = "Entries end highlight")
```

### `strategy.close()` and `strategy.close_all()` {#strategyclose-and-strategyclose_all}

The [strategy.close()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.close) and [strategy.close\_all()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.close_all) commands generate orders to exit from an open position. Unlike [strategy.exit()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.exit), which creates _price-based_ exit orders (e.g., [stop-loss](https://www.tradingview.com/pine-script-docs/concepts/strategies/#take-profit-and-stop-loss)), these commands generate [market orders](https://www.tradingview.com/pine-script-docs/concepts/strategies/#market-orders) that the [broker emulator](https://www.tradingview.com/pine-script-docs/concepts/strategies/#broker-emulator) fills on the next available tick, irrespective of the price.

The example script below demonstrates a simple strategy that enters and exits a position using only [market orders](https://www.tradingview.com/pine-script-docs/concepts/strategies/#market-orders). The script executes a [strategy.entry()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.entry) call to place a “buy” market order once every 50 bars, then calls the [strategy.close()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.close) command to close any open “buy” trade 25 bars later:

```pine
//@version=6
strategy("Close demo", "test", overlay = true)

//@variable Is `true` on every 50th bar.
buyCond = bar_index % 50 == 0
//@variable Is `true` on every 25th bar except for those that are divisible by 50.
sellCond = bar_index % 25 == 0 and not buyCond

if buyCond
    strategy.entry("buy", strategy.long)
if sellCond
    strategy.close("buy")

bgcolor(buyCond  ? color.new(color.blue, 90) : na)
bgcolor(sellCond ? color.new(color.red, 90) : na)
```

Notice that the [strategy.close()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.close) call in this script uses “buy” as its required `id` argument. Unlike [strategy.exit()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.exit), this command’s `id` parameter specifies the _entry ID_ of an open trade. It **does not** represent the ID of the resulting exit order. If a market position consists of multiple open trades with the same entry ID, a single [strategy.close()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.close) call with that ID as its `id` argument generates a single [market order](https://www.tradingview.com/pine-script-docs/concepts/strategies/#market-orders) to close all of those trades in one transaction.

The following script creates a “buy” order using a [strategy.entry()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.entry) call once every 25 bars, and it calls the [strategy.close()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.close) command with `"buy"` as its `id` argument to close all open trades with that entry ID once every 100 bars. The market order from the [strategy.close()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.close) call closes the entire position in this case because every open trade in the position has the same “buy” entry ID:

```pine
//@version=6
strategy("Multiple close demo", "test", overlay = true, pyramiding = 3)

//@variable Is `true` on every 100th bar.
sellCond = bar_index % 100 == 0
//@variable Is `true` on every 25th bar except for those that are divisible by 100.
buyCond = bar_index % 25 == 0 and not sellCond

if buyCond
    strategy.entry("buy", strategy.long)
if sellCond
    strategy.close("buy")

bgcolor(buyCond  ? color.new(color.blue, 90) : na)
bgcolor(sellCond ? color.new(color.red, 90) : na)
```

Note that:

-   We included `pyramiding = 3` in the [strategy()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy) declaration statement, allowing the script to generate up to three entries per position using [strategy.entry()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.entry) calls.

The [strategy.close\_all()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.close_all) command generates a [market order](https://www.tradingview.com/pine-script-docs/concepts/strategies/#market-orders) to close any open position. Unlike [strategy.close()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.close), this command _does not_ link to any specific entry ID. Using the [strategy.close\_all()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.close_all) command is helpful when a strategy must exit as soon as possible from a position consisting of multiple open trades with different entry IDs.

The script below places “A”, “B”, and “C” entry orders sequentially based on the number of open trades as tracked by the [strategy.opentrades](https://www.tradingview.com/pine-script-reference/v6/#var_strategy.opentrades) variable. Then, it calls the [strategy.close\_all()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.close_all) command to create a single order that closes the entire position on the following bar:

```pine
//@version=6
strategy("Close multiple ID demo", "test", overlay = true, pyramiding = 3)

switch strategy.opentrades
    0 => strategy.entry("A", strategy.long)
    1 => strategy.entry("B", strategy.long)
    2 => strategy.entry("C", strategy.long)
    3 => strategy.close_all()
```

### `strategy.cancel()` and `strategy.cancel_all()` {#strategycancel-and-strategycancel_all}

The [strategy.cancel()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.cancel) and [strategy.cancel\_all()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.cancel_all) commands allow strategies to cancel _unfilled_ orders before the [broker emulator](https://www.tradingview.com/pine-script-docs/concepts/strategies/#broker-emulator) processes them. These order cancellation commands are most helpful when working with _price-based orders_, including all orders from [strategy.exit()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.exit) calls and the orders from [strategy.entry()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.entry) or [strategy.order()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.order) calls that include `limit` or `stop` arguments.

The [strategy.cancel()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.cancel) command has a required `id` parameter, which specifies the ID of the entry or exit orders to cancel. The [strategy.cancel\_all()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.cancel_all) command does not have such a parameter because it cancels _all_ unfilled orders, regardless of ID.

The following strategy calls the [strategy.entry()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.entry) command to place a “buy” [limit order](https://www.tradingview.com/pine-script-docs/concepts/strategies/#limit-orders) 500 ticks below the closing price while executing on the bar that is 100 bars before the latest bar. Then, on the next bar, it calls the [strategy.cancel()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.cancel) command to cancel the order before the broker emulator executes it.

The following strategy places a “buy” [limit order](https://www.tradingview.com/pine-script-docs/concepts/strategies/#limit-orders) 500 ticks below the closing price 100 bars before the last chart bar with [strategy.entry()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.entry), and it cancels the order on the next bar with [strategy.cancel()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.cancel). The script highlights the chart’s background to signify when it places and cancels the “buy” order, and it draws a horizontal [line](https://www.tradingview.com/pine-script-docs/visuals/lines-and-boxes/#lines) at the order’s price. As shown below, our example chart shows no entry marker when the market price crosses the horizontal line, because the strategy already cancels the order (when the chart’s background is orange) before the price reaches that level:

```pine
//@version=6
strategy("Cancel demo", "test", overlay = true)

//@variable Draws a horizontal line at the `limit` price of the "buy" order.
var line limitLine = na

//@variable Is `color.green` when the strategy places the "buy" order, `color.orange` when it cancels the order.
color bgColor = na

if last_bar_index - bar_index == 100
    float limitPrice = close - syminfo.mintick * 500
    strategy.entry("buy", strategy.long, limit = limitPrice)
    limitLine := line.new(bar_index, limitPrice, bar_index + 1, limitPrice, extend = extend.right)
    bgColor := color.new(color.green, 50)

if last_bar_index - bar_index == 99
    strategy.cancel("buy")
    bgColor := color.new(color.orange, 50)

bgcolor(bgColor)
```

The [strategy.cancel()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.cancel) command cancels _all_ unfilled orders that have the specified entry ID. It does nothing if the specified `id` represents the ID of an order that does not exist. If the strategy has more than one unfilled order with the same entry ID, the command cancels _all_ of them at once.

Below, we modified the previous script to place a “buy” limit order on three consecutive bars, starting 100 bars before the last chart bar. After placing all three orders, the strategy cancels them using a single [strategy.cancel()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.cancel) call with “buy” as the `id` argument, resulting in no new trades when the market price reaches any of the order prices indicated by the horizontal lines:

```pine
//@version=6
strategy("Multiple cancel demo", "test", overlay = true, pyramiding = 3)

//@variable Draws a horizontal line at the `limit` price of the "buy" order.
var line limitLine = na

//@variable Is `color.green` when the strategy places the "buy" order, `color.orange` when it cancels the order.
color bgColor = na

if last_bar_index - bar_index <= 100 and last_bar_index - bar_index >= 98
    float limitPrice = close - syminfo.mintick * 500
    strategy.entry("buy", strategy.long, limit = limitPrice)
    limitLine := line.new(bar_index, limitPrice, bar_index + 1, limitPrice, extend = extend.right)
    bgColor := color.new(color.green, 50)

if last_bar_index - bar_index == 97
    strategy.cancel("buy")
    bgColor := color.new(color.orange, 50)

bgcolor(bgColor)
```

Note that:

-   We included `pyramiding = 3` in the [strategy()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy) declaration statement, allowing three successive entries from [strategy.entry()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.entry) calls per position. The script would also achieve the same result without this setting if it called the [strategy.order()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.order) command instead, because a strategy’s [pyramiding](https://www.tradingview.com/pine-script-docs/concepts/strategies/#pyramiding) setting _does not_ affect orders from that command.

The [strategy.cancel()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.cancel) and [strategy.cancel\_all()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.cancel_all) commands can cancel orders of any type, including [market orders](https://www.tradingview.com/pine-script-docs/concepts/strategies/#market-orders). However, it is important to note that either command can cancel a market order only if its call occurs on the _same_ script execution as the order placement command. The cancellation command has _no effect_ if the call happens after that point, because the [broker emulator](https://www.tradingview.com/pine-script-docs/concepts/strategies/#broker-emulator) always fills market orders by the _next available tick_.

The example below places a “buy” market order with a [strategy.entry()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.entry) call 100 bars before the latest bar. Then, it attempts to cancel that order with a [strategy.cancel\_all()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.cancel_all) call on the next bar. The cancellation command _does not_ affect the “buy” order in this case. The broker emulator fills the market order on the next bar’s opening tick, _before_ the script executes the [strategy.cancel\_all()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.cancel_all) call:

```pine
//@version=6
strategy("Cancel market demo", overlay = true)

//@variable Is `color.green` when the strategy places the "buy" order, `color.orange` when it tries to cancel the order.
color bgColor = na

if last_bar_index - bar_index == 100
    strategy.entry("buy", strategy.long)
    bgColor := color.new(color.green, 50)

if last_bar_index - bar_index == 99
    strategy.cancel_all()
    bgColor := color.new(color.orange, 50)

bgcolor(bgColor)
```

## Position sizing {#position-sizing}

Pine Script strategies feature two ways to control the sizes of the orders that open and modify positions:

-   Set a default _fixed_ quantity type and value for the orders. Programmers can specify defaults for these properties by including [`default_qty_type` and `default_qty_value`](https://www.tradingview.com/pine-script-docs/language/declaration-statements/#default_qty_type-and-default_qty_value) arguments in the [strategy()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy) declaration statement. Script users can adjust these defaults via the “Default order size” inputs in the “Settings/Properties” tab.
-   Include a _non-na_ `qty` argument in the [strategy.entry()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.entry) or [strategy.order()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.order) call. If a call to either of these commands specifies a non-na `qty` value, that call _ignores_ the strategy’s default quantity type and value and places an order for `qty` contracts/shares/lots/units instead.

The following example uses [strategy.entry()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.entry) calls with different `qty` arguments for long and short trades. When the current bar’s [low](https://www.tradingview.com/pine-script-reference/v6/#var_low) equals the `lowest` value, the script places a “Buy” order to enter a long position of `longAmount` units. Otherwise, when the [high](https://www.tradingview.com/pine-script-reference/v6/#var_high) equals the `highest` value, it places a “Sell” order to enter a short position of `shortAmount` units:

```pine
//@version=6
strategy(
    "Orders with specified quantities demo", overlay = true, 
    default_qty_type = strategy.cash, default_qty_value = 5000
)

int   length      = input.int(20, "Length", 1)
float longAmount  = input.float(4.0, "Long Amount", 0.0)
float shortAmount = input.float(2.0, "Short Amount", 0.0)

float highest = ta.highest(length)
float lowest  = ta.lowest(length)

switch
    low == lowest   => strategy.entry("Buy", strategy.long, longAmount)
    high == highest => strategy.entry("Sell", strategy.short, shortAmount)
```

Notice that although we’ve included `default_qty_type` and `default_qty_value` arguments in the [strategy()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy) declaration statement, the strategy _does not_ use this default setting to size its orders. Instead, the specified `qty` value in the entry commands takes precedence. To use the strategy’s default order size, we must _remove_ the `qty` arguments from the [strategy.entry()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.entry) calls or set their values to [na](https://www.tradingview.com/pine-script-reference/v6/#var_na).

Below, we edited the previous script by including [ternary](https://www.tradingview.com/pine-script-docs/language/operators/#-ternary-operator) expressions for the `qty` arguments in both [strategy.entry()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.entry) calls. These expressions replace input values of 0 with [na](https://www.tradingview.com/pine-script-reference/v6/#var_na). Now, if the specified `longAmount` or `shortAmount` value is 0, the corresponding entry orders use the strategy’s default order size instead, as we see below:

```pine
//@version=6
strategy(
    "Orders using default quantities demo", overlay = true, 
    default_qty_type = strategy.cash, default_qty_value = 5000
)

int   length      = input.int(20, "Length", 1)
float longAmount  = input.float(0.0, "Long Amount", 0.0)
float shortAmount = input.float(0.0, "Short Amount", 0.0)

float highest = ta.highest(length)
float lowest  = ta.lowest(length)

switch
    low == lowest   => strategy.entry("Buy", strategy.long, longAmount == 0.0 ? na : longAmount)
    high == highest => strategy.entry("Sell", strategy.short, shortAmount == 0.0 ? na : shortAmount)
```

## Closing a market position {#closing-a-market-position}

By default, strategies close a market position using the _First In, First Out (FIFO)_ method. If a strategy uses this behavior, any exit order closes or reduces the position starting with the _first_ open trade, even if the exit command specifies the entry ID of a _different_ open trade. To override this default behavior, include `close_entries_rule = "ANY"` in the [strategy()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy) declaration statement.

The following example script places “Buy1” and “Buy2” entry orders sequentially, starting 100 bars before the latest chart bar. When the position size is 0, it calls the [strategy.entry()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.entry) command to place the “Buy1” order for five units. After the strategy’s position size matches the size of that order, the second [strategy.entry()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.entry) call places the “Buy2” order for ten units. The strategy then creates “bracket” exit orders [for both entries](https://www.tradingview.com/pine-script-docs/concepts/strategies/#exits-for-multiple-entries) using a single [strategy.exit()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.exit) call without a `from_entry` argument. For visual reference, the script also plots the [strategy.position\_size](https://www.tradingview.com/pine-script-reference/v6/#var_strategy.position_size) value in a separate pane:

```pine
//@version=6
strategy("Exit demo", pyramiding = 2)

float positionSize = strategy.position_size

if positionSize == 0 and last_bar_index - bar_index <= 100
    strategy.entry("Buy1", strategy.long, 5)
else if positionSize == 5
    strategy.entry("Buy2", strategy.long, 10)
else if positionSize == 15
    strategy.exit("bracket", loss = 10, profit = 10)

plot(positionSize, "Position size", color.lime, 4, plot.style_histogram)
```

Note that:

-   We included `pyramiding = 2` in the [strategy()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy) declaration statement, allowing two successive entries from [strategy.entry()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.entry) calls per position.

Each time the market price triggers an exit order, the above script exits from the open position, starting with the _oldest_ open trade. This FIFO behavior applies even if we explicitly specify an exit from “Buy2” before “Buy1” in the code.

The script version below calls the [strategy.close()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.close) command with “Buy2” as the `id` argument, and it includes “Buy1” as the `from_entry` argument in the [strategy.exit()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.exit) call. The [market order](https://www.tradingview.com/pine-script-docs/concepts/strategies/#market-orders) from the [strategy.close()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.close) call executes on the next available tick. Therefore, the [broker emulator](https://www.tradingview.com/pine-script-docs/concepts/strategies/#broker-emulator) fills it _before_ the [take-profit](https://www.tradingview.com/pine-script-docs/concepts/strategies/#take-profit-and-stop-loss) and [stop-loss](https://www.tradingview.com/pine-script-docs/concepts/strategies/#take-profit-and-stop-loss) orders from the [strategy.exit()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.exit) call:

```pine
//@version=6
strategy("Exit demo", pyramiding = 2)

float positionSize = strategy.position_size

if positionSize == 0 and last_bar_index - bar_index <= 100
    strategy.entry("Buy1", strategy.long, 5)
else if positionSize == 5
    strategy.entry("Buy2", strategy.long, 10)
else if positionSize == 15
    strategy.close("Buy2")
    strategy.exit("bracket", "Buy1", loss = 10, profit = 10)

plot(positionSize, "Position size", color.lime, 4, plot.style_histogram)
```

The market order from the script’s [strategy.close()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.close) call is for 10 units because it links to the open trade with the “Buy2” entry ID. A user might expect this strategy to close that trade completely when the order executes. However, the [“Trades” tab](https://www.tradingview.com/pine-script-docs/concepts/strategies/#trades-tab) in the strategy report shows that _five_ units of the order close the “Buy1” trade _first_ because it is the oldest, and the remaining five units close _half_ of the “Buy2” trade. Then, the “bracket” orders from the [strategy.exit()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.exit) call close the rest of the position later:

Note that:

-   If we included `close_entries_rule = "ANY"` in the [strategy()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy) declaration statement, the market order from the [strategy.close()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.close) call would close the open trade with the “Buy2” entry ID _first_, and then the “bracket” orders from the [strategy.exit()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.exit) call would close the trade with the “Buy1” entry ID.

## OCA groups {#oca-groups}

_One-Cancels-All (OCA)_ groups allow a strategy to fully or partially _cancel_ specific orders when the [broker emulator](https://www.tradingview.com/pine-script-docs/concepts/strategies/#broker-emulator) executes another order from the same group. To assign an order to an OCA group, include an `oca_name` argument in the call to the [order placement command](https://www.tradingview.com/pine-script-docs/concepts/strategies/#order-placement-and-cancellation). The [strategy.entry()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.entry) and [strategy.order()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.order) commands also allow programmers to specify an _OCA type_, which defines whether a strategy [cancels](https://www.tradingview.com/pine-script-docs/concepts/strategies/#strategyocacancel), [reduces](https://www.tradingview.com/pine-script-docs/concepts/strategies/#strategyocareduce), or [does not modify](https://www.tradingview.com/pine-script-docs/concepts/strategies/#strategyocanone) the order after executing other orders.

> [!NOTE]
> All order placement commands that issue orders for the same OCA group must specify the same group name **and** OCA type. If two commands have the same `oca_name` value but _different_ `oca_type` values, the strategy considers them to be from **two distinct groups**. In other words, an OCA group **cannot** mix the [strategy.oca.cancel](https://www.tradingview.com/pine-script-reference/v6/#const_strategy.oca.cancel), [strategy.oca.reduce](https://www.tradingview.com/pine-script-reference/v6/#const_strategy.oca.reduce), and [strategy.oca.none](https://www.tradingview.com/pine-script-reference/v6/#const_strategy.oca.none) OCA types.

### `strategy.oca.cancel` {#strategyocacancel}

If an order placement command uses [strategy.oca.cancel](https://www.tradingview.com/pine-script-reference/v6/#const_strategy.oca.cancel) as its `oca_type` argument, the strategy completely _cancels_ the resulting order if another order from the same OCA group executes first.

To understand how this OCA type impacts a strategy’s orders, consider the following script, which places orders when the `ma1` value crosses the `ma2` value. If the [strategy.position\_size](https://www.tradingview.com/pine-script-reference/v6/#var_strategy.position_size) value is 0 when the cross occurs, the strategy places two [stop orders](https://www.tradingview.com/pine-script-docs/concepts/strategies/#stop-and-stop-limit-orders) using [strategy.order()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.order) calls. The first is a long order at the bar’s high, and the second is a short order at the bar’s low. If the strategy already has an open position during the cross, it calls the [strategy.close\_all()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.close_all) command to close the position with a [market order](https://www.tradingview.com/pine-script-docs/concepts/strategies/#market-orders):

```pine
//@version=6
strategy("OCA Cancel Demo", overlay=true)

float ma1 = ta.sma(close, 5)
float ma2 = ta.sma(close, 9)

if ta.cross(ma1, ma2)
    if strategy.position_size == 0
        strategy.order("Long",  strategy.long, stop = high)
        strategy.order("Short", strategy.short, stop = low)
    else
        strategy.close_all()

plot(ma1, "Fast MA", color.aqua)
plot(ma2, "Slow MA", color.orange)
```

Depending on the price action, the strategy might fill _both_ stop orders before creating the closing market order. In that case, the strategy exits the position without executing the [strategy.close\_all()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.close_all) call because both orders have the same size. We see this behavior in the chart below, where the strategy alternated between filling the “Long” and “Short” orders a few times without executing an order from the [strategy.close\_all()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.close_all) command:

To eliminate cases in which the strategy fills the “Long” and “Short” orders before evaluating the [strategy.close\_all()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.close_all) call, we can instruct it to _cancel_ one of the orders after the broker emulator fills the other. Below, we included “Entry” as the `oca_name` argument and [strategy.oca.cancel](https://www.tradingview.com/pine-script-reference/v6/#const_strategy.oca.cancel) as the `oca_type` argument in both [strategy.order()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.order) calls. Now, after the either the “Long” or “Short” order fills, the strategy cancels the other order and waits for the market order from the [strategy.close\_all()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.close_all) command to close the position:

```pine
//@version=6
strategy("OCA Cancel Demo", overlay=true)

float ma1 = ta.sma(close, 5)
float ma2 = ta.sma(close, 9)

if ta.cross(ma1, ma2)
    if strategy.position_size == 0
        strategy.order("Long",  strategy.long, stop = high, oca_name = "Entry", oca_type = strategy.oca.cancel)
        strategy.order("Short", strategy.short, stop = low, oca_name = "Entry", oca_type = strategy.oca.cancel)
    else
        strategy.close_all()

plot(ma1, "Fast MA", color.aqua)
plot(ma2, "Slow MA", color.orange)
```

### `strategy.oca.reduce` {#strategyocareduce}

If an order placement command uses [strategy.oca.reduce](https://www.tradingview.com/pine-script-reference/v6/#const_strategy.oca.reduce) as its OCA type, the strategy _does not_ cancel the resulting order entirely if another order with the same OCA name executes first. Instead, it _reduces_ the order’s size by the filled number of contracts/shares/lots/units. This behavior is particularly useful for custom exit strategies.

The following example demonstrates a _long-only_ strategy that generates a single stop-loss order and two take-profit orders for each new entry. When a faster moving average crosses over a slower one, the script calls the [strategy.entry()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.entry) command with the argument `qty = 6` to create an entry order. Then, it uses three [strategy.order()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.order) calls to create a [stop order](https://www.tradingview.com/pine-script-docs/concepts/strategies/#stop-and-stop-limit-orders) at the `stop` value and two [limit orders](https://www.tradingview.com/pine-script-docs/concepts/strategies/#limit-orders) at the `limit1` and `limit2` values. The [strategy.order()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.order) call for the “Stop” order uses the argument `qty = 6`, and the two calls for the “Limit 1” and “Limit 2” orders both use `qty = 3`:

```pine
//@version=6
strategy("Multiple limits without reduction demo", overlay = true, behind_chart = false)

var float stop   = na
var float limit1 = na
var float limit2 = na

bool longCondition = ta.crossover(ta.sma(close, 5), ta.sma(close, 9))
if longCondition and strategy.position_size == 0
    stop   := close * 0.99
    limit1 := close * 1.01
    limit2 := close * 1.02
    strategy.entry("Long",  strategy.long, 6)
    strategy.order("Stop",  strategy.short, stop = stop, qty = 6)
    strategy.order("Limit 1", strategy.short, limit = limit1, qty = 3)
    strategy.order("Limit 2", strategy.short, limit = limit2, qty = 3)

bool showPlot = strategy.position_size != 0
plot(showPlot ? stop   : na, "Stop",    color.red,   style = plot.style_linebr)
plot(showPlot ? limit1 : na, "Limit 1", color.green, style = plot.style_linebr)
plot(showPlot ? limit2 : na, "Limit 2", color.green, style = plot.style_linebr)
```

After adding this strategy to the chart, we see that it does not work as initially intended. The problem with this script is that the orders from the [strategy.order()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.order) command **do not** belong to an OCA group by default (unlike [strategy.exit()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.exit), whose orders automatically belong to a [strategy.oca.reduce](https://www.tradingview.com/pine-script-reference/v6/#const_strategy.oca.reduce) OCA group). Because the strategy does not assign the [strategy.order()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.order) calls to any OCA group, it cannot reduce any unfilled stop or limit orders after a pending order executes. Consequently, if the [broker emulator](https://www.tradingview.com/pine-script-docs/concepts/strategies/#broker-emulator) fills the stop order and at least one of the limit orders, the traded quantity **exceeds** the open long position, resulting in an open _short_ position:

For our long-only strategy to work as we intended, we must instruct it to _reduce_ the sizes of the unfilled stop/limit orders after one of them executes to prevent selling a larger quantity than the open long position.

Below, we specified “Bracket” as the `oca_name` argument and [strategy.oca.reduce](https://www.tradingview.com/pine-script-reference/v6/#const_strategy.oca.reduce) as the `oca_type` argument in all the script’s [strategy.order()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.order) calls. These changes tell the strategy to reduce the sizes of the orders in the “Bracket” group each time the broker emulator fills one of them. This version of the strategy never simulates a short position, because the total size of its filled stop and limit orders never _exceeds_ the long position’s size:

```pine
//@version=6
strategy("Multiple limits with reduction demo", overlay = true, behind_chart = false)

var float stop   = na
var float limit1 = na
var float limit2 = na

bool longCondition = ta.crossover(ta.sma(close, 5), ta.sma(close, 9))
if longCondition and strategy.position_size == 0
    stop   := close * 0.99
    limit1 := close * 1.01
    limit2 := close * 1.02
    strategy.entry("Long",  strategy.long, 6)
    strategy.order("Stop",  strategy.short, stop = stop, qty = 6, oca_name = "Bracket", oca_type = strategy.oca.reduce)
    strategy.order("Limit 1", strategy.short, limit = limit1, qty = 3, oca_name = "Bracket", oca_type = strategy.oca.reduce)
    strategy.order("Limit 2", strategy.short, limit = limit2, qty = 6, oca_name = "Bracket", oca_type = strategy.oca.reduce)

bool showPlot = strategy.position_size != 0
plot(showPlot ? stop   : na, "Stop",    color.red,   style = plot.style_linebr)
plot(showPlot ? limit1 : na, "Limit 1", color.green, style = plot.style_linebr)
plot(showPlot ? limit2 : na, "Limit 2", color.green, style = plot.style_linebr)
```

Note that:

-   We also changed the `qty` value of the “Limit 2” order to 6 instead of 3 because the strategy reduces that order’s amount by three units after it executes the “Limit 1” order. Keeping the `qty` value of 3 would cause the second limit order’s size to decrease to 0 after the strategy fills the first limit order, resulting in no effect on the position.

### `strategy.oca.none` {#strategyocanone}

If an order placement command uses [strategy.oca.none](https://www.tradingview.com/pine-script-reference/v6/#const_strategy.oca.none) as its `oca_type` value, all orders from that command execute _independently_ of any OCA group. This value is the default `oca_type` argument for the [strategy.order()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.order) and [strategy.entry()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.entry) commands.

## Currency {#currency}

Pine Script strategies can simulate trades using different account currencies in their calculations. To set the default account currency for a strategy, include one of the `currency*` variables as the `currency` argument in the [strategy()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy) declaration statement. The default argument is [currency.NONE](https://www.tradingview.com/pine-script-reference/v6/#const_currency.NONE), which specifies that the strategy uses the quoted currency on the chart by default (the currency indicated by the [syminfo.currency](https://www.tradingview.com/pine-script-reference/v6/#var_syminfo.currency) variable). Users can change the strategy’s account currency via the “Initial capital” inputs in the “Settings/Properties” tab and at the top of the [strategy report](https://www.tradingview.com/pine-script-docs/concepts/strategies/#strategy-report).

If a strategy script uses an account currency that differs from the chart’s currency, it uses the _previous daily value_ of a corresponding currency pair from the most popular exchange to determine the conversion rate for all necessary calculations. If no available exchange provides the conversion rate directly, the strategy uses a [spread](https://www.tradingview.com/support/solutions/43000502298-spread-charts/) to calculate the rate. The strategy multiplies all monetary values, such as simulated profits and losses, by the retrieved rate to express them in the account currency. Likewise, it inverts the rate to convert values in the account currency to prices in the quoted currency. To retrieve the rate that the strategy uses for currency conversion, call the [request.currency\_rate()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.currency_rate) function with [syminfo.currency](https://www.tradingview.com/pine-script-reference/v6/#var_syminfo.currency) as the `from` argument and [strategy.account\_currency](https://www.tradingview.com/pine-script-reference/v6/#var_strategy.account_currency) as the `to` argument.

> [!TIP]
> Programmers can also directly convert values expressed in a strategy’s account currency to the chart’s currency, and vice versa, by using the [strategy.convert\_to\_symbol()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.convert_to_symbol) and [strategy.convert\_to\_account()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.convert_to_account) functions.

The following example demonstrates how currency conversion affects a strategy’s monetary values, and how conversion rate calculations in a strategy match the calculations used by the `request.*()` functions.

On each of the latest 500 bars, the strategy below places an entry market order with the [strategy.entry()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.entry) command, then places a [take-profit](https://www.tradingview.com/pine-script-docs/concepts/strategies/#take-profit-and-stop-loss) and [stop-loss](https://www.tradingview.com/pine-script-docs/concepts/strategies/#take-profit-and-stop-loss) order _one tick_ away from the entry price using the [strategy.exit()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.exit) command. Each entry order uses the size `1.0 / syminfo.mintick`. Therefore, the resulting profit or loss from each trade is [syminfo.pointvalue](https://www.tradingview.com/pine-script-reference/v6/#var_syminfo.pointvalue) units of chart currency per tick. We included [currency.EUR](https://www.tradingview.com/pine-script-reference/v6/#const_currency.EUR) as the `currency` argument in the [strategy()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy) declaration statement, causing the strategy to convert necessary values to _Euros_ by default.

The script calculates the absolute one-bar change in the ratio of the strategy’s net profit to the symbol’s point value to derive the value of _one unit_ of the chart’s currency in Euros. It plots the result alongside the value returned by a [request.currency\_rate()](https://www.tradingview.com/pine-script-reference/v6/#fun_request.currency_rate) call that uses [syminfo.currency](https://www.tradingview.com/pine-script-reference/v6/#var_syminfo.currency) and [strategy.account\_currency](https://www.tradingview.com/pine-script-reference/v6/#var_strategy.account_currency) as the `from` and `to` arguments. As shown on the chart below, both plots show the same values, confirming that the strategy and the data request use the same daily conversion rate in their calculations:

```pine
//@version=6
strategy("Currency Test", currency = currency.EUR)

if last_bar_index - bar_index < 500
    // Place an entry order with a size that results in a P/L of `syminfo.pointvalue` units of chart currency per tick. 
    strategy.entry("LE", strategy.long, math.round_to_mintick(1.0 / syminfo.mintick))
    // Place exit orders one tick above and below the "LE" entry price, 
    // meaning each trade closes with one point of profit or loss in the chart's currency.
    strategy.exit("LX", "LE", profit = 1, loss = 1)

// Plot the absolute change in `strategy.netprofit / syminfo.pointvalue`, which represents 1 chart unit of profit/loss. 
plot(
     math.abs(ta.change(strategy.netprofit / syminfo.pointvalue)), "1 chart unit of profit/loss in EUR", 
     color = color.fuchsia, linewidth = 4
 )
// Plot the requested currency rate.
plot(request.currency_rate(syminfo.currency, strategy.account_currency), "Requested conversion rate", color.lime)
```

Note that:

-   If a strategy executes on a chart with a timeframe higher than “1D”, it uses the data from _one day before_ each _historical_ bar’s closing time in currency conversions. For example, on a weekly chart, the strategy performs currency conversions using the confirmed rate from the previous Thursday’s close. However, it still uses the latest confirmed daily rate on _realtime_ bars.

## Altering calculation behavior {#altering-calculation-behavior}

Similar to an indicator, a strategy script executes across all historical chart bars in order, then continues to execute across realtime bars that become available after updates from the data feed. However, while indicators always executes _once per bar_ on historical bars and _once per tick_ on realtime bars, a strategy script can execute _differently_ on historical bars, realtime bars, or both, depending on the selected strategy properties.

> [!TIP]
> Understanding how these settings impact a strategy requires some knowledge of Pine’s [Execution model](https://www.tradingview.com/pine-script-docs/language/execution-model/). Therefore, we recommend reviewing [the basics](https://www.tradingview.com/pine-script-docs/language/execution-model/#the-basics) of the model before exploring the information below.

Unless otherwise specified, a strategy script executes strictly _once per closed bar_, regardless of whether a bar is part of the historical dataset or a new bar from the realtime data feed. If the current bar is open, the strategy _waits_ until the bar closes before updating calculations or placing new orders. Additionally, each new order in the strategy’s simulation has a _one-tick delay_ by default. Therefore, when using these default behaviors, a strategy places orders on a bar only when it reaches the bar’s close, and the earliest point at which the [broker emulator](https://www.tradingview.com/pine-script-docs/concepts/strategies/#broker-emulator) can fill those orders is at the _open_ of the following bar.

Users can configure a strategy to perform _more than one_ execution per historical or realtime bar, allowing for more granular calculations and order fills, by selecting the checkboxes in the strategy’s [Script execution](https://www.tradingview.com/support/solutions/43000786178/) settings. Users can also change the strategy’s order delay on closed bars to zero ticks by selecting the “None” option from the “Order execution delay” input in the “Settings/Properties” tab. Programmers can set the default script execution and order delay behaviors by including arguments for the `calc_on_every_tick`, `calc_on_order_fills`, `calc_on_every_history_tick`, and `process_orders_on_close` parameters in the [strategy()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy) declaration statement.

The sections below explain how each of these parameters affects a strategy’s default calculation behaviors.

> [!IMPORTANT]
> Modifying a strategy’s “Script execution” settings can cause [repainting](https://www.tradingview.com/pine-script-docs/concepts/repainting/) in the strategy’s calculations and results, because they define different behaviors for historical and realtime bars, and both types of bars often have different levels of intrabar detail. Such differences can affect the behavior of [order placement commands](https://www.tradingview.com/pine-script-docs/concepts/strategies/#order-placement-and-cancellation), [Pine Logs](https://www.tradingview.com/pine-script-docs/writing/debugging/#pine-logs), [alerts](https://www.tradingview.com/pine-script-docs/concepts/alerts/), and any variables or fields that use the [`varip` declaration mode](https://www.tradingview.com/pine-script-docs/language/variable-declarations/#varip). Therefore, when modifying these settings, we recommend inspecting a strategy’s logic carefully to ensure that it behaves as intended.

### `calc_on_every_tick` {#calc_on_every_tick}

The `calc_on_every_tick` parameter of the [strategy()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy) declaration statement specifies the default behavior of the strategy’s executions on _realtime bars_. If the value is `true`, the script executes to update its calculations on _every new tick_ in an open realtime bar, similar to an indicator. The default value is `false`. Script users can override the specified default by selecting the _“On realtime bar tick”_ checkbox from the [Script execution](https://www.tradingview.com/support/solutions/43000786178/) settings in the “Settings/Properties” tab and at the top of the [strategy report](https://www.tradingview.com/pine-script-docs/concepts/strategies/#strategy-report).

Configuring a strategy to execute on each realtime tick is sometimes useful when _forward testing_ or updating _visuals_ on live data, because it allows the strategy to perform more granular calculations using the latest price and volume updates for an open bar. However, depending on the strategy’s logic, using this setting can also cause the strategy to behave _differently_ across historical and realtime bars, leading to [repainting](https://www.tradingview.com/pine-script-docs/concepts/repainting/) in the strategy’s orders and performance results. After a script reloads, all _elapsed_ realtime bars from the previous run become _historical_ bars in the new run, and the new historical bars **do not** preserve intrabar data from previous realtime ticks. Consequently, if a strategy places orders on the ticks within an open bar, it might not be able to reproduce the same orders after reloading.

The following example demonstrates how executing a strategy’s logic on each new tick can significantly change the behavior of orders on realtime bars. If the current value of the [close](https://www.tradingview.com/pine-script-reference/v6/#var_close) variable is the highest value in the series over a specified length, the script below calls the [strategy.entry()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.entry) command to place a long entry order. Otherwise, if the [close](https://www.tradingview.com/pine-script-reference/v6/#var_close) value is the lowest over the same length, the script uses a separate [strategy.entry()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.entry) call to place a short entry order. We included `calc_on_every_tick = true` in the script’s [strategy()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy) declaration statement to allow a new execution on each realtime tick by default:

```pine
//@version=6
strategy(
    "Executions on realtime ticks demo", overlay = true, behind_chart = false, 
    pyramiding = 20, calc_on_every_tick = true
)

//@variable The number of bars in the highest and lowest calculations.
int lengthInput = input.int(15, "Length", minval = 1)

// Calculate the highest and lowest `close` values over the input length.
float highest = ta.highest(close, lengthInput)
float lowest  = ta.lowest(close, lengthInput)

// Place a long market order if the `close` value equals the `highest` value.
if close == highest
    strategy.entry("Buy", strategy.long)
// Otherwise, place a short market order if the `close` value equals the `lowest` value.
if close == lowest
    strategy.entry("Sell", strategy.short)

// Highlight the background of realtime bars.
bgcolor(barstate.isrealtime ? color.new(color.orange, 80) : na, title = "Realtime bar highlight")

// Plot the `highest` and `lowest` series.
plot(highest, "Highest close", color = color.lime)
plot(lowest,  "Lowest close",  color = color.red)
```

Note that:

-   The script uses a [pyramiding](https://www.tradingview.com/pine-script-docs/concepts/strategies/#pyramiding) value of 20, which allows to 20 entries per position using calls to the [strategy.entry()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.entry) command.
-   The script indicates realtime bars by highlighting the chart’s background in orange when the [barstate.isrealtime](https://www.tradingview.com/pine-script-reference/v6/#var_barstate.isrealtime) value is `true`.

Because this strategy allows executions on every realtime tick, it updates its calculations and can place new orders after _each new update_ from the data feed. Below, we applied the script to a chart and let it run on several realtime bars. On the chart’s historical bars, the script places up to _one_ market order per bar, and the broker emulator fills each order at the _open_ of the following bar. By contrast, on realtime bars (the bars with an orange background), the script places _multiple_ orders per bar — one for every _tick_ on which the latest available [close](https://www.tradingview.com/pine-script-reference/v6/#var_close) value equals the highest or lowest value over the specified length. Additionally, the broker emulator fills most of the orders on each highlighted bar _before_ the next bar opens, because every update to a realtime bar is a _valid tick_ for filling orders:

After we refresh our chart and run the script on the same bars again, the _elapsed_ realtime bars from the previous script run become _historical_ bars in the new run, and the script’s behavior _changes_ on those bars. Rather than placing multiple orders per bar on the former realtime bars, the script places only _one_ order on each closed bar whose _final_ price equals the highest or lowest value, and the broker emulator fills that order at the _open_ of the following bar:

Note that:

-   This strategy also behaves differently if we enable executions after _each order fill_ or on _each historical tick_, because both settings allow _additional_ executions and fill prices for orders on _historical bars_. See the [`calc_on_order_fills`](https://www.tradingview.com/pine-script-docs/concepts/strategies/#calc_on_order_fills) and [`calc_on_every_history_tick`](https://www.tradingview.com/pine-script-docs/concepts/strategies/#calc_on_every_history_tick) sections below to learn more.

> [!TIP]
> When using the “On realtime bar tick” setting, users can help align script behaviors on historical and realtime bars by enabling the “On history bar tick” setting. When active, the script executes on each available tick across _historical_ bars to _approximate_ how the strategy would behave when executing on those bars as they formed. Allowing executions on historical ticks does not _eliminate_ repainting. However, it can help reduce significant differences between the script’s behaviors on historical and realtime bars for more realistic results. See the [`calc_on_every_history_tick`](https://www.tradingview.com/pine-script-docs/concepts/strategies/#calc_on_every_history_tick) section for more information.

### `calc_on_order_fills` {#calc_on_order_fills}

The `calc_on_order_fills` parameter of the [strategy()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy) declaration statement affects the default behavior of the strategy’s executions on both historical bars _and_ realtime bars. If the value is `true`, the script performs an additional execution on _each tick_ where the [broker emulator](https://www.tradingview.com/pine-script-docs/concepts/strategies/#broker-emulator) _fills_ an order. The default value is `false`. Script users can override the specified default behavior by selecting the _“On order fill”_ checkbox from the [Script execution](https://www.tradingview.com/support/solutions/43000786178/) settings in the “Settings/Properties” tab and at the top of the [strategy report](https://www.tradingview.com/pine-script-docs/concepts/strategies/#strategy-report).

When using this setting, a strategy updates its calculations and can place new orders _immediately_ after an order fills, without waiting for a bar’s closing tick. Those extra calculations update the `strategy.*` built-ins with more granular information that is otherwise not available until the closing tick, such as the current size or the average price of a new or modified position. This behavior is sometimes useful when backtesting or forward testing strategies that enter and exit trades mid-bar.

The example script below uses the [strategy.entry()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.entry) command to place a “Buy” entry order on any bar where the value of the [strategy.position\_size](https://www.tradingview.com/pine-script-reference/v6/#var_strategy.position_size) variable is 0. It also calls the [strategy.exit()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.exit) command on each bar to place “Exit” bracket orders to close any active “Buy” trade. The script calculates the stop-loss and take-profit prices for the exit orders based on the value of the [strategy.position\_avg\_price](https://www.tradingview.com/pine-script-reference/v6/#var_strategy.position_avg_price) variable.

The [strategy()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy) declaration statement includes the argument `calc_on_order_fills = true`. Therefore, in addition to updating calculations on each bar’s close, the strategy performs new calculations on each tick where the broker emulator fills a “Buy” or “Exit” order by default. Each time an “Exit” order fills, the strategy’s position size reverts to 0, triggering a new “Buy” order. The emulator then fills the “Buy” order on the next available tick, and the [strategy.position\_avg\_price](https://www.tradingview.com/pine-script-reference/v6/#var_strategy.position_avg_price) value automatically updates on that tick to store the average price of the new position. The strategy then uses the updated price to set the prices of new “Exit” orders. This cycle of intrabar entries and exits repeats across the bars in the dataset:

```pine
//@version=6
strategy("Executions after order fills demo", overlay = true, calc_on_order_fills = true)

float stopSize   = input.float(5.0, "SL %", minval = 0.0) / 100.0
float profitSize = input.float(5.0, "TP %", minval = 0.0) / 100.0

// The condition that triggers the entry order is `true` after each tick where an "Exit"
// order fills.
if strategy.position_size == 0.0
    strategy.entry("Buy", strategy.long)

// After each order fill, the values of these variables update immediately rather 
// than only at the bar's closing tick.
float stopLoss   = strategy.position_avg_price * (1.0 - stopSize)
float takeProfit = strategy.position_avg_price * (1.0 + profitSize)

strategy.exit("Exit", stop = stopLoss, limit = takeProfit)
```

Note that:

-   This strategy can [repaint](https://www.tradingview.com/pine-script-docs/concepts/repainting/) after running on realtime bars. As the strategy runs on historical bars, the broker emulator uses the chart’s OHLC prices or the prices from a lower timeframe to determine the ticks available for filling orders, depending on the selected level of [historical bar detail](https://www.tradingview.com/pine-script-docs/concepts/strategies/#adjusting-historical-bar-detail). By contrast, _each new update_ to a realtime bar is a valid tick on which an order can fill. Consequently, when _elapsed_ realtime bars become _historical_ bars after the script reloads, the strategy may place or fill orders on those bars at _different_ prices or times.
-   If we deactivate executions after order fills, the strategy would _not_ place new orders _before_ a bar closes. Instead, it would wait for a bar’s closing tick before placing any “Buy” order. The broker emulator would then fill the order at the open of the following bar, and the [strategy.position\_avg\_price](https://www.tradingview.com/pine-script-reference/v6/#var_strategy.position_avg_price) variable would return a usable value for calculating the “Exit” order prices on that bar.

It’s crucial to note that enabling some strategies to execute after order fills can cause [lookahead bias](https://www.tradingview.com/pine-script-docs/concepts/strategies/#lookahead-bias) on historical bars. As a strategy executes across a dataset’s history while using this setting, [built-in variables](https://www.tradingview.com/pine-script-docs/language/built-ins/#built-in-variables) that store price and volume data for the current bar — including [high](https://www.tradingview.com/pine-script-reference/v6/#var_high), [low](https://www.tradingview.com/pine-script-reference/v6/#var_low), [close](https://www.tradingview.com/pine-script-reference/v6/#var_close), and [volume](https://www.tradingview.com/pine-script-reference/v6/#var_volume) — consistently hold the bar’s **final values**. Consequently, if the strategy uses these built-ins to control order logic on the ticks _within_ a historical bar, it may produce **misleading** backtest results, as the logic relies on data that would **not** be available in live trading until the market reaches the bar’s closing tick. Furthermore, the strategy’s apparent future awareness on historical bars is _impossible_ to reproduce on _realtime bars_. The bottom of the [strategy report](https://www.tradingview.com/pine-script-docs/concepts/strategies/#strategy-report) typically displays a _warning banner_ when the “On order fill” execution setting is active to inform users about this behavior.

The following example demonstrates a simple strategy that produces lookahead bias when using the “On order fill” execution setting. The script calls the [strategy.entry()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.entry) command to place a long market order, then calls the [strategy.exit()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.exit) command to place a take-profit order at the current bar’s [high](https://www.tradingview.com/pine-script-reference/v6/#var_high) value. By default, the script places the entry order at the current bar’s closing tick, and the broker emulator fills that order at the open of the following bar. The order fill on that bar triggers an additional execution, causing the [strategy.exit()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.exit) call to set the exit level to that bar’s high price. As shown below, on most historical bars where the script enters a new long trade, it then exits the trade at the bar’s _exact high_. This behavior is **misleading**, because knowing the exact high of a bar on the opening tick — let alone numerous consecutive times — is _impossible_ to achieve in real-world trading:

```pine
//@version=6
strategy(
    "Lookahead bias in orders demo", calc_on_order_fills = true, 
    default_qty_type = strategy.percent_of_equity, default_qty_value = 2
)

//@variable A persistent variable that updates to hold the current bar's opening time.
//          The script uses this variable to limit the executions of the order placement 
/           commands.
varip int openTime = 0

//@variable Holds a string to append to each order's ID to indicate the trade number.
string num = str.tostring(strategy.closedtrades)

if openTime != time
    strategy.entry("Buy" + num, strategy.long)

    // Create a take-profit order to exit the open trade at the current `high` value.
    // When an entry order fills on a historical bar's opening tick, the call below
    // typically places the take-profit order at that bar's **final** high price.
    // This behavior is extremely **misleading**, and not reproducible on realtime bars, 
    // because it is impossible to know the final high of a bar before the bar closes. 
    strategy.exit("Exit" + num, limit = high)

openTime := time
```

Note that:

-   This script declares the `openTime` variable using the [varip](https://www.tradingview.com/pine-script-reference/v6/#kw_varip) keyword. If a variable declaration uses this keyword, the variable persists across all executions without resetting to a previous state. The script uses this variable in the [if](https://www.tradingview.com/pine-script-reference/v6/#kw_if) structure to limit the placement of new orders to the current bar’s first tick — where the variable’s value does not yet match the current bar’s opening time. To learn more about the behavior of this keyword, refer to the [`varip`](https://www.tradingview.com/pine-script-docs/language/variable-declarations/#varip) section of the [Variable declarations](https://www.tradingview.com/pine-script-docs/language/variable/declarations/) page.
-   This script behaves very differently on _realtime bars_. Rather than exiting trades at a bar’s exact high, the strategy exits each trade at the bar’s _developing_ high as of the next tick or the current tick, because the final high price on a realtime bar is _unknown_ until after the bar closes.

> [!TIP]
> To enable script executions on historical intrabar ticks _without_ causing this form of lookahead bias, use the _“On history bar tick”_ execution setting rather than “On order fill”. See the [`calc_on_every_history_tick`](https://www.tradingview.com/pine-script-docs/concepts/strategies/#calc_on_every_history_tick) section below to learn more.

### `calc_on_every_history_tick` {#calc_on_every_history_tick}

The `calc_on_every_history_tick` parameter of the [strategy()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy) declaration statement specifies the default behavior of the strategy’s executions on _historical bars_. This parameter requires a _named argument_, which includes the parameter’s name and the assigned value (e.g., `calc_on_every_history_tick = true`). If the value is `true`, the script executes to update its calculations on _every available tick_ in each historical bar. The default value is `false`. Script users can override the specified default by selecting the _“On history bar tick”_ checkbox from the [Script execution](https://www.tradingview.com/support/solutions/43000786178/) settings in the “Settings/Properties” tab and at the top of the [strategy report](https://www.tradingview.com/pine-script-docs/concepts/strategies/#strategy-report).

> [!NOTE]
> The “On history bar tick” feature is available only to users who have a Premium or Ultimate [plan](https://www.tradingview.com/pricing/). Additionally, the feature requires a _standard_ chart type. If the setting is enabled by default, the strategy raises a runtime error if it is not compatible with the user’s plan or the current chart.

A strategy that uses the “On history bar tick” setting treats historical bars similarly to _realtime bars_. As the script executes on the available ticks of each historical bar, multiple built-in variables that hold price and volume data for the bar, including [high](https://www.tradingview.com/pine-script-reference/v6/#var_high), [low](https://www.tradingview.com/pine-script-reference/v6/#var_low), [close](https://www.tradingview.com/pine-script-reference/v6/#var_close), and [volume](https://www.tradingview.com/pine-script-reference/v6/#var_volume), _update on every tick_ to approximate the progression of values that would have been visible to the strategy when the bar was still open. This behavior helps reduce the risk of [lookahead bias](https://www.tradingview.com/pine-script-docs/concepts/strategies/#lookahead-bias) while executing on historical ticks, as the strategy cannot access the _final_ price or volume data for a historical bar before reaching the bar’s closing tick.

> [!NOTE]
> This setting does not affect the behavior of variables that track [bar states](https://www.tradingview.com/pine-script-docs/concepts/bar-states/). For example, the value of the [barstate.isconfirmed](https://www.tradingview.com/pine-script-reference/v6/#var_barstate.isconfirmed) variable is always `true` on historical bars. It never changes to `false` during the executions across a historical bar’s intrabar ticks while the “On history bar tick” setting is active.

The visibility of values within each historical bar depends on the selected level of [historical bar detail](https://www.tradingview.com/pine-script-docs/concepts/strategies/#adjusting-historical-bar-detail). If the strategy uses high historical detail, it determines the high, low, close, and volume values available on each tick by requesting data from a _lower timeframe_. Otherwise, if the strategy uses the default level of detail, the values available on each tick depend on the [broker emulator’s assumptions](https://www.tradingview.com/pine-script-docs/concepts/strategies/#broker-emulator) about the price action within the historical bar. The following tables describe the values stored by the [high](https://www.tradingview.com/pine-script-reference/v6/#var_high), [low](https://www.tradingview.com/pine-script-reference/v6/#var_low), [close](https://www.tradingview.com/pine-script-reference/v6/#var_close), and [volume](https://www.tradingview.com/pine-script-reference/v6/#var_volume) variables on each tick in a historical bar when using the default level of detail, based on the assumed order of price action within the bar:

**Open → High → Low → Close**

| Tick | `high` | `low` | `close` | `volume` |
| --- | --- | --- | --- | --- |
| 1 | Open | Open | Open | Total volume \* 0.25 |
| 2 | High | Open | High | Total volume \* 0.5 |
| 3 | High | Low | Low | Total volume \* 0.75 |
| 4 | High | Low | Close | Total volume |

**Open → Low → High → Close**

| Tick | `high` | `low` | `close` | `volume` |
| --- | --- | --- | --- | --- |
| 1 | Open | Open | Open | Total volume \* 0.25 |
| 2 | Open | Low | Low | Total volume \* 0.5 |
| 3 | High | Low | High | Total volume \* 0.75 |
| 4 | High | Low | Close | Total volume |

Note that:

-   Variables that store values calculated from the chart’s OHLCV data also update on each tick based on these assumptions. For example, if the _second_ tick of the current historical bar is at the _high_, the value of the [ohlc4](https://www.tradingview.com/pine-script-reference/v6/#var_ohlc4) variable equals `(open + open + high + high) / 4`. However, the value still equals `(open + high + low + close) / 4` on the bar’s final tick.

The following example demonstrates a strategy that enters trades on the ticks within historical bars when using the “On history bar tick” execution setting. The script declares two persistent variables named `openTime` and `firstPrice` to track time and price information from the first tick on which it executes in each historical bar. On each tick where the `openTime` value does not match the bar’s opening time, the script reassigns that variable to store the current timestamp, then reassigns the `firstPrice` variable to hold the [close](https://www.tradingview.com/pine-script-reference/v6/#var_close) value (i.e., the current price) for that tick.

The script uses the `firstPrice` value and the 10-bar highest and lowest prices to control order placement commands. On each tick where the current price is less than the `firstPrice` value and not equal to the current highest or lowest price, the script calls the [strategy.entry](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.entry) command to place a long market order, then calls the [strategy.exit()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.exit) command to place take-profit and stop-loss orders at the highest and lowest values:

```pine
//@version=6
strategy(
    "Executions on all historical ticks demo", overlay = true, behind_chart = false,
    default_qty_type = strategy.percent_of_equity, default_qty_value = 2,
    calc_on_every_history_tick = true
)

//@variable A persistent variable that tracks the current bar's opening time.
varip int openTime = 0
//@variable A persistent variable that tracks the `close` value on each bar's *first* tick (i.e., the opening price).
varip float firstPrice = na

// The local scope of this structure executes only on the first tick of each bar, because the `time` variable's 
// value changes only once per bar.
if time != openTime
    // Update the `openTime` variable, preventing subsequent executions of the scope on the same bar.
    openTime   := time
    // Update the `firstPrice` variable to store the current `close` value.
    // When using the "On history bar tick" setting, the value is the price as of the historical bar's *first tick*.
    // When not using the setting, the value consistently equals the historical bar's *final* closing price. 
    firstPrice := close

// Calculate the highest high and lowest low over the latest 10 bars.
float highest = ta.highest(high, 10)
float lowest  = ta.lowest(low, 10)

// When using the "On history bar tick" setting, this variable's value is `true` for each tick historical tick 
// where the *current* price is less than the price on the bar's first tick and not equal to the current highest high 
// or lowest low. 
// Without this setting enabled, the condition is *never* `true` on historical bars, because `close` and `firstPrice` 
// both refer to the *same value*.
bool entryCondition = close < firstPrice and close != highest and close != lowest
 
if entryCondition
    // Place a long entry order on the current tick.
    strategy.entry("Entry", strategy.long)
    // Place take-profit and stop-loss orders at the current tick's `highest` and `lowest` values, respectively.
    strategy.exit("Exit", "Entry", limit = highest, stop = lowest)

// Plot the `highest` and `lowest` series for visual reference.
plot(highest, "10-bar high", color.green)
plot(lowest, "10-bar low", color.red)
```

Note that:

-   This script simulates historical trades only while using the “On history bar tick” setting. Without the setting enabled, the value of the [close](https://www.tradingview.com/pine-script-reference/v6/#var_close) variable on historical bars always represents the _final_ closing price rather than the intrabar price on each tick. Therefore, when the script reassigns the `firstPrice` variable on any historical bar, that variable also stores the bar’s final price, and the `close < firstPrice` condition consistently evaluates to `false`, resulting in _no trades_.
-   By default, this script does not simulate trades on _realtime bars_, because it executes only _once_ at each bar’s close on that part of the dataset. To enable trades on realtime bars, select the “On realtime bar tick” checkbox in the “Script execution” settings or add `calc_on_every_tick = true` to the [strategy()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy) declaration statement.
-   By default, the broker emulator fills the strategy’s orders based on the _chart prices_ of each historical bar, because the script uses the default level of [historical bar detail](https://www.tradingview.com/pine-script-docs/concepts/strategies/#adjusting-historical-bar-detail). Users can simulate the trades using more granular prices by selecting the “High” option in the script’s [Bar detalization](https://www.tradingview.com/support/solutions/43000786180/) settings or including `use_bar_magnifier = true` in the declaration statement.

### `process_orders_on_close` {#process_orders_on_close}

The `process_orders_on_close` parameter of the [strategy()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy) declaration statement specifies the default behavior of orders created on a bar’s _closing tick_. If the value is `true`, the [broker emulator](https://www.tradingview.com/pine-script-docs/concepts/strategies/#broker-emulator) can fill orders created on a bar’s close immediately, on the _same tick_, rather than waiting for the next available tick. If the value is `false` (the default), the earliest point at which the emulator can fill orders created on a bar’s closing tick is on the _next_ tick, at the _open_ of the following bar. Script users can override the specified default behavior via the “Order execution delay” dropdown menu in the “Settings/Properties” tab.

Strategies apply a one-tick delay to order fills by default because creating and filling an order on the same tick is typically _unrealistic_ in real-world trading. However, simulating order fills at a bar’s close can be useful in some scenarios, such as when backtesting manual strategies in which traders exit a position immediately before the market closes, or when using [order fill alerts](concepts/alerts/#order-fill-events) to potentially trigger real-world orders before the start of the next trading session.

> [!NOTE]
> Sending alerts to a third-party service for filling orders when using this setting might not work as a trader intends, especially in non-continuous markets, because the alerts still occur _after_ the session closes. Depending on the external service and the type of market, real-world orders based on such alerts might not fill until after the market opens again.
>
> This setting _does not_ affect the execution of orders that a strategy places _before_ a bar’s closing tick. If a strategy places orders on the ticks within a bar, the broker emulator fills those orders on the next available tick.

The following example demonstrates how removing the execution delay in orders at a bar’s close changes the timing of trades. The initial script below uses the [strategy.entry()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.entry) command to place alternating long and short [market orders](https://www.tradingview.com/pine-script-docs/concepts/strategies/#market-orders) across the dataset. The script places an order only at the close of each bar, then highlights the chart’s background in blue or red to indicate the order direction. The [strategy()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy) declaration statement does not include a `process_orders_on_close` argument. Therefore, the broker emulator applies a one-tick delay to all orders placed on a bar’s close by default. As shown by the trade markers on the chart’s bars, after the strategy creates an order, the broker emulator fills the order at the open of the following bar:

```pine
//@version=6
strategy("Default delay demo", overlay = true)

//@variable A translucent blue or red on each bar where a `strategy.entry()` call executes.
color entryColor = na

if bar_index % 10 == 0
    // Place a long entry order and assign a blue color to `entryColor` if a short position or no position is open.
    if strategy.position_size <= 0
        strategy.entry("Buy", strategy.long)
        entryColor := color.new(color.blue, 80)
    // Otherwise, place a short entry order and assign a red color to `entryColor`.
    else
        strategy.entry("Sell", strategy.short)
        entryColor := color.new(color.red, 80)

// Highlight the chart's background to indicate when the entry orders occur.
bgcolor(entryColor, title = "Entry order highlight")
```

If we add `process_orders_on_close = true` to the [strategy()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy) declaration statement, broker emulator fills each market order on the same closing tick where the script creates it by default. After applying this change to the script above, the trade markers on the chart now align with the displayed background colors, indicating that each order fills immediately rather than at the next bar’s opening tick:

```pine
//@version=6
strategy("No delay for orders on close demo", overlay = true, process_orders_on_close = true)

//@variable A translucent blue or red on each bar where a `strategy.entry()` call executes.
color entryColor = na

if bar_index % 10 == 0
    // Place a long entry order and assign a blue color to `entryColor` if a short position or no position is open.
    if strategy.position_size <= 0
        strategy.entry("Buy", strategy.long)
        entryColor := color.new(color.blue, 80)
    // Otherwise, place a short entry order and assign a red color to `entryColor`.
    else
        strategy.entry("Sell", strategy.short)
        entryColor := color.new(color.red, 80)

// Highlight the chart's background to indicate when the entry orders occur.
bgcolor(entryColor, title = "Entry order highlight")
```

> [!TIP]
> The [strategy.close()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.close) and [strategy.close\_all()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.close_all) commands feature an `immediately` parameter, which enables programmers to remove the delay for closing market orders _without_ affecting the behavior of other order placement commands. If the value is `true`, the broker emulator fills the resulting market order on the same tick where the strategy calls the command. If `false` (the default), the behavior of the order depends on the strategy’s “Order execution delay” setting.

## Simulating trading costs {#simulating-trading-costs}

Strategy performance reports are more relevant and meaningful when they include potential real-world trading costs. Without modeling the potential costs associated with their trades, traders may overestimate a strategy’s historical profitability, potentially leading to suboptimal decisions in live trading. Pine Script strategies include inputs and parameters for simulating trading costs in performance results.

### Commission {#commission}

Commission is the fee that a broker/exchange charges when executing orders. The commission can be a flat fee per order, a fee per contract/share/lot/unit, or a percentage of the total transaction value. Programmers can specify the default commission settings for their strategies by including [`commission_type` and `commission_value`](https://www.tradingview.com/pine-script-docs/declaration-statements/#commission_type-and-commission_value) arguments in the [strategy()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy) declaration statement. If not specified, the strategy applies no commission to filled orders. Users can override the specified defaults by adjusting the “Commision” inputs in the strategy’s “Settings/Properties” tab.

The following script is a simple strategy that enters a long position using 2% of its available equity when the current [close](https://www.tradingview.com/pine-script-reference/v6/#var_close) value is the highest value over a user-specified length, and closes the trade when the [close](https://www.tradingview.com/pine-script-reference/v6/#var_close) value is the lowest across that same number of bars. Because the declaration statement does not specify any `commission_*` arguments, the strategy simulates transactions without any commission by default:

```pine
//@version=6
// This declaration statement's `default_qty_*` arguments specify that `strategy.entry()` calls create orders
// with 2% of the available equity by default.
// It does not specify `commission_*` arguments, so this strategy applies no commission by default. 
strategy(
    "Commission demo", overlay = true, default_qty_value = 2, default_qty_type = strategy.percent_of_equity
)

//@variable The number of bars for the `ta.highest()` and `ta.lowest()` calculations.
int lengthInput = input.int(10, "Length", minval = 1)

// Determine the highest and lowest `close` values over `lengthInput` bars.
float highest = ta.highest(close, lengthInput)
float lowest  = ta.lowest(close, lengthInput)

switch close
	// Place a long market order if the current `close` value is equal to the `highest` value.
    highest => strategy.entry("Long", strategy.long)
	// Place an order to close the position if the `close` value is equal to the `lowest` value.
    lowest  => strategy.close("Long")

// Plot the calculated `highest` and `lowest` series across the bars for reference.
plot(highest, "Highest close", color.new(color.lime, 50), 2)
plot(lowest,  "Lowest close",  color.new(color.red, 50),  2)
```

Note that:

-   The [`default_qty_type` and `default_qty_value`](https://www.tradingview.com/pine-script-docs/language/declaration-statements/#default_qty_type-and-default_qty_value) arguments in the [strategy()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy) statement set the strategy’s default order type and size. This default order size applies only to orders from [strategy.entry()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.entry) and [strategy.order()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.order) calls that do _not_ include a `qty` argument. See the [Position sizing](https://www.tradingview.com/pine-script-docs/concepts/strategies/#position-sizing) section above for more information.

For our daily “NASDAQ:AAPL” chart above, the results in the [strategy report](https://www.tradingview.com/pine-script-docs/concepts/strategies/#strategy-report) show that the strategy had a positive equity growth of 18.67% over the testing range. However, these backtesting results do not account for any fees that the broker/exchange may charge.

Let’s see what happens to these results when we add commission to every trade in the strategy simulation. In the example below, we modified the previous script to include the arguments `commission_type = strategy.commission.percent` and `commission_value = 1` in the [strategy()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy) declaration statement. With this change, the broker emulator now applies a commission of 1% of the transaction size to each filled order by default.

As shown below, after applying 1% commission to the strategy’s orders on the same dataset, the strategy report shows a significantly reduced net profit, as well as increased volatility in the strategy’s cumulative returns and an elevated maximum drawdown. These results highlight the impact that commission can have on a strategy’s simulated performance:

```pine
//@version=6
// This declaration statement's `default_qty_*` arguments specify that `strategy.entry()` calls create orders
// with 2% of the available equity by default.
// The `commission_*` arguments specify that the strategy simulates a commission of 1% of each transaction's value.
strategy(
    "Commission demo", overlay = true, default_qty_value = 2, default_qty_type = strategy.percent_of_equity,
    commission_type = strategy.commission.percent, commission_value = 1
)

//@variable The number of bars for the `ta.highest()` and `ta.lowest()` calculations.
lengthInput = input.int(10, "Length", minval = 1)

// Determine the highest and lowest `close` values over `lengthInput` bars.
float highest = ta.highest(close, lengthInput)
float lowest  = ta.lowest(close, lengthInput)

switch close
    // Place a long market order if the current `close` value is equal to the `highest` value.
    highest => strategy.entry("Long", strategy.long)
    // Place an order to close the position if the `close` value is equal to the `lowest` value.
    lowest  => strategy.close("Long")

// Plot the calculated `highest` and `lowest` series across the bars for reference.
plot(highest, "Highest close", color.new(color.lime, 50), 2)
plot(lowest,  "Lowest close",  color.new(color.red, 50),  2)
```

### Slippage and unfilled limits {#slippage-and-unfilled-limits}

In real-world trading, orders may execute at prices that differ from what the trader intended, due to volatility, liquidity, order size, and other market factors. Such differences can profoundly impact a strategy’s performance. The disparity between the expected fill price of an order and the actual fill price is known as _slippage_. Slippage is dynamic and unpredictable, making it impossible to simulate precisely. However, applying a small, fixed amount of slippage to every order in a backtest or forward test can help overall performance results align more closely with reality. Programmers can specify a strategy’s default slippage amount, as a fixed number of _ticks_, by including a `slippage` argument in the [strategy()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy) declaration statement. The default argument is 0. Script users can override the specified default by adjusting the “Slippage” input in the strategy’s “Settings/Properties” tab.

The following example demonstrates how simulating slippage impacts the fill prices of [market orders](https://www.tradingview.com/pine-script-docs/concepts/strategies/#market-orders) in a strategy test. The script below places a “Buy” market order of 2% equity when the market price is above a rising EMA, then closes the position with another market order when the price dips below the EMA while it’s falling. The [strategy()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy) declaration statement includes the argument `slippage = 20`. Therefore, by default, each order fills 20 ticks beyond the intended price in the _unfavorable_ direction (higher for long orders, and lower for short orders). The script plots the each order’s expected fill price along with the simulated fill price after slippage to visually compare the difference:

```pine
//@version=6
strategy(
     "Slippage Demo", overlay = true, slippage = 20,
     default_qty_value = 2, default_qty_type = strategy.percent_of_equity
 )

int length = input.int(5, "Length")

//@variable Exponential Moving Average with an input `length`.
float ma = ta.ema(close, length)

//@variable Is `true` when `ma` has increased and `close` is above it, `false` otherwise.
bool longCondition = close > ma and ma > ma[1]
//@variable Is `true` when `ma` has decreased and `close` is below it, `false` otherwise.
bool shortCondition = close < ma and ma < ma[1]

// Enter a long market position on `longCondition` and close the position on `shortCondition`. 
if longCondition    
    strategy.entry("Buy", strategy.long)
if shortCondition
    strategy.close("Buy")

//@variable The `bar_index` of the position's entry order fill.
int entryIndex = strategy.opentrades.entry_bar_index(0)
//@variable The `bar_index` of the position's close order fill.
int exitIndex  = strategy.closedtrades.exit_bar_index(strategy.closedtrades - 1)

//@variable The fill price simulated by the strategy.
float fillPrice = switch bar_index
    entryIndex => strategy.opentrades.entry_price(0)
    exitIndex  => strategy.closedtrades.exit_price(strategy.closedtrades - 1)

//@variable The expected fill price of the open market position.
float expectedPrice = not na(fillPrice) ? open : na

color expectedColor = na
color filledColor   = na

if bar_index == entryIndex
    expectedColor := color.green
    filledColor   := color.blue
else if bar_index == exitIndex
    expectedColor := color.red
    filledColor   := color.fuchsia

plot(ma, color = color.new(color.orange, 50))

plotchar(not na(fillPrice) ? open : na, "Expected fill price", "—", location.absolute, expectedColor)
plotchar(fillPrice, "Fill price after slippage", "—", location.absolute, filledColor)
```

Note that:

-   Because the strategy applies constant slippage to _all_ order fills, some orders can fill _outside_ the candle range in the simulation. Therefore, we recommend exercising caution with this setting, as adding excessive simulated slippage can produce unrealistically _worse_ testing results.

Some traders might assume that they can avoid the adverse effects of slippage by using [limit orders](https://www.tradingview.com/pine-script-docs/concepts/strategies/#limit-orders). While market orders execute as soon as possible, irrespective of the price, limit orders execute at the specified price or a better value. They cannot execute at a _worse_ price. However, in real-world trading, some limit orders _might not fill_ when the market price reaches the specified value, due to insufficient liquidity or price action. Programmers can simulate the possibility of _unfilled_ limit orders in their scripts by using the `backtest_fill_limits_assumption` parameter in the [strategy()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy) declaration statement. The parameter specifies the number of ticks that the price must move _beyond_ each limit order’s level to trigger an order fill at that level. The default argument is 0, meaning that the broker emulator fills any limit order immediately if the price reaches the order’s level.

> [!NOTE]
> The “Limit order execution” input in a strategy’s “Settings/Properties” tab overrides the default assumption for filling limit orders. This input includes the options to fill each limit order when either the market price reaches the specified level or it moves _one tick_ beyond that level. If the `backtest_fill_limits_assumption` argument is greater than 1, users can restore the default assumption specified in the declaration statement after using this input by selecting “Reset settings” from the “Defaults” dropdown menu at the bottom of the “Properties” tab.

The following example script places a limit order for 2% of the strategy’s equity at a bar’s [hlcc4](https://www.tradingview.com/pine-script-reference/v6/#var_hlcc4) price if the current [high](https://www.tradingview.com/pine-script-reference/v6/#var_high) value is the highest price over a specified number of bars and no pending entry orders are active. The strategy closes the market position and cancels all pending orders if the current [low](https://www.tradingview.com/pine-script-reference/v6/#var_low) value is the lowest price over the same length. Each time that the strategy triggers an order, it draws a horizontal line at the `limitPrice` value. It then updates the line on each bar until it closes the position or cancels the order:

```pine
//@version=6
strategy(
     "Verify price for limits example", overlay = true,
     default_qty_type = strategy.percent_of_equity, default_qty_value = 2
 )

int length = input.int(25, title = "Length")

//@variable Draws a line at the limit price of the most recent entry order.
var line limitLine = na

// Highest high and lowest low
highest = ta.highest(length)
lowest  = ta.lowest(length)

// Place an entry order and draw a new line when the the `high` equals the `highest` value and `limitLine` is `na`.
if high == highest and na(limitLine)
    float limitPrice = hlcc4
    strategy.entry("Long", strategy.long, limit = limitPrice)
    limitLine := line.new(bar_index, limitPrice, bar_index + 1, limitPrice)

// Close the open market position, cancel orders, and set `limitLine` to `na` when the `low` equals the `lowest` value.
if low == lowest
    strategy.cancel_all()
    limitLine := na
    strategy.close_all()

// Update the `x2` value of `limitLine` if it isn't `na`.
if not na(limitLine)
    limitLine.set_x2(bar_index + 1) 

plot(highest, "Highest High", color = color.new(color.green, 50))
plot(lowest, "Lowest Low", color = color.new(color.red, 50))
```

By default, the broker emulator assumes that all limit orders are guaranteed to fill when the market price reaches their values. However, that may not be the case in a real-world market. Let’s add price verification to our script’s limit orders to account for potentially unfilled ones. In the example below, we added `backtest_fill_limits_assumption = 3` to the [strategy()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy) statement. With this change, the broker emulator assumes that there is sufficient liquidity to fill a limit order only if the price moves _three ticks_ beyond the order’s level. As shown below, using this fill assumption prevents the execution of some orders and changes the _times_ of others:

> [!IMPORTANT]
> Using a nonzero `backtest_fill_limits_assumption` argument can affect the _times_ at which limit orders execute, as shown above. However, regardless of the fill time, the broker emulator still executes verified limit orders at their specified _prices_. This behavior is a necessary compromise to preserve the intended fill prices of limit orders without causing _lookahead bias_, but it can also cause the orders to execute at times that may _not_ be possible in real-word trading, especially if the argument is a large value. Therefore, we recommend that users exercise caution and understand this price-time limitation when applying price verification to limit orders.

## Risk management {#risk-management}

Designing a strategy that performs well, especially across a broad range of markets, is a challenging task. Most strategies are designed for specific market patterns or conditions and can produce uncontrolled losses when applied to other datasets. Therefore, a strategy’s risk management behavior can be critical to its performance. Programmers can specify risk management criteria in their strategy scripts by using the `strategy.risk.*()` commands.

Strategies can incorporate any number of risk management criteria in any combination. All risk management commands execute _on every tick and order execution event_, regardless of any changes to the strategy’s [calculation behavior](https://www.tradingview.com/pine-script-docs/concepts/strategies/#altering-calculation-behavior). There is no way to deactivate any of these commands on specific script executions. Irrespective of a risk management command’s location, the command _always_ applies to the strategy unless the programmer removes the call from the code. Below, we list the available risk management commands and the behaviors they define:

[strategy.risk.allow\_entry\_in()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.risk.allow_entry_in)

This command overrides the market direction allowed for all calls to the [strategy.entry()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.entry) command. If a user specifies the trade direction with the [strategy.risk.allow\_entry\_in()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.risk.allow_entry_in) function (e.g., [strategy.direction.long](https://www.tradingview.com/pine-script-reference/v6/#const_strategy.direction.long)), the strategy enters trades only in that direction. If the script calls an entry command in the opposite direction of an open market position, the strategy generates a [market order](https://www.tradingview.com/pine-script-docs/concepts/strategies/#market-orders) to _close_ the position without entering a new trade in that direction.

[strategy.risk.max\_cons\_loss\_days()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.risk.max_cons_loss_days)

This command cancels all pending orders, closes any open market position, and stops all additional trade actions after the strategy simulates a defined number of trading days with consecutive losses.

[strategy.risk.max\_drawdown()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.risk.max_drawdown)

This command cancels all pending orders, closes any open market position, and stops all additional trade actions after the strategy’s drawdown reaches the amount specified in the function call.

[strategy.risk.max\_intraday\_filled\_orders()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.risk.max_intraday_filled_orders)

This command specifies the maximum number of filled orders per trading day (or per chart bar if the chart’s timeframe is higher than “1D”). If the strategy fills more orders than the specified limit, the command cancels all pending orders, closes any open market position, and halts trading activity until the end of the current session.

[strategy.risk.max\_intraday\_loss()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.risk.max_intraday_loss)

This command controls the maximum allowed loss per trading day (or per chart bar if the chart’s timeframe is higher than “1D”). If the strategy’s losses reach the specified threshold, the command cancels all pending orders, closes the open market position, and stops all trading activity until the end of the current session.

[strategy.risk.max\_position\_size()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.risk.max_position_size)

This command specifies the maximum possible position size when calling the [strategy.entry()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.entry) command. If the size of an entry order results in a market position that exceeds the specified threshold, the strategy reduces the order quantity so that the resulting position does not exceed the limit.

## Margin and leverage {#margin-and-leverage}

_Margin_ is the minimum percentage of a market position that a trader must hold in their account as collateral to open and maintain that position. With a margin of 100%, the trader must cover the entire position using their account’s available funds. With a margin of 25%, the trader must cover only _one-fourth_ of each position using their account’s funds to maintain a _loan_ for the other three-fourths from the broker. Most brokers define a trader’s margin requirements based on a specified _leverage_ amount, where leverage is the _inverse_ of margin. For example, a leverage ratio of 4:1 is equivalent to 25% margin. A trader can open a position for up to _four times_ their available funds when using this amount of leverage, because they must maintain only a fourth of each position’s size as collateral. In other words, the trader has four times the purchasing power that they would otherwise have when trading using only their account’s funds.

The `margin_long` and `margin_short` parameters of the [strategy()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy) declaration statement define the default required _margin percentages_ for long and short trades, respectively. The strategy _converts_ the specified percentages to leverage ratios and uses those ratios as the default values for the “Long leverage” and “Short leverage” inputs in the “Settings/Properties” tab. The default argument for both parameters is 100, which is equivalent to a leverage ratio of 1:1 .

> [!IMPORTANT]
> A margin requirement of less than 0.2% (i.e., leverage greater than 500
>
> ) is typically _unrealistic_ in a real-world market. Using unrealistic levels of margin in a strategy can cause very _misleading_ backtest results. Furthermore, using a margin of 0% is _extremely_ misleading because it is equivalent to _infinite_ leverage, which is impossible to achieve in any live trading environment. Therefore, when setting a strategy’s margin via the `margin_*` parameters, or adjusting leverage using the “Leverage \*” inputs, we recommend specifying realistic values that align with current market conditions.

Trading with less than 100% margin can significantly increase a trader’s potential profits and their potential **losses**. In real-world trading, if the loss from a position causes the trader’s available margin to fall below the required margin, the broker issues a _margin call_, which is a demand for the trader to immediately deposit additional funds to cover the loss. If the trader fails to meet the demand, or if the losses reach beyond the broker’s limits, the broker forcibly _liquidates_ all or part of the position to prevent further losses that the trader cannot cover.

To simulate this process in strategies, the broker emulator generates _margin call events_ if a strategy’s available funds fall below the required margin percentage. Each time that a margin call event occurs, the emulator immediately liquidates _four times_ the number of contracts/shares/lots/units required to cover the loss to help prevent continuous margin calls across subsequent bars. The emulator uses the following algorithm to determine the liquidated quantity for each event:

1.  Calculate the amount of capital spent on the position: `Money Spent = Quantity * Entry Price`
2.  Calculate the Market Value of Security (MVS): `MVS = Position Size * Current Price`
3.  Calculate the Open Profit as the difference between `MVS` and `Money Spent`. If the position is short, multiply this value by -1.
4.  Calculate the strategy’s equity value: `Equity = Initial Capital + Net Profit + Open Profit`
5.  Calculate the margin ratio: `Margin Ratio = Margin Percent / 100`
6.  Calculate the margin value, which is the cash required to cover the hypothetical account’s portion of the position: `Margin = MVS * Margin Ratio`
7.  Calculate the strategy’s available funds: `Available Funds = Equity - Margin`
8.  Calculate the total amount of money lost: `Loss = Available Funds / Margin Ratio`
9.  Calculate the number of contracts/shares/lots/units the account must liquidate to cover the loss, truncated to the same decimal precision as the minimum position size for the current instrument: `Cover Amount = TRUNCATE(Loss / Current Price).`
10.  Multiply the quantity required to cover the loss by four to determine the margin call size: `Margin Call Size = Cover Amount * 4`

> [!NOTE]
> A short trade typically entails _borrowing_ shares from a broker to sell them at the current market price, then buying the shares back at a different price and returning them to the lender. The trade thus produces a profit if the trader purchases the shares at a _lower_ price, or a loss if they purchase the shares at a _higher_ price. Consequently, unlike long trades, short trades carry the risk of _uncapped losses_, because there is no definite limit on how far an instrument’s price can rise. Therefore, short trades in a strategy that uses _any_ nonzero margin are subject to forced liquidation from margin call events, even if the `margin_short` argument is 100 or the “Short leverage” input value is 1.

To examine the above calculations in detail, the following example applies the [Supertrend Strategy](https://www.tradingview.com/support/solutions/43000645068/) built-in script to a daily “NASDAQ:TSLA” chart. In the strategy’s properties, we set the order size to 300% of equity, and we set the strategy’s long leverage to 4. On the chart below, the strategy opens a long trade, then the broker emulator triggers margin call event a few bars later:

In the above image, the script enters the long position at the bar’s opening price on 16 Sep 2010. The strategy purchases 682,438 shares (Position Size) at 4.43 USD (Entry Price) using 25% margin. Then, on 23 Sep 2010, when the price drops to 3.9 USD (Current Price), the emulator triggers the margin call event and forcibly liquidates _111,052 shares_. The calculations below explain how the broker emulator determines the liquidation quantity for this event:

```
Money spent: 682438 * 4.43 = 3023200.34
MVS: 682438 * 3.9 = 2661508.2
Open Profit: −361692.14
Equity: 1000000 + 0 − 361692.14 = 638307.86
Margin Ratio: 25 / 100 = 0.25
Margin: 2661508.2 * 0.25 = 665377.05
Available Funds: 638307.86 - 665377.05 = -27069.19
Money Lost: -27069.19 / 0.25 = -108276.76
Cover Amount: TRUNCATE(-108276.76 / 3.9) = TRUNCATE(-27763.27) = -27763
Margin Call Size: -27763 * 4 = - 111052
```

> [!TIP]
> Programmers can use the [strategy.margin\_liquidation\_price](https://www.tradingview.com/pine-script-reference/v6/#var_strategy.margin_liquidation_price) variable to retrieve the current _price level_ at which the broker emulator will trigger a margin call event and forcibly liquidate part or all of an open position if the market price reaches it. For more information about how margin and leverage work in strategies, as well as details on how to calculate the liquidation price, refer to the [How to simulate trading with leverage in Pine Script](https://www.tradingview.com/support/solutions/43000717375/) article in our Help Center.

## Using strategy information in scripts {#using-strategy-information-in-scripts}

Numerous built-ins within the `strategy.*` namespace and its _sub-namespaces_ provide convenient solutions for programmers to use a strategy’s trade and performance information, including data shown in the [strategy report](https://www.tradingview.com/pine-script-docs/concepts/strategies/#strategy-report), directly within their code’s logic and calculations.

Several `strategy.*` variables store essential information about a strategy, including its starting capital, equity, profits and losses, run-up and drawdown, and open position:

-   [strategy.account\_currency](https://www.tradingview.com/pine-script-reference/v6/#var_strategy.account_currency)
-   [strategy.initial\_capital](https://www.tradingview.com/pine-script-reference/v6/#var_strategy.initial_capital)
-   [strategy.equity](https://www.tradingview.com/pine-script-reference/v6/#var_strategy.equity)
-   [strategy.netprofit](https://www.tradingview.com/pine-script-reference/v6/#var_strategy.netprofit) and [strategy.netprofit\_percent](https://www.tradingview.com/pine-script-reference/v6/#var_strategy.netprofit_percent)
-   [strategy.grossprofit](https://www.tradingview.com/pine-script-reference/v6/#var_strategy.grossprofit) and [strategy.grossprofit\_percent](https://www.tradingview.com/pine-script-reference/v6/#var_strategy.grossprofit_percent)
-   [strategy.grossloss](https://www.tradingview.com/pine-script-reference/v6/#var_strategy.grossloss) and [strategy.grossloss\_percent](https://www.tradingview.com/pine-script-reference/v6/#var_strategy.grossloss_percent)
-   [strategy.openprofit](https://www.tradingview.com/pine-script-reference/v6/#var_strategy.openprofit) and [strategy.openprofit\_percent](https://www.tradingview.com/pine-script-reference/v6/#var_strategy.openprofit_percent)
-   [strategy.max\_runup](https://www.tradingview.com/pine-script-reference/v6/#var_strategy.max_runup) and [strategy.max\_runup\_percent](https://www.tradingview.com/pine-script-reference/v6/#var_strategy.max_runup_percent)
-   [strategy.max\_drawdown](https://www.tradingview.com/pine-script-reference/v6/#var_strategy.max_drawdown) and [strategy.max\_drawdown\_percent](https://www.tradingview.com/pine-script-reference/v6/#var_strategy.max_drawdown_percent)
-   [strategy.position\_size](https://www.tradingview.com/pine-script-reference/v6/#var_strategy.position_size)
-   [strategy.position\_avg\_price](https://www.tradingview.com/pine-script-reference/v6/#var_strategy.position_avg_price)
-   [strategy.position\_entry\_name](https://www.tradingview.com/pine-script-reference/v6/#var_strategy.position_entry_name)

Additionally, the namespace features multiple variables that store general trade information, such as the number of open and closed trades, the number of winning and losing trades, average trade profits, and maximum trade sizes:

-   [strategy.opentrades](https://www.tradingview.com/pine-script-reference/v6/#var_strategy.opentrades)
-   [strategy.closedtrades](https://www.tradingview.com/pine-script-reference/v6/#var_strategy.closedtrades)
-   [strategy.wintrades](https://www.tradingview.com/pine-script-reference/v6/#var_strategy.wintrades)
-   [strategy.losstrades](https://www.tradingview.com/pine-script-reference/v6/#var_strategy.losstrades)
-   [strategy.eventrades](https://www.tradingview.com/pine-script-reference/v6/#var_strategy.eventrades)
-   [strategy.avg\_trade](https://www.tradingview.com/pine-script-reference/v6/#var_strategy.avg_trade) and [strategy.avg\_trade\_percent](https://www.tradingview.com/pine-script-reference/v6/#var_strategy.avg_trade_percent)
-   [strategy.avg\_winning\_trade](https://www.tradingview.com/pine-script-reference/v6/#var_strategy.avg_winning_trade) and [strategy.avg\_winning\_trade\_percent](https://www.tradingview.com/pine-script-reference/v6/#var_strategy.avg_winning_trade_percent)
-   [strategy.avg\_losing\_trade](https://www.tradingview.com/pine-script-reference/v6/#var_strategy.avg_losing_trade) and [strategy.avg\_losing\_trade\_percent](https://www.tradingview.com/pine-script-reference/v6/#var_strategy.avg_losing_trade_percent)
-   [strategy.max\_contracts\_held\_all](https://www.tradingview.com/pine-script-reference/v6/#var_strategy.max_contracts_held_all)
-   [strategy.max\_contracts\_held\_long](https://www.tradingview.com/pine-script-reference/v6/#var_strategy.max_contracts_held_long)
-   [strategy.max\_contracts\_held\_short](https://www.tradingview.com/pine-script-reference/v6/#var_strategy.max_contracts_held_short)

Programmers can use these variables to display relevant strategy information on their charts, create customized trading logic based on strategy data, calculate custom performance metrics, and more.

The following example demonstrates a few simple use cases for these `strategy.*` variables. The script uses them in its order placement and display calculations. When the calculated `rank` value crosses above 10 and the [strategy.opentrades](https://www.tradingview.com/pine-script-reference/v6/#var_strategy.opentrades) value is 0, the script calls the [strategy.entry()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.entry) command to place a “Buy” [market order](https://www.tradingview.com/pine-script-docs/concepts/strategies/#market-orders). On the following bar, where that order fills, the script calls [strategy.exit()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.exit) to create a [stop-loss](https://www.tradingview.com/pine-script-docs/concepts/strategies/#take-profit-and-stop-loss) order at a user-specified percentage below the [strategy.position\_avg\_price](https://www.tradingview.com/pine-script-reference/v6/#var_strategy.position_avg_price) value. If the `rank` crosses above 80 during the open trade, the script calls [strategy.close()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.close) to exit the position on the next bar.

The script creates a [table](https://www.tradingview.com/pine-script-reference/v6/#type_table) to display [formatted strings](https://www.tradingview.com/pine-script-docs/concepts/strings/#formatting-strings) representing information from several of the above `strategy.*` variables on the main chart pane. The text in the table shows the strategy’s net profit and net profit percentage, the account currency, the number of winning trades and the win percentage, the ratio of the average winning trade to the average losing trade, and the profit factor (the ratio of the gross profit to the gross loss). The script also plots the [strategy.equity](https://www.tradingview.com/pine-script-reference/v6/#var_strategy.equity) series in a separate pane and highlights the pane’s background based on the [strategy.openprofit](https://www.tradingview.com/pine-script-reference/v6/#var_strategy.openprofit) value:

```pine
//@version=6
strategy(
     "Using strategy information demo", default_qty_type = strategy.percent_of_equity, default_qty_value = 5, 
     margin_long = 100, margin_short = 100
 )

//@variable The number of bars in the `rank` calculation.
int lengthInput = input.int(50, "Length", 1)
//@variable The stop-loss percentage.
float slPercentInput = input.float(4.0, "SL %", 0.0, 100.0) / 100.0

//@variable The percent rank of `close` prices over `lengthInput` bars.
float rank = ta.percentrank(close, lengthInput)
// Entry and exit signals.  
bool entrySignal = ta.crossover(rank, 10) and strategy.opentrades == 0
bool exitSignal  = ta.crossover(rank, 80) and strategy.opentrades == 1

// Place orders based on the `entrySignal` and `exitSignal` occurrences. 
switch
    entrySignal    => strategy.entry("Buy", strategy.long)
    entrySignal[1] => strategy.exit("SL", "Buy", stop = strategy.position_avg_price * (1.0 - slPercentInput))
    exitSignal     => strategy.close("Buy")

if barstate.islastconfirmedhistory or barstate.isrealtime
    //@variable A table displaying strategy information on the main chart pane. 
    var table dashboard = table.new(
         position.top_right, 2, 10, border_color = chart.fg_color, border_width = 1, force_overlay = true
     )
    //@variable The strategy's currency.
    string currency = strategy.account_currency
    // Display the net profit as a currency amount and percentage.
    dashboard.cell(0, 1, "Net P/L")
    dashboard.cell(
         1, 1, str.format("{0, number, 0.00} {1} ({2}%)", strategy.netprofit, currency, strategy.netprofit_percent), 
         text_color = chart.fg_color, bgcolor = strategy.netprofit > 0 ? color.lime : color.red
     )
    // Display the number of winning trades as an absolute value and percentage of all completed trades. 
    dashboard.cell(0, 2, "Winning trades")
    dashboard.cell(
         1, 2, str.format("{0} ({1, number, #.##%})", strategy.wintrades, strategy.wintrades / strategy.closedtrades), 
         text_color = chart.fg_color, bgcolor = strategy.wintrades > strategy.losstrades ? color.lime : color.red
     )
    // Display the ratio of average trade profit to average trade loss. 
    dashboard.cell(0, 3, "Avg. win / Avg. loss")
    dashboard.cell(
         1, 3, str.format("{0, number, #.###}", strategy.avg_winning_trade / strategy.avg_losing_trade), 
         text_color = chart.fg_color, 
         bgcolor = strategy.avg_winning_trade > strategy.avg_losing_trade ? color.lime : color.red
     )
    // Display the profit factor, i.e., the ratio of gross profit to gross loss. 
    dashboard.cell(0, 4, "Profit factor")
    dashboard.cell(
         1, 4, str.format("{0, number, #.###}", strategy.grossprofit / strategy.grossloss), text_color = chart.fg_color, 
         bgcolor = strategy.grossprofit > strategy.grossloss ? color.lime : color.red
     )

// Plot the current equity in a separate pane and highlight the pane's background while there is an open position.
plot(strategy.equity, "Total equity", strategy.equity > strategy.initial_capital ? color.teal : color.maroon, 3)
bgcolor(
     strategy.openprofit > 0 ? color.new(color.teal, 80) : strategy.openprofit < 0 ? color.new(color.maroon, 80) : na, 
     title = "Open position highlight"
 )
```

Note that:

-   This script creates a [stop-loss](https://www.tradingview.com/pine-script-docs/concepts/strategies/#take-profit-and-stop-loss) order one bar after the entry order because it uses [strategy.position\_avg\_price](https://www.tradingview.com/pine-script-reference/v6/#var_strategy.position_avg_price) to determine the price level. This variable has a non-na value only when the strategy has an _open position_.
-   The script draws the table only on the last historical bar and all realtime bars because the _historical_ states of [tables](https://www.tradingview.com/pine-script-docs/visuals/tables/) are **never visible**. See the [Reducing drawing updates](https://www.tradingview.com/pine-script-docs/writing/profiling-and-optimization/#reducing-drawing-updates) section of the [Profiling and optimization](https://www.tradingview.com/pine-script-docs/writing/profiling-and-optimization/) page for more information.
-   The [table.new()](https://www.tradingview.com/pine-script-reference/v6/#fun_table.new) call includes `force_overlay = true` to display the table on the main chart pane.

### Individual trade information {#individual-trade-information}

The `strategy.*` namespace features two sub-namespaces that provide access to _individual trade_ information: `strategy.opentrades.*` and `strategy.closedtrades.*`. The `strategy.opentrades.*` built-ins return data for _incomplete_ (open) trades, and the `strategy.closedtrades.*` built-ins return data for _completed_ (closed) trades. With these built-ins, programmers can use granular trade data in their scripts, allowing for more detailed strategy analysis and advanced calculations.

Both sub-namespaces contain several similar functions that return information about a trade’s orders, simulated costs, and profit/loss, including:

-   [strategy.opentrades.entry\_id()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.opentrades.entry_id) / [strategy.closedtrades.entry\_id()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.closedtrades.entry_id)
-   [strategy.opentrades.entry\_price()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.opentrades.entry_price) / [strategy.closedtrades.entry\_price()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.closedtrades.entry_price)
-   [strategy.opentrades.entry\_bar\_index()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.opentrades.entry_bar_index) / [strategy.closedtrades.entry\_bar\_index()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.closedtrades.entry_bar_index)
-   [strategy.opentrades.entry\_time()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.opentrades.entry_time) / [strategy.closedtrades.entry\_time()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.closedtrades.entry_time)
-   [strategy.opentrades.entry\_comment()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.opentrades.entry_comment) / [strategy.closedtrades.entry\_comment()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.closedtrades.entry_comment)
-   [strategy.opentrades.size()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.opentrades.size) / [strategy.closedtrades.size()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.closedtrades.size)
-   [strategy.opentrades.profit()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.opentrades.profit) / [strategy.closedtrades.profit()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.closedtrades.profit)
-   [strategy.opentrades.profit\_percent()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.opentrades.profit_percent) / [strategy.closedtrades.profit\_percent()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.closedtrades.profit_percent)
-   [strategy.opentrades.commission()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.opentrades.commission) / [strategy.closedtrades.commission()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.closedtrades.commission)
-   [strategy.opentrades.max\_runup()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.opentrades.max_runup) / [strategy.closedtrades.max\_runup()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.closedtrades.max_runup)
-   [strategy.opentrades.max\_runup\_percent()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.opentrades.max_runup_percent) / [strategy.closedtrades.max\_runup\_percent()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.closedtrades.max_runup_percent)
-   [strategy.opentrades.max\_drawdown()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.opentrades.max_drawdown) / [strategy.closedtrades.max\_drawdown()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.closedtrades.max_drawdown)
-   [strategy.opentrades.max\_drawdown\_percent()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.opentrades.max_drawdown_percent) / [strategy.closedtrades.max\_drawdown\_percent()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.closedtrades.max_drawdown_percent)
-   [strategy.closedtrades.exit\_id()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.closedtrades.exit_id)
-   [strategy.closedtrades.exit\_price()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.closedtrades.exit_price)
-   [strategy.closedtrades.exit\_time()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.closedtrades.exit_time)
-   [strategy.closedtrades.exit\_bar\_index()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.closedtrades.exit_bar_index)
-   [strategy.closedtrades.exit\_comment()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.closedtrades.exit_comment)

Note that:

-   Most built-ins within these namespaces are _functions_. However, the `strategy.opentrades.*` namespace also features a unique _variable_: [strategy.opentrades.capital\_held](https://www.tradingview.com/pine-script-reference/v6/#var_strategy.opentrades.capital_held). Its value represents the amount of capital _reserved_ by _all_ open trades.
-   Only the `strategy.closedtrades.*` namespace has `.exit_*()` functions that return information about _exit orders_.

All `strategy.opentrades.*()` and `strategy.closedtrades.*()` functions have a `trade_num` parameter, which accepts an “int” value representing the index of the open or closed trade. The index of the first open/closed trade is 0, and the last trade’s index is _one less_ than the value of the [strategy.opentrades](https://www.tradingview.com/pine-script-reference/v6/#var_strategy.opentrades)/[strategy.closedtrades](https://www.tradingview.com/pine-script-reference/v6/#var_strategy.closedtrades) variable.

The following example places up to five long entry orders per position, each with a unique ID, and it calculates metrics for specific closed trades.

The strategy places a new entry order when the [close](https://www.tradingview.com/pine-script-reference/v6/#var_close) crosses above the `median` value without reaching the `highest` value, but only if the number of open trades is less than five. It exits each position using [stop-loss](https://www.tradingview.com/pine-script-docs/concepts/strategies/#take-profit-and-stop-loss) orders from [strategy.exit()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.exit) or a [market order](https://www.tradingview.com/pine-script-docs/concepts/strategies/#market-orders) from [strategy.close\_all()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.close_all). Each successive entry order’s ID depends on the number of open trades. The first entry ID in each position is `"Buy0"`, and the last possible entry ID is `"Buy4"`.

The script calls `strategy.closedtrades.*()` functions within a [for](https://www.tradingview.com/pine-script-reference/v6/#kw_for) loop to access closed trade entry IDs, profits, entry bar indices, and exit bar indices. It uses this information to calculate the total number of closed trades with the specified entry ID, the number of winning trades, the average number of bars per trade, and the total profit from all the trades. The script then organizes this information in a [formatted string](https://www.tradingview.com/pine-script-docs/concepts/strings/#formatting-strings) and displays the result using a single-cell [table](https://www.tradingview.com/pine-script-reference/v6/#type_table):

```pine
//@version=6
strategy(
     "Individual trade information demo", pyramiding = 5, default_qty_type = strategy.percent_of_equity, 
     default_qty_value = 1, margin_long = 100, margin_short = 100
 )

//@variable The number of bars in the `highest` and `lowest` calculation. 
int lengthInput = input.int(50, "Length", 1)
string idInput = input.string("Buy0", "Entry ID to analyze", ["Buy0", "Buy1", "Buy2", "Buy3", "Buy4"])

// Calculate the highest, lowest, and median `close` values over `lengthInput` bars.
float highest = ta.highest(close, lengthInput)
float lowest  = ta.lowest(close, lengthInput)
float median  = 0.5 * (highest + lowest)

// Define entry and stop-loss orders when the `close` crosses above the `median` without touching the `highest` value.
if ta.crossover(close, median) and close != highest and strategy.opentrades < 5
    strategy.entry("Buy" + str.tostring(strategy.opentrades), strategy.long) 
    if strategy.opentrades == 0
        strategy.exit("SL", stop = lowest)
// Close the entire position when the `close` reaches the `lowest` value.
if close == lowest
    strategy.close_all()

// The total number of closed trades with the `idInput` entry, the number of wins, the average number of bars, 
// and the total profit.
int   trades  = 0
int   wins    = 0
float avgBars = 0
float totalPL = 0.0

if barstate.islastconfirmedhistory or barstate.isrealtime
    //@variable A single-cell table displaying information about closed trades with the `idInput` entry ID. 
    var table infoTable = table.new(position.middle_center, 1, 1, color.purple)
    // Iterate over closed trade indices.
    for tradeNum = 0 to strategy.closedtrades - 1
        // Skip the rest of the current iteration if the `tradeNum` closed trade didn't open with an `idInput` entry.
        if strategy.closedtrades.entry_id(tradeNum) != idInput
            continue
        // Accumulate `trades`, `wins`, `avgBars`, and `totalPL` values.
        float profit = strategy.closedtrades.profit(tradeNum)
        trades  += 1
        wins    += profit > 0 ? 1 : 0
        avgBars += strategy.closedtrades.exit_bar_index(tradeNum) - strategy.closedtrades.entry_bar_index(tradeNum) + 1
        totalPL += profit
    avgBars /= trades

    //@variable A formatted string containing the calculated closed trade information. 
    string displayText = str.format(
         "ID: {0}\n\nTotal trades: {1}\nWin trades: {2}\nAvg. bars: {3}\nTotal P/L: {4} {5}",
         idInput, trades, wins, avgBars, totalPL, strategy.account_currency
     )
    // Populate the table's cell with `displayText`. 
    infoTable.cell(0, 0, displayText, text_color = color.white, text_halign = text.align_left, text_size = size.large)

// Plot the highest, median, and lowest values on the main chart pane. 
plot(highest, "Highest close", force_overlay = true)
plot(median, "Median close", force_overlay = true)
plot(lowest, "Lowest close", force_overlay = true)
```

Note that:

-   This strategy can open up to five long trades per position by default because we included `pyramiding = 5` in the [strategy()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy) declaration statement. See the [pyramiding](https://www.tradingview.com/pine-script-docs/concepts/strategies/#pyramiding) section for more information.
-   The [strategy.exit()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy.exit) call in this script persists and generates exit orders for every entry in the open position because it does not include a `from_entry` argument. See the [Exits for multiple entries](https://www.tradingview.com/pine-script-docs/concepts/strategies/#exits-for-multiple-entries) section to learn more about this behavior.

## Strategy alerts {#strategy-alerts}

Pine Script indicators (not strategies) have two different mechanisms to set up custom alert conditions: the [alertcondition()](https://www.tradingview.com/pine-script-reference/v6/#fun_alertcondition) function, which defines a separate alert trigger per function call, and the [alert()](https://www.tradingview.com/pine-script-reference/v6/#fun_alert) function, which defines a single alert trigger based on all calls in the code, but provides greater flexibility in the number of calls, alert messages, etc.

Pine Script strategies cannot create alert triggers using the [alertcondition()](https://www.tradingview.com/pine-script-reference/v6/#fun_alertcondition) function, but they can create triggers with the [alert()](https://www.tradingview.com/pine-script-reference/v6/#fun_alert) function. Additionally, each [order placement command](https://www.tradingview.com/pine-script-docs/concepts/strategies/#order-placement-and-cancellation) comes with its own built-in alert functionality that does not require any additional code to implement. As such, any strategy that uses an order placement command can issue alerts upon order execution. The precise mechanics of such built-in strategy alerts are described in the [Order Fill events](https://www.tradingview.com/pine-script-docs/concepts/alerts/#order-fill-events) section of the [Alerts](https://www.tradingview.com/pine-script-docs/concepts/alerts/) page.

If a strategy uses both the [alert()](https://www.tradingview.com/pine-script-reference/v6/#fun_alert) function and functions that create orders in the same script, the “Create Alert” dialog box provides a choice between the conditions to use as a trigger: [alert()](https://www.tradingview.com/pine-script-reference/v6/#fun_alert) events, order fill events, or both.

For many trading strategies, the delay between a triggered alert and a live trade can be a critical performance factor. By default, strategy scripts execute [alert()](https://www.tradingview.com/pine-script-reference/v6/#fun_alert) function calls only on the close of realtime bars, as if they used the [alert.freq\_once\_per\_bar\_close](https://www.tradingview.com/pine-script-reference/v6/#const_alert.freq_once_per_bar_close) frequency, regardless of the `freq` argument in the call. Users can change the allowed alert frequency by enabling the “On realtime bar tick” or “On order fill” [Script execution](https://www.tradingview.com/support/solutions/43000786178/) setting. See the [Altering calculation behavior](https://www.tradingview.com/pine-script-docs/concepts/strategies/#altering-calculation-behavior) section above to learn more about these settings.

Order fill alert triggers do not suffer the same limitations as the triggers from [alert()](https://www.tradingview.com/pine-script-reference/v6/#fun_alert) calls. Alerts from order fill events execute _immediately_, regardless of the script’s execution settings. Therefore, they are often more suitable for sending alerts to third parties for automation. Users can specify the default message for order fill alerts by using the `//@strategy_alert_message` compiler annotation. The text included in the annotation populates the “Message” field in the “Create alert” dialog box.

The following script shows a simple example of a default order fill alert message. Above the [strategy()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy) declaration statement, the script includes the `//@strategy_alert_message` annotation with [placeholders](https://www.tradingview.com/support/solutions/43000531021-how-to-use-a-variable-value-in-alert/) for the trade action, current position size, ticker name, and fill price values in the message text:

```pine
//@version=6
//@strategy_alert_message {{strategy.order.action}} {{strategy.position_size}} {{ticker}} @ {{strategy.order.price}}
strategy("Alert Message demo", overlay = true)
float fastMa = ta.sma(close, 5)
float slowMa = ta.sma(close, 10)

if ta.crossover(fastMa, slowMa)
    strategy.entry("buy", strategy.long)

if ta.crossunder(fastMa, slowMa)
    strategy.entry("sell", strategy.short)

plot(fastMa, "Fast MA", color.aqua)
plot(slowMa, "Slow MA", color.orange)
```

This script populates the “Create alert” dialog box with its default message when the user selects its name from the “Condition” section:

Each time the alert occurs, the strategy _replaces_ the placeholders in the alert message with their corresponding values. For example:

## Notes on testing strategies {#notes-on-testing-strategies}

Testing and tuning strategies in historical and live market conditions can provide insight into a strategy’s characteristics, potential weaknesses, and _possibly_ its future potential. However, traders should always be cautious of the biases and limitations of simulated strategy results, especially when using the results to support live trading decisions. This section outlines some caveats associated with strategy validation and tuning, and lists possible solutions to mitigate their effects.

> [!IMPORTANT]
> Although testing strategies on existing data might provide traders useful information about a strategy’s qualities, it’s crucial to understand that neither the past nor the present guarantees the _future_. Financial markets can change rapidly and unpredictably, leading to uncontrollable, unforeseen losses. Additionally, simulated results may not fully account multiple real-world factors that can impact trading performance. Therefore, we recommend that traders thoroughly understand the limitations of strategy simulations, and consider them only “parts of the whole” in their validation processes rather than basing decisions solely on the results.

### Backtesting and forward testing {#backtesting-and-forward-testing}

_Backtesting_ is a technique to evaluate the past performance of a trading strategy or model by simulating and analyzing its results on historical market data. This technique assumes that a strategy’s results on past data can provide insight into its strengths and weaknesses. When backtesting, many traders adjust the parameters of a strategy in an attempt to optimize its results. Analysis and optimization of historical results can help traders to gain a deeper understanding of a strategy’s characteristics. However, traders should always understand the risks and limitations when basing their live trading decisions on optimized backtest results.

It is prudent to also use realtime analysis as a tool for evaluating a trading system on a forward-looking basis. _Forward testing_ aims to gauge the performance of a strategy in live market conditions, where factors such as trading costs, slippage, and liquidity can meaningfully affect overall trading performance. While forward testing has the distinct advantage of not being affected by some types of biases (e.g., lookahead bias or “future data leakage”), it does carry the disadvantage of being limited in the quantity of data to test. Therefore, although it can provide helpful insights into a strategy’s performance in current market conditions, forward testing is not typically used on its own.

### Lookahead bias {#lookahead-bias}

One typical issue in backtesting strategies that request alternate timeframe data, use repainting variables such as [timenow](https://www.tradingview.com/pine-script-reference/v6/#var_timenow), or [alter calculation behavior](https://www.tradingview.com/pine-script-docs/concepts/strategies/#altering-calculation-behavior) for intrabar order fills, is the leakage of _future_ data into the _past_ during evaluation. We refer to this behavior as _lookahead bias_. Not only is lookahead bias a common cause of unrealistic strategy results, because the future is always unknown, but it is also one of the common causes of strategy [repainting](https://www.tradingview.com/pine-script-docs/concepts/repainting/).

Traders can often confirm whether a strategy has lookahead bias by forward testing it on realtime data, where no known data exists beyond the latest bar. Because there is no _future_ data to leak into the past on realtime bars, the strategy will behave _differently_ on historical and realtime bars if it is affected by lookahead bias.

To eliminate lookahead bias in a strategy:

-   Do not use repainting variables that leak future values into the past in the strategy’s order placement or cancellation logic.
-   Do not include [barmerge.lookahead\_on](https://www.tradingview.com/pine-script-reference/v6/#const_barmerge.lookahead_on) in `request.*()` calls without offsetting the data series with the [history-referencing operator](https://www.tradingview.com/pine-script-docs/language/operators/#-history-referencing-operator), especially when requesting data from a higher timeframe. See the [`lookahead`](https://www.tradingview.com/pine-script-docs/concepts/other-timeframes-and-data/#lookahead) section of the [Other timeframes and data](https://www.tradingview.com/pine-script-docs/concepts/other-timeframes-and-data/) page for more information.
-   When using the “On order fill” [Script execution](https://www.tradingview.com/support/solutions/43000786178/) setting, avoid using the _current_ values of built-in variables that hold price or volume data for each bar. During the script’s historical executions, the current value of a variable such as [close](https://www.tradingview.com/pine-script-reference/v6/#var_close) always refers to the _final value_ as of the bar’s _closing tick_, even while the script executes on the bar’s _first_ tick. See the [`calc_on_order_fills`](https://www.tradingview.com/pine-script-docs/concepts/strategies/#calc_on_order_fills) section above to learn more about this behavior.

### Selection bias {#selection-bias}

Selection bias occurs when a trader analyzes performance results for a set of specific instruments, timeframes, or testing ranges while ignoring others. This bias can distort the trader’s perspective of the strategy’s robustness, thus impacting trading decisions and performance optimizations. Traders can reduce the effects of selection bias by evaluating their strategies on multiple, ideally _diverse_ datasets, and avoiding ignoring poor performance results or “cherry-picking” testing ranges.

### Overfitting {#overfitting}

A common problem when optimizing a strategy based on backtest results is _overfitting_ (“curve fitting”), which refers to tailoring the strategy for improved performance on specific datasets. A strategy that suffers from overfitting often fails to perform well on new, _unseen_ data. One widely-used approach to help reduce overfitting and promote better generalization is to split an instrument’s data into two or more parts to test the strategy outside the sample used for optimization. This process is often known as “in-sample” (IS) and “out-of-sample” (OOS) backtesting.

When using this approach, traders optimize strategy parameters on the IS data. Then, they test the optimized configuration on the OOS data without additional fine-tuning. Although this technique and other, more robust approaches might provide a glimpse into how a strategy might fare after optimization, traders should still exercise caution. No trading strategy can _guarantee_ future performance, regardless of the data used for optimization and testing, because the future is inherently _unknown_.

### Trade limit {#trade-limit}

By default, strategies preserve information for up to the latest _9000 trades_. If a strategy simulates _more_ than 9000 trades, it _trims_ the information for the oldest closed trades and maintains data for only the most recent trades. Information for trimmed trades or their orders are **not** visible in the [strategy report](https://www.tradingview.com/pine-script-docs/concepts/strategies/#strategy-report) interface or any downloaded XLSX or CSV files. Likewise, the `strategy.closedtrades.*()` functions return [na](https://www.tradingview.com/pine-script-reference/v6/#var_na) for all trimmed trades. However, if the strategy uses [Deep Backtesting](https://www.tradingview.com/support/solutions/43000666199/) mode, it maintains information for _every_ closed trade and does not trim any orders from the report.

Programmers can retrieve the index of the oldest _untrimmed_ trade, which corresponds to the oldest trade listed in the strategy report’s [“Trades” tab](https://www.tradingview.com/pine-script-docs/concepts/strategies/#trades-tab), by using the [strategy.closedtrades.first\_index](https://www.tradingview.com/pine-script-reference/v6/#var_strategy.closedtrades.first_index) variable. Scripts can use the index in `strategy.closedtrades.*()` calls to retrieve information for the oldest available closed trade. If the strategy simulates fewer than 9000 trades or runs in Deep Backtesting mode, the variable’s value is 0, representing the actual first trade in the simulation.

> [!NOTE]
> Deep Backtesting mode only affects the testing range of results displayed in the _strategy report_. When using this mode, the strategy’s trade markers, [plots](https://www.tradingview.com/pine-script-docs/visuals/plots/), [alerts](https://www.tradingview.com/pine-script-docs/concepts/alerts/), and [Pine Logs](https://www.tradingview.com/pine-script-docs/writing/debugging/#pine-logs) are calculated using only the available chart data, regardless of the specified testing range.

To learn more about retrieving trade data using the `strategy.closedtrades.*()` built-ins, refer to the [Individual trade information](https://www.tradingview.com/pine-script-docs/concepts/strategies/#individual-trade-information) section above.
