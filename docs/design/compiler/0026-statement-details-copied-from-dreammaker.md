# Statement compilation details matched to DreamMaker

Several DM statements compile in ways the source syntax does not suggest, and several of those
details affect runtime behavior. dmasm replicates each of them so compiled procs behave like
DreamMaker's:

- **Local declarations:** After the first `for` loop with a head, label, `switch`, or `pick()` in a
  proc, DreamMaker initializes local declarations without a value to `null`. Before that point, it
  does not. A `for ()` with no head does not count.
- **Assignment operator overloads:** DreamMaker begins the body of an assignment operator overload,
  such as `operator+=`, with `. = src` at the header's line, so an overload that returns nothing
  leaves the variable unchanged.
- **`del`:** DreamMaker clears the reference before deleting the object, and deleting `src` ends
  the proc.
- **Folded `if` chains:** When an `if` chain folds, a false arm after the last live arm still
  emits its jump to the end, as if an `else` were present.
- **`switch`:** DreamMaker lists the values of each arm from last to first. A case of `-0` is
  stored as `0`, which the compiler ensures by adding `0.0` to the value.
- **`in` after an assignment:** `x = a in L` parses as `(x = a) in L`, and DreamMaker ignores the
  result of the `in` test.
- **Parameter defaults:** DreamMaker reads a parameter default written without spaces as a path
  when it is a variable, or a field of a variable such as `a.b`, divided by a number, as in `a/2`.

## Applies to

- `dmasm/compiler/src/compiler/procs.rs`: `clear_new_vars` in `ProcCtx` (set in
  `Compiler::emit_ins` and `emit_statement`)
- `dmasm/compiler/src/compiler/statements.rs`: overload start, folded `If`, `Switch` and `Del`
  arms, `constant_value`, `in` case of `emit_expr_statement_inner`, `path_division`
