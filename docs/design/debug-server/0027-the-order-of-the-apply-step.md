# The order of a reload's apply step

Once `append` has added every new entry, `apply` links them into the running world. Although the
sequence is not obviously constrained at first glance, the order of its steps matters. Computed
initializers run DM code that may use anything the reload adds.

`apply` runs in this order:

1. Parent changes are applied to the type tree.
2. Removed procs are unlinked from type lists, name maps, and `..()` chains.
3. New procs are linked once every new proc has its entry, and new type init procs are set on their
   types.
4. Override numbers are renumbered.
5. New code is installed, and breakpoints are restored.
6. Renames update the name lookups.
7. Settings, type fields, built-in variables, appearances, and static values are written.
8. Computed initializers run last so they see the new procs, variables, and types. When the world's
   starting call is the global init proc, it is made to run the new code. A global init that must
   run now, along with each new variable's initializer, runs through `/proc/auxtools_expr_stub`.
   See [unnamed initializer procs and global init](0043-unnamed-initializer-procs-and-global-init.md)
   for when a changed global init applies at all.

## Applies to

- `debug_server/src/hot_reload.rs`: `apply`
