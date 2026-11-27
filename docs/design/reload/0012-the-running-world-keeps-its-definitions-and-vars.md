# Definition order and retained procs and variables in the running world

A type's proc can have a declaration and multiple overrides, numbered in the running world in an
order fixed at build time, with DreamMaker placing the declaration at the type's first declaration
of the proc. Meanwhile, old bytecode still calls global procs by ID and may remain active on a call
stack after a hot reload, and a running build may be older than the old snapshot being compared, so
a variable present in the old snapshot might not exist in the world.

These constraints determine how hot reload handles definition order and removals:

- **Definition order:** An edit that changes the order of a proc's existing definitions is refused
  (`DefinitionOrderChanged`), because moving a declaration past an override would make a fresh
  build invoke them in a different order than the running world. When definitions are inserted or
  removed, the plan keeps source order and the runtime renumbers the definitions after applying the
  plan.
- **Removed type proc definitions:** A removed definition of a type's proc is unlinked from type
  lookup and override chains so calls reach the definition it overrode.
- **Removed global proc definitions:** A removed global proc definition is removed only from the
  numbering and kept in the running world because old bytecode may still call it. The reload notes
  this with `RemovedProcRetained`.
- **Removed variable declarations:** A removed variable declaration stays in the running world.
  Because only the world knows whether it has the variable, removed variables are queried rather
  than modified, and the reload notes this with `RemovedVarRetained` or `RemovedGlobalRetained`.

## Applies to

- `dmasm/reload/src/reload.rs`: `RemovedProc`, `RemovedVar`, `definition_retained`, `var_retained`,
  definition checks in `compare_types`
