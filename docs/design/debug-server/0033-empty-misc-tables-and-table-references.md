# Empty misc tables and locals-table references

Variable tables, declared-variable tables, proc lists, locals tables, and built-in value lists are
BYOND misc entries. Each misc entry begins with a 16-bit count, and its reference count is a `u32`
at offset 4. In a built world, an empty variable table or declared-variables table is stored as
`NONE` rather than as an empty misc entry, and a type with no procs has no list entry at all.
BYOND does not create empty misc entries well.

Hot reloads follow the same conventions and maintain locals-table reference counts accordingly:

- An empty variable table, declared-variables table, or built-in value list is stored as `NONE`.
  When a removal empties a type's proc list, the reload stores `NONE`.
- A locals table created by the reload (`LocalsTable::Made`) holds its own reference, which the
  reload transfers to the proc receiving the table.
- A locals table shared with another proc (`LocalsTable::Shared`) carries no reference for its new
  owner, so the reload takes one for the proc receiving it.

## Applies to

- `debug_server/src/hot_reload.rs`: `TypeVars::intern`, `LocalsTable`, `apply_code`, the proc list
  update in `apply`, and `misc_references` (used by test commands)
- `debug_server/src/reload_inherit.rs`: `set_listed`
