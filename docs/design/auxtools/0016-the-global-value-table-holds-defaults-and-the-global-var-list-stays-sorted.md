# Global variable defaults and the sorted global variable list

BYOND's global value table is indexed by variable ID and holds global variables, static variables,
and variable defaults. The code does not show that new local variables start from these values.
Separately, BYOND's global variable list is a sorted misc entry of `(variable ID, flags)` pairs that
`global.vars` searches with binary search.

- **Writing global values:** `set_global_value` and `set_global_words` write the slot for a variable
  ID. Because that slot is also the variable's default, writing it changes the default that other
  code starts from. `set_global_value` acquires a reference to the new value, whereas
  `set_global_words` takes over the caller's reference. Both release the old value.
- **Adding global variables:** `add_global_vars` inserts each new pair in sorted position, interns
  the result as a new misc entry, and points the global at it. The sort key is the string ID of each
  variable's name rather than the text of the name. Adding a pair out of order would cause
  `global.vars` to miss variables.

## Applies to

- `auxtools/src/runtime_tables.rs`: `global_values`, `global_vars`, `set_global_value`,
  `set_global_words`, `global_value`, `add_global_vars`, `add_variable`
