---
title: User-defined functions
source: https://www.tradingview.com/pine-script-docs/language/user-defined-functions/
section: language
---

# User-defined functions {#user-defined-functions}

## Introduction {#introduction}

_User-defined functions_ are functions written by programmers, as opposed to the [built-in functions](https://www.tradingview.com/pine-script-docs/language/built-ins/#built-in-functions) provided by Pine Script®. They help to encapsulate custom calculations that scripts perform conditionally or repeatedly, or to isolate logic in a single location for modularity and readability. Programmers often write functions to extend the capabilities of their scripts when no existing built-ins fit their needs.

A function definition consists of two main parts: a _header_ and a _body_.

**Header**

The header declares the function’s _signature_, i.e., its _name_ and _parameters_. A script _calls_ the function by creating an expression containing the function’s name followed by parentheses (e.g., `f()`). If the function has declared parameters, calls to the function list _arguments_ (values or references) for those parameters within the parentheses (e.g., `f(x = 1)`).

**Body**

The function’s body is the code that follows the header. Each _call_ to the function performs the tasks defined by the expressions and statements in the function’s body.

Function definitions in Pine can use either of the following formats:

-   [Single-line](https://www.tradingview.com/pine-script-docs/language/user-defined-functions/#single-line-functions) format, where the function’s header and body occupy only _one_ line of code. This format is best suited for defining compact functions containing only minimal statements or expressions.
-   [Multiline](https://www.tradingview.com/pine-script-docs/language/user-defined-functions/#multiline-functions) format, where the _first_ line defines the function’s header, and all _indented_ lines that follow define the function’s body. This format is optimal for functions that require [conditional structures](https://www.tradingview.com/pine-script-docs/language/conditional-structures/), [loops](https://www.tradingview.com/pine-script-docs/language/loops/), or multiple other statements.

Programmers can define functions for use only in a specific script, or create _library functions_ for use in other scripts. Refer to our [Style guide](https://www.tradingview.com/pine-script-docs/writing/style-guide/) for recommendations on where to include function definitions in a source code. To learn more about library functions and their unique requirements, see the [Libraries](https://www.tradingview.com/pine-script-docs/concepts/libraries/) page.

Regardless of format, location, or use, several common characteristics and [limitations](https://www.tradingview.com/pine-script-docs/language/user-defined-functions/#limitations) apply to every user-defined function, including the following:

-   The function’s definition must be in the [global scope](https://www.tradingview.com/pine-script-docs/faq/programming/#what-does-scope-mean). Programmers cannot define functions inside the body of another function or the local blocks of any other structure.
-   The function cannot modify its declared parameters or any global variables.
-   The function can include calls to most other functions within its body, but it _cannot_ include calls to _itself_ or to functions that must be called from the _global scope_.
-   Each written call to the function must have consistent parameter [types](https://www.tradingview.com/pine-script-docs/language/type-system/#types), and therefore consistent _argument_ types, across all executions.
-   Each call returns the result of evaluating the _final_ statement or separate expression defined in the function’s body, and that result inherits the _strongest_ [type qualifier](https://www.tradingview.com/pine-script-docs/language/type-system/#qualifiers) used in the call’s calculations. As with parameter types, the call’s returned types must be consistent across executions.
-   Each written call to the function establishes a _new scope_ from the function’s definition. The parameters, variables, and expressions created in that scope are _unique_ and have an _independent_ history; other calls to the function do not directly affect them.

## Structure and syntax {#structure-and-syntax}

A function definition can occupy a single line of code or multiple lines, depending on the expressions and statements that the function requires. The single-line and multiline formats are similar, with the key difference being the placement of the function’s _body_.

Single-line functions define their header and body on the same line of code:

```
<functionHeader> => <functionBody>
```

In contrast, multiline functions define the body on separate lines of code following the header. The code block following the header line has an indentation of _four spaces_ or a single tab:

```
<functionHeader> =>
    <functionBody>
```

Both formats use the following syntax for defining the function’s _header_:

```
[export ]<functionName>([[[paramQualifier ]<paramType> ]<paramName>[ = defaultValue], …]) =>
```

Where:

-   All parts within square brackets (`[]`) represent _optional_ syntax, and all parts within angle brackets (`<>`) represent _required_ syntax.
-   [export](https://www.tradingview.com/pine-script-reference/v6/#kw_export) is the optional keyword for exporting the function from a library, enabling its use in other scripts. See the [Libraries](https://www.tradingview.com/pine-script-docs/concepts/libraries/) page to learn more.
-   `functionName` is the function’s identifier (name). The script calls the function by referencing this identifier, followed by parentheses.
-   `paramName` is the identifier for a declared _parameter_. The script can supply a specific _argument_ (value or reference) to the parameter in each function call. A function header can contain zero or more parameter declarations.
-   `defaultValue` is the parameter’s default argument. If not specified, each call to the function requires an argument for the parameter. Otherwise, supplying an argument is optional.
-   `paramQualifier` and `paramType` are qualifier and type _keywords_, which together specify the parameter’s [qualified type](https://www.tradingview.com/pine-script-docs/language/type-system/). Using these keywords is optional in most cases. If the declaration does not include them, the compiler determines the parameter’s type information automatically. See the [Declaring parameter types](https://www.tradingview.com/pine-script-docs/language/user-defined-functions/#declaring-parameter-types) section to learn more.

TipProgrammers can also place the [method](https://www.tradingview.com/pine-script-reference/v6/#kw_method) keyword immediately before a function’s name to declare the function as a _method_ for a _specific type_. All methods must include at least _one_ parameter, and the first parameter _requires_ a declared type. Refer to the [Methods](https://www.tradingview.com/pine-script-docs/language/methods/) page for more information.

Below is an example of a simple function header. The header declares that the function’s name is `myFunction`, and that the function has two parameters named `param1` and `param2`:

```pine
myFunction(param1, param2) =>
```

Note that:

-   Neither parameter has a _default_ argument. Therefore, the script must supply arguments to these parameters in _every_ `myFunction()` call.
-   Because the function’s parameters do not include [type keywords](https://www.tradingview.com/pine-script-docs/language/user-defined-functions/#type-keywords), they inherit the _same_ qualified type as their argument in each separate call.

The following two sections explain the body structure of single-line and multiline functions.

### Single-line functions {#single-line-functions}

A single-line function’s body begins and ends on the _same_ line of code as the header. This format is convenient for defining compact functions that execute only simple statements and do not use [conditional structures](https://www.tradingview.com/pine-script-docs/language/conditional-structures/) or [loops](https://www.tradingview.com/pine-script-docs/language/loops/). The syntax to define a single-line function is as follows:

```
<functionHeader> => {statement, }<returnExpression>
```

Where:

-   `functionHeader` declares the function’s _name_ and _parameters_, as explained in the [previous section](https://www.tradingview.com/pine-script-docs/language/user-defined-functions/#structure-and-syntax).
-   `statement`, in curly brackets, represents zero or more statements or expressions that the function evaluates _before_ returning a result. The function must separate all individual statements in its body with _commas_.
-   `returnExpression` is the _final_ expression, variable, or [tuple](https://www.tradingview.com/pine-script-docs/language/type-system/#tuples) in the function’s body. Each function call _returns_ the result of evaluating this code.

The following example defines an `add()` function in single-line format. The function includes two parameters named `val1` and `val2`. The body of the function contains a single [+](https://www.tradingview.com/pine-script-reference/v6/#op_+) operation that _adds_ or [concatenates](https://www.tradingview.com/pine-script-docs/concepts/strings/#concatenation) the parameter values, depending on their [types](https://www.tradingview.com/pine-script-docs/language/type-system/#types). Each call to the function returns the result of that operation:

```pine
add(val1, val2) => val1 + val2
```

A script that includes this function definition can call `add()` with different arguments for `val1` and `val2`. The type of value returned by each call depends on these arguments. For example, the script below executes a few calls to `add()`, then passes their results to the `series`, `title`, and `linewidth` parameters in a call to [plot()](https://www.tradingview.com/pine-script-reference/v6/#fun_plot):

![image](https://www.tradingview.com/pine-script-docs/_astro/User-defined-functions-Structure-and-syntax-Single-line-functions-1.ZUSKhP6e_1lYyw4.webp)

```pine
//@version=6
indicator("Simple single-line function demo")

add(val1, val2) => val1 + val2

float  a = add(open, close)              // `open + close`  ("series float")
int    b = add(bar_index, 1)             // `bar_index + 1` ("series int")
int    c = add(2, 3)                     //  5              ("const int")
string d = add(add("Test", " "), "plot") // `"Test plot"`   ("const string")

// Create a plot using the `a`, `b`, `c`, and `d` values.
plot(a / b, title = d, linewidth = c)
```

Note that:

-   The `val1` and `val2` parameters automatically _inherit_ the qualified types of their arguments in each `add()` call, because the function’s header does not [declare their types](https://www.tradingview.com/pine-script-docs/language/user-defined-functions/#declaring-parameter-types) using type and qualifier keywords.
-   Although the parameters accept arguments of any type, except for [void](https://www.tradingview.com/pine-script-docs/language/type-system/#void), an `add()` function call compiles successfully only if the arguments have types that are _compatible_ with the [+](https://www.tradingview.com/pine-script-reference/v6/#op_+) operator, such as “int”, “float”, or “string”. As shown above, if the arguments are numbers (“int” or “float” values), the function performs _addition_. If they are strings, the function performs _concatenation_.
-   The value returned by each `add()` call inherits the _strongest_ [type qualifier](https://www.tradingview.com/pine-script-docs/language/type-system/#qualifiers) used in the calculation. The first two calls return “series” results because both use at least one “series” value. In contrast, the other calls return “const” results, because all values in their calculations are _constants_.

The body of a single-line function can contain a _comma-separated list_ of statements and expressions. Each call to the function evaluates the list from left to right, treating each item as a separate line of code. The call returns the result of evaluating the _final_ expression or statement in the list.

For example, the following script contains a `zScore()` function defined in single-line format. The function computes the [z-score](https://en.wikipedia.org/wiki/Standard_score) of a `source` series over `length` bars. Its body declares two variables, `mean` and `sd`, to hold the average and standard deviation of the series. The final expression in the body uses these variables to calculate the function’s returned value. On each bar, the script calls the `zScore()` function using [close](https://www.tradingview.com/pine-script-reference/v6/#var_close) as the `source` argument and 20 as the `length` argument, then plots the result:

![image](https://www.tradingview.com/pine-script-docs/_astro/User-defined-functions-Structure-and-syntax-Single-line-functions-2.BetlAr8S_Z1NQuye.webp)

```pine
//@version=6
indicator("Single-line function with more than one statement demo")

//@function Calculates the z-score of a `source` series over `length` bars.
zScore(float source, int length) => mean = ta.sma(source, length), sd = ta.stdev(source, length), (source - mean) / sd

//@variable The 20-bar z-score of `close` values.
float osc = zScore(close, 20)

// Plot the `osc` series as color-coded columns.
plot(osc, "Z-score", osc > 0 ? color.green : color.red, style = plot.style_columns)
```

Note that:

-   The `source` parameter requires an “int” or “float” value because its declaration includes the [float](https://www.tradingview.com/pine-script-reference/v6/#type_float) keyword. The `length` parameter requires an “int” value because its declaration uses the [int](https://www.tradingview.com/pine-script-reference/v6/#fun_int) keyword. See the [Declaring parameter types](https://www.tradingview.com/pine-script-docs/language/user-defined-functions/#declaring-parameter-types) section to learn more.
-   The `//@variable` and `//@function` comments are [annotations](https://www.tradingview.com/pine-script-docs/language/script-structure/#compiler-annotations) that document identifiers in the code. The `//@function` annotation provides documentation for the `zScore()` function. Users can hover over the function’s name in the Pine Editor, or write a function call, to view the annotation’s formatted text in a pop-up window. See the [Documenting functions](https://www.tradingview.com/pine-script-docs/language/user-defined-functions/#documenting-functions) section for more information.

TipAlthough it is possible to write single-line functions containing multiple statements, as shown above, using the [multiline](https://www.tradingview.com/pine-script-docs/language/user-defined-functions/#multiline-functions) format is often preferred for readability. Programmers can list the statements on separate lines and add comments for each one.

### Multiline functions {#multiline-functions}

A multiline function defines its body using a _block_ of code following the header line. The general syntax is as follows:

```
<functionHeader> =>
    [statements]
    …
    <returnExpression>
```

Where:

-   `functionHeader` declares the function’s _name_ and _parameters_. See the [Structure and syntax](https://www.tradingview.com/pine-script-docs/language/user-defined-functions/#structure-and-syntax) section above to learn the syntax for function headers.
-   `statements` is an optional block of statements and expressions that the function evaluates _before_ returning a result. Single lines in the body can also contain _multiple_ statements separated by commas.
-   `returnExpression` is the _final_ statement, expression, variable, or tuple at the end of the body. Each call to the function returns the result of this code. If the end of the function’s body is a comma-separated list of statements or expressions, `returnExpression` is the code at the end of that list. If the final statement is a [conditional structure](https://www.tradingview.com/pine-script-docs/language/conditional-structures/) or [loop](https://www.tradingview.com/pine-script-docs/language/loops/), the function returns that structure’s result.
-   Each line of code following the function’s header has an indentation of _four spaces_ or a tab, signifying that it belongs to the function’s scope. The function’s body ends on the final indented line; all non-indented code following the definition belongs to the _global_ scope.

The multiline format is well-suited for defining functions that perform multiple tasks. Programmers can organize all the function’s statements across different lines and document them with comments for readability.

In the following example, we modified the second script from the [Single-line functions](https://www.tradingview.com/pine-script-docs/language/user-defined-functions/#single-line-functions) section to define an equivalent `zScore()` function using the multiline format. The function’s [variable declarations](https://www.tradingview.com/pine-script-docs/language/variable-declarations/) and return expression are now on _separate_ indented lines, and we’ve added comments inside the body to describe the function’s calculations:

```pine
//@version=6
indicator("Multiline function demo")

//@function Calculates the z-score of a `source` series over `length` bars.
zScore(float source, int length) =>
    // Calculate the mean and standard deviation of the series.
    float mean = ta.sma(source, length)
    float sd   = ta.stdev(source, length)
    // Compute the z-score using the `mean` and `sd` values, and return the result.
    (source - mean) / sd

//@variable The 20-bar z-score of `close` values.
float osc = zScore(close, 20)

// Plot the `osc` series as color-coded columns.
plot(osc, "Z-score", osc > 0 ? color.green : color.red, style = plot.style_columns)
```

Note that:

-   We added the [float](https://www.tradingview.com/pine-script-reference/v6/#type_float) keyword to the `mean` and `sd` declarations to declare their [types](https://www.tradingview.com/pine-script-docs/language/type-system/#types) in the code. The keyword is _optional_ in both variable declarations because the compiler can determine the correct types from the assigned values, but including it helps promote readability.

Programmers often define multiline functions to encapsulate complex or logical tasks involving [loops](https://www.tradingview.com/pine-script-docs/language/loops/) or [conditional structures](https://www.tradingview.com/pine-script-docs/language/conditional-structures/). If the final part of a function’s body contains one of these structures, a call to the function returns the result of evaluating that structure.

For example, the `smoothMedian()` function in the script below calculates the median of a `source` series over `length` bars, then smooths the result using a moving average specified by the `avgType` parameter. The function compares the `avgType` value in a [switch](https://www.tradingview.com/pine-script-reference/v6/#kw_switch) statement to select the type of average that it returns. The script calls the function to calculate the median of [close](https://www.tradingview.com/pine-script-reference/v6/#var_close) values over 10 bars, smoothed by an [EMA](https://www.tradingview.com/support/solutions/43000592270-exponential-moving-average/) with the same length, and then plots the result on the chart:

![image](https://www.tradingview.com/pine-script-docs/_astro/User-defined-functions-Structure-and-syntax-Multiline-functions-1.BxkB4CE1_HT9WV.webp)

```pine
//@version=6
indicator("Multiline function with conditional structure demo", overlay = true, behind_chart = false)

//@function Calculates the median of `source` over `length` bars, then returns a moving average of the median.
smoothMedian(float source, int length, string avgType = "ema") =>

    //@variable The median of the `source` series.
    float median = ta.median(source, length)

    // Calculate the EMA, SMA, and WMA of `median`.
    float ema = ta.ema(median, length)
    float sma = ta.sma(median, length)
    float wma = ta.wma(median, length)

    // Return `ema`, `sma`, or `wma`, depending on the `avgType` value.
    switch avgType
        "ema" => ema
        "sma" => sma
        "wma" => wma

//@variable The EMA of the 10-bar median of `close` values.
float medSmooth = smoothMedian(close, 10)

// Plot the `medSmooth` series as a color-coded line.
plot(medSmooth, "Smoothed median", close > medSmooth ? color.teal : color.maroon, 3)
```

Note that:

-   The `smoothMedian()` call in this example does not supply an argument to the `avgType` parameter. Specifying an argument is _optional_, because the parameter declaration includes a _default argument_ (`"ema"`).
-   We included empty lines between each section for readability. The lines that follow an empty line are considered part of the function as long as they are indented correctly.

## Functions that return multiple results {#functions-that-return-multiple-results}

Sometimes, a function must perform multiple calculations and return separate results for each one. Programmers can write functions that return _multiple_ values or references by using a [tuple](https://www.tradingview.com/pine-script-docs/language/type-system/#tuples) as the _final statement_ in the function’s body. A tuple is a comma-separated list of expressions enclosed in square brackets (`[]`). Every call to a function with a tuple at the end of its body returns the result of evaluating each listed expression in order.

For example, the `sumDiff()` function below calculates both the sum and difference of the values passed to its `val1` and `val2` parameters. Its body consists of a two-item tuple containing the addition and subtraction [operations](https://www.tradingview.com/pine-script-docs/language/operators/):

```pine
//@function Returns a tuple containing the sum and difference between the `val1` and `val2` values, respectively.
sumDiff(float val1, float val2) =>
    [val1 + val2, val1 - val2]
```

Note that:

-   The `val1` and `val2` parameters include the [float](https://www.tradingview.com/pine-script-reference/v6/#type_float) keyword in their declarations. Therefore, both parameters are of the type “float”, and they accept only “float” or “int” arguments. See the [Declaring parameter types](https://www.tradingview.com/pine-script-docs/language/user-defined-functions/#declaring-parameter-types) section for more information.

Because the end of the function’s body is a two-item tuple, each call to the function always returns two separate values. To assign the function’s results to variables, the script must use a [tuple declaration](https://www.tradingview.com/pine-script-docs/language/variable-declarations/#tuple-declarations) containing one _new_ variable for each returned item. It is _not_ possible to reassign existing tuples or to use previously declared variables within them.

The following example calls `sumDiff()` to calculate the sum and difference between two pseudorandom values. The script uses a tuple declaration containing two variables, `sum` and `diff`, to store the values returned by the call. Then, it plots the values of those variables on the chart:

![image](https://www.tradingview.com/pine-script-docs/_astro/User-defined-functions-Functions-that-return-multiple-results-1.CVZCb-PO_1728g7.webp)

```pine
//@version=6
indicator("Functions that return multiple results demo")

//@function Returns a tuple containing the sum and difference between the `val1` and `val2` values, respectively.
sumDiff(float val1, float val2) =>
    [val1 + val2, val1 - val2]

// Call `sumDiff()` with two pseudorandom arguments, and use a tuple declaration with one variable for each separate result.
[sum, diff] = sumDiff(math.random(), math.random())

// Plot the `sum` and `diff` series.
plot(sum,  "Sum",        color.teal,   3)
plot(diff, "Difference", color.maroon, 3)
```

In some cases, a script might not require _all_ the results from the tuple returned by a function call. Instead of writing unique identifiers for every variable in the tuple declaration, programmers can [use an underscore](https://www.tradingview.com/pine-script-docs/language/variable-declarations/#using-an-underscore-as-an-identifier) as the identifier for each variable that the script does not require. All variables with the name `_` are _not usable_ in the script’s calculations.

For example, if we change `sum` to `_` in the previous script’s tuple declaration, the first value returned by the `sumDiff()` call is _inaccessible_ to the script. Attempting to use the identifier elsewhere in the code causes a _compilation error_, as shown by the first [plot()](https://www.tradingview.com/pine-script-reference/v6/#fun_plot) call in the script version below:

```pine
//@version=6
indicator("Using `_` in a tuple declaration demo")

//@function Returns a tuple containing the sum and difference between the `val1` and `val2` values, respectively.
sumDiff(float val1, float val2) =>
    [val1 + val2, val1 - val2]

// Declare a tuple with `_` as the first identifier, meaning the first item returned by `sumDiff()` (the sum) is
// inaccessible to the script.
[_, diff] = sumDiff(math.random(), math.random())

// The first `plot()` call now causes an error, because all `_` variables are *not usable* in a script's calculations.
plot(_,    "Sum",        color.teal,   3)
plot(diff, "Difference", color.maroon, 3)
```

NoteThe items in a function’s returned tuple can have _different_ [types](https://www.tradingview.com/pine-script-docs/language/type-system/#types). However, functions _cannot_ return multiple results with different [type qualifiers](https://www.tradingview.com/pine-script-docs/language/type-system/#qualifiers). Therefore, _all_ items in a returned tuple automatically inherit the _same qualifier_. If the tuple does not rely on a “series” value, all the items in the tuple typically inherit the _“simple”_ qualifier. Otherwise, all items inherit the _“series”_ qualifier. See the [Tuples](https://www.tradingview.com/pine-script-docs/language/type-system/#tuples) section of the [Type system](https://www.tradingview.com/pine-script-docs/language/type-system/) page for an example.

## Declaring parameter types {#declaring-parameter-types}

User-defined function definitions can prefix each parameter declaration with type and qualifier _keywords_, enabling strict control over the qualified types that a script can pass to the parameter in any function call. If a declaration does not include these keywords, the compiler automatically determines the parameter’s qualified type based on its arguments and the function’s structure.

The sections below explain how type and qualifier keywords affect the behavior of function parameters. For detailed information about Pine’s types and qualifiers, refer to the [Type system](https://www.tradingview.com/pine-script-docs/language/type-system/) page.

### Type keywords {#type-keywords}

Parameter declarations prefixed by _type keywords_ — such as [int](https://www.tradingview.com/pine-script-reference/v6/#fun_int), [float](https://www.tradingview.com/pine-script-reference/v6/#type_float), [string](https://www.tradingview.com/pine-script-reference/v6/#type_string), or [label](https://www.tradingview.com/pine-script-reference/v6/#type_label) — declare the [types](https://www.tradingview.com/pine-script-docs/language/type-system/#types) of data that the parameters represent in any function call. If a parameter declaration includes a type keyword, it accepts only arguments of that type, or arguments that Pine can automatically [cast](https://www.tradingview.com/pine-script-docs/language/type-system/#type-casting) to that type.

If a function parameter does _not_ have a type keyword in its declaration, its type is initially _undefined_. In each separate call to the function, the parameter automatically inherits the _same_ type as its specified argument. In other words, the parameter can take on _any type_, except for [void](https://www.tradingview.com/pine-script-docs/language/type-system/#void), depending on the function call.

The following example demonstrates this behavior. The user-defined `pass()` function in the script below returns the value of the `source` parameter without performing additional calculations. The parameter’s declaration does not include a type keyword. The script executes five calls to the function with different argument types and then uses their results in code that accepts those types. This script compiles successfully, because each call’s version of the `source` parameter inherits its argument’s type:

![image](https://www.tradingview.com/pine-script-docs/_astro/User-defined-functions-Declaring-parameter-types-Type-keywords-1.zDkkXyfc_ZDD8OY.webp)

```pine
//@version=6
indicator("Undefined parameter types demo")

//@function Returns the value of the `source` argument without modification.
//          Each written call to the function can accept an argument of *any* type except for "void".
pass(source) =>
    source

plotSeries  = pass(bar_index)                         // "series int"
lineWidth   = pass(3)                                 // "const int"
plotTitle   = pass("Test plot")                       // "const string"
plotColor   = pass(chart.fg_color)                    // "input color"
plotDisplay = pass(display.all - display.status_line) // "const plot_display"

// Create a plot using the values from the five `pass()` calls.
plot(plotSeries, plotTitle, plotColor, lineWidth, display = plotDisplay)
```

We can restrict the `source` parameter’s type, and thus the arguments it can accept, by including a type keyword in its declaration. For example, in the modified script below, we added the [int](https://www.tradingview.com/pine-script-reference/v6/#fun_int) keyword to the declaration to specify that the parameter’s type is “int”. With this change, the last three `pass()` calls now cause a compilation error, because the parameter no longer allows “string”, “color”, or “plot\_display” arguments:

```pine
//@version=6
indicator("Declared parameter type demo")

//@function Returns the value of the `source` argument.
//          The argument in each call must be an integer. No other type is allowed.
pass(int source) =>
    source

// These two calls are valid, because their arguments are "int" values.
plotSeries = pass(bar_index)
lineWidth  = pass(3)

// These three calls cause an error, because the `pass()` function no longer allows non-integer arguments.
plotTitle   = pass("Test plot")
plotColor   = pass(chart.fg_color)
plotDisplay = pass(display.all - display.status_line)

// Create a plot using the values from the five `pass()` calls.
plot(plotSeries, plotTitle, plotColor, lineWidth, display = plotDisplay)
```

Note that:

-   If a parameter declaration includes a type keyword _without_ a qualifier keyword, such as [const](https://www.tradingview.com/pine-script-reference/v6/#type_const) or [simple](https://www.tradingview.com/pine-script-reference/v6/#type_simple), the compiler automatically sets the qualifier to _“series”_ or _“simple”_, depending on the function’s calculations. In this example, the `source` parameter’s type is “series int”, because the function’s logic does not require a “simple” or weaker qualifier. See the [Qualifier keywords](https://www.tradingview.com/pine-script-docs/language/user-defined-functions/#qualifier-keywords) section to learn more.

In most cases, type keywords are _optional_ in parameter declarations. However, specifying a parameter’s type is _required_ if:

-   The parameter’s default argument is an [`na` value](language/type-system/#na-value).
-   The parameter declaration includes a _qualifier keyword_.
-   The function definition includes the [export](https://www.tradingview.com/pine-script-reference/v6/#kw_export) keyword. Exported functions require declared types for _every_ parameter. See the [Libraries](https://www.tradingview.com/pine-script-docs/concepts/libraries/) page to learn more.
-   The function definition includes the [method](https://www.tradingview.com/pine-script-reference/v6/#kw_method) keyword, and the parameter is the _first_ one listed in the header. See the [User-defined methods](https://www.tradingview.com/pine-script-docs/language/methods/#user-defined-methods) section of the [Methods](https://www.tradingview.com/pine-script-docs/language/methods/) page for more information.

TipEven when not required, we recommend declaring parameter types where possible. Type keywords help promote readability, and they enable the Pine Editor to provide relevant code suggestions. Additionally, parameters with declared types help prevent _unintended_ arguments in function calls.

### Qualifier keywords {#qualifier-keywords}

A _qualifier keyword_ ([const](https://www.tradingview.com/pine-script-reference/v6/#type_const), [simple](https://www.tradingview.com/pine-script-reference/v6/#type_simple), or [series](https://www.tradingview.com/pine-script-reference/v6/#type_series)) preceding a [type keyword](https://www.tradingview.com/pine-script-docs/language/user-defined-functions/#type-keywords) in a parameter declaration specifies the parameter’s [type qualifier](https://www.tradingview.com/pine-script-docs/language/type-system/#qualifiers). The keyword also indicates when the _argument_ for the parameter must be accessible, and whether that argument can _change_ across bars:

`const`

The parameter has the “const” qualifier. Its argument must be available at _compile time_, and that argument cannot change during any execution. Only the parameters of _non-exported_ functions can use the [const](https://www.tradingview.com/pine-script-reference/v6/#type_const) keyword.

`simple`

The parameter has the “simple” qualifier. Its argument must be a “simple” value or a value with a _weaker_ qualifier (“input” or “const”). The value cannot change after the _first bar_.

`series`

The parameter has the “series” qualifier. It can accept an argument with _any_ type qualifier, because “series” is the _highest_ one in Pine’s [qualifier hierarchy](https://www.tradingview.com/pine-script-docs/language/type-system/#qualifiers). The argument for the parameter in each function call _can change_ on any execution.

NoteQualifier keywords affect only the parameters that accept [value types](https://www.tradingview.com/pine-script-docs/language/type-system/#value-types). They **do not** affect those that accept [reference types](https://www.tradingview.com/pine-script-docs/language/type-system/#reference-types). Instances of reference types _always_ have the “series” qualifier, regardless of how the script uses them. Therefore, all parameters of these types automatically inherit the “series” qualifier. See the [Type system](https://www.tradingview.com/pine-script-docs/language/type-system/) page to learn more.

Qualifier keywords are always _optional_ in parameter declarations. If a declaration does not include a qualifier keyword, the compiler uses the following logic to assign a qualifier to the parameter:

-   If the declaration does _not_ include a type keyword, the compiler assigns the parameter the _same_ qualifier _and_ type as its argument in each written function call. For example, if the script passes a “const int” argument in one function call, the parameter of that call inherits the type “const int”. If it uses a “series float” value in another call to the function, the parameter of that call inherits the type “series float”.
-   If the declaration _does_ include a type keyword, the compiler first tests whether the parameter can use the _“series”_ qualifier. If the function contains a call to another function in its body, and that call _cannot_ accept a “series” value, the compiler then checks whether the _“simple”_ qualifier works. If neither the “series” nor “simple” qualifier is compatible with the function’s calculations, a _compilation error_ occurs.

To demonstrate these behaviors, let’s revisit the `pass()` function from the [Type keywords](https://www.tradingview.com/pine-script-docs/language/user-defined-functions/#type-keywords) section. The function returns the value of its `source` parameter without additional calculations. If the parameter does not include _any_ keywords in its declaration, its qualified type is that of its _argument_ in each separate function call:

```pine
//@function Returns the value of the `source` argument without modification.
//          Each written call to the function can accept an argument of *any* type except for "void".
pass(source) =>
    source
```

The script below calls `pass()` using an “int” value with the “const” qualifier, then uses the returned value as the `length` argument in a call to [ta.ema()](https://www.tradingview.com/pine-script-reference/v6/#fun_ta.ema) and plots the result. This script compiles successfully because our `pass()` call returns the same qualified type as its argument (“const int”), and the `length` parameter of [ta.ema()](https://www.tradingview.com/pine-script-reference/v6/#fun_ta.ema) can accept a value of that type:

![image](https://www.tradingview.com/pine-script-docs/_astro/User-defined-functions-Declaring-parameter-types-Qualifier-keywords-1.UmWjQFYB_2kXXpm.webp)

```pine
//@version=6
indicator("Qualifier inheritance demo")

//@function Returns the value of the `source` argument without modification.
//          Each written call to the function can accept an argument of *any* type except for "void".
pass(source) =>
    source

//@variable The EMA smoothing length.
//          This `pass()` call's `source` parameter automatically inherits the "const" qualifier from its argument.
//          Therefore, the returned type is "const int".
int lengthVal = pass(14)

//@variable The EMA of `close - open`.
//          This call works as expected, because the `length` parameter of `ta.ema()` can accept "int" values with
//          "simple" or weaker qualifiers.
float emaDiff = ta.ema(close - open, length = lengthVal)

// Plot the `emaDiff` series.
plot(emaDiff, "Smoothed difference", color.purple, 3)
```

If we add [int](https://www.tradingview.com/pine-script-reference/v6/#fun_int) to the `source` declaration, the parameter then requires an “int” value, but it **does not** directly inherit the _same_ type qualifier as its argument. Instead, the compiler first checks if it can assign _“series”_ to the parameter, then tries using _“simple”_ if “series” does not work.

Our `pass()` function does not use the `source` parameter in any local function calls that require a “simple int” value, so the compiler sets its qualifier to **“series”**. Consequently, the function’s returned type is always _“series int”_, even if the `source` argument is a “const” value. Adding this change to the previous script thus causes a _compilation error_, because the `length` parameter of [ta.ema()](https://www.tradingview.com/pine-script-reference/v6/#fun_ta.ema) cannot accept a “series” argument; only “simple” or weaker qualifiers are allowed:

```pine
//@version=6
indicator("Qualifier promotion demo")

//@function Returns the value of the `source` parameter. Requires an integer.
pass(int source) =>
    source

//@variable The EMA smoothing length.
//          This `pass()` call's `source` parameter **does not** inherit the "const" qualifier.
//          Its type is promoted to "series int".
int lengthVal = pass(14)

// This call causes a *compilation error*. The `length` parameter cannot accept a "series int" value.
float emaDiff = ta.ema(close - open, length = lengthVal)

// Plot the `emaDiff` series.
plot(emaDiff, "Smoothed difference", color.purple, 3)
```

We can restrict the type qualifier of our function’s `source` parameter by adding a _qualifier keyword_ to its declaration. In the script version below, we prefixed the declaration with the [simple](https://www.tradingview.com/pine-script-reference/v6/#type_simple) keyword. Now, the `source` parameter’s type is _“simple int”_ instead of “series int”. With this change, the script _does not_ cause an error, because the `pass()` function’s returned type is now compatible with the `length` parameter of [ta.ema()](https://www.tradingview.com/pine-script-reference/v6/#fun_ta.ema):

![image](https://www.tradingview.com/pine-script-docs/_astro/User-defined-functions-Declaring-parameter-types-Qualifier-keywords-2.3l-V8A-N_Z1fiiVi.webp)

```pine
//@version=6
indicator("Declared parameter qualifier demo")

//@function Returns the value of the `source` argument. Requires an "int" value with
//          "simple" or a weaker qualifier.
pass(simple int source) =>
    source

//@variable The EMA smoothing length.
//          This `pass()` call's result is a "simple int" value, because the function definition explicitly
//          declares that the `source` parameter's qualifier is "simple".
int lengthVal = pass(14)

// This call does not cause an error. The qualified type of `lengthVal` matches the `length` parameter's type.
float emaDiff = ta.ema(close - open, length = lengthVal)

// Plot the `emaDiff` series.
plot(emaDiff, "Smoothed difference", color.teal, 3)
```

For some types of calculations, using “series” values might not cause a compilation error, but if the values do not remain consistent across all bars, the calculations produce incorrect or unintended results. When wrapping such calculations in a function, declaring the relevant parameters with the [simple](https://www.tradingview.com/pine-script-reference/v6/#type_simple) keyword — or [const](https://www.tradingview.com/pine-script-reference/v6/#type_const) when appropriate — ensures that only _unchanging_ arguments are allowed in each call, preventing unintended behavior.

Let’s look at an example. The following `calcAvg()` function calculates a moving average of a `source` series over `length` bars. The function compares the value of its `avgType` parameter in a [switch](https://www.tradingview.com/pine-script-reference/v6/#kw_switch) statement to select a built-in `ta.*()` call to use for the average calculation:

```pine
//@function Calculates a moving average of a `source` series over `length` bars.
//          The selected average calculation depends on the `avgType` value.
calcAvg(float source, int length, string avgType) =>
    switch avgType
        "ema" => ta.ema(source, length)
        "sma" => ta.sma(source, length)
        "wma" => ta.wma(source, length)
        "hma" => ta.hma(source, length)
```

The compiler raises a _warning_ about this function’s structure inside the Pine Editor, because using `calcAvg()` with a _dynamic_ `avgType` argument can cause _unintended results_. If the `ta.*()` call executed by the function changes on any bar, it affects the _history_ of values used in the average calculations. See the [Time series in scopes](https://www.tradingview.com/pine-script-docs/language/execution-model/#time-series-in-scopes) section of the [Execution model](https://www.tradingview.com/pine-script-docs/language/execution-model/) page for advanced details about this behavior.

The script below executes two `calcAvg()` calls and plots their returned values. The first call consistently uses `"ema"` as its `avgType` argument, and the other alternates between using `"ema"` and `"sma"` as the argument. The second call’s result _does not_ often align with the first call’s result, even on the bars where its `avgType` argument is `"ema"`, because both `ta.*()` calls require _consistent_ evaluation to calculate the averages for _consecutive_ bars:

![image](https://www.tradingview.com/pine-script-docs/_astro/User-defined-functions-Declaring-parameter-types-Qualifier-keywords-3.CmT8Y_eI_2110wI.webp)

```pine
//@version=6
indicator("Inconsistent behavior demo", overlay = true, behind_chart = false)

//@function Calculates a moving average of a `source` series over `length` bars.
//          The selected average calculation depends on the `avgType` value.
calcAvg(float source, int length, string avgType) =>
    switch avgType
        "ema" => ta.ema(source, length)
        "sma" => ta.sma(source, length)
        "wma" => ta.wma(source, length)
        "hma" => ta.hma(source, length)

//@variable The 10-bar EMA of *consecutive* `close` values.
float avg1 = calcAvg(close, 10, "ema")
//@variable The EMA of `close` values from *even* bar indices, or the SMA of values from *odd* bar indices.
//          Either average uses only the values from *every other bar* in its calculation, **not** consecutive bars.
float avg2 = calcAvg(close, 10, bar_index % 2 == 0 ? "ema" : "sma")

// Plot `avg1` and `avg2` for comparison.
plot(avg1, "Consistent EMA", color.blue, 4)
plot(avg2, "Inconsistent EMA/SMA", color.purple, 3)
```

To ensure that any `calcAvg()` call calculates consistent averages without modifying the function’s logic, we can prevent it from using _dynamic_ `avgType` arguments by prefixing the parameter declaration with the [simple](https://www.tradingview.com/pine-script-reference/v6/#type_simple) keyword. With this change, the compiler does _not_ raise a warning about the function — as long as the script evaluates calls to the function on _every bar_ — because any `calcAvg()` call must always use the same `ta.*()` function. Now, if the script attempts to pass a “series string” value to `avgType`, a compilation error occurs:

```pine
//@version=6
indicator("Promoting consistency with qualifier keywords demo", overlay = true, behind_chart = false)

//@function Calculates a moving average of a `source` series over `length` bars.
//          The selected average calculation depends on the `avgType` value.
//          The average type *cannot change* across bars.
calcAvg(float source, int length, simple string avgType) =>
    switch avgType
        "ema" => ta.ema(source, length)
        "sma" => ta.sma(source, length)
        "wma" => ta.wma(source, length)
        "hma" => ta.hma(source, length)

// This `calcAvg()` call works successfully, because the `avgType` argument is a "const string" value.
float avg1 = calcAvg(close, 10, "ema")
// This `calcAvg()` call now causes an error, because a dynamic `avgType` argument is *not allowed*.
float avg2 = calcAvg(close, 10, bar_index % 2 == 0 ? "ema" : "sma")

// Plot `avg1` and `avg2` for comparison.
plot(avg1, "Consistent EMA", color.blue, 4)
plot(avg2, "Inconsistent EMA/SMA", color.purple, 3)
```

## Documenting functions {#documenting-functions}

Pine Script features [annotations](https://www.tradingview.com/pine-script-docs/language/script-structure/#compiler-annotations) that programmers can use to document a function, its parameters, and its result directly in the source code. Annotations are _comments_ that issue special _instructions_ to the compiler or the Pine Editor. The editor can display the formatted text from function annotations in a pop-up window as users work with the function in their code. Additionally, the “Publish script” window uses the text from function annotations to generate default descriptions for [libraries](https://www.tradingview.com/pine-script-docs/concepts/libraries/).

To annotate a function, add comment lines containing valid annotation syntax directly above the function’s header. Each annotation comment must start with `@`, immediately followed by the keyword that indicates its purpose. Below is the syntax for all annotations that apply exclusively to function and [method](https://www.tradingview.com/pine-script-docs/language/methods/#user-defined-methods) definitions:

`//@function <description>`

The [//@function](https://www.tradingview.com/pine-script-reference/v6/#an_@function) annotation defines the _main description_ of the function or method. This annotation is where the programmer documents the function’s purpose and key behaviors. The Pine Editor displays the formatted `description` text in its pop-up window while the user hovers over the function’s name or writes a function call.

`//@param <parameterName> <description>`

The [//@param](https://www.tradingview.com/pine-script-reference/v6/#an_@function) annotation defines the description of a specific function _parameter_. The Pine Editor displays the `description` text beneath the parameter’s type in its pop-up window while the user _writes an argument_ for that parameter in a function call.

In this syntax, `parameterName` is the name of one of the function’s _parameters_. If the annotation does not include a parameter name, or if the specified name does not match one of the listed parameters, the annotation is _ignored_.

`//@returns <description>`

The [//@returns](https://www.tradingview.com/pine-script-reference/v6/#an_@returns) annotation defines the description of the function’s _returned data_. The Pine Editor displays the `description` text at the bottom of the pop-up window that appears while the user hovers over the function’s name.

NoteRedundant annotations are automatically _ignored_. If two or more `//@function` or `//@returns` annotations are above a function header, only the **last** one adds a description to the function or its return expression. If two or more `//@param` annotations above the header share the same parameter name, only the **first** one defines the corresponding parameter’s description.

The following code block defines a `mixEMA()` function, which calculates an EMA of a `source` series and then mixes the EMA with that series by a specified amount (`mix`). Above the function definition, we included `//@function`, `//@param`, and `//@returns` annotations to document its purpose, parameters, and result, respectively. Users can view the formatted text from these annotations by hovering over the `mixEMA` identifier in the Pine Editor or writing a `mixEMA()` function call:

```pine
//@function         Calculates an EMA of a source series, and mixes it with the `source` value.
//@param source     The series of values to process.
//@param length     The length of the EMA's smoothing parameter.
//@param mix        Optional. The mix ratio. Requires a value from 0 to 1. The default is 1.
//@returns          The mixture between the `source` value and its EMA.
mixEMA(float source, int length, float mix = 1.0) =>
    // Raise an error if the `mix` value is outside the supported range.
    if mix < 0 or mix > 1
        runtime.error("The `mix` value must be a number between 0 and 1.")

    // Calculate the EMA of `source`.
    float ma = ta.ema(source, length)
    // Mix the `source` and `ma` values and return the result.
    float result = (1.0 - mix) * source + mix * ma
```

Note that:

-   The pop-up window that shows annotation text automatically _removes_ most leading or repeated whitespaces in its formatted result.

The description attached to any function annotation can occupy _more than one_ comment line in the source code, but only if there are _no blank lines_ between each comment line. For example, the following code block contains `//@function` and `//@param` annotations that both define descriptions across _three_ comment lines:

```pine
//@function         Calculates an EMA of a source series with a specified `length`, and then
//                  computes a mixture between the EMA and the original value based on the `mix` value.
//                  For consistency, calls to this function should execute on every bar.
//@param source     The series of values to process.
//@param length     The length of the EMA's smoothing parameter.
//@param mix        Optional. The mix ratio. Requires a value from 0 to 1. If 0, the result is the `source` value.
//                  If 1, the result is the EMA. If between 0 and 1, the result is a mixture between the two values.
//                  The default is 1.
//@returns          The mixture between the `source` value and its EMA.
mixEMA(float source, int length, float mix = 1.0) =>
    // Raise an error if the `mix` value is out of the supported range.
    if mix < 0 or mix > 1
        runtime.error("The `mix` value must be a number between 0 and 1.")

    // Calculate the EMA of `source`.
    float ma = ta.ema(source, length)
    // Mix the `source` and `ma` values and return the result.
    float result = (1.0 - mix) * source + mix * ma
```

Note that each separate comment line in an annotation block _does not_ typically create a new line of text in the displayed description. Programmers can create annotations with _multiline_ descriptions by adding _empty_ comment lines to the annotation block. For example:

```pine
//@function         Calculates an EMA of a source series with a specified `length`, and then
//                  computes a mixture between the EMA and the original value based on the `mix` value.
//
//                  (**New text line**) For consistency, calls to this function should execute on every bar.
mixEMA(float source, int length, float mix = 1.0) =>
    // Raise an error if the `mix` value is out of the supported range.
    if mix < 0 or mix > 1
        runtime.error("The `mix` value must be a number between 0 and 1.")

    // Calculate the EMA of `source`.
    float ma = ta.ema(source, length)
    // Mix the `source` and `ma` values and return the result.
    float result = (1.0 - mix) * source + mix * ma
```

Annotations support a limited range of [Markdown](https://en.wikipedia.org/wiki/Markdown#Examples) syntax, which enables _custom text formats_ in the Pine Editor’s pop-up window. The example below shows some of the syntax that the window can render. To view the annotation’s results, copy the code and hover over the `f` identifier inside the Pine Editor:

```pine
//@function This annotation shows some common Markdown syntax that the Pine Editor can render in its pop-up window.
//
// ---
//
// `Monospace with gray background`
//
// *Italic text*
//
// **Bold text**
//
// ***Bold and italic***
//
// ~Strikethrough~
//
// ---
//
// > Block quotation
//
// ---
//
// Bulleted list:
// - Item 1
// - Item 2
//
// ---
//
// Numbered list:
// 1. Item 1
// 1. Item 2
//
// ---
//
// ```
// // Code block format
// float x = 1.5
// ```
//
// ---
//
// Hyperlink:
//
// The [@function](https://www.tradingview.com/pine-script-reference/v6/#an_@function) annotation is very flexible.
//
// ---
//
// # Heading 1
// ## Heading 2
// ### Heading 3
//
// ---
f() => int(na)
```

NoteThe annotation syntax in the above example affects only the appearance of text displayed by the Pine Editor’s autosuggest feature; it **does not** affect text formatting in _script publications_. See the [Title and description](https://www.tradingview.com/pine-script-docs/writing/publishing/#title-and-description) section of the [Publishing scripts](https://www.tradingview.com/pine-script-docs/writing/publishing/) page to learn the formatting syntax for publication descriptions.

## Function scopes {#function-scopes}

All variables, expressions, and statements have a designated [scope](https://www.tradingview.com/pine-script-docs/faq/programming/#what-does-scope-mean), which refers to the part of the script where they are defined and accessible. Every script has one _global_ scope and zero or more _local_ scopes.

Each function definition creates a distinct local scope. All code written in the function’s header and body belongs exclusively to that definition; no code outside the function can access its declared variables or parameters.

For example, the following script defines a function named `myFun()`. The function’s body contains a variable named `result`. The script attempts to use a [plot()](https://www.tradingview.com/pine-script-reference/v6/#fun_plot) call, in the global scope, to display the value of that variable. This script causes a compilation error, because the global scope _cannot_ access the function’s _local_ identifiers, and there is no global variable named `result`:

```pine
//@version=6
indicator("Inaccessible scope demo")

//@function Multiplies `x` by a pseudorandom value and returns the result.
myFun(float x = 10) =>
    // This variable belongs to the function definition; other scopes cannot access it.
    float result = math.random() * x

// This `plot()` call causes a compilation error. There is no *global* `result` variable available for plotting.
plot(result)
```

The only way for the script to use the function’s code is to execute a _function call_. Additionally, because the code in the function definition is _inaccessible_ to other scopes, the script can declare separate variables in other parts of the code with the _same_ identifiers as the function’s local variables and parameters. For example, the following script executes a call to the `myFun()` function and assigns its returned value to a new, _global_ variable _also_ named `result`. The [plot()](https://www.tradingview.com/pine-script-reference/v6/#fun_plot) call in this script does not cause an error, because only the global `result` variable is available to that call:

![image](https://www.tradingview.com/pine-script-docs/_astro/User-defined-functions-Function-scopes-1.WDscmOKC_Z1HYWMK.webp)

```pine
//@version=6
indicator("Function variable vs. global variable demo")

//@function Multiplies `x` by a pseudorandom value and returns the result.
myFun(float x = 10) =>
    // This variable belongs to the function definition; other scopes cannot access it.
    float result = math.random() * x

// This variable stores the result returned by a `myFun()` call.
// Although it shares the same name as the variable in `myFun()`, it is a completely separate variable declared
// in the global scope, after the function definition.
float result = myFun()

// This `plot()` call does not cause an error, because `result` refers to the *global* variable.
plot(result, "Global `result` series", linewidth = 3)
```

All local scopes in a script, including the scope of a function definition, are embedded into the script’s global scope. Therefore, while the global scope cannot access any variables within a function’s scope, the function _can_ access any global variables declared _above_ its definition.

For instance, the following script declares a `globalVar` variable before defining the `myFun()` function, then uses that variable in the function’s body. This script compiles successfully, because the function definition has access to any global variables declared before its location in the code:

![image](https://www.tradingview.com/pine-script-docs/_astro/User-defined-functions-Function-scopes-2.CcNLbFZI_Y6NNV.webp)

```pine
//@version=6
indicator("Global variables in functions demo")

//@variable A global variable that holds a pseudorandom value.
var float globalVar = math.random()

//@function Multiplies `x` by a pseudorandom value, adds the value of `globalVar`, then returns the result.
//          This function can use `globalVar` in its body because the script declares the variable *first*.
myFun(float x = 10) =>
    math.random() * x + globalVar

// Assign the result of a default `myFun()` call to a new variable. The call uses `globalVar` internally.
float result = myFun()

// Plot the `result` series.
plot(result, "`result` series", color.teal, 3)
```

It’s important to note that all global variables used in a function’s body cannot change their assigned values or references during any execution of a function call. Therefore, although user-defined functions can _access_ some global variables, they _cannot modify_ those variables using the reassignment or compound assignment [operators](https://www.tradingview.com/pine-script-docs/language/operators/). Similarly, a function cannot reassign its declared parameters, because the parameters of each function call must always hold the same values or references passed to them from the scope where that call occurs.

The example script below demonstrates this limitation. It declares a global variable named `counter` with an initial value of 0. Then, it defines an `increment()` function that uses the [+=](https://www.tradingview.com/pine-script-reference/v6/#op_+=) operator to change the `counter` variable’s assigned value, causing a compilation error:

```pine
//@version=6
indicator("Cannot modify global variables demo")

//@variable A global variable to hold a counter value.
var int counter = 0

//@function Attempts to use `+=` to increment the global `counter` value by the value of `amountToAdd`.
//          This definition causes an error, because global variables cannot be modified
//          during the execution of a user-defined function call.
increment(int amountToAdd = 1) =>
    counter += amountToAdd

plot(increment())
```

TipIt is possible to modify the data accessed by a global variable from inside a function’s scope if that variable is of a [reference type](https://www.tradingview.com/pine-script-docs/language/type-system/#reference-types), such as a [collection type](https://www.tradingview.com/pine-script-docs/language/type-system/#collections) or [user-defined type](https://www.tradingview.com/pine-script-docs/language/type-system/#user-defined-types). Instead of reassigning the variable, the script can use _setter functions_ or _field reassignments_, depending on the type, to modify the _object_ that the variable _refers_ to. For examples of this advanced technique, see the [Extracting data from local scopes](https://www.tradingview.com/pine-script-docs/writing/debugging/#extracting-data-from-local-scopes) section of the [Debugging](https://www.tradingview.com/pine-script-docs/writing/debugging/) page and the scope-related sections of the [Arrays](https://www.tradingview.com/pine-script-docs/language/arrays/), [Matrices](https://www.tradingview.com/pine-script-docs/language/matrices/), and [Maps](https://www.tradingview.com/pine-script-docs/language/maps/) pages.

### Scope of a function call {#scope-of-a-function-call}

A function’s definition acts as a _template_ for each function call. Every call _written_ in the source code establishes a _separate_ local scope using that definition. The parameters, variables, and expressions created for each function call are _unique_ to that call, and they leave _independent_ historical trails in the script’s [time series](https://www.tradingview.com/pine-script-docs/language/execution-model/#time-series). Because each written call has a separate scope, with an independent history, no two calls to the same function directly affect each other.

For example, the script below contains a user-defined function named `accumulate()`. The function’s structure declares a _persistent_ variable named `total` with an initial value of 0, then adds the value of its `source` parameter to that variable and returns the result. The script uses two separate `accumulate()` calls inside a [ternary operation](https://www.tradingview.com/pine-script-docs/language/operators/#-ternary-operator), and then plots the result returned by that expression. The operation triggers the first written call on every _third_ bar, and the second on all other bars. Both calls use a value of 1 as their `source` argument:

```pine
//@version=6
indicator("Independent call scopes demo")

//@function Accumulates the values from a `source` series across executions.
//          Each written call to this function defines a *separate* scope with unique versions of `source` and `total`.
accumulate(float source) =>
    //@variable A *persistent* variable declared on only one bar, initialized with a value of 0.
    var float total = 0.0
    // Add the `source` value to `total` and return the variable's new value.
    total += source

// Use two calls to `accumulate()` with the same `source` argument inside a ternary expression.
// Each call has its own version of `total`, which updates only on bars where the script *evaluates* that call.
// These two calls thus return **different** results, because the script does not evaluate both of them on the
// *same* number of bars.
float altCallSeries = bar_index % 3 == 0 ? accumulate(1) : accumulate(1)

plot(altCallSeries, "Alternating call results", color.purple, 3)
```

Note that:

-   The `total` variable and its assigned value _persist_ across bars because the variable declaration includes the [var](https://www.tradingview.com/pine-script-reference/v6/#kw_var) keyword. See the [Declaration modes](https://www.tradingview.com/pine-script-docs/language/variable-declarations/#declaration-modes) section of the [Variable declarations](https://www.tradingview.com/pine-script-docs/language/variable-declarations/) page to learn more.

Both `accumulate()` calls in this script might seem identical. However, each one has a _separate_ scope with distinct versions of the `source` parameter and the `total` variable. The first call **does not** affect the second, and vice versa. As shown below, the script’s plotted value _decreases_ on every third bar before increasing again on subsequent bars. This behavior occurs because each `accumulate()` call’s version of `total` increases its value only on bars where the script _evaluates_ that call, and the script evaluates the first call on about _half_ as many bars as the second:

![image](https://www.tradingview.com/pine-script-docs/_astro/User-defined-functions-Function-scopes-Scope-of-a-function-call-1.BvL7oJKj_Z1uOD5g.webp)

It is crucial to emphasize that each function call _written_ in the source code has **one** unique scope created from the function’s definition. Repeated evaluations of the same written call **do not** create additional scopes with separate local variables and history. For example, a function call written in the body of a [loop](https://www.tradingview.com/pine-script-docs/language/loops/) performs calculations using the _same_ local series on _every_ iteration; it does _not_ calculate on different versions of those series for each separate iteration.

The following script demonstrates this behavior. The source code includes two written calls to our previous `accumulate()` function. The script evaluates the first call in the global scope, and the second inside the body of a [for](https://www.tradingview.com/pine-script-reference/v6/#kw_for) loop that performs 10 iterations per execution. It then plots the results of both calls in a separate pane:

```pine
//@version=6
indicator("Written call scopes in structures demo")

//@function Accumulates the values from a `source` series across executions.
//          Each written call to this function defines a **separate** scope with unique versions of `source` and `total`.
accumulate(float source) =>
    //@variable A *persistent* variable declared on only one bar, initialized with a value of 0.
    var float total = 0.0
    // Add the `source` value to `total` and return the variable's new value.
    total += source

//@variable Holds the result of evaluating `accumulate(1)` in the global scope. The value increases by *one* on each bar.
float globalCallRes = accumulate(1)

//@variable Initialized with `na`, then reassigned the result of an `accumulate()` call within a loop.
float loopedCallRes = na

for _ = 1 to 10
    // Reassign the `loopedCallRes` variable using the result of `accumulate(1)`.
    // This written call establishes **one** new scope with a unique version of the `total` variable.
    // The script evaluates that scope on every loop iteration, thus adding 1 to the *same* version of `total`
    // *10 times* per bar.
    loopedCallRes := accumulate(1)

// Plot the results of global and looped calls for comparison.
plot(globalCallRes, "Global call result", color.teal, 3)
plot(loopedCallRes, "Looped call result", color.red,  3)
```

A newcomer to Pine might expect the results of both `accumulate()` calls to be equal. However, the result of the call inside the loop is _10 times_ that of the call evaluated in the global scope. This difference occurs because the call written in the loop does not have separate scopes for each iteration; every evaluation of that call modifies the _same_ version of the persistent `total` variable. Consequently, the value returned by the loop’s `accumulate()` call increases by **10** instead of one on each bar:

![image](https://www.tradingview.com/pine-script-docs/_astro/User-defined-functions-Function-scopes-Scope-of-a-function-call-2.BKLDlUMm_Z1uJzyY.webp)

Note that:

-   Both `accumulate()` calls return the _same_ result if we remove [var](https://www.tradingview.com/pine-script-reference/v6/#kw_var) from the `total` variable declaration. Without the keyword, each call’s version of the `total` variable no longer _persists_. Instead, every evaluation of the call _re-declares_ the variable and initializes it to 0 before adding the value of the `source` argument.

## Function overloading {#function-overloading}

Function _overloads_ are _unique versions_ of a function that share the same name but differ in their required [parameter types](https://www.tradingview.com/pine-script-docs/language/user-defined-functions/#declaring-parameter-types). Overloading enables function calls using the same identifier to perform _different tasks_ based on their specified arguments. Programmers often write overloads to group similar calculations for specific types under a single function name, offering a convenient alternative to defining several related functions with unique names.

In Pine Script, a function overload is valid only if its definition satisfies one of the following conditions:

-   It has a different number of _required_ parameters (parameters _without_ default arguments) than that of any other defined overload.
-   It has the same number of required parameters as another overload, but at least _one_ required parameter has a different [qualified type](https://www.tradingview.com/pine-script-docs/language/type-system/) than the one declared at the _same position_ in the other overload’s header.

The following code block defines three overloads of a custom `negate()` function. Each overload has a different parameter type and performs a distinct task. The first overload uses the [\-](https://www.tradingview.com/pine-script-reference/v6/#op_-) operator to negate a “float” or “int” value; the second uses the [not](https://www.tradingview.com/pine-script-reference/v6/#kw_not) operator to negate a “bool” value; and the third calculates the sRGB negative of a “color” value using the built-in `color.*()` functions:

```pine
//@function     (Overload 1) Changes the sign of a number.
//@param value  The "float" or "int" value to process.
//@returns      The negated number.
negate(float value) =>
    -value

//@function     (Overload 2) Negates a Boolean value.
//@param value  The "bool" value to process.
//@returns      `false` if the value is `true`, or `true` if the value is `false`.
negate(bool value) =>
    not value

//@function     (Overload 3) Calculates the sRGB negative (i.e., complement) of a color.
//@param value  The "color" value to process.
//@returns      The "color" value whose red, green, and blue components are opposites of the `value` color's components
//              in the sRGB color space.
negate(color value) =>
    color.rgb(255 - color.r(value), 255 - color.g(value), 255 - color.b(value), color.t(value))
```

Note that:

-   The `value` parameter of each `negate()` overload automatically inherits the _“series”_ qualifier, because its declaration includes a [type keyword](https://www.tradingview.com/pine-script-docs/language/user-defined-functions/#type-keywords) _without_ a [qualifier keyword](https://www.tradingview.com/pine-script-docs/language/user-defined-functions/#qualifier-keywords), and the overload does not use the parameter in a local function call that requires a “simple” or weaker qualifier.
-   We included [annotations](https://www.tradingview.com/pine-script-docs/language/script-structure/#compiler-annotations) to [document](https://www.tradingview.com/pine-script-docs/language/user-defined-functions/#documenting-functions) each separate overload. As a user writes a `negate()` call, the Pine Editor shows the documentation for _one_ of the overloads in a pop-up window. While the window is open, the user can view the documentation for the _other_ overloads by using the Up and Down arrow keys.

With our function overloads defined, we can use `negate()` calls for different type-specific tasks. The script below uses a call to each overload. First, it uses the second overload to calculate the opposite of the condition `close > open`. It then uses the negated condition to determine the value of a plotted series. The plotted value is the result of `negate(close)` if the condition is `true`, and the value of [close](https://www.tradingview.com/pine-script-reference/v6/#var_close) otherwise. The script colors the plot using the negative of the chart’s background color (`negate(chart.bg_color)`):

![image](https://www.tradingview.com/pine-script-docs/_astro/User-defined-functions-Function-overloading-1.BTe80Gur_ZyArF7.webp)

```pine
//@version=6
indicator("Function overloading demo")

//@function     (Overload 1) Changes the sign of a number.
//@param value  The "float" or "int" value to process.
//@returns      The negated number.
negate(float value) =>
    -value

//@function     (Overload 2) Negates a Boolean value.
//@param value  The "bool" value to process.
//@returns      `false` if the value is `true`, or `true` if the value is `false`.
negate(bool value) =>
    not value

//@function     (Overload 3) Calculates the sRGB negative (i.e., complement) of a color.
//@param value  The "color" value to process.
//@returns      The "color" value whose red, green, and blue components are opposites of the `value` color's components
//              in the sRGB color space.
negate(color value) =>
    color.rgb(255 - color.r(value), 255 - color.g(value), 255 - color.b(value), color.t(value))

//@variable `true` if the current `close` value is *not* greater than the value of `open`. Otherwise, `false`.
bool notUpBar = negate(close > open)
//@variable The `-close` value if the `notUpBar` condition is `true`, or the `close` value if the condition is `false`.
float plotSeries = notUpBar ? negate(close) : close
//@variable The sRGB negative of the chart's background color.
color plotColor = negate(chart.bg_color)

// Plot the value of `plotSeries` and color it using `plotColor`.
plot(plotSeries, "Test plot", plotColor, style = plot.style_area)
```

It’s important to emphasize that a function overload is valid only if its signature contains a _unique_ set of qualified types for its _required_ parameters. Differences in _optional_ parameters (the parameters that have default arguments) **do not** affect the validity of overloads. The compiler raises an _error_ if two overloads share the same required parameter types and differ only in optional parameters, because it cannot determine which overload to use for _all_ possible function calls.

For example, the following code block defines a fourth `negate()` overload with two “float” parameters. The overload’s first parameter requires an argument, but the second one _does not_ because it has a default value. With this addition, any `negate()` call with a single “float” argument becomes _ambiguous_. Such a call might refer to the first overload or the fourth, and the compiler _cannot_ confirm which one to use in that case. Therefore, the extra overload causes a compilation error:

```pine
//@function     (Overload 1) Changes the sign of a number.
//@param value  The "float" or "int" value to process.
//@returns      The negated number.
negate(float value) =>
    -value

//@function     (Overload 2) Negates a Boolean value.
//@param value  The "bool" value to process.
//@returns      `false` if the value is `true`, or `true` if the value is `false`.
negate(bool value) =>
    not value

//@function     (Overload 3) Calculates the sRGB negative (i.e., complement) of a color.
//@param value  The "color" value to process.
//@returns      The "color" value whose red, green, and blue components are opposites of the `value` color's components
//              in the sRGB color space.
negate(color value) =>
    color.rgb(255 - color.r(value), 255 - color.g(value), 255 - color.b(value), color.t(value))

// This overload is **invalid**.
// Although the function has two parameters, `scale` has a default argument. The only parameter that requires an
// argument is `value`, and that parameter shares the same type as the required parameter in Overload 1.
// Any `negate()` call with a single "float" argument is *ambiguous* in this case, so the compiler raises an error.
negate(float value, float scale = 1.0) =>
    -value * scale
```

Another crucial limitation to note is that the _names_ of parameters **do not** affect the validity of overloads. The compiler validates overloads by analyzing only the _qualified types_ of their required parameters at each _position_ in the function headers, because function calls can use _positional_ or _named_ arguments for each parameter. Named arguments specify the parameter to which they apply. For example, `f(x = 10)` passes the value 10 to the `x` parameter of the `f()` call. In contrast, positional arguments omit names and apply to the parameters in order: the first argument to the first parameter, the second to the second parameter, and so on.

If two overloads have the same parameter types but different parameter names, a compilation error occurs, because any call that uses positional arguments is ambiguous. For example, the code block below defines a separate `negate()` overload with a single “float” parameter named `comparedValue`. That overload causes a compilation error, as the required parameter’s qualified type matches that of the `value` parameter in the first overload:

```pine
//@function     (Overload 1) Changes the sign of a number.
//@param value  The "float" or "int" value to process.
//@returns      The negated number.
negate(float value) =>
    -value

//@function     (Overload 2) Negates a Boolean value.
//@param value  The "bool" value to process.
//@returns      `false` if the value is `true`, or `true` if the value is `false`.
negate(bool value) =>
    not value

//@function     (Overload 3) Calculates the sRGB negative (i.e., complement) of a color.
//@param value  The "color" value to process.
//@returns      The "color" value whose red, green, and blue components are opposites of the `value` color's components
//              in the sRGB color space.
negate(color value) =>
    color.rgb(255 - color.r(value), 255 - color.g(value), 255 - color.b(value), color.t(value))

// This overload is **invalid**.
// Although the required parameter has a different name from that of Overload 1, it is possible to call the
// function using a *positional* argument. The compiler cannot determine whether calls such as `negate(close)`
// should refer to the first overload or this one, so it raises an error.
negate(float comparedValue) =>
    not (comparedValue > open)
```

In Pine, function overloads can contain calls to overloads with the _same_ function name in their bodies, but only if those overloads are defined first. However, just like non-overloaded functions, an overload _cannot_ use calls to _itself_ within its body.

For example, the script version below defines a fourth `negate()` overload with two required “float” parameters. The new overload calculates the product of its arguments, then calls the _first_ overload of `negate()` to change the result’s sign. This script compiles successfully, because the fourth overload uses a _separate_ `negate()` implementation in its scope, _not_ a call to itself:

![image](https://www.tradingview.com/pine-script-docs/_astro/User-defined-functions-Function-overloading-2.CU45Rhl7_JVeGn.webp)

```pine
//@version=6
indicator("Overloads calling other overloads demo")

//@function      (Overload 1) Changes the sign of a number.
//@param value   The "float" or "int" value to process.
//@returns       The negated number.
negate(float value) =>
    -value

//@function      (Overload 2) Negates a Boolean value.
//@param value   The "bool" value to process.
//@returns       `false` if the value is `true`, or `true` if the value is `false`.
negate(bool value) =>
    not value

//@function      (Overload 3) Calculates the sRGB negative (i.e., complement) of a color.
//@param value   The "color" value to process.
//@returns       The "color" value whose red, green, and blue components are opposites of the `value` color's components
//               in the sRGB color space.
negate(color value) =>
    color.rgb(255 - color.r(value), 255 - color.g(value), 255 - color.b(value), color.t(value))

//@function      (Overload 4) Changes the sign of the product of two numbers.
//@param value1  The first "float" or "int" value to process.
//@param value2  The second value to process.
//@returns       The negated product.
negate(float value1, float value2) =>
    // Using this call in the overload's body does not cause an error, because it is *non-recursive*.
    // It refers to the *first* `negate()` implementation, not the fourth.
    negate(value1 * value2)

//@variable `true` if the current `close` value is *not* greater than the value of `open`. Otherwise, `false`.
bool notUpBar = negate(close > open)
//@variable `-close * 2` if the `notUpBar` condition is `true`, or `close * 2` if the condition is `false`.
float plotSeries = notUpBar ? negate(close, 2) : negate(close, -2)
//@variable The sRGB negative of `color.green`.
color plotColor = negate(color.green)

// Plot the value of `plotSeries` and color it using `plotColor`.
plot(plotSeries, "Test plot", plotColor, style = plot.style_area)
```

Note that:

-   If we change the fourth overload to use a `negate()` call with _two_ “float” arguments, a compilation error occurs because the overload calls _itself_ in that case. Even if Pine allowed recursive functions, using such an overload would cause an error because it creates the equivalent of an _infinite loop_, where a single call activates a second call, the second call activates a third one, and so on.

## Limitations {#limitations}

This section explains several key limitations common to all user-defined functions. These same limitations also apply to [user-defined methods](https://www.tradingview.com/pine-script-docs/language/methods/#user-defined-methods).

### No global-only built-in function calls {#no-global-only-built-in-function-calls}

The body of a user-defined function can contain calls to most other functions or [methods](https://www.tradingview.com/pine-script-docs/language/methods/), or previously defined [overloads](https://www.tradingview.com/pine-script-docs/language/user-defined-functions/#function-overloading) with the same function name. However, a function _cannot_ use calls to the following _built-in_ functions inside its scope:

-   Declaration statements: [indicator()](https://www.tradingview.com/pine-script-reference/v6/#fun_indicator), [strategy()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy), and [library()](https://www.tradingview.com/pine-script-reference/v6/#fun_library).
-   Plot-related functions: [plot()](https://www.tradingview.com/pine-script-reference/v6/#fun_plot), [hline()](https://www.tradingview.com/pine-script-reference/v6/#fun_hline), [fill()](https://www.tradingview.com/pine-script-reference/v6/#fun_fill), [plotshape()](https://www.tradingview.com/pine-script-reference/v6/#fun_plotshape), [plotchar()](https://www.tradingview.com/pine-script-reference/v6/#fun_plotchar), [plotarrow()](https://www.tradingview.com/pine-script-reference/v6/#fun_plotarrow), [plotbar()](https://www.tradingview.com/pine-script-reference/v6/#fun_plotbar), [plotcandle()](https://www.tradingview.com/pine-script-reference/v6/#fun_plotcandle), [barcolor()](https://www.tradingview.com/pine-script-reference/v6/#fun_barcolor), [bgcolor()](https://www.tradingview.com/pine-script-reference/v6/#fun_bgcolor), and [alertcondition()](https://www.tradingview.com/pine-script-reference/v6/#fun_alertcondition).

Global variables can be assigned values returned by functions in order to be used in calls to the above functions. However, the calls to these functions are allowed only in the script’s [global scope](https://www.tradingview.com/pine-script-docs/faq/programming/#what-does-scope-mean), outside the operands of [ternary operations](https://www.tradingview.com/pine-script-docs/language/operators/#-ternary-operator) or other conditional expressions.

### Consistent types for each call {#consistent-types-for-each-call}

Each written call to a user-defined function must have _consistent_ parameter types and return types during each evaluation of that call. The call _cannot_ accept varying argument types throughout the script’s runtime, nor can it change its return type. Similar limitations also apply to other structures and expressions, including [loops](https://www.tradingview.com/pine-script-docs/language/loops/), [conditional structures](https://www.tradingview.com/pine-script-docs/language/conditional-structures/), and ternary operations.

### Cannot modify global variables or parameters {#cannot-modify-global-variables-or-parameters}

The body of a user-defined function cannot use the _reassignment_ or _compound assignment_ [operators](https://www.tradingview.com/pine-script-docs/language/operators/) to modify variables declared in the _global scope_. Likewise, it cannot use these operators to reassign function _parameters_. The value or reference assigned to a global variable or the parameter of a function call _cannot change_ while the script evaluates the call. See the [Function scopes](https://www.tradingview.com/pine-script-docs/language/user-defined-functions/#function-scopes) section for an example.

### No nested definitions {#no-nested-definitions}

The body of a user-defined function _cannot_ contain the definition of another user-defined function. Likewise, [loops](https://www.tradingview.com/pine-script-docs/language/loops/) and [conditional structures](https://www.tradingview.com/pine-script-docs/language/conditional-structures/) cannot include function definitions in their local blocks. Function and [method](https://www.tradingview.com/pine-script-docs/language/methods/#user-defined-methods) definitions are allowed only in the script’s _global scope_.

For example, the following `f()` function definition causes a compilation error, because it attempts to define another function, `g()`, within its body:

```pine
//@version=6
indicator("Invalid nested definition demo")

f(x, y) =>
    // Defining a separate function inside the body of `f()` is not allowed.
    g(a, b) => math.sqrt(a * a + b * b)
    g(x, y) / (x + y)

plot(f(open, close))
```

Instead of attempting to define `g()` within the scope of `f()`, we can move the definition of `g()` above `f()` in the global scope:

```pine
//@version=6
indicator("Moving nested definitions to the global scope demo")

// If we move `g()` to the global scope, above `f()`, the `f()` function can use it, and no error occurs.
g(a, b) => math.sqrt(a * a + b * b)

f(x, y) =>
    g(x, y) / (x + y)

plot(f(open, close))
```

### No recursive functions {#no-recursive-functions}

In contrast to functions in some programming languages, user-defined functions in Pine Script _cannot_ contain calls to _themselves_ within their bodies. Instead of using recursive function structures, programmers can replace those structures with equivalent [loop](https://www.tradingview.com/pine-script-docs/language/loops/) calculations.

For instance, the following `gcd()` function causes a compilation error because it includes a `gcd()` call within its body:

```pine
//@version=6
indicator("Invalid recursive function demo")

gcd(int x, int y) =>
    // This expression causes an error. `gcd()` cannot call itself.
    y == 0 ? x : gcd(y, x % y)

plot(gcd(15, 20))
```

Instead of attempting to use recursion, we can reformat the function’s structure to perform the necessary calculations using _iteration_. The example version below achieves our intended calculations using a [while](https://www.tradingview.com/pine-script-reference/v6/#kw_while) loop and no recursive `gcd()` calls:

```pine
//@version=6
indicator("Replacing recursion with loops demo")

// This function performs the same intended calculations as the invalid recursive structure.
// It uses separate local variables to track values, and modifies those variables in a `while` loop
// until it finds the greatest common divisor for two positive integers.
gcd(int x, int y) =>
    int a = x
    int b = y
    while b != 0
        int remainder = a % b
        a := b
        b := remainder
    a

plot(gcd(15, 20))
```
