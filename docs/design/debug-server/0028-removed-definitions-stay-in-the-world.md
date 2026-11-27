# Removed definitions stay in the world

Running frames, sleeping frames, and old call sites can still use a proc or variable that a hot
reload removes (see
[staging hot reload changes and preserving running code](0025-a-reload-never-frees-code-that-may-still-run.md)).
Removed definitions therefore remain allocated in the running world:

- A reload that removes a global proc or a non-verb proc on `/world` keeps its entry and takes it
  out of override numbering. A removed `/world` proc is also removed from the world's proc name map
  so new code no longer reaches it.
- A reload that removes or renames a type proc unlinks its name from the type's lists and name maps
  while keeping its entry for calls that still run it.
- Completion and member lists skip procs whose path no longer links to them.
- Type variables removed by a reload stay in the running world. A later reload that adds a type
  variable with the same name reuses the old declaration and emits a warning that existing static
  values do not change. It refuses the reload if the `static`, `const`, or `tmp` flags differ.

## Applies to

- `debug_server/src/hot_reload.rs`: `prepare_removal`, `prepare_new_var`
- `debug_server/src/server/values.rs`: `each_proc`
