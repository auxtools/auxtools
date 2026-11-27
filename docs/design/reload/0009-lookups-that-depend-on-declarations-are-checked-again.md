# Re-checking type-dependent lookups when declarations change

DreamMaker resolves fields and procs in a chain from the declared types it knows at compile time,
and falls back to an untyped lookup where no declared type is available. Hot reload tracks
declaration, return-type, and destination-type changes so any code whose compile-time lookup can
change is checked again.

## Untyped lookups

In three positions, DreamMaker has no type and resolves the name as an untyped `x:name` lookup,
matching a declaration of that name on any type:

- after a colon
- immediately after an index, because DreamMaker ignores the list's element type
- immediately after a call, because DreamMaker may have no type for the proc's return value

`Changes::colon_names` tracks variable names that any type starts or stops declaring. Code that
performs an untyped `x:name` or `x:name()` lookup of such a name in any of the three positions
above is checked again. As a result, adding or removing a variable declaration anywhere can
recompile distant code that uses the same name after a colon, an index, or a call.

## Declared return types and `astype()`

Two other lookups depend directly on declared types:

- **Proc return types:** A field read through a proc's return value is resolved from the proc's
  declared return type, so a changed proc return type is tracked like a retyped variable.
- **`astype()` assignments:** DreamMaker casts `astype(x)` to the declared type of the variable it
  is assigned to regardless of `x`'s type, including through `||`, `&&`, and `? :`. An `astype(x)`
  assigned to a variable (directly or through `||`, `&&`, or `? :`) counts as a read of that
  variable's type, because changing the variable's declared type changes the compiled code.

## Applies to

- `dmasm/reload/src/reload.rs`: `Changes::colon_names`, `colon_fields`, `colon_calls`,
  `colon_names_of`, `implicit_type_changes` (`destination_casts`), return type check in
  `compare_types`
