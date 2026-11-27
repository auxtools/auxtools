# `DbgLine` placement from source structure

DreamMaker emits `DbgFile` and `DbgLine` instructions only for worlds built with `#define DEBUG`,
allowing the debugger and runtime errors to map bytecode to source lines. Because DreamMaker places
`DbgLine` based on the source text rather than the compiled code, with many special cases, dmasm
emits the same lines so that breakpoints, stepping, and runtime error line numbers in a reloaded
proc match a DreamMaker build. It emits debug lines only when the proc source indicates that the
world has debug info (`ProcSource::debug_info`).

## Proc start and lookahead lines

- DreamMaker's first debug line in a proc is the first nonempty line after the header. An empty
  macro expansion or a `;` counts as nonempty, and an empty body can take the next proc's header
  line.
- A lookahead debug line can precede the last recorded line, such as a condition's final operator
  appearing before its closing parenthesis. The compiler records it anyway and marks the next term
  to record its own line.
- For `return "[x]"`, DreamMaker records the lookahead line before the `return` statement's own
  line.
- In a statement whose operator spans lines, as in `f(a,\n\tb) + 1`, DreamMaker records the
  operator's line before the statement's line.

## Compound terms, conditions, loops, and assignments

- DreamMaker records the line of a multiline compound term, and of the first term after a variable
  default or a lookahead line. A default that folds away still counts because DreamMaker decides
  from the source rather than from the folded result.
- Multiline `if` and `while` conditions record the line of their closing parenthesis. The next term
  then records its own line even if the line number is unchanged.
- In a `for (k, v in L)` loop, the value variable is declared after reading into the loop body, so
  DreamMaker records the body's line and then the loop's line again.
- The place of `-=` and similar operators receives no debug line of its own, whereas `=` to a list
  element gets one.

## Comparison and testing

Basing line placement on folded code instead of the source places lines incorrectly for defaults
and conditions that fold away. While differing debug lines do not change what a proc executes, the
comparison tool counts a proc as an exact match only when every word matches, debug lines included.
It drops `DbgFile` and `DbgLine` only when judging whether the code is equivalent. Many of these
cases are checked directly by the compiler's tests.

## Applies to

- `dmasm/compiler/src/compiler/procs.rs`: `ProcSource::debug_info`
- `dmasm/compiler/src/compiler.rs`: line handling at the start of `Compiler::emit_expr`
- `dmasm/compiler/src/compiler/statements.rs`: proc start, lookahead cases in `emit_statement`,
  `condition_end`, the `ForKeyValue` arm, and the assignment place
