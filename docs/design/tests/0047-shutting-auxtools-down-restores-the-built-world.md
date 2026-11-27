# Restoring the built world when auxtools shuts down

A world not started by the launcher does not keep hot reloads across a reboot. When auxtools shuts
down, it restores the state the world had before any reload; BYOND then frees the restored data
itself, and the rebooted world starts fresh from its `.dmb`.

`hot_reload_test` verifies this cleanup by reloading several procs and a variable default, checking
that the lines with code have changed, and rebooting. After the reboot, `world/Del`'s override must
have run, every reloaded proc and variable default must return its built result again, and the
lines with code in `reload.dm` and `main.dm` must match the build.

## Applies to

- `tests/hot_reload_test/src/main.rs`: "reboot after reload" checks
