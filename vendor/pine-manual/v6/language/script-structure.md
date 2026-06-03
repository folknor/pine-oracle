---
title: Script structure
source: https://www.tradingview.com/pine-script-docs/language/script-structure/
section: language
---

# Script structure {#script-structure}

A Pine script follows this general structure:

```
<version>
<declaration_statement>
<code>
```

## Version {#version}

A [compiler annotation](https://www.tradingview.com/pine-script-docs/language/script-structure/#compiler-annotations) in the following form tells the compiler which of the versions of Pine Script® the script is written in:

```pine
//@version=6
```

-   The version number is a number from 1 to 6.
-   The compiler annotation is not mandatory. When omitted, version 1 is assumed. It is strongly recommended to always use the latest version of the language.
-   While it is synctactically correct to place the version compiler annotation anywhere in the script, it is much more useful to readers when it appears at the top of the script.

Notable changes to the current version of Pine Script are documented in the [Release notes](https://www.tradingview.com/pine-script-docs/release-notes/).

## Declaration statement {#declaration-statement}

All Pine scripts must contain one declaration statement, which is a call to one of these functions:

-   [indicator()](https://www.tradingview.com/pine-script-reference/v6/#fun_indicator)
-   [strategy()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy)
-   [library()](https://www.tradingview.com/pine-script-reference/v6/#fun_library)

The declaration statement:

-   Identifies the type of the script, which in turn dictates which content is allowed in it, and how it can be used and executed.
-   Sets key properties of the script such as its name, where it will appear when it is added to a chart, the precision and format of the values it displays, and certain values that govern its runtime behavior, such as the maximum number of drawing objects it will display on the chart. With strategies, the properties include parameters that control backtesting, such as initial capital, commission, slippage, etc.

Each script type has distinct basic requirements. Scripts that do not meet these criteria cause a compilation error:

-   Indicators must call at least one function that creates a script output, such as [plot()](https://www.tradingview.com/pine-script-reference/v6/#fun_plot), [plotshape()](https://www.tradingview.com/pine-script-reference/v6/#fun_plotshape), [barcolor()](https://www.tradingview.com/pine-script-reference/v6/#fun_barcolor), [line.new()](https://www.tradingview.com/pine-script-reference/v6/#fun_line.new), [log.info()](https://www.tradingview.com/pine-script-reference/v6/#fun_log.info), [alert()](https://www.tradingview.com/pine-script-reference/v6/#fun_alert), etc.
-   [Strategies](https://www.tradingview.com/pine-script-docs/concepts/strategies/) must call at least one [order placement command](https://www.tradingview.com/pine-script-docs/concepts/strategies/#order-placement-and-cancellation) or other output function.
-   [Libraries](https://www.tradingview.com/pine-script-docs/concepts/libraries/) must [export](https://www.tradingview.com/pine-script-reference/v6/#kw_export) at least one user-defined [function](https://www.tradingview.com/pine-script-docs/language/user-defined-functions/), [method](https://www.tradingview.com/pine-script-docs/language/methods/#user-defined-methods), [type](https://www.tradingview.com/pine-script-docs/language/type-system/#user-defined-types), or [enum](https://www.tradingview.com/pine-script-docs/language/enums/).

## Code {#code}

Lines in a script that are not [comments](https://www.tradingview.com/pine-script-docs/language/script-structure/#comments) or [compiler annotations](https://www.tradingview.com/pine-script-docs/language/script-structure/#compiler-annotations) are _statements_, which implement the script’s algorithm. A statement can be one of these:

-   [variable declaration](https://www.tradingview.com/pine-script-docs/language/variable-declarations/)
-   [variable reassignment](https://www.tradingview.com/pine-script-docs/language/variable-declarations/#variable-reassignment)
-   [function definition](https://www.tradingview.com/pine-script-docs/language/user-defined-functions/#structure-and-syntax)
-   [built-in function call](https://www.tradingview.com/pine-script-docs/language/built-ins/#built-in-functions), [user-defined function call](https://www.tradingview.com/pine-script-docs/language/user-defined-functions/) or [a library function call](https://www.tradingview.com/pine-script-docs/concepts/libraries/#using-a-library)
-   [if](https://www.tradingview.com/pine-script-reference/v6/#kw_if), [for](https://www.tradingview.com/pine-script-reference/v6/#kw_for), [while](https://www.tradingview.com/pine-script-reference/v6/#kw_while), [switch](https://www.tradingview.com/pine-script-reference/v6/#kw_switch), [type](https://www.tradingview.com/pine-script-reference/v6/#kw_type), or [enum](https://www.tradingview.com/pine-script-reference/v6/#kw_enum) _structure_.

Statements can be arranged in multiple ways:

-   Some statements can be expressed in one line, like most variable declarations, lines containing only a function call or single-line function declarations. Lines can also be [wrapped](https://www.tradingview.com/pine-script-docs/language/script-structure/#line-wrapping) (continued on multiple lines). Multiple one-line statements can be concatenated on a single line by using the comma as a separator.
-   Others statements such as structures or multiline function definitions always require multiple lines because they require a _local block_. A local block must be indented by a tab or four spaces. Each local block defines a distinct _local scope_.
-   Statements in the _global scope_ of the script (i.e., which are not part of local blocks) cannot begin with white space (a space or a tab). Their first character must also be the line’s first character. Lines beginning in a line’s first position become by definition part of the script’s _global scope_.

A simple valid Pine Script indicator can be generated in the Pine Script Editor by using the “Open” button and choosing “New blank indicator”:

```pine
//@version=6
indicator("My Script")
plot(close)
```

This indicator includes three local blocks, one in the `barIsUp()` function declaration, and two in the variable declaration using an [if](https://www.tradingview.com/pine-script-reference/v6/#kw_if) structure:

```pine
//@version=6

indicator("", "", true)    // Declaration statement (global scope)

barIsUp() =>    // Function declaration (global scope)
    close > open    // Local block (local scope)

plotColor = if barIsUp()  // Variable declaration (global scope)
    color.green     // Local block (local scope)
else
    color.red       // Local block (local scope)

bgcolor(color.new(plotColor, 70))   // Call to a built-in function  (global scope)
```

You can bring up a simple Pine Script strategy by selecting “New blank strategy” instead:

```pine
//@version=6
strategy("My Strategy", overlay=true, margin_long=100, margin_short=100)

longCondition = ta.crossover(ta.sma(close, 14), ta.sma(close, 28))
if (longCondition)
    strategy.entry("My Long Entry Id", strategy.long)

shortCondition = ta.crossunder(ta.sma(close, 14), ta.sma(close, 28))
if (shortCondition)
    strategy.entry("My Short Entry Id", strategy.short)
```

## Comments {#comments}

Double slashes (`//`) define comments in Pine Script. Comments can begin anywhere on the line. They can also follow Pine Script code on the same line:

```pine
//@version=6
indicator("")
// This line is a comment
a = close // This is also a comment
plot(a)
```

The Pine Editor has a keyboard shortcut to comment/uncomment lines: `ctrl` + `/`. You can use it on multiple lines by highlighting them first.

## Line wrapping {#line-wrapping}

Scripts can use _line wrapping_ to define a long _single line_ of code across _multiple_ lines. Generally, each wrapped line after the first can use any indentation length _except_ multiples of four, because Pine uses four-space or tab indentations to define [local code blocks](https://www.tradingview.com/pine-script-docs/faq/programming/#what-does-scope-mean).

For example, consider the following line of code:

```pine
float x = open + high + low + close
```

We can distribute any part of this single line of code across two or more lines. Within a wrapped statement or expression, the subsequent lines can use different indentation lengths, and can also include [comments](https://www.tradingview.com/pine-script-docs/language/script-structure/#comments) without disrupting the code:

```pine
float x = open +
  high +           // Indented by 2 spaces.
     low +         // Indented by 5 spaces.
          close    // Indented by 10 spaces.
```

If parts of a wrapped expression are enclosed in _parentheses_ `( )`, such as function calls or parameter declarations, the wrapped lines within the parentheses _do not_ have any restriction on their indentation lengths. Therefore, those wrapped lines can use any indentation length _including_ multiples of four.

For example, this script demonstrates various ways that expressions enclosed in parentheses can wrap across multiple lines:

```pine
//@version=6
indicator("Line wrapping within parentheses demo")

// We can enclose operations in parentheses to wrap them across multiple lines using a four-space indentation.
float x = (open +
    high +              
    low +          
    close)

// We can wrap a long function call across two lines, for a minimal line wrapping style.
plot(ta.sma(close, 14), title = "Avg close", color = color.new(color.purple, 70), style = plot.style_area,
 force_overlay = true, display = display.all - display.status_line)     // Indented by one space.

// We can also wrap a long function call across multiple lines, each with different indentation lengths.
// The parentheses enclosing the wrapped lines can start and end on separate lines than the wrapped content.
plot(
 series = x, title = "Sum OHLC",                              // Indented by one space.
   color = (x >= x[1] ? color.green : color.red),             // Indented by three spaces.
    linewidth = 4,                                            // Indented by four spaces.
        style = plot.style_stepline                           // Indented by eight spaces.
)                                                             // No indentation.
```

Expressions inside _local_ code blocks can also use line wrapping. A local block requires indenting each line that belongs to its scope by four spaces or a tab relative to the local block’s header. Therefore, we recommend indenting any wrapped lines inside local blocks by a _larger_ indentation than that of the block’s scope for readability. For example:

```pine
upDown(float s) =>
    // These lines are indented by four spaces relative to the `upDown()` function header to belong to its local scope.
    var int ud = 0
    bool isEqual   = s == s[1]
    bool isGrowing = s > s[1]
    // Within the local block, this statement wraps across multiple lines, where each line uses   
    // an indentation length that is larger than the indentation that signifies the local block's scope.  
    ud := isEqual ?
           0 :
           isGrowing ?
               (ud <= 0 ?
                    1 :
                    ud + 1) :
               (ud >= 0 ?
                    -1 :
                    ud - 1)
```

Scripts can also create line-wrapped expressions by using [multiline strings](https://www.tradingview.com/pine-script-docs/concepts/strings/#multiline-strings). A multiline string, enclosed by three pairs of quotation marks (e.g., `"""..."""`) or apostrophes (e.g., `'''...'''`), can occupy multiple lines in the Pine Editor, where each part of the code between the `"""` or `'''` delimiters represents _literal text_. If an expression uses a string spanning multiple lines as an operand or argument, Pine still treats that expression as part of a _single_ line of code. For example:

```pine
//@version=6
indicator("Line wrapping with multiline strings demo", overlay = true)

//@variable A string indicating whether the bar is rising, falling, or neutral.
//          The strings in the ternary operations span multiple visible lines, but the entire expression is 
//          considered part of a single line. 
string labelText = close > open ? """
Bar is rising.
""" : close < open ? """
Bar is falling.
""" : """
Bar is neutral.
"""

// Draw a label to display the text from the current `labelText` string.
label.new(bar_index, close, labelText)
```

Note that:

-   A multiline string treats _all_ characters between the `"""` or `'''` delimiters as literal text, including the _leading spaces_ on each visible code line. If a line in a multiline string definition is indented by any number of spaces, relative to _column 0_ in the Pine Editor, the line in the resulting string also includes that indentation.

## Compiler annotations {#compiler-annotations}

Compiler annotations are [comments](https://www.tradingview.com/pine-script-docs/language/script-structure/#comments) that issue special instructions for a script:

-   `//@version=` specifies the PineScript version that the compiler will use. The number in this annotation should not be confused with the script’s version number, which updates on every saved change to the code.
-   `//@description` sets a custom description for scripts that use the [library()](https://www.tradingview.com/pine-script-reference/v6/#fun_library) declaration statement.
-   `//@function`, `//@param` and `//@returns` add custom descriptions for a [user-defined function](https://www.tradingview.com/pine-script-docs/language/user-defined-functions/) or [method](https://www.tradingview.com/pine-script-docs/language/methods/), its parameters, and its result when placed above the function declaration.
-   `//@type` adds a custom description for a [user-defined type (UDT)](https://www.tradingview.com/pine-script-docs/language/type-system/#user-defined-types) when placed above the type declaration.
-   `//@enum` adds a custom description for an [enum types](https://www.tradingview.com/pine-script-docs/language/type-system/#enum-types) when placed above the enum declaration.
-   `//@field` adds a custom description for the field of a [user-defined type (UDT)](https://www.tradingview.com/pine-script-docs/language/type-system/#user-defined-types) or an [enum types](https://www.tradingview.com/pine-script-docs/language/type-system/#enum-types) when placed above the type or enum declaration.
-   `//@variable` adds a custom description for a variable when placed above its declaration.
-   `//@strategy_alert_message` provides a default message for strategy scripts to pre-fill the “Message” field in the alert creation dialog.

The Pine Editor also features two specialized annotations, `//#region` and `//#endregion`, that create _collapsible_ code regions. Clicking the dropdown arrow next to a `//#region` line collapses all the code between that line and the nearest `//#endregion` annotation below it.

This example draws a triangle using three interactively selected points on the chart. The script illustrates how one can use compiler and Editor annotations to document code and make it easier to navigate:

```pine
//@version=6
indicator("Triangle", "", true)

//#region ———————————————————— Constants and inputs

int   TIME_DEFAULT  = 0
float PRICE_DEFAULT = 0.0

x1Input = input.time(TIME_DEFAULT,   "Point 1", inline = "1", confirm = true)
y1Input = input.price(PRICE_DEFAULT, "",        inline = "1", tooltip = "Pick point 1", confirm = true)
x2Input = input.time(TIME_DEFAULT,   "Point 2", inline = "2", confirm = true)
y2Input = input.price(PRICE_DEFAULT, "",        inline = "2", tooltip = "Pick point 2", confirm = true)
x3Input = input.time(TIME_DEFAULT,   "Point 3", inline = "3", confirm = true)
y3Input = input.price(PRICE_DEFAULT, "",        inline = "3", tooltip = "Pick point 3", confirm = true)
//#endregion

//#region ———————————————————— Types and functions

// @type            Used to represent the coordinates and color to draw a triangle.
// @field time1     Time of first point.
// @field time2     Time of second point.
// @field time3     Time of third point.
// @field price1    Price of first point.
// @field price2    Price of second point.
// @field price3    Price of third point.
// @field lineColor Color to be used to draw the triangle lines.
type Triangle
    int   time1
    int   time2
    int   time3
    float price1
    float price2
    float price3
    color lineColor

//@function Draws a triangle using the coordinates of the `t` object.
//@param t  (Triangle) Object representing the triangle to be drawn.
//@returns  The ID of the last line drawn.
drawTriangle(Triangle t) =>
    line.new(t.time1, t.price1, t.time2, t.price2, xloc = xloc.bar_time, color = t.lineColor)
    line.new(t.time2, t.price2, t.time3, t.price3, xloc = xloc.bar_time, color = t.lineColor)
    line.new(t.time1, t.price1, t.time3, t.price3, xloc = xloc.bar_time, color = t.lineColor)
//#endregion

//#region ———————————————————— Calculations

// Draw the triangle only once on the last historical bar.
if barstate.islastconfirmedhistory
    //@variable Used to hold the Triangle object to be drawn.
    Triangle triangle = Triangle.new()

    triangle.time1  := x1Input
    triangle.time2  := x2Input
    triangle.time3  := x3Input
    triangle.price1 := y1Input
    triangle.price2 := y2Input
    triangle.price3 := y3Input
    triangle.lineColor := color.purple

    drawTriangle(triangle)
//#endregion
```
