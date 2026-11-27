# Resolving proc paths as DreamMaker does

Reloaded code names procs by path and must resolve them using the same rules as DreamMaker, which
differ depending on whether `/proc/` or `/verb/` appears in the path.

## DreamMaker and BYOND rules

- BYOND keeps a list of overrides for each proc path. An override written in the source before the
  declaration comes first in that list.
- A path with `/proc/` or `/verb/` names the declaration.
- A path without `/proc/` or `/verb/` names the first override whose own path is exactly that path.
  DreamMaker refuses the path when the declaration is the only definition.
- `text2path()` does not resolve a proc path without `/proc/` or `/verb/`.
- When a new declaration is added that existing declarations now reach with `..()`, those
  declarations become overrides and their paths lose `/proc/`. A new override leaves the other
  procs' paths alone.

## Resolution and relinking

- `declaration` finds the declaration of a path with `/proc/` or `/verb/` by searching for the
  entry whose own path contains `/proc/` or `/verb/`, falling back to the first entry.
- `first_override` follows DreamMaker's rule for paths without `/proc/`.
- `get_type` tries a type value first. When that fails, it calls `first_override` and builds a
  `ProcRef` value from the proc's ID.
- When a reload adds a declaration, any declarations relinked to it lose `/proc/` from their paths.
  Only a new declaration triggers this update.

## Applies to

- `debug_server/src/assemble_env.rs`: `declaration`, `declares`, `own_path`, `first_override`,
  `get_proc_index`, `get_type`
- `debug_server/src/hot_reload.rs`: the relinking step of the new-proc code
