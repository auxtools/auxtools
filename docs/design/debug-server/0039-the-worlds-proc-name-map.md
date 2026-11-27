# The world's proc name map

Procs on `/world` do not appear in any type's name map. The world maintains a name map of its own.
That map lists later definitions of a proc first, after any pinned procs. The launcher's helper
overrides are pinned (see [the launcher's helper procs](0022-the-launchers-helper-procs.md)). BYOND
builds the world's map after global initialization and before `world/New`, overwriting any edits
made earlier.

Hot reloads edit the world's map according to those constraints:

- Renaming a `/world` proc edits the world's map instead of a type map. The renamed proc is removed
  from its old key and inserted at the position that preserves the map's order: after the pinned
  procs and its own later definitions, and before the rest.
- A new `/world` proc that is not a verb is placed in the world's map by override number.
- Edits to the world's map wait until BYOND has built it. A reload applied before then queues its
  world name edits and applies them at the first proc call after the map is built.

## Applies to

- `debug_server/src/hot_reload.rs`: `Rename`, `renamed`, `rename_world_proc`, `edit_world_names`,
  `make_waiting_world_edits`, `PINNED_WORLD_PROCS`, and the `/world` branch of the new-proc step
