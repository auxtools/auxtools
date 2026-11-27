# Finding the slots of statics and consts

BYOND stores the statics and consts of types and procs among the globals under their own names. A
type's statics and consts appear in its declared variables, whereas a proc's appear only as the IDs
its bytecode reads for its own locals. Because BYOND names a proc static's slot after the local and
nothing in the world distinguishes it from a global's slot, several slots can share the same name.
In addition, all overrides of a proc share one scope for their static locals, so a static's scope
alone cannot indicate which override declared it.

Hot reloads disambiguate global, type, and proc slots with four rules:

- The debugger records the slots of proc statics and consts that reloads created or found in
  `PROC_LOCALS`. A slot remains classified as a local's even after the local is renamed.
- To find a true global by name, the reload first excludes IDs that are type statics, recorded proc
  locals, or IDs that procs read for their own locals.
- For a proc static in a global initializer, the reload uses the owner named by the compiler for
  each initializer to find the static's slot in the right override.
- A type static or const resolves on its declaring type before name-based guesses, taking a static
  staged by the current reload or the only slot with that name first. This matters after a type
  changes parent, because its code may have referred to another variable of the same name, and the
  rule keeps it on the declared one.

## Applies to

- `debug_server/src/hot_reload.rs`: `PROC_LOCALS`, `owned_slots`, `prepare_redeclared`,
  `ReloadEnv::get_global_index`
