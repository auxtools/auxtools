# Rebuilding the global init proc for reboots after a hot reload

DreamMaker stores constant defaults of global and static variables in the `.dmb`'s variable table
and generates a single proc to initialize the rest before the map is created. After a hot reload,
the running world still holds the build's variable table and init proc, so a reboot would restore
the old values. To prevent that, the reload compiles its own global init proc (`GlobalInit`) for the
world to run on reboot.

If `GlobalInit` fails to compile, changed initializers require a rebuild instead of a reboot (see
[all-or-nothing hot reloads and deferred changes](0001-a-reload-is-applied-whole-or-not-at-all.md)).

## Structure of `GlobalInit`

The overrides of a proc share the scope of their static variables, and a proc's statics are
numbered by name within that scope, with `const` locals numbered alongside them. `GlobalInit`
captures both the initialization bytecode and the slot metadata the running world needs:

- Its code first writes changed variable-table defaults (setting any value that is now computed to
  `null`), and then runs the computed initializers in
  [dmasm's initializer order](0005-dmasm-orders-global-initializers-its-own-way.md).
- Each assignment is a `SetVar global(name)` paired with its declaring scope (`StaticScope`).
- `proc_statics` counts, for each proc path, the statics of each name. `const` locals are counted
  along with them, matching proc-scope numbering so the counts align with the world's slots.
- `owners` has one entry per `SetVar` in the code, in order. Each entry names the proc whose static
  that `SetVar` writes (by the proc's key in the running world), or is `None` for a global or a
  type's static. Because overrides share their statics' scope, the proc identifies whose slots the
  initializer sets and reads.

The running world relies on `proc_statics` and `owners` to find the slots each assignment writes.

## Matching old and new proc statics

Before old and new initializers are compared, the old code's proc statics are updated to use the
new snapshot's proc keys and the numbers of the new statics they match. Statics are matched by
their enclosing blocks, the same way the launcher matches slots (see
[matching `static` and `const` locals by enclosing blocks](0016-static-and-const-locals-are-matched-by-their-blocks.md)).
Statics in scope of those initializers receive the new numbers as well. An old static with no match
gets a number that no new static uses, and a static belonging to a removed proc gets no proc. Each
new static's initializer is therefore compared with the old initializer of the static it matches,
rather than whichever static previously held its number.

## Applies to

- `dmasm/reload/src/reload/global_init.rs`: `GlobalInit`, `StaticOwner`, `owners`, `proc_statics`,
  `renumber_old_statics`, `global_init_proc`
