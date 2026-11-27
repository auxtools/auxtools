# Changing a type's parent

A reload can change the parents of several types at once, including moving a type into another
subtree whose own parent also changes. In a proc entry, `NONE` as the next override marks a
declaration or an override of a built-in proc such as `New()` that has no code. When a type changes
parent, DreamMaker either refuses such a proc or links it to the new parent's proc.

Reparenting stages the new type tree and name maps in a fixed order before writing anything to the
world (see [preparing reloads before writing to the world](0025-a-reload-never-frees-code-that-may-still-run.md)):

- All new parents are looked up by path before any change is staged, so every lookup sees the world
  as it was.
- A new parent that is a subtype of the moved type is refused.
- Parent changes are staged from the outermost type inward, sorted by depth, so a nested change
  sees the ancestry given by the outer change.
- For each of the type's own procs whose next override lies outside the type's own chain, the
  reload sets the next override to the first proc of that name in the new parent's lookup, unless
  that lookup entry is marked as not linking. Otherwise, it sets `NONE`.

## Applies to

- `debug_server/src/reload_reparent.rs`: `prepare`, `prepare_one`
