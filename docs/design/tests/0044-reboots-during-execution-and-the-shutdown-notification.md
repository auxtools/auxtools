# Reboots during execution and the shutdown notification

A proc that wakes between `world/Del` and the next world's load still belongs to the old world, so
the debugger waits for the new world before applying anything to it. From `world/Del`, the debug
server sends "the world is shutting down" (`Response::ShuttingDown`) to clients only when the world
is genuinely stopping, meaning no reboot is under way and `Del()` did not fail.

A reboot sends no shutdown notification, and clients reconnect to the next world. The debug adapter
reconnects after any closed connection while the world process stays alive, so the notification
only controls whether the close is reported as a lost connection: once it has seen
`Response::ShuttingDown`, the adapter does not print "Waiting to reconnect". Sending the
notification during a reboot would hide a real lost connection afterward, while omitting it on a
real shutdown would make every normal stop look like a lost connection.

`launcher_test`'s `reboots` covers both behaviors:

- **Mid-loop reboots:** Runs `auxtools reboot` eight times, waiting 0, 13, 26, and so on up to
  91 ms before each reboot. Because each round of the attach host's loop takes 100 ms, the reboots
  land at different points in the world's work. Afterward, the world must be running and evaluate
  `6 * 7`, and the launcher's log must contain neither "ID mismatch" nor "panicked".
- **Shutdown log check:** Runs `auxtools stop` on a world whose `Del()` sleeps and verifies that
  the log output written since the stop contains no "Waiting to reconnect". Restricting the check
  to that slice avoids counting the earlier reboots in the test, which occur before any shutdown
  notification and cause clients to reconnect.

## Applies to

- `tests/launcher_test/src/main.rs`: `reboots`
- `debug_server/src/launched.rs`: `world_del`
- `debug_server/src/server/sessions.rs`: `world_shutting_down`
- `auxtools-cli/src/adapter/lifecycle.rs`: `connection_closed`
