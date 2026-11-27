# Early evaluation return when `world.Del()` or `world.Reboot()` sleeps

When `world.Del()` or `world.Reboot()` sleeps in BYOND, the evaluation that called it returns before
the proc finishes. The launcher's `stop` and `reboot` commands therefore wait for the world itself
rather than treating the evaluation's early completion as a finished stop or reboot.

The attach host's `world/Del` and `world/Reboot` use marker files to exercise slow and failing
shutdowns or reboots:

- `slow_del` and `slow_reboot` make `world/Del` and `world/Reboot` sleep for 0.3 seconds.
- `refuse_del` makes `world/Del` return without calling `..()`.
- `fail_del` makes `world/Del` crash.
- `refuse_reboot` makes `world/Reboot` log "attach test: reboot refused" and return.

`launcher_test` verifies two behaviors against these hooks:

- `reboot` and `stop` wait through the sleep and succeed when the world goes on to reboot or stop.
- With `--timeout 1`, `reboot` and `stop` give up with exit code `2` when the world sleeps and
  continues running, and the world can still be paused afterward.

## Applies to

- `tests/launcher_test_host/attach_host.dm`: `/world/Del`, `/world/Reboot`
- `tests/launcher_test/src/main.rs`: `reboots`
