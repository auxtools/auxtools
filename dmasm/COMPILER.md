# Proc compiler

`dmasm::compiler::compile_proc` compiles a DM proc parsed by SpacemanDMM's `dreammaker` crate into a
list of nodes that assemble to BYOND bytecode. The compiler targets the running world's BYOND build
so hot reload can replace a proc with the compiled result.

## Target BYOND build

Pass the running world's build through `ProcSource::byond`, `compile_expr_for`, or
`reload::ParseOptions::byond`. `ByondVersion::read_as` configures parsing for that build.
`compiler/src/target.rs` holds a `ByondVersion` check for every build where dmasm's output changes,
and `compiler/src/compiler/versions.rs` holds the arithmetic that varies across builds.

Probes covered builds 515.1637, 515.1647, 516.1659, 516.1685, 516.1688, and 516.1689, with
additional probes pinpointing behavior changes across 515.1637–516.1660 as well as 516.1664,
516.1668–516.1672, 516.1676, 516.1681, and 516.1684.

- **Built-ins and syntax.** BYOND 515 uses `Sin`, `Cos`, and `Tan` instead of their `*2`
  instructions. Calls to a built-in proc that the target build lacks compile as global proc calls.
  `alist()` and key/value iteration require BYOND 516, and `__PROC__` requires 515.1644.
- **Integer literals.** Before 515.1644, any integer literal above 2147483647 (including hexadecimal
  and octal literals) clamps to 2147483647. Literals with a decimal point or an exponent, such as
  `3000000000.0` and `3e9`, are unaffected.
