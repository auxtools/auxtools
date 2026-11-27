# The launcher's helper procs

When the debugger attaches to a launched world, it uses a hot reload to install helper procs: global
procs such as `/proc/auxtools_expr_stub`, `/proc/auxtools_keepalive`, and the spawn helpers, along
with overrides of `/world/New`, `/world/Reboot`, and `/world/Del` that report to the debugger.

## Preserving world definitions and ordering overrides

A world may already declare some of the global helper procs itself, and a later hot reload can add
or change the world's own definitions of the same world procs:

- A global helper that the world already declares is not added again, so the world's definition
  stays. A world with its own `/proc/auxtools_keepalive` keeps it and then cannot run spawned code.
- The helper overrides of world procs are added after the world's own definitions. The helper
  `/world/Del` calls `..()` first, so the world's `Del` runs before the helper reports.
- The helper overrides are left out of source override numbering with `Proc::set_apart` and pinned
  in the world's proc name map. A reload that adds or changes world definitions places them below
  the helpers, so the helpers keep running first.

## Reboot reporting and next-world initialization

During a reboot, BYOND proceeds with the reboot even if `world/Del` throws. A reboot that proceeds
never returns from `..()` in `/world/Reboot`. When `..()` returns or throws, the world declined
the reboot (for example, in an override of its own that does not call `..()`).

- The helper `/world/Reboot` reports the reboot before calling `..()`. If `..()` throws, it reports
  that the reboot was declined and throws the error again. If `..()` returns, it reports that the
  reboot was declined.
- On a reboot, the helper `/world/Del` arms initialization for the next world. After that point, a
  proc call in the ending world could be mistaken for the next world's first proc call. As described
  in [two-stage initialization and world-load deferral](../auxtools/0020-when-each-init-stage-runs-and-what-the-world-load-init-waits-for.md),
  `init_on_next_world_load` guards against this only when it finds the helper `/world/Del` override.
  The helper therefore passes its own set-apart override so the post-load step can tell the ending
  world from the next one.

## Keepalive loop and spawn queue

`/proc/auxtools_keepalive` sets `waitfor = FALSE` and sleeps immediately. Calling it during attach
starts its loop and returns control to attach right away.

The keepalive loop serves as the queue for spawned evaluations. It obtains a list from
`/proc/auxtools_spawn_inbox` once, and the debug server adds an entry to that list for each spawn
it queues. Each tick, the loop reads only the list's length. Only when the list has entries does it
empty the list first and call a proc, starting with `/proc/auxtools_spawn_next`. An idle loop
therefore makes no proc call while the world reboots and cannot trigger the debugger's
initialization in the ending world.

## Applies to

- `debug_server/src/launched.rs`: `HELPER_PROCS`, `HELPER_OVERRIDES`, `helper_procs`,
  `add_helper_procs`, `start`, `world_del`, `world_reboot`, `world_reboot_declined`, `spawn_inbox`,
  `spawn_next`
- `debug_server/src/server/spawn.rs`: `spawn_inbox`
- `debug_server/src/server/eval.rs`: queueing of a spawn
- `debug_server/src/hot_reload.rs`: `pin_world_proc`
