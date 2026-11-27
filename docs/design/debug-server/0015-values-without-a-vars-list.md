# Showing values without a `vars` list

The Variables view displays an object's variables, but several BYOND value types do not expose a
normal `vars` list:

- On BYOND 516, the primitive types `vector`, `pixloc`, `callee`, and `savefile` have no `vars`
  list.
- A `callee`'s fields can be read only while its proc is running.
- BYOND does not reference-count a `savefile`'s `vars` list and reports "Bad ref" when it is
  released.
- Reading `vars` on a typepath returns a value, not a list.
- In an `alist`, numbers act as keys, so entries cannot be reached by position.

To handle these cases, the debug server inspects them without relying on a standard `vars` list:

- `primitive_fields` hardcodes the fields of the BYOND 516 primitive types:
  - `vector`: `x`, `y`, `z`, `len`, `size`
  - `pixloc`: `x`, `y`, `z`, `step_x`, `step_y`, `loc`
  - `callee`: `proc`, `file`, `line`, `src`, `usr`, `args`, `caller`
  - `savefile`: `type`, `name`, `cd`, `dir`, `eof`, `byond_version`, `byond_build`
- A `savefile`'s `vars` list is never kept.
- `is_object` treats a value as an object only if it has known primitive fields or its `vars` is a
  list. `global` (the world value with ID `1`) also counts as an object.
- Entries in an `alist` are read as key and value pairs, and their expressions index by key only.

## Applies to

- `debug_server/src/server/values.rs`: `primitive_fields`, `is_object`, the `alist` branch of the
  list reader
