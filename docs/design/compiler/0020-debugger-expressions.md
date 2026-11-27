# Compiling debugger expressions as procs

To evaluate expressions, watch expressions, and conditions in a paused frame, dmasm compiles each
expression into a small proc. Because that proc executes outside the paused frame, the frame's
locals, arguments, and `.` are passed in as parameters, bare names must resolve as they would in
the frame, and every exit must hand back any values the expression changed.

## Name resolution and `global.vars`

`ExprScope` controls how bare names and calls resolve inside a compiled expression:

- A bare name resolves to a parameter, then a `src` variable, then a global, in that order.
- A bare call resolves to a `src` proc, then a global proc.
- Without a scope, other names and calls resolve as globals.
- BYOND's types declare `vars` at the root as a `const` with no value, so the compiler treats
  `vars` as a normal name (`src`'s variable, or else a global) rather than the root `const`.

A bare name alone cannot distinguish a global from a type's static variable of the same name, so
BYOND's `global.vars` list is the only exact way to reach globals. That list is read-only, though
its entries can be assigned. The scope therefore passes BYOND's `global.vars` list in as a named
parameter (`ExprScope::globals`). Inside the expression, `global.vars` reads that parameter as a
value unless a real global named `vars` exists, allowing its entries to be read and assigned.

## Return values and places

- Every exit of the compiled proc returns both the expression's result and the parameters so the
  caller can copy modified values back into the frame.
- `End` returns `.`. When `.` is one of the parameters, that parameter is the frame's own `.`.
- When debugger code uses the place of `a?.b?.c ||= v` or `&&=` as a value, each `?.` and `?[`
  jumps to the end of the expression and leaves a place on the stack rather than a value
  (`safe_place`).

The caller in the debug server must pass parameters in the order the compiled proc expects and copy
returned values back into the frame. Because a debugger expression has no source file, its
diagnostics have no file name and the caller supplies one.

## Applies to

- `dmasm/compiler/src/compiler.rs`: `ExprScope`, exit rewriting in `compile_statements`,
  `safe_place` field
- `dmasm/compiler/src/compiler/follow.rs`: `Global` arm
- `dmasm/compiler/src/compiler/procs.rs`: `vars` check in `const_lookup`
