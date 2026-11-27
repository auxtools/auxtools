# Hooking `auxtools_launched_world_del` in the debug library

When a world declares `/proc/auxtools_launched_world_del`, the debug library hooks that proc as it
starts. Because new code for a hooked proc never runs, reloading it produces a warning.

To test this warning, the hot reload host declares `/proc/auxtools_launched_world_del` with a body
that crashes, and `hot_reload_test` reloads a new body for it and expects the reload to warn that
the proc is hooked.

## Applies to

- `tests/hot_reload_test_host/main.dm`: `/proc/auxtools_launched_world_del`
- `tests/hot_reload_test/src/main.rs`: "hooked-proc warning" check
- `debug_server/src/launched.rs`: `start`
