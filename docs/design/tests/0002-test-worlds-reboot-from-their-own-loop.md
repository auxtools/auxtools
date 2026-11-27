# Rebooting test worlds from their own loop

Test worlds trigger `world.Reboot()` from their own `main_loop` rather than directly inside a
debugger evaluation, because an evaluation runs inside the debug server's request handling and a
reboot shuts the debug server down.

Calling `world.Reboot()` directly from an evaluation would ask the debug server to shut itself down
from inside its own request handling, which it cannot do. Instead, a test that needs a reboot
evaluates `request_reboot()`, which only sets `reboot_requested`. The world's `main_loop` sees that
flag, exits its loop, and calls `world.Reboot()`.

Because the reboot occurs a tick or so after the evaluation returns, tests wait for the connection
to close before connecting to the new world.

## Applies to

- `tests/debug_test_host/debug_test_host.dm`: `request_reboot`, `main_loop`
- `tests/hot_reload_test_host/main.dm`: `request_reboot`, `main_loop`
