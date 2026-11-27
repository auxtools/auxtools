# Restoring proc overrides before BYOND builds proc name maps

Early in world startup, BYOND has not yet built its proc name maps. Inserting an entry into a
type's map before BYOND builds it causes a duplicate entry once BYOND constructs the map, and the
world's own name map is built after global init and before `world/New`, so any edits written to it
earlier are lost.

Because a hot reload restored by the launcher after a reboot can add an override, such as a new
`/datum/rb_over/New()`, before these maps exist, auxtools leaves a type's map untouched while BYOND
has not built it and delays edits to the world's map until the first proc call after it is built.

`dap_test`'s `override_restored_before_map` tests this by building a world with a map file that
places one `/obj/placed`, adding `/datum/rb_over/New()` via a reload, and rebooting twice. After
each reboot, the placed object must have been created exactly once, and `new /datum/rb_over` must
run the restored override once.

## Applies to

- `tests/dap_test/src/groups/reloading.rs`: `override_restored_before_map`
- `auxtools/src/runtime_tables.rs`: `set_named_procs`, `world_names_built`
- `debug_server/src/hot_reload.rs`: `edit_world_names`
