---
title: Conditional structures
source: https://www.tradingview.com/pine-script-docs/language/conditional-structures/
section: language
---

# Conditional structures {#conditional-structures}

## Introduction {#introduction}

The conditional structures in Pine Script® are [if](https://www.tradingview.com/pine-script-reference/v6/#kw_if), [switch](https://www.tradingview.com/pine-script-reference/v6/#kw_switch), and [once](https://www.tradingview.com/pine-script-reference/v6/#kw_once).

Scripts can use all of these structures for their side effects, i.e., the actions they perform, like reassigning values to variables, or calling functions. The [if](https://www.tradingview.com/pine-script-reference/v6/#kw_if) and [switch](https://www.tradingview.com/pine-script-reference/v6/#kw_switch) structures can also return a value or a tuple which can then be assigned to a variable (or multiple variables in the case of tuples). The [once](https://www.tradingview.com/pine-script-reference/v6/#kw_once) structure cannot return a value.

Scripts can use an [if](https://www.tradingview.com/pine-script-reference/v6/#kw_if), [switch](https://www.tradingview.com/pine-script-reference/v6/#kw_switch), or [once](https://www.tradingview.com/pine-script-reference/v6/#kw_once) structure inside the local scope of a loop, function, conditional structure, or other structure.

Some Pine Script built-in functions are **not** callable from within the local blocks of conditional structures, including [barcolor()](https://www.tradingview.com/pine-script-reference/v6/#fun_barcolor), [bgcolor()](https://www.tradingview.com/pine-script-reference/v6/#fun_bgcolor), [plot()](https://www.tradingview.com/pine-script-reference/v6/#fun_plot), [plotshape()](https://www.tradingview.com/pine-script-reference/v6/#fun_plotshape), [plotchar()](https://www.tradingview.com/pine-script-reference/v6/#fun_plotchar), [plotarrow()](https://www.tradingview.com/pine-script-reference/v6/#fun_plotarrow), [plotcandle()](https://www.tradingview.com/pine-script-reference/v6/#fun_plotcandle), [plotbar()](https://www.tradingview.com/pine-script-reference/v6/#fun_plotbar), [hline()](https://www.tradingview.com/pine-script-reference/v6/#fun_hline), [fill()](https://www.tradingview.com/pine-script-reference/v6/#fun_fill), [alertcondition()](https://www.tradingview.com/pine-script-reference/v6/#fun_alertcondition), [indicator()](https://www.tradingview.com/pine-script-reference/v6/#fun_indicator), [strategy()](https://www.tradingview.com/pine-script-reference/v6/#fun_strategy), and [library()](https://www.tradingview.com/pine-script-reference/v6/#fun_library).

This restriction does not entail their functionality cannot be controlled by conditions evaluated by your script — only that it cannot be done by including them in conditional structures. Note that while `input*.()` function calls are allowed in local blocks, their functionality is the same as if they were in the script’s _global scope_.

The local blocks in conditional structures must be indented by four spaces or a tab.

## `if` structure {#if-structure}

### `if` used for its side effects {#if-used-for-its-side-effects}

An [if](https://www.tradingview.com/pine-script-reference/v6/#kw_if) structure used for its side effects has the following syntax:

```
if <expression>
    <local_block>
{else if <expression>
    <local_block>}
[else
    <local_block>]
```

where:

-   Parts enclosed in square brackets (`[]`) can appear zero or one time, and those enclosed in curly braces (`{}`) can appear zero or more times.
-   <expression> must be of “bool” type or be auto-castable to that type, which is only possible for “int” or “float” values (see the [Type system](https://www.tradingview.com/pine-script-docs/language/type-system/#types) page).
-   <local\_block> consists of zero or more statements followed by a return value, which can be a tuple of values. It must be indented by four spaces or a tab.
-   There can be zero or more `else if` clauses.
-   There can be zero or one `else` clause.

When the <expression> following the [if](https://www.tradingview.com/pine-script-reference/v6/#kw_if) evaluates to [true](https://www.tradingview.com/pine-script-reference/v6/#const_true), the first local block is executed, the [if](https://www.tradingview.com/pine-script-reference/v6/#kw_if) structure’s execution ends, and the value(s) evaluated at the end of the local block are returned.

When the <expression> following the [if](https://www.tradingview.com/pine-script-reference/v6/#kw_if) evaluates to [false](https://www.tradingview.com/pine-script-reference/v6/#const_false), the successive `else if` clauses are evaluated, if there are any. When the <expression> of one evaluates to [true](https://www.tradingview.com/pine-script-reference/v6/#const_true), its local block is executed, the [if](https://www.tradingview.com/pine-script-reference/v6/#kw_if) structure’s execution ends, and the value(s) evaluated at the end of the local block are returned.

When no <expression> has evaluated to [true](https://www.tradingview.com/pine-script-reference/v6/#const_true) and an `else` clause exists, its local block is executed, the [if](https://www.tradingview.com/pine-script-reference/v6/#kw_if) structure’s execution ends, and the value(s) evaluated at the end of the local block are returned.

When no <expression> has evaluated to [true](https://www.tradingview.com/pine-script-reference/v6/#const_true) and no `else` clause exists, [na](https://www.tradingview.com/pine-script-reference/v6/#var_na) is returned. The only exception to this is if the structure returns “bool” values — in that case, [false](https://www.tradingview.com/pine-script-reference/v6/#const_false) is returned instead.

Using [if](https://www.tradingview.com/pine-script-reference/v6/#kw_if) structures for their side effects can be useful to manage the order flow in strategies, for example. While the same functionality can often be achieved using the `when` parameter in `strategy.*()` calls, code using [if](https://www.tradingview.com/pine-script-reference/v6/#kw_if) structures is easier to read:

```pine
if (ta.crossover(source, lower))
    strategy.entry("BBandLE", strategy.long, stop=lower,
                   oca_name="BollingerBands",
                   oca_type=strategy.oca.cancel, comment="BBandLE")
else
    strategy.cancel(id="BBandLE")
```

Restricting the execution of your code to specific bars can be done using [if](https://www.tradingview.com/pine-script-reference/v6/#kw_if) structures, as we do here to restrict updates to our label to the chart’s last bar:

```pine
//@version=6
indicator("", "", true)
var ourLabel = label.new(bar_index, na, na, color = color(na), textcolor = color.orange)
if barstate.islast
    label.set_xy(ourLabel, bar_index + 2, hl2[1])
    label.set_text(ourLabel, str.tostring(bar_index + 1, "# bars in chart"))
```

Note that:

-   We initialize the `ourLabel` variable on the script’s first bar only, as we use the [var](https://www.tradingview.com/pine-script-reference/v6/#kw_var) declaration mode. The value used to initialize the variable is provided by the [label.new()](https://www.tradingview.com/pine-script-reference/v6/#fun_label.new) function call, which returns a label ID pointing to the label it creates. We use that call to set the label’s properties because once set, they will persist until we change them.
-   What happens next is that on each successive bar the Pine Script runtime will skip the initialization of `ourLabel`, and the [if](https://www.tradingview.com/pine-script-reference/v6/#kw_if) structure’s condition ([barstate.islast](https://www.tradingview.com/pine-script-reference/v6/#var_barstate.islast)) is evaluated. It returns `false` on all bars until the last one, so the script does nothing on most historical bars after bar zero.
-   On the last bar, [barstate.islast](https://www.tradingview.com/pine-script-reference/v6/#var_barstate.islast) becomes true and the structure’s local block executes, modifying on each chart update the properties of our label, which displays the number of bars in the dataset.
-   We want to display the label’s text without a background, so we make the label’s background [na](https://www.tradingview.com/pine-script-reference/v6/#var_na) in the [label.new()](https://www.tradingview.com/pine-script-reference/v6/#fun_label.new) function call, and we use `hl2[1]` for the label’s _y_ position because we don’t want it to move all the time. By using the average of the **previous** bar’s [high](https://www.tradingview.com/pine-script-reference/v6/#var_high) and [low](https://www.tradingview.com/pine-script-reference/v6/#var_low) values, the label doesn’t move until the moment when the next realtime bar opens.
-   We use `bar_index + 2` in our [label.set\_xy()](https://www.tradingview.com/pine-script-reference/v6/#fun_label.set_xy) call to offset the label to the right by two bars.

### `if` used to return a value {#if-used-to-return-a-value}

An [if](https://www.tradingview.com/pine-script-reference/v6/#kw_if) structure used to return one or more values has the following syntax:

```
[<declaration_mode>] [<type>] <identifier> = if <expression>
    <local_block>
{else if <expression>
    <local_block>}
[else
    <local_block>]
```

where:

-   Parts enclosed in square brackets (`[]`) can appear zero or one time, and those enclosed in curly braces (`{}`) can appear zero or more times.
-   <declaration\_mode> is the variable’s [declaration mode](https://www.tradingview.com/pine-script-docs/language/variable-declarations/#declaration-modes)
-   <type> is optional, as in almost all Pine Script variable declarations (see [types](https://www.tradingview.com/pine-script-docs/language/type-system/#types))
-   <identifier> is the variable’s [name](https://www.tradingview.com/pine-script-docs/language/identifiers/)
-   <expression> can be a literal, a variable, an expression or a function call.
-   <local\_block> consists of zero or more statements followed by a return value, which can be a tuple of values. It must be indented by four spaces or a tab.
-   The value assigned to the variable is the return value of the <local\_block>, or [na](https://www.tradingview.com/pine-script-reference/v6/#var_na) if no local block is executed. If other local blocks return “bool” values, [false](https://www.tradingview.com/pine-script-reference/v6/#const_false) will be returned instead.

This is an example:

```pine
//@version=6
indicator("", "", true)
string barState = if barstate.islastconfirmedhistory
    "islastconfirmedhistory"
else if barstate.isnew
    "isnew"
else if barstate.isrealtime
    "isrealtime"
else
    "other"

f_print(_text) => 
    var table _t = table.new(position.middle_right, 1, 1)
    table.cell(_t, 0, 0, _text, bgcolor = color.yellow)
f_print(barState)
```

It is possible to omit the _else_ block. In this case, if the `condition` is false, an _empty_ value (`na`, `false`, or `""`) will be assigned to the `var_declarationX` variable.

This is an example showing how [na](https://www.tradingview.com/pine-script-reference/v6/#var_na) is returned when no local block is executed. If `close > open` is `false` in here, [na](https://www.tradingview.com/pine-script-reference/v6/#var_na) is returned:

```pine
x = if close > open
    close
```

Scripts can contain `if` structures with nested `if` and other conditional structures. For example:

```pine
if condition1
    if condition2
        if condition3
            expression
```

However, nesting these structures is not recommended from a performance perspective. When possible, it is typically more optimal to compose a single `if` statement with multiple logical operators rather than several nested `if` blocks:

```pine
if condition1 and condition2 and condition3
    expression
```

## `switch` structure {#switch-structure}

The [switch](https://www.tradingview.com/pine-script-reference/v6/#kw_switch) structure exists in two forms. One switches on the different values of a key expression:

```
[[<declaration_mode>] [<type>] <identifier> = ]switch <expression>
    {<expression> => <local_block>}
    => <local_block>
```

The other form does not use an expression as a key; it switches on the evaluation of different expressions:

```
[[<declaration_mode>] [<type>] <identifier> = ]switch
    {<expression> => <local_block>}
    => <local_block>
```

where:

-   Parts enclosed in square brackets (`[]`) can appear zero or one time, and those enclosed in curly braces (`{}`) can appear zero or more times.
-   <declaration\_mode> is the variable’s [declaration mode](https://www.tradingview.com/pine-script-docs/language/variable-declarations/#declaration-modes)
-   <type> is optional, as in almost all Pine Script variable declarations (see [types](https://www.tradingview.com/pine-script-docs/language/type-system/#types))
-   <identifier> is the variable’s [name](https://www.tradingview.com/pine-script-docs/language/identifiers/)
-   <expression> can be a literal, a variable, an expression or a function call.
-   <local\_block> consists of zero or more statements followed by a return value, which can be a tuple of values. It must be indented by four spaces or a tab.
-   The value assigned to the variable is the return value of the <local\_block>, or [na](https://www.tradingview.com/pine-script-reference/v6/#var_na) if no local block is executed.
-   The `=> <local_block>` at the end allows you to specify a return value which acts as a default to be used when no other case in the structure is executed.

Only one local block of a [switch](https://www.tradingview.com/pine-script-reference/v6/#kw_switch) structure is executed. It is thus a _structured switch_ that doesn’t _fall through_ cases. Consequently, `break` statements are unnecessary.

Both forms are allowed as the value used to initialize a variable.

As with the [if](https://www.tradingview.com/pine-script-reference/v6/#kw_if) structure, if no local block is executed, the expression returns either [false](https://www.tradingview.com/pine-script-reference/v6/#const_false) (when other local blocks return a “bool” value) or [na](https://www.tradingview.com/pine-script-reference/v6/#var_na) (in all other cases).

### `switch` with an expression {#switch-with-an-expression}

Let’s look at an example of a [switch](https://www.tradingview.com/pine-script-reference/v6/#kw_switch) using an expression:

```pine
//@version=6
indicator("Switch using an expression", "", true)

string maType = input.string("EMA", "MA type", options = ["EMA", "SMA", "RMA", "WMA"])
int maLength = input.int(10, "MA length", minval = 2)

float ma = switch maType
    "EMA" => ta.ema(close, maLength)
    "SMA" => ta.sma(close, maLength)
    "RMA" => ta.rma(close, maLength)
    "WMA" => ta.wma(close, maLength)
    => 
        runtime.error("No matching MA type found.")
        float(na)

plot(ma)
```

Note that:

-   The expression we are switching on is the variable `maType`, which is of “input int” type (see here for an explanation of what the “[input](https://www.tradingview.com/pine-script-docs/language/type-system/#input)” qualifier is). Since it cannot change during the execution of the script, this guarantees that whichever MA type the user selects will be executing on each bar, which is a requirement for functions like [ta.ema()](https://www.tradingview.com/pine-script-reference/v6/#fun_ta.ema) which require a “simple int” argument for their `length` parameter.
-   If no matching value is found for `maType`, the [switch](https://www.tradingview.com/pine-script-reference/v6/#kw_switch) executes the last local block introduced by `=>`, which acts as a catch-all. We generate a runtime error in that block. We also end it with `float(na)` so the local block returns a value whose type is compatible with that of the other local blocks in the structure, to avoid a compilation error.

### `switch` without an expression {#switch-without-an-expression}

This is an example of a [switch](https://www.tradingview.com/pine-script-reference/v6/#kw_switch) structure which does not use an expression:

```pine
//@version=6
strategy("Switch without an expression", "", true)

bool longCondition  = ta.crossover( ta.sma(close, 14), ta.sma(close, 28))
bool shortCondition = ta.crossunder(ta.sma(close, 14), ta.sma(close, 28))

switch
    longCondition  => strategy.entry("Long ID", strategy.long)
    shortCondition => strategy.entry("Short ID", strategy.short)
```

Note that:

-   We are using the [switch](https://www.tradingview.com/pine-script-reference/v6/#kw_switch) to select the appropriate strategy order to emit, depending on whether the `longCondition` or `shortCondition` “bool” variables are `true`.
-   The building conditions of `longCondition` and `shortCondition` are exclusive. While they can both be `false` simultaneously, they cannot be `true` at the same time. The fact that only **one** local block of the [switch](https://www.tradingview.com/pine-script-reference/v6/#kw_switch) structure is ever executed is thus not an issue for us.
-   We evaluate the calls to [ta.crossover()](https://www.tradingview.com/pine-script-reference/v6/#fun_ta.crossover) and [ta.crossunder()](https://www.tradingview.com/pine-script-reference/v6/#fun_ta.crossunder) **prior** to entry in the [switch](https://www.tradingview.com/pine-script-reference/v6/#kw_switch) structure. Not doing so, as in the following example, would prevent the functions being executed on each bar, which would result in a compiler warning and erratic behavior:

```pine
//@version=6
strategy("Switch without an expression", "", true)

switch
    // Compiler warning! Will not calculate correctly!
    ta.crossover( ta.sma(close, 14), ta.sma(close, 28)) => strategy.entry("Long ID", strategy.long)
    ta.crossunder(ta.sma(close, 14), ta.sma(close, 28)) => strategy.entry("Short ID", strategy.short)
```

## `once` structure {#once-structure}

Unlike the [if](https://www.tradingview.com/pine-script-reference/v6/#kw_if) and [switch](https://www.tradingview.com/pine-script-reference/v6/#kw_switch) structures, which are evaluated on every execution of their containing scope, the [once](https://www.tradingview.com/pine-script-reference/v6/#kw_once) structure is evaluated only once, when its condition is true for the first time, and then never again after that. See the [`once` on the realtime bar](https://www.tradingview.com/pine-script-docs/language/conditional-structures/#once-on-the-realtime-bar) section below for details of the behavior of the [once](https://www.tradingview.com/pine-script-reference/v6/#kw_once) structure on the realtime bar, where it _can_ fire more than once.

The [once](https://www.tradingview.com/pine-script-reference/v6/#kw_once) structure has the following syntax:

```
once [<condition>]
    <statements>
```

where:

-   `<condition>` is an optional parameter, `true` by default, that takes an argument of type “series bool”. This conditional expression controls the structure’s execution. The structure executes its local block when the expression evaluates to `true`. It prevents additional executions of the block after the condition is `true` for the first time on a closed bar.
-   `<statements>` is the block of statements and expressions that execute when the condition evaluates to `true`. The block must be indented by four spaces or a single tab.

The following example uses the [once](https://www.tradingview.com/pine-script-reference/v6/#kw_once) structure to draw a label the first time that price closes above a moving average. It also draws another label on the same condition, with equivalent code using an [if structure](https://www.tradingview.com/pine-script-docs/language/conditional-structures/#if-structure) and a Boolean flag:

```pine
//@version=6
indicator("`once` simple demo", overlay = true)

//@variable The simple moving average of `close` prices, using the input length.
mySMA = ta.sma(close, 20)

//@function Prints a label above or below the current bar at a given price with the supplied text.
printLabel(float y, string txt, bool isAbove = true) =>
    label.new(bar_index, y, txt, xloc.bar_index, yloc.price, #2195f382, 
      isAbove ? label.style_label_down : label.style_label_up, chart.fg_color)

// The first time price closes above the SMA, print a label.
once close > mySMA
    printLabel(mySMA, "First close above\nthe moving average\ndetected using `once`.")

//@variable This persistent flag is `true` if we have printed a label; `false` otherwise.
var bool printedLabel = false
// The first time price closes above the SMA, print a label.
if close > mySMA and printedLabel == false
    printLabel(mySMA, "First close above \nthe moving average\ndetected using `if` plus a flag.", false)
    printedLabel := true  // Set the flag so that this `if` structure does not fire again.

// Plot the SMA line.
plot(mySMA)
```

Note that:

-   The equivalent logic using the [if structure](https://www.tradingview.com/pine-script-docs/language/conditional-structures/#if-structure) and a Boolean flag achieves the same result, but uses more code and is less performant.

Unlike the other conditional structures, the [if](https://www.tradingview.com/pine-script-reference/v6/#kw_if) and [switch](https://www.tradingview.com/pine-script-reference/v6/#kw_switch) structures, the [once](https://www.tradingview.com/pine-script-reference/v6/#kw_once) structure does _not_ return usable values. Scripts cannot assign a `once` statement or a call to a function that ends with a `once` statement to a variable or a tuple of variables.

The following example script demonstrates this limitation. The first structure correctly reassigns a “string” variable _inside_ a [once](https://www.tradingview.com/pine-script-reference/v6/#kw_once) block. The second structure uses an [if structure](https://www.tradingview.com/pine-script-docs/language/conditional-structures/#if-structure) and a Boolean flag to emulate a [once](https://www.tradingview.com/pine-script-reference/v6/#kw_once) block; it _returns_ a value that is additively assigned to the same “string” variable. The third structure is commented out. It attempts to _return_ a string from a [once](https://www.tradingview.com/pine-script-reference/v6/#kw_once) block and additively assign it the same “string” variable as the previous structures. Uncommenting this third structure causes a _compilation error_, because a [once](https://www.tradingview.com/pine-script-reference/v6/#kw_once) structure _cannot_ return usable values. When the script is in a form that compiles, it displays the final string in a label:

```pine
//@version=6
indicator("`once` compilation error demo", overlay = true)

//@variable The text to display in the label.
var string txt = ""

// Structure 1: Assign a success message to the `txt` variable once, *inside* the `once` structure.
once close > open
    txt := "Structure 1 works."

//@variable A flag indicating whether Structure 2 fired.
var bool condition2Fired = false
// Structure 2: If this condition has not fired before, set the `condition2Fired` flag to `true` and return a
// success message.
txt += if close > open and not condition2Fired
    condition2Fired := true  // Set the flag.
    "\nStructure 2 works."   // Return a string. It is appended to the `txt` variable's value.

// // Uncommenting this section causes a compilation error, because it attempts to use the returned value from 
// // the `once` structure, and `once` structures cannot return values. 
// txt += once close > open
//     txt := "\nStructure 3 works."

// Draw a label on the last bar showing the `txt` string.
if barstate.islast
    label.new(bar_index + 1, open, txt, style = label.style_label_left)
```

### `once` on the realtime bar {#once-on-the-realtime-bar}

On the live realtime bar, a [once](https://www.tradingview.com/pine-script-reference/v6/#kw_once) structure that has not yet fired is evaluated on _each tick_. If the structure’s condition is `true`, its statements are executed. However, unless that tick is the _closing_ tick of the bar, the structure’s state is _reset_ by Pine’s rollback process on the bar’s next tick. If the structure’s condition is `true` on a subsequent tick of the same bar, its statements are executed again.

In this way, a [once](https://www.tradingview.com/pine-script-reference/v6/#kw_once) structure can execute its statements multiple times on one realtime bar.

The block must execute on a bar’s _closing tick_ in order to make the [once](https://www.tradingview.com/pine-script-reference/v6/#kw_once) structure inactive. An inactive [once](https://www.tradingview.com/pine-script-reference/v6/#kw_once) structure never executes its statements again.

This behavior can affect code that is _not_ reset by rollback, including [varip variables](https://www.tradingview.com/pine-script-docs/language/variable-declarations/#varip), function calls that create [Pine Logs](https://www.tradingview.com/pine-script-docs/writing/debugging/#pine-logs), [strategy](https://www.tradingview.com/pine-script-docs/concepts/strategies/) commands, and [`alert()` calls](https://www.tradingview.com/pine-script-docs/concepts/alerts/#alert-function-events). Refer to the [Executions on realtime bars](https://www.tradingview.com/pine-script-docs/language/execution-model/#executions-on-realtime-bars) section of our User Manual for advanced details about rollback and its exceptions.

If a script uses code that is not reset by rollback, one way to prevent multiple executions of a [once](https://www.tradingview.com/pine-script-reference/v6/#kw_once) structure is to append `and barstate.isconfirmed` to its condition. This addition means that the condition can be `true` only on the closing tick of a realtime bar, or on a historical bar.

The following example script demonstrates this behavior. It declares two different counter variables using the [varip](https://www.tradingview.com/pine-script-reference/v6/#kw_varip) keyword, so that their values persist across intrabar rollbacks. It increments these counters inside two [once](https://www.tradingview.com/pine-script-reference/v6/#kw_once) blocks. Both structures are inside an `if barstate.isrealtime` block so that they are active when the script starts executing on its first realtime bar. The first [once](https://www.tradingview.com/pine-script-reference/v6/#kw_once) structure uses an empty condition, so it increases its counter and logs its message on every tick of the first realtime bar. The second [once](https://www.tradingview.com/pine-script-reference/v6/#kw_once) block uses `barstate.isconfirmed` as its condition, so it increases its counter and logs its message only on the _closing tick_ of the first realtime bar. We can see that both counters are zero on historical bars. On the first realtime bar on which the script executes, the on-close counter increments to 1 because the block that increments it runs only once, while the per-tick counter increases beyond 1 because its block runs several times during the bar. On subsequent bars, the values of both counters persist without incrementing, because the [once](https://www.tradingview.com/pine-script-reference/v6/#kw_once) blocks cannot run again. The log messages show the same behavior as the plots:

```pine
//@version=6
indicator("`once` upon a realtime bar")

//@variable counts how many times the first `count` block executes per bar.
varip int realTimeCount = 0
//@variable counts how many times the second `count` block, which runs only on the closing tick, executes per bar.
varip int closingTickCount = 0

// Evaluate our `once` blocks only on realtime bars, so that a closed bar does not fire them once and forever.
if barstate.isrealtime
    // Fires on every tick of the first realtime bar on which the script executes.
    once
        realTimeCount += 1  // Increase the realtime count by one.
        log.info("Close > open on this tick.")  // Log a message.
    // Fires on the *closing tick* of the first realtime bar on which the script executes.
    once barstate.isconfirmed
        closingTickCount += 1  // Increase the closing count by one.
        log.info("🕛 Close > open on the closing tick.")  // Log a message.
 
// Plot both counts.
plot(realTimeCount,    "Once per realtime tick")
plot(closingTickCount, "Once per realtime bar", color.red)
// Plot a zero line.
hline(0, linestyle = hline.style_dotted)
```

Note that:

-   If the counter variables are instead declared using the [var keyword](https://www.tradingview.com/pine-script-docs/language/variable-declarations/#var), they both increment to 1 and remain there, because their values are _rolled back_ after each realtime tick.

## Matching local block type requirement {#matching-local-block-type-requirement}

When multiple local blocks are used in structures, the type of the return value of all its local blocks must match. This applies only if the structure is used to assign a value to a variable in a declaration, because a variable can only have one type, and if the statement returns two incompatible types in its branches, the variable type cannot be properly determined. If the structure is not assigned anywhere, its branches can return different values.

This code compiles fine because [close](https://www.tradingview.com/pine-script-reference/v6/#var_close) and [open](https://www.tradingview.com/pine-script-reference/v6/#var_open) are both of the `float` type:

```pine
x = if close > open
    close
else
    open
```

This code does not compile because the first local block returns a `float` value, while the second one returns a `string`, and the result of the `if`\-statement is assigned to the `x` variable:

```pine
// Compilation error!
x = if close > open
    close
else
    "open"
```
