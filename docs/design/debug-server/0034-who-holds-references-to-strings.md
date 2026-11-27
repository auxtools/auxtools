# String reference ownership in world tables

BYOND counts references to strings, and a reload writes string IDs into proc entries, type records,
mob entries, variable tables, built-in value lists, and bytecode. If a string's reference count
drops to zero while something still holds its ID, that ID can name a different string later. The
debugger tracks every string it installs in world tables and releases a reference only when
replacing its own string, while strings interned for assembled bytecode have their references
leaked for the lifetime of the world.

## BYOND's references and table tracking

BYOND manages string references differently across structures:

- Strings from the `.dmb` hold no reference that the debugger may release.
- When BYOND adds a proc entry, it takes a reference to the path and to each setting string, but
  none to the basename. Replacing one of those strings later releases BYOND's reference.
- An existing variable's entry holds a reference to its name string.
- The world's table of variable names holds a reference to each built-in variable's name string.
- Bytecode refers to strings by ID and holds no counted reference.

The debugger records every string it installs with one reference each and releases a reference only
when replacing a string it owns:

- `PROC_STRINGS`: strings in proc entries, keyed by proc and word. For a new proc, the path and
  setting strings created by the reload are recorded as held, while the debugger leaks its own
  reference to the basename.
- `TYPE_STRINGS`: strings in a type's record or mob entry, keyed by type and place.
- `TABLE_STRINGS`: strings in type variable tables created by the reload, keyed by type and
  variable name.
- `LISTED_STRINGS`: built-in value list entries written by the reload, keyed by type and name. Each
  entry holds a reference to its name and to its text value.

Strings from the `.dmb` are never released. A variable table that names an existing variable or a
built-in variable takes no reference of its own for the name. All tracking tables are cleared when
the world ends.

## Strings interned during assembly

When the assembler requests a string's ID, `get_string_index` interns the string and leaks the
reference immediately so the string is never released while code using it can still run. Replaced
code stays in memory until the world ends.

Because `get_string_index` leaks on every lookup before assembly finishes, several cases accumulate
unreleased references:

- A lookup leaks a reference even if its code is never installed, such as when reload assembly
  fails on a later instruction. Code assembled for an evaluation runs once and is replaced by the
  next evaluation, and its references are leaked as well.
- `get_variable_name_index` calls `get_string_index` before searching the world's table of variable
  names, so a variable-name lookup leaks a reference whether or not it finds the name. A found name
  is an index into that table, and the code holds no string ID for it.
- Each lookup of the same string adds one more reference.

As a result, strings interned for assembled code are never released, and neither are strings looked
up for code that failed to assemble or for an evaluation.

## Applies to

- `debug_server/src/hot_reload.rs`: `PROC_STRINGS`, `hold_setting`, `TYPE_STRINGS`,
  `TABLE_STRINGS`, `hold_table_strings`, `LISTED_STRINGS`, `hold_listed`, `inherit_listed`,
  `append_entries`
- `debug_server/src/reload_inherit.rs`: `copy_table_entry`
- `debug_server/src/assemble_env.rs`: `get_string_index`, `get_variable_name_index`
