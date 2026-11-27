# Two-stage initialization and world-load deferral

auxtools initializes in two stages. The full initialization locates BYOND's functions and installs
detours, while the partial initialization reads the proc table, sets up the bytecode manager and
interned strings, and installs compile-time hooks. A partial shutdown ends a world, and the partial
initialization runs again for the next world.

## Stage ordering and per-world tables

BYOND allocates a fresh `variable_names` table for each world, so caching its address from the full
initialization would leave auxtools pointing at the previous world's table. `resolve_partial`
therefore reads the address of `variable_names` and its count on every partial initialization, and
shutdown clears both pointers. The count sits 4 bytes before the table pointer on Linux from
`515.1644`, and 4 bytes after it on all other builds.

Functions registered for the full initialization always run after the partial initialization stage
so that the proc table and compile-time hooks are already ready when they execute:

- `auxtools_init` runs both stages in a single call.
- `init_on_world_load` runs the full initialization early and sets `FULL_INIT_FUNCS_PENDING`. The
  registered full-init functions then run at the first proc call, after the partial initialization
  stage completes and before the partial-init functions run.

## Waiting for world load to finish

`init_on_world_load` finishes initialization on BYOND's thread at the first proc call. However,
while a world is still loading, BYOND runs initialization procs on `InitVarRecorder` values (tag
`0x3D`) while holding live pointers into the type table, and running deferred initialization during
that phase could reallocate the type table under BYOND's pointers.

To prevent that, `on_proc_call` skips any call whose `src` is an `InitVarRecorder`. The next proc
call after those init procs still occurs before global init and `world/New`. After
`init_on_next_world_load` is called with an added proc, `on_proc_call` also waits until that proc's
entry is gone so the deferred initialization does not run in the world that is still ending.

## Applies to

- `auxtools/src/symbols.rs`: `resolve_partial`
- `auxtools/src/lib.rs`: `init_with`, `init_on_world_load`, `init_on_next_world_load`,
  `FULL_INIT_FUNCS_PENDING`, `WORLD_LOAD_PENDING`, `ENDING_WORLD`, `on_proc_call`,
  `shutdown_partial`
