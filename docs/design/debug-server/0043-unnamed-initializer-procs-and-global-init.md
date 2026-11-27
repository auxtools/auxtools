# Unnamed initializer procs and global init

DreamMaker generates procs without paths for global and static initializers, for `in` lists, and
for type variable initializers. BYOND names type initializers `/TYPE/proc/(init)` after its initial
scan when the world loads, so a proc read early can be unnamed at first and acquire a name later.
DreamMaker emits the global init proc after the type init procs and `in` list procs, so any unnamed
procs numbered after it come from hot reloads. BYOND also names a map instance's init proc
`TYPE/proc/(map_init)` when it loads a world. In some codebases, such as tgstation, map areas
initialize before global init runs.

## Naming and identifying unnamed procs

- When a cached proc has no path, the debugger reads the proc again by ID before treating it as
  unnamed. A named type initializer is shown with its path and override `0`.
- An unnamed proc in BYOND's `in` list table is shown as `(in list)`, with its index in that table
  as the override number. Any other unnamed proc is shown as `(var initializer)`.
- Global init is the highest-numbered unnamed proc that is neither an `in` list nor added by a
  reload, and it is shown under the protocol's `GLOBAL_INIT` name. A proc that is some type's init
  proc is never taken for global init.
- The journal does not cache a frame whose proc still has a placeholder name starting with `(`, so
  the frame receives the real name once BYOND assigns it.

## Prefab initializers

A prefab init proc added by a reload is constructed the same way DreamMaker builds them: a copy of
a template entry with its `name`, `desc`, `category`, `basename`, parameters, and next override set
to `NONE`, and its path set to `TYPE/proc/(map_init)` (the name BYOND would give it).

## Reloading global init

A changed global init applies only while a launched world is starting and its resources have not
loaded yet. In that window:

- If the world has no global init proc, the new initializer runs immediately through the evaluation
  stub.
- If global init is on the running stack and is not the starting call, it has already run, so the
  reload warns that a rebuild is needed. Another start proc on the stack does not count, because
  map areas can initialize before global init.
- Otherwise, the global init proc receives the new code, and if it is the starting call, startup
  runs the new code.

Outside that window, the reload warns that global initialization has already run. A reload that
replaces global init drops the separate initializers of its new variables.

## Applies to

- `debug_server/src/unnamed.rs`: `proc_ref`, `UnnamedProcKinds`, `IN_LIST`, `OTHER`
- `debug_server/src/server/journaling.rs`: `journal_stack`
- `debug_server/src/hot_reload.rs`: `StagedInstances::find_or_add`, `proc_template`,
  `prepare_global_init`, `global_init_proc`
