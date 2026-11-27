# Apply order and contents of a reload plan

A `Plan` is the update `dmasm-reload` hands to the running world. Because its parts depend on one
another (a change to a type requires the type to exist, a variable default on a subtype requires
the variable to be declared first, and a replacement proc definition must not sit alongside the
definition it replaces), both the code building a plan and the receiver in the debug server must
preserve and apply these lists in a fixed order:

1. `new_types`: listed parents first, applied before every change that depends on them.
2. `reparented`: listed parents first, applied after `new_types` and before the other changes.
3. `removed_procs`: removed definitions of types' procs are unlinked before replacement definitions
   are added.
4. `new_vars`: applied before defaults on the subtypes that inherit them.
5. `redeclared`: applied after `new_vars` and before defaults.
6. `builtin_inherited`: restored from the running parents after `builtin_defaults`.

Two additional rules govern what a `Plan` carries:

- `ProcReload::nodes` always includes `DbgFile` and `DbgLine` instructions, whereas DreamMaker
  emits them only for worlds built with `DEBUG`. The receiver strips them for an existing proc that
  has none in the running world, and keeps them for a new proc only when the world's procs have
  them.
- `Plan::len` counts only the changes the plan applies to the world. Removed variables are excluded
  because the world is only queried about whether it has them (see
  [definition order and retained procs and variables](0012-the-running-world-keeps-its-definitions-and-vars.md)).
  A plan containing only removed variables is therefore empty.

## Applies to

- `dmasm/reload/src/reload.rs`: `Plan`, `Plan::len`, `ProcReload`
- `debug_server/src/hot_reload.rs`: `adapt_debug_info`
