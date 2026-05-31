---
title: Style guide
source: https://www.tradingview.com/pine-script-docs/writing/style-guide/
section: writing
---

# Style guide {#style-guide}

## Introduction {#introduction}

This style guide provides recommendations on how to name variables and organize your Pine scripts in a standard way that works well. Scripts that follow our best practices will be easier to read, understand and maintain.

You can see scripts using these guidelines published from the [TradingView](https://www.tradingview.com/u/TradingView/#published-scripts) and [PineCoders](https://www.tradingview.com/u/PineCoders/#published-scripts) accounts on the platform.

## Naming Conventions {#naming-conventions}

We recommend the use of:

-   `camelCase` for all identifiers, i.e., variable or function names: `ma`, `maFast`, `maLengthInput`, `maColor`, `roundedOHLC()`, `pivotHi()`.
-   All caps `SNAKE_CASE` for constants: `BULL_COLOR`, `BEAR_COLOR`, `MAX_LOOKBACK`.
-   The use of qualifying suffixes when it provides valuable clues about the type or provenance of a variable: `maShowInput`, `bearColor`, `bearColorInput`, `volumesArray`, `maPlotID`, `resultsTable`, `levelsColorArray`.

## Script organization {#script-organization}

The Pine Script® compiler is quite forgiving of the positioning of specific statements or the version [compiler annotation](https://www.tradingview.com/pine-script-docs/language/script-structure/#compiler-annotations) in the script. While other arrangements are syntactically correct, this is how we recommend organizing scripts:

```
<license>
<version>
<declaration_statement>
<import_statements>
<constant_declarations>
<inputs>
<function_declarations>
<calculations>
<strategy_calls>
<visuals>
<alerts>
```

### <license> {#license}

If you publish your open-source scripts publicly on TradingView (scripts can also be published privately), your open-source code is by default protected by the Mozilla license. You may choose any other license you prefer.

The reuse of code from those scripts is governed by our [House Rules on Script Publishing](https://www.tradingview.com/support/solutions/43000590599) which preempt the author’s license.

The standard license comments appearing at the beginning of scripts are:

```pine
// This source code is subject to the terms of the Mozilla Public License 2.0 at https://mozilla.org/MPL/2.0/
// © username
```

### <version> {#version}

This is the [compiler annotation](https://www.tradingview.com/pine-script-docs/language/script-structure/#compiler-annotations) defining the version of Pine Script the script will use. If none is present, v1 is used. For v6, use:

```pine
//@version=6
```

### <declaration\_statement> {#declaration_statement}

This is the mandatory declaration statement which defines the type of your script. It must be a call to either [indicator()](https://www.tradingview.com/pine-script-reference/v6/#fun_indicator), [strategy()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy), or [library()](https://www.tradingview.com/pine-script-reference/v6/#fun_library).

### <import\_statements> {#import_statements}

If your script uses one or more Pine Script [libraries](https://www.tradingview.com/pine-script-docs/concepts/libraries/), your [import](https://www.tradingview.com/pine-script-reference/v6/#kw_import) statements belong here.

### <constant\_declarations> {#constant_declarations}

Scripts can declare variables qualified as “const”, i.e., ones referencing a constant value.

We refer to variables as “constants” when they meet these criteria:

-   Their declaration uses the optional `const` keyword (see our User Manual’s section on [type qualifiers](https://www.tradingview.com/pine-script-docs/language/type-system/#qualifiers) for more information).
-   They are initialized using a literal (e.g., `100` or `"AAPL"`) or a built-in qualified as “const” (e.g., `color.green`).
-   Their value does not change during the script’s execution.

We use `SNAKE_CASE` to name these variables and group their declaration near the top of the script. For example:

```pine
// ————— Constants
int     MS_IN_MIN   = 60 * 1000
int     MS_IN_HOUR  = MS_IN_MIN  * 60
int     MS_IN_DAY   = MS_IN_HOUR * 24

color   GRAY        = #808080ff
color   LIME        = #00FF00ff
color   MAROON      = #800000ff
color   ORANGE      = #FF8000ff
color   PINK        = #FF0080ff
color   TEAL        = #008080ff
color   BG_DIV      = color.new(ORANGE, 90)
color   BG_RESETS   = color.new(GRAY, 90)

string  RST1        = "No reset; cumulate since the beginning of the chart"
string  RST2        = "On a stepped higher timeframe (HTF)"
string  RST3        = "On a fixed HTF"
string  RST4        = "At a fixed time"
string  RST5        = "At the beginning of the regular session"
string  RST6        = "At the first visible chart bar"
string  RST7        = "Fixed rolling period"

string  LTF1        = "Least precise, covering many chart bars"
string  LTF2        = "Less precise, covering some chart bars"
string  LTF3        = "More precise, covering less chart bars"
string  LTF4        = "Most precise, 1min intrabars"

string  TT_TOTVOL     = "The 'Bodies' value is the transparency of the total volume candle bodies. Zero is opaque, 100 is transparent."
string  TT_RST_HTF    = "This value is used when '" + RST3 +"' is selected."
string  TT_RST_TIME   = "These values are used when '" + RST4 +"' is selected.
  A reset will occur when the time is greater or equal to the bar's open time, and less than its close time.\nHour: 0-23\nMinute: 0-59"
string  TT_RST_PERIOD = "This value is used when '" + RST7 +"' is selected."
```

In this example:

-   The `RST*` and `LTF*` constants will be used as tuple elements in the `options` argument of `input.*()` calls.
-   The `TT_*` constants will be used as `tooltip` arguments in `input.*()` calls. Note how we use a line continuation for long string literals.
-   We do not use [var](https://www.tradingview.com/pine-script-reference/v6/#kw_var) to initialize constants. The Pine Script runtime is optimized to handle declarations on each bar, but using [var](https://www.tradingview.com/pine-script-reference/v6/#kw_var) to initialize a variable only the first time it is declared incurs a minor penalty on script performance because of the maintenance that [var](https://www.tradingview.com/pine-script-reference/v6/#kw_var) variables require on further bars.

Note that:

-   Literals used in more than one place in a script should always be declared as a constant. Using the constant rather than the literal makes it more readable if it is given a meaningful name, and the practice makes code easier to maintain. Even though the quantity of milliseconds in a day is unlikely to change in the future, `MS_IN_DAY` is more meaningful than `1000 * 60 * 60 * 24`.
-   Constants only used in the local block of a function or [if](https://www.tradingview.com/pine-script-reference/v6/#kw_if), [while](https://www.tradingview.com/pine-script-reference/v6/#kw_while), etc., statement for example, can be declared in that local block.

### <inputs> {#inputs}

It is **much** easier to read scripts when all their inputs are in the same code section. Placing that section at the beginning of the script also reflects how they are processed at runtime, i.e., before the rest of the script is executed.

Suffixing input variable names with `input` makes them more readily identifiable when they are used later in the script: `maLengthInput`, `bearColorInput`, `showAvgInput`, etc.

```pine
// ————— Inputs
string  resetInput              = input.string(RST2,        "CVD Resets",                       inline = "00", options = [RST1, RST2, RST3, RST4, RST5, RST6, RST7])
string  fixedTfInput            = input.timeframe("D",      "  Fixed HTF:  ",                   tooltip = TT_RST_HTF)
int     hourInput               = input.int(9,              "  Fixed time hour:  ",             inline = "01", minval = 0, maxval = 23)
int     minuteInput             = input.int(30,             "minute",                           inline = "01", minval = 0, maxval = 59, tooltip = TT_RST_TIME)
int     fixedPeriodInput        = input.int(20,             "  Fixed period:  ",                inline = "02", minval = 1, tooltip = TT_RST_PERIOD)
string  ltfModeInput            = input.string(LTF3,        "Intrabar precision",               inline = "03", options = [LTF1, LTF2, LTF3, LTF4])
```

### <function\_declarations> {#function_declarations}

All user-defined functions must be defined in the script’s global scope; nested function definitions are not allowed in Pine Script.

Optimal function design should minimize the use of global variables in the function’s scope, as they undermine function portability. When it can’t be avoided, those functions must follow the global variable declarations in the code, which entails they can’t always be placed in the <function\_declarations> section. Such dependencies on global variables should ideally be documented in the function’s comments.

It will also help readers if you document the function’s objective, parameters and result. The same syntax used in [libraries](https://www.tradingview.com/pine-script-docs/concepts/libraries/) can be used to document your functions. This can make it easier to port your functions to a library should you ever decide to do so:

```pine
//@version=6
indicator("<function_declarations>", "", true)

string SIZE_LARGE  = "Large"
string SIZE_NORMAL = "Normal"
string SIZE_SMALL  = "Small"

string sizeInput = input.string(SIZE_NORMAL, "Size", options = [SIZE_LARGE, SIZE_NORMAL, SIZE_SMALL])

// @function        Used to produce an argument for the `size` parameter in built-in functions.
// @param userSize  (simple string) User-selected size.
// @returns         One of the `size.*` built-in constants.
// Dependencies     SIZE_LARGE, SIZE_NORMAL, SIZE_SMALL
getSize(simple string userSize) =>
    result = 
      switch userSize
        SIZE_LARGE  => size.large
        SIZE_NORMAL => size.normal
        SIZE_SMALL  => size.small
        => size.auto

if ta.rising(close, 3)
    label.new(bar_index, na, yloc = yloc.abovebar, style = label.style_arrowup, size = getSize(sizeInput))
```

### <calculations> {#calculations}

This is where the script’s core calculations and logic should be placed. Code can be easier to read when variable declarations are placed near the code segment using the variables. Some programmers prefer to place all their non-constant variable declarations at the beginning of this section, which is not always possible for all variables, as some may require some calculations to have been executed before their declaration.

### <strategy\_calls> {#strategy_calls}

Strategies are easier to read when strategy calls are grouped in the same section of the script.

### <visuals> {#visuals}

This section should ideally include all the statements producing the script’s visuals, whether they be plots, drawings, background colors, candle-plotting, etc. See the Pine Script user manual’s section on [Z-index](https://www.tradingview.com/pine-script-docs/visuals/overview/#z-index) for more information on how the relative depth of visuals is determined.

### <alerts> {#alerts}

Alert code will usually require the script’s calculations to have executed before it, so it makes sense to put it at the end of the script.

## Spacing {#spacing}

A space should be used on both sides of all operators, except unary operators (`-1`). A space is also recommended after all commas and when using named function arguments, as in `plot(series = close)`:

```pine
int a = close > open ? 1 : -1
var int newLen = 2
newLen := min(20, newlen + 1)
float a = -b
float c = d > e ? d - e : d
int index = bar_index % 2 == 0 ? 1 : 2
plot(close, color = color.red)
```

## Line wrapping {#line-wrapping}

[Line wrapping](https://www.tradingview.com/pine-script-docs/language/script-structure/#line-wrapping) can make long lines of code easier to read by defining a _single line_ of code across _multiple_ lines in the script. Generally, scripts can wrap lines using any indentation length that is _not_ a multiple of four, because the four-space or tab indentation defines [local blocks](https://www.tradingview.com/pine-script-docs/faq/programming/#what-does-scope-mean) in Pine.

However, if a wrapped line is enclosed in _parentheses_, such as in function calls or parameter declarations, it can use any indentation length without restriction, _including_ a multiple of four. For example:

```pine
//@version=6
indicator("Line wrapping demo")

// A wrapped line that is *not* enclosed in parentheses can use any indentation length *except* a multiple of four.
//@variable The difference between the current and previous `close` values. 
float closeDiff = 
  close                         // Indented by two spaces.
  - close[1]                    // Indented by two spaces.

// A wrapped line that *is* enclosed in parentheses *can* use four-space indentation.
//@variable The percentage difference between the current and previous `close` values. 
float percentChange = (
    (closeDiff)                 // Indented by four spaces.
    / close[1] * 100  )         // Indented by four spaces.

// Within the same expression, each wrapped line can use different indentation lengths.
// The parentheses enclosing wrapped lines can also be wrapped on separate lines.
plot(
 percentChange, title = "Percent change",                            // Indented by one space.
   color = (percentChange >= 0 ? color.green : color.red),           // Indented by three spaces.
    linewidth = 8,                                                   // Indented by four spaces.
        style = plot.style_histogram, format = format.percent        // Indented by eight spaces.
)                                                                    // No indentation.
```

Line wrapping is also useful when working with long [single-line strings](https://www.tradingview.com/pine-script-docs/concepts/strings/#single-line-strings). For example, instead of defining a lengthy string on a single line of code, programmers can split that string into smaller parts and [concatenate](https://www.tradingview.com/pine-script-docs/concepts/strings/#concatenation) them using the [+](https://www.tradingview.com/pine-script-reference/v6/#op_+) operator to wrap the expression across multiple lines for readability:

```pine
//@version=6
indicator("Defining a long string across multiple lines demo")

//@variable A single, long string created by concatenating three smaller single-line strings.
// These wrapped lines are indented by five spaces. If enclosed in parentheses, they can use four spaces instead.
var string newString = "This is one long string result that is defined "
     + "across multiple lines of code by concatenating smaller strings. "
     + "When output, the text appears without line breaks until we include the \n newline escape sequence."

if barstate.isfirst
    // Output the `newString` result in the Pine Logs pane.
    log.info(newString)
```

It is possible to use various line wrapping styles within the same script and even within the same expression, as seen in the first example above. To keep the code organized and easy to read, we recommend maintaining a _consistent_ line wrapping style within the same script where possible. For instance, programmers can choose to align wrapped lines to their nearest tab space, or to wrap lines minimally only once they exceed the Pine Editor’s line length guide.

This example script shows a consistent line wrapping style that lists each argument in a function call on a wrapped line indented by four spaces. It wraps the function’s closing parentheses on a separate line without indentation to align it vertically with the beginning of the expression and signify the end of the wrapped code:

```pine
//@version=6
indicator("Consistent line wrapping style demo")

//@variable The number of values to show in the plot, counting backwards from the last bar.
int lengthInput = input.int(
    defval = 10, 
    title = "Show last",
    minval = 1
)

// Plot the last `lengthInput` values of the `close` series.
plot(
    series = close,
    title = "Close",
    color = color.blue,
    linewidth = 3,
    show_last = lengthInput
)
```

Similar line-wrapping behaviors apply to [multiline strings](https://www.tradingview.com/pine-script-docs/concepts/strings/#multiline-strings), which can span multiple visible lines in the code. All parts of the code between a multiline string’s `"""` or `'''` delimiters, including leading spaces and line breaks, represent _literal text_. Pine treats expressions that use multiline strings as part of a single code line, similar to other types of line-wrapped code. See the [Line wrapping](https://www.tradingview.com/pine-script-docs/language/script-structure/#line-wrapping) section of the [Script structure](https://www.tradingview.com/pine-script-docs/language/script-structure/) page for an example.

For readability, it is often helpful to define multiline strings separately in the _global scope_, or in line-wrapped expressions with _no indentation_. For example:

```pine
//@version=6
indicator("Multiline string style demo")

//@variable A string created using multiline syntax.
string multilineStr = """
This string spans multiple lines in the editor, but it's considered part of a single line of code.

All parts of the code here, including line breaks, represent literal text.

Starting the definition in non-indented code can help readability, because any indentation,
    <- such as this
        <- or this
is automatically included in the string.
"""

//@variable References an array of strings created using multiline syntax.
var array<string> multiStrArray = array.from(
"""When defining multiline strings inside expressions,
such as function calls,
""",

"""starting each string on a separate, non-indented line in the editor can help promote readability."""
)

// Draw labels to display text from the multiline string and array of strings on the last historical bar.
if barstate.islastconfirmedhistory
    label.new(bar_index, 0, multilineStr, textalign = text.align_left)
    label.new(bar_index, 0, multiStrArray.join(), style = label.style_label_up, textalign = text.align_left)
```

## Vertical alignment {#vertical-alignment}

Vertical alignment using tabs or spaces can be useful in code sections containing many similar lines such as constant declarations or inputs. They can make mass edits much easier using the Pine Editor’s multi-cursor feature (`ctrl` + `alt` + `🠅`):

```pine
// Colors used as defaults in inputs.
color COLOR_AQUA  = #0080FFff
color COLOR_BLACK = #000000ff
color COLOR_BLUE  = #013BCAff
color COLOR_CORAL = #FF8080ff
color COLOR_GOLD  = #CCCC00ff
```

## Explicit typing {#explicit-typing}

Including the type of variables when declaring them is not required. However, it helps make scripts easier to read, navigate, and understand. It can help clarify the expected types at each point in a script’s execution and distinguish a variable’s declaration (using `=`) from its reassignments (using `:=`). Using explicit typing can also make scripts easier to [debug](https://www.tradingview.com/pine-script-docs/writing/debugging/).
