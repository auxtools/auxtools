# Implicit types for `new`, `locate()`, and `astype()`

A bare `new`, a `locate()` with no type, and a one-argument `astype(x)` take their type from the
variable they are assigned to. DreamMaker determines that type using rules that reach further in
some positions, and stop sooner in others, than the source suggests, and rejects the code when no
type can be found. dmasm replicates these rules:

- DreamMaker applies the destination's type to every implicit `new`, `locate()`, and `astype()`
  inside an assignment's value, including nested operands and arguments. A nested assignment
  supplies its own destination's type instead.
- For an assignment to a list element, the type is the holder's declared type. For a list declared
  as `/list`, that type is `/list`, and `args` counts as a `/list`.
- DreamMaker types a variable read through a proc's return value, but not the return value itself,
  so `astype(make())` is an error.
- DreamMaker does not type the place when a call appears after the place's last safe access.
  `make().child = new` is an error, whereas `make()?.child = new` creates the type `child` is
  declared with.
- Before BYOND build 516.1673, DreamMaker rejects an implicit `new`, `locate()`, or `astype()`
  value in a `||=` or `&&=` through `?.`. From 516.1673 onward, it types the value as usual.

Where DreamMaker rejects a value because its destination has no type, dmasm rejects it as well,
except in debugger expressions.

The build check for `||=` and `&&=` through `?.` uses the `keeps_safe_logical_null_tests`
predicate, named after another change introduced in BYOND build 516.1673 that shares the same
threshold. Because changing a variable's declared type changes what an implicit `new` creates
elsewhere, hot reload must find and recompile any procs that take that type.

## Applies to

- `dmasm/compiler/src/compiler.rs`: `implicit_type` field, `assignment_destination`,
  `destination_type`, `place_type_name`, `implied_type`
- `dmasm/compiler/src/compiler/procs.rs`: `expr_type`, `place_typed`
- `dmasm/compiler/src/target.rs`: `keeps_safe_logical_null_tests`
