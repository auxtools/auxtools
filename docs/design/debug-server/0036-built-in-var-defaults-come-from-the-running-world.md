# Built-in variable defaults from the running world and variable tables

The compiler's source metadata does not always know BYOND's default for a built-in variable, so a
reload reads inherited built-in defaults from the running world's type tables. On types derived
from `/sound`, `/particles`, `/exception`, `/regex`, and `/icon`, built-in variables are stored as
entries in the type's variable table under the world's variable IDs rather than as fields of the
type record, and such a type has a variable-table entry for a built-in variable only when it sets
that variable.

- When a type stops setting a built-in variable or changes parent, its inherited built-in defaults
  are copied from the parent's entry in the running world.
- When a reload sets a built-in default on a type whose built-ins live in its variable table and
  the type has no entry for that variable yet, the reload inserts a new variable-table entry in key
  order. The entry uses the world's variable ID for the name, or a new variable name with a `null`
  value when none exists.
- When such a type changes parent, its own built-in entries move with its variable table.

## Applies to

- `debug_server/src/reload_inherit.rs`: `copy_builtin`, `copy_table_entry`, `builtin_in_table`,
  `prepare`, `apply`
- `debug_server/src/hot_reload.rs`: the variable-default step handling `builtin_in_table`
- `debug_server/src/reload_reparent.rs`: `prepare_one`
- `dmasm/reload/src/reload.rs`: `VAR_TABLE_BUILTINS`