- **Folding.** BYOND 515 does not fold `ceil`, `floor`, `round`, `trunc`, or `fract`. BYOND 516 also
  folds `isnan` and `isinf`, subject to the flag quirk described under [Warnings](#warnings).

## Validation

A comparison harness compiles every proc in a codebase, assembles it against DreamMaker's `.dmb`,
and compares the resulting disassembly with DreamMaker's. Results from 2026-11-10:

| Codebase commit | BYOND | Procs | Identical | Equivalent | Known differences |
| --- | --- | --- | --- | --- | --- |
| cmss13 `f712f7b` | 516.1687 | 28,330 | 19,456 | 8,872 | 2 |
| tgstation `928bb812` | 516.1685 | 66,712 | 43,255 | 23,422 | 35 |
| goonstation `3e27edf` | 516.1687 | 44,522 | 29,643 | 14,875 | 2 |
| tgstation `addb78ce0b` | 516.1659 | 65,478 | 42,499 | 22,944 | 35 |
| tgstation `6a18424329` | 515.1647 | 58,937 | 38,860 | 20,041 | 36 |
| tgstation `7faadfdc16` | 515.1637 | 58,171 | 38,343 | 19,791 | 37 |

"Identical" procs match DreamMaker's output including debug information. "Equivalent" procs match
once debug information is omitted and the forms under [Equivalent encodings](#equivalent-encodings)
are normalized. Every remaining proc differs in the known ways described in this document.

The harness also patches the compiled bytecode into the `.dmb` and runs each codebase's unit tests
(150 for cmss13, 486 for tgstation, and 22 for goonstation), producing the same results as
DreamMaker's builds. Differential fuzzing compared 1,536 worlds of 80 generated procs each.

The codebase harness, its bytecode patcher, and the fuzzer live outside this repository. This
repository includes the `compare_dreammaker` example, which performs the same comparison against the
small worlds in `compiler/tests/dreammaker/`. See
[Comparing with DreamMaker](README.md#comparing-with-dreammaker) for how to run it.

## Equivalent encodings

The comparison normalizes these differences:

- Static versus dynamic proc calls. DreamMaker selects one based on compilation order, which dmasm
  does not know, so dmasm always emits dynamic calls.
- Label names and switch-target encodings. The comparison compares target destinations instead.
- Unneeded cache loads of plain `src`, arguments, or locals.

Cache reuse through fields is not normalized because it changes runtime behavior. For example, if
`a.b.f()` changes `a.b`, DreamMaker still reads the old object from the cache for a subsequent
`a.b.x`. dmasm follows the same rules for reusing and clearing the cache.

## Known differences

| Source or case | DreamMaker | dmasm |
| --- | --- | --- |
| Parameter default `a.b/10` without spaces | Parses a path and drops the division. `x/-10` becomes a subtraction | Divides |
| `a?.f(b?.x)?.y = v` | Calls `f` on the object that evaluating the argument left in the cache | Calls `f` on its own object |
| `a?.b \|\|= v` or `&&=` before 516.1673 | Drops the null checks. With multiple `?.` accesses, reads and writes the variable of the first object | Keeps the null checks and the final target |
| `/T/name` subtype block where `name` is a proc of `T` | Adds the block to the proc | Parses a subtype |
| Empty proc at the end of a file | Names the next included file in debug information | Names the proc's own source file |
| `*p[i] = x` from 516.1668 | Drops the index and writes through the cache | Writes the indexed element |
| `*(&x)` before 516.1668 | Reads or writes `*x`, dereferencing `x`'s value (a "bad pointer" error unless `x` holds a pointer) | Reads or writes `x` |
| `isnan()` or `isinf()` of a non-constant value, in 516.1670 | Does not read the test flag, crashing the server | Reads the flag |
| Out-of-range constants in `&`, `\|`, `^`, `<<`, `>>` | Converts to 24 bits in ways that vary across builds and cases | Uses the low 24 bits of the nearest 32-bit integer |
| `pick()` ending in a weight-zero entry, before 516.1689 | Jumps outside the proc if no earlier entry is chosen | Selects the last entry whose weight is not a constant zero |
| `continue LABEL` to a labeled block containing `do` ... `while`, inside a counted `for` loop | Pops the `for` loop's range (`PopN`) before jumping, crashing the server | Leaves the range on the stack |

dmasm rejects hexadecimal literals longer than eight digits and octal literals that do not fit in an
`i32`. DreamMaker converts them in an unusual way that dmasm does not replicate (for example,
`0x100000000` becomes `2**60`).

## Warnings

### Diagnostic collections

`CompiledProc::warnings` reports two categories: intentional differences from DreamMaker and notable
DreamMaker quirks that dmasm replicates. Hot reload forwards these diagnostics.

`CompiledProc::byond_bugs` filters down to bugs in DreamMaker builds, omitting dmasm workaround
details and compatibility-only differences. It covers the bugs listed under
[Bugs affecting both DreamMaker and reloads](#bugs-affecting-both-dreammaker-and-reloads), division
in parameter defaults, safe-call objects, logical assignment before 516.1673, indexed pointers from
516.1668, `*(&x)` before 516.1668, the copied quirks below, and `pick()` ending in a zero weight
before 516.1689. `Snapshot::known_bugs` collects these across a project for `auxtools check`.

Every warning is a `diagnostic::Diagnostic` carrying a source line and an ID. Compatibility
differences each have a distinct ID, while each BYOND bug has a `Byond` ID shared by its warning and
its `byond_bugs` entry.

Specifically, the compiler warns on division in parameter defaults, safe-call objects, logical
assignment before 516.1673, indexed pointer assignment from 516.1668, `*(&x)` before 516.1668,
out-of-range bit operands, `pick()` ending in a zero weight before 516.1689, angles that fold
differently on glibc 2.41, and colors that fold differently on Windows.

Differences only in debug locations or encoding do not emit warnings. Across the validation
codebases, most warnings come from divisions in tgstation's parameter defaults, while older builds
also trigger the safe list assignment quirk.

### Copied DreamMaker quirks

dmasm replicates and warns on two DreamMaker quirks:

- In 516.1648–516.1669 and 516.1671, a constant `isnan` or `isinf` test emits its result followed by
  `GetFlag`, returning the flag from the preceding test. From 516.1670 onward (including 516.1671),
  a test inside another constant expression such as `!isnan(1)` folds with that expression.
- Before 516.1685, modifying a list element through `?.` when the right-hand value uses the cache
  takes the target list from the cache value left behind by that expression.

One quirk is copied without a warning: after `a.b.f()`, DreamMaker keeps `a.b` in the cache, so a
later `a.b.x` in the same control-flow path uses the old object even if `f()` assigned another
object to `a.b`. Because the pattern is common and only matters when the call replaces the field,
dmasm stays silent. If a call can replace a field, read that field into a local variable first.

### Bugs affecting both DreamMaker and reloads

These bugs (tracked in `compiler/src/compiler/known_bugs.rs`) affect both DreamMaker builds and
reloads:

- `continue LABEL` targeting a labeled `do` ... `while` inside a counted `for` loop crashes the
  server.
- Nested list loops separated by a `try`: catching an error in the inner loop causes the outer loop
  to use the inner loop's iterator, which can end the outer loop early, loop forever, or crash.
- `operator<<=` and `operator>>=` discard their result. `operator++` and `operator--` used as
  statements are called without an argument and discard their result. `operator**` on a type with
  `operator*` calls `operator*` for `a ** 2`.
- `operator<=>` from 516.1674 is called by `list.Find()` and `list ^ x`, and from 516.1689 those
  treat `0` as a match. `operator~=` and `operator~!` in 516.1674–516.1688 discard their result.
- `Uncross()` on an area from 516.1683 ignores its result.
- Reading `*p[i]` from 516.1668 behaves like its assignment counterpart above.

## Constant folding and platform differences

The compiler always folds constants the way Linux DreamMaker does and exposes no option for the
build platform.

### Math

BYOND 515 converts trigonometric inputs straight to radians. BYOND 516 normalizes the angle and
reduces it by quadrants, yielding exact values on the axes for `sin` and `cos`. For `tan`, BYOND 516
reduces the angle to 45 degrees or less, computes in `f64`, rounds to `f32`, and then takes the
reciprocal and applies the sign as needed. Each step affects the result, so they must stay in that
exact order. `arctan(x, -0)` treats its second argument as zero, and `%%` folds with `f64` `fmod`.

Inverse trigonometric functions fold in `f64` and round once to `f32`. DreamMaker on Windows, and on
Linux before 515.1644, calls the `f64` functions. Starting in 515.1644, Linux DreamMaker calls
glibc's `f32` functions. Before glibc 2.41, those returned the unrounded x87 result, keeping the
folded value identical. From glibc 2.41 onward, they round the radians to `f32`, shifting some
constants by one `f32` rounding step (for example, `arctan(2)` is `63.434948` normally and
`63.43495` on glibc 2.41). This behavior was verified on glibc 2.35, 2.36, 2.39, and 2.41, and the
compiler warns whenever glibc 2.41 would produce a different value.

### Colors

`rgb()` folds in every color space. RGB and alpha channels truncate to integers, while HSV, HSL, and
HCY channels round with halves rounded up. HSL clamps saturation and luminance to `0–100`, HSV
clamps them starting in 516.1657, and HCY does not clamp. HCY uses a green weight of `0.557` for
`rgb()` and `0.587` for `rgb2num()`.

Starting in 515.1647, Linux DreamMaker normalizes percentage inputs in `f32`, computes in `f64`, and
rounds each channel to `f32` before rounding to an integer. Windows and 515.1637 use a different
intermediate precision, so channels near a half-integer can differ by one (276 of 31,752 sampled
colors differed between Linux and Windows on 516.1688). The compiler rounds like Linux and warns
near those boundaries. For builds before 515.1647, it defers such colors to runtime.

`rgb2num()` folds RGB results into lists of integers. It accepts short or long hex colors (with or
without alpha), named colors in any case, or `null` (which means white). Any color space other than
1, 2, and 3 selects RGB. Inside a condition, `rgb2num()` is always true regardless of color space,
so the compiler skips building the list. HSV, HSL, and HCY results are deferred to runtime because
their folded precision varies across builds and platforms.

At runtime, color conversion follows the platform running DreamDaemon, and angle conversion on Linux
follows DreamDaemon's glibc just as DreamMaker does. Windows DreamDaemon rounds radians to `f32` in
every build (verified under Wine). Sweeps of numeric, text, statement, and built-in code found no
other platform differences apart from bit operations on out-of-range operands.

## Debug lines

In debug builds, the compiler emits `DbgFile` and `DbgLine` records to match DreamMaker. Breakpoints
rely on these records, moving to the next recorded line when placed on a line without one.

- Statements record their own lines, including `var` without an initializer, `var/const`, and `set`.
  `else`, `;`, empty macro expansions, comments, and lines removed by `#if` do not record a line.
- Before the first statement, a proc records the first nonempty line after its header, which may be
  an empty macro expansion, a `;`, the body's opening brace, or the name of a multiline macro call.
  An empty body records the next nonempty line, usually the following proc's header
  (`ProcValue::first_line`).
- Parameter defaults record the header's line. A return type (`as num`) records the header's line a
  second time after the defaults.
- DreamMaker also records lines it reads ahead before compiling the current construct. The parser
  supplies `ProcValue::next_lines`, `end_lines`, and `operator_lines` so dmasm can reproduce those
  records:
  - An initializer beginning with interpolated text (`var/s = "[x]"`) records the next nonempty line
    before the value, then the line where the text ends. Directives and empty macros count as lines
    here, but comments do not. `return "[x]"` records that next line before its own line. The same
    records appear at switch-case entries and loop exits when those statements occur there.
  - In `for (var/k, v in L)`, the value variable records the body's first line, then the loop's line
    again.
  - Multiline calls with named arguments and multiline `if` or `while` conditions record the line of
    the closing parenthesis. Interpolated text records its ending line. A binary operator, `?`, or
    `.` starting a continuation line records that line, and the next term records its own line even
    when it is the same line. Expression statements record those operator lines before the
    statement's own line.

Debug lines compared with DreamMaker on 2026-11-13 across the same validation codebases:

| Codebase | Procs | Same, start | Same, first-line rules | Same, all rules |
| --- | --- | --- | --- | --- |
| cmss13 | 28,330 | 26,311 | 27,291 | 27,949 |
| tgstation | 66,712 | 58,470 | 64,363 | 66,166 |
| goonstation | 44,522 | 42,576 | 43,285 | 44,341 |
| tgstation 515.1637 | 58,171 | 51,780 | 56,395 | 57,697 |
| tgstation 515.1647 | 58,937 | 52,397 | 57,119 | 58,459 |
| tgstation 516.1659 | 65,478 | 57,575 | 63,278 | 64,953 |

Remaining debug-line differences:

| Case | Procs (cmss13, tgstation, goonstation) | DreamMaker | dmasm |
| --- | --- | --- | --- |
| A proc defined by a macro (`GLOBAL_LIST_INIT`, goonstation's `HELP_MESSAGE_OVERRIDE`) | 188, 290, 25 | Records lines after the macro invocation | Records the macro's line |
| The last proc of an included file | 9, 39, 22 | Records the `.dme`'s `#include` line among the proc's lines, and for an empty proc records the first line of the next file | Records lines from the proc's own file |
| Other multiline code: statements from macro expansions (`LAZYADD`, `DB_VIEW`), conditions with multiple operators continued with `\`, multiline list-entry assignments, ternaries in text | 184, 217, 130 | Records further lines read ahead, including blank lines inside a macro expansion | Records the lines described above |

These differences can shift breakpoints after a hot reload. When DreamMaker records a read-ahead
line, its bytecode can pause on a statement earlier.

## Compiler compatibility rules

Each rule below affects generated bytecode and is covered by focused tests.

### Variables, constants, and lookup

- User `const` variables (including local constants) have slots among the globals. A standalone read
  loads the global slot, while a larger constant expression folds the value. Built-in constants such
  as `NORTH` are emitted directly.
- Static locals are block-scoped. Their scope (`StaticScope::Proc`) is the proc's path paired with
  the zero-based index of that declaration among declarations of the same name in the proc.
- Static and `const` access via `x::a::b` does not evaluate the objects before `b`. `/type::proc()`
  selects that type's override, `:` lookup searches subtypes depth-first, and a call on an unknown
  object uses the display name of the first declaration.
- After the first `for` loop, label, `switch`, or `pick()`, a variable declared without an
  initializer is explicitly set to `null`.
- `var/L[n]` emits `n` followed by `EmptyList`, whereas a multidimensional declaration uses
  `new /list`. In `var/x = a in L`, `L` is the declaration's input clause, while the statement
  `x = a in L` parses as `(x = a) in L`.

### Cache tracking and assignments

- `for` loops, ternary `else` branches, and logical operands whose values are discarded clear the
  compiler's tracked cache state. A label preserves tracked state only when a single control-flow
  path reaches it. Switch cases sharing a target count as one path, and a fallthrough after `return`
  still counts.
- Assignment statements clear cached field chains, while list-entry writes, most declarations, and
  value-position assignments do not (among declarations, `var/L[] = new()` does clear them). Calls
  outside the cache clear chains, while calls on an object, `..()`, and `spawn` preserve them.
  `x.y ||= v` resolves its target before testing it.
- Logical assignments resolve fields and list entries once, saving the cache around the right-hand
  expression whenever that expression needs the cache. Static and `const` reads through `.` and
  `global.x` do not need a save, whereas `::` accesses do. If the right-hand expression indexes a
  list, the target's key is saved as well.
- Safe assignment emits null checks for every field, call, and index in the chain, using
  `SetCacheJmpIfNull` for value expressions and `SetCachePopJmpIfNull` for statement targets.
- Because `PickSwitch` always jumps, the compiler preserves its tracked cache state for the first
  option.

### Expressions, operators, and folding

- An empty argument compiles to `PushVal null`, while an explicit `null` compiles to `GetVar null`.
  An associative key `null = x` becomes the string `"null"`.
- In return values, call arguments, and parenthesized expressions, associative `+`, `*`, `&`, `|`,
  and `^` expressions are regrouped before folding. Math built-ins fold except `min` and `max`,
  reversed `clamp` bounds are swapped, and `X * X` uses `PowSquare` for supported values (excluding
  interpolated strings and fractions).
- Binary bitwise operations clamp operands to `i32` and retain 24 bits. `~` converts through `u32`,
  with tests covering negative and overflow results.

### Control flow and loops

- `spawn` without a block applies to the next statement and retains the targets of enclosing loops.
  A labeled `break` leaves the block. Exiting a `try` block emits `Catch` or `TryJmp`.
- `for (var/datum/T/x)` iterates over all datums of type `T`, whereas adding `in world` iterates
  over atoms and checks their type (finding no datums). `for (var/list/T/x in L)` checks that `x`
  is a `T`.
- `for (var/T/k, v in L)` filters and checks the type of the key just as `for (var/T/x in L)` does.

### Built-ins, parameters, and prefabs

- `del(x)` sets a writable target to `null` before deleting the object. `del src` ends the proc, and
  `del(world)` only deletes.
- `input()` takes four arguments (filling missing ones with `null`) plus an `in` source that is
  either built-in or on the stack. `findlasttext` defaults to `0` for `start` and `1` for `end`.
- Parameters with an absolute path, and `...`, do not receive parameter slots.
- Prefabs use entries in the instance table with init procs, ordering variable assignments by string
  ID with source locations before computed values. The assembler resolves instances through
  `AssembleEnv::get_instance`.

## State carried between emitters

### Speculative compilation

Two checks inspect code before constant folding: `names_other_file` checks whether a folded
`/type::var` default comes from another file, and `unfolded_has` checks whether the value in
`x.y ||= v` uses the cache. Both recompile the expression with folding disabled through
`Compiler::speculate`, which discards the trial code and restores every other compiler field
(including warnings) whether or not the trial succeeded.

### Fields passed between emitters

These fields pass state from one emitter to a later one. Each is set just before the code that
consumes it, and inserting unrelated emission in between breaks the invariant noted below:

- `regroup_root` is set before a return value, argument, or parenthesized expression and consumed by
  the next `emit_inner_expr`. For binary operations, `emit_inner_expr` forwards it to
  `binary_ops::emit` as `regroup_chain`, and otherwise drops it. Intervening code would be regrouped
  instead.
- `place` is set before the target of a non-`=` assignment (in `emit_place`, the statement form, and
  `emit_conditional`) and consumed by the next `emit_inner_expr`, suppressing read-ahead and
  compound-term line records. Arms that compile rewritten expressions (`&(x)`, `*(p)`,
  `locate(X) in a in b`) or parenthesized places compile the inner expression without `place` so
  those inner expressions record lines, while the statement form strips parentheses first. Only a
  multiline place could differ, and DreamMaker rejected both such places that were tested.
- `safe_place` is set by `emit_conditional` in debugger mode. `emit_inner_expr` takes it and
  restores it right before `follow::emit_typed` runs on the place's follows after the term, keeping
  it invisible to code compiled for the term.
- `list_elem_type` is set by `emit_inner_expr` immediately before `follow::emit_typed`, which
  consumes it first.
- `skip_compound` is set for `return "[x]"` after its read-ahead line and consumed by the return
  value's `emit_inner_expr`. Because such a value is never `.`, the field is always consumed.
- `explicit_src` is set when a chain begins with `src` (as in `src.x` or `new src.a.b()`) and
  cleared by the next instruction (the chain's first load). Call arguments save and restore it so
  the receiver's `src.` does not leak into them. Any instruction emitted before that load would
  clear it, causing the chain to reuse whatever a bare `x` left in the cache.
- `line_pending` marks that a default's line or a text initializer's read-ahead line was recorded,
  and `read_on_pending` marks that a read-ahead line was recorded. While either is set, the next
  compound term records its own line. Any line record clears both flags, so they persist until the
  next line rather than the next expression. Without debug information, no line is recorded and
  leaving them set has no effect.
- `other_file` marks that the last line record named another file (such as a default's file, or
  would have if the default had not folded), prompting the next line record to re-emit the proc's
  own file first. Without debug information, it remains set harmlessly.
- `mul_lhs` suppresses `PowSquare` for an inner multiplication that is the left operand of another
  multiplication. Parentheses do not change this rule.
- `condition_chain` is set while compiling an `if` or `while` condition whose top-level operator is
  `&&` or `||`, and is consumed by that outer `&&` or `||`.

### Scoped fields

These fields are set around a single construct and cleared afterward:

- `implicit_type` holds the declared type of an assignment target. The first implicit `new` or
  `astype()` in the right-hand expression takes that type, matching DreamMaker (for example, in
  `var/T/x = f(new)`, `new` constructs a `T`).
- `global_field` is set by field reads. It is cleared before compiling the right-hand value of a
  `||=` statement and consumed afterward.
- `proc_macro_ok` is set for the proc argument of `call(x, __PROC__)`. Before 515.1644, DreamMaker
  accepts `__PROC__` only in that position and rejects it everywhere else, so this flag lets the
  compiler accept it there on older builds. From 515.1644 onward, it is accepted everywhere.
- `no_fold` and `live_only` are set during trial compilations.

`read_to` is reset at every statement.

## Debugger code

In addition to compiling source procs with `compile_proc`, `Compiler` compiles code entered in the
debugger: a proc body via `compile_body_in`, or a single expression via the `compile_expr_*`
functions. The debugger selects between the two with `is_body`. Expressions compile as the body
`return EXPR` without debug lines, sharing the same code path validated against DreamMaker. Both
forms execute inside the debugger's evaluation stub and return `list(result, params...)`.

Debugger code compiles as a global proc (the stub) against a tree of BYOND's built-in types parsed
once, with `Compiler::debugger` set. When a paused frame is available, `expr_scope` answers queries
about that frame. Debugger compilation differs from ordinary proc compilation only in the cases
below:

| Where | Construct | In debugger code |
| --- | --- | --- |
| `Compiler::proc_macros_known`, `term::emit` | `__TYPE__`, `__PROC__`, `..()`, `.()` | Rejected because the stub is not the paused proc and has no parent |
| `Compiler::emit_global_by_name`, `procs::find_var` | Bare name | Searches the body's locals, then parameters (frame variables), then `src` variables, then globals, failing with `"undefined variable"` if none matches. Globals are read through the world's global list or by the ID supplied by the scope. Without a scope, the name resolves as a global |
| `procs::const_lookup` | `vars` | Treated like any other name because BYOND's types declare it at the root as a `const` with no value |
| `term::emit`, `procs::emit_bare_call` | `f()` | Compiles as `src.f()` if the scope reports that `src` has the proc, and as a global proc call otherwise |
| `follow::emit_typed` | `x.f()` | Called by name (`CallName`) when BYOND's built-in types lack that proc, since world types are unknown and name lookup also finds verbs |
| `follow::commit_field_buffer` | `global.x` | Handled like a bare global, with `global.vars` returning the scope's list |
| `term::emit`, `Compiler::emit_inner_expr` | `/type/proc/name`, `/type::name()` | Looked up in the world through the scope |
| `Compiler::emit_inner_expr`, `fold::static_const` | `/type::var`, `x::a::b` | Read at runtime rather than folded or resolved from a type tree |
| `builtin_procs::emit_code`, `builtin_procs::declared_world_type`, `statements::implicit_type_for` | `astype(x)`, `istype(x)`, `var/T/x = new` | A declared type absent from BYOND's built-in types is emitted by its written name and resolved during assembly |
| `term::prefab_value` | `/type{var = value}` | Rejected because the created instance would remain in the world's tables |
| `binary_ops::emit`, `fold::binary_value` | Bit operand beyond 32 bits | Left for the world to convert at runtime |
| `assignment::emit_conditional` | `a?.b?.c \|\|= v`, `a?[i] &&= v` as a value | Compiled via the `Safe` arms of `follow::emit_typed`, which leave a place. In ordinary procs, only plain `a?.b.c` compiles as a value and the other forms are rejected |

`compiler/tests/compile_body.rs` compiles expressions both standalone and as `return EXPR` and
verifies that the emitted instructions match.

## Unsupported forms

Compilation returns an error for unsupported constructs, including savefile-to-savefile reads
(`S["a"] >> S["b"]`), standalone `load_resource()`, and non-constant switch cases. Callers must
retain the existing bytecode or reject the reload when compilation fails.

Assembly also requires every referenced resource, string, proc, and prefab instance to be provided
through the assembler environment. Resources must already exist in the world's `.rsc`, and hot
reload callers must register any new entries in the runtime tables.

## Parser dependency

`Cargo.toml` depends on [auxtools/SpacemanDMM](https://github.com/auxtools/SpacemanDMM) for parser
and preprocessor fixes required by the compiler: separate `alist` terms, empty arguments, type
clauses in `for ... in`, macro stringification, source-location macros, raw strings, declaration
flags, pragma handling, and metadata for absolute parameters.
