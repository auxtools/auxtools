# Stopping and rebooting a world through DM evaluation

The launcher stops or reboots a world by evaluating `world.Del()` or `world.Reboot()` rather than
terminating the OS process, so the world's own shutdown and reboot code runs. That DM code can
sleep, raise a runtime error, or (in the case of `Reboot()`) decline or delay the reboot. A world
can also be paused when asked to stop, and while paused it would never get to run its shutdown code.
A successful `world.Del()` or `world.Reboot()` closes the debugger connection without sending an
answer, so code that waits for a response will always time out on success.

## Sending the request

- `world.Del()` and `world.Reboot()` are sent as `Request::Eval` with `ending: true`. That flag
  resumes a paused world and suppresses pauses so code that sleeps during shutdown, or code that
  stops or reboots the world, can run to completion.
- Only the debug adapter's own `world.Del()` and `world.Reboot()` requests, and the CLI's `stop`
  and `reboot` commands, set `ending`. Evaluations typed by the user never set it.
- A world that obeys closes the connection without answering. Receiving an answer means the call
  slept or failed with a runtime error. After a failed `world.Del()`, the world continues running
  and keeps its debugger.

## Distinguishing a shutdown from a reboot

When a world stops, the debug server sends `Response::ShuttingDown` to every connected client,
followed by `Disconnect`, and waits up to 1 second for its writers to drain. It sends no
`ShuttingDown` for a reboot, which allows clients to tell a shutdown apart from a reboot.

## Debug adapter behavior

- **Stop:** The adapter sets `ending`, sends the request without waiting for an answer, and watches
  the world process. If the process has not exited after `TERMINATE_GRACE` (5 seconds), the adapter
  prints `"Shutdown timed out after 5 seconds. Terminating process PID."` and terminates the
  process.
- **Launched world stopping on its own:** Receiving `ShuttingDown` starts the same 5-second timer.
  If the process the adapter launched is still running when the timer expires, the adapter prints
  `"The world shut down, but its process is still running after 5 seconds. Terminating process
  PID."` and terminates it. The adapter does not reconnect to such a world when its connection
  closes. A DreamDaemon that has given up on BYOND's hub, or has no network, waits 20 seconds after
  its world ends before exiting, and the debugger's listener in that process still accepts a
  connection and sends its hello. Reconnecting would leave the adapter asking for the BYOND version
  and waiting for an answer that never arrives, unresponsive to its client until the process exits.
  An attached world's process is not the adapter's to terminate, so nothing changes for it. A
  launched world's session therefore ends at most 5 seconds after the world does, regardless of what
  DreamDaemon waits for.
- **Reboot:** The adapter sends the request, sets `rebooting`, and waits for the world to
  reconnect. Because `Reboot()` may decline or delay, it starts a timer of `REBOOT_PATIENCE`
  (60 seconds). If the world has not reconnected by then, the adapter clears `rebooting` and reports
  that `world/Reboot()` may still be running or may have returned without rebooting.
- **Overlapping state flags:** Ending, rebooting, shutting down, unavailable, and waiting for code
  are tracked as separate flags rather than a single enum because these states overlap (for example,
  a world asked to stop can also be unavailable). `ending` remains set after the world stops until a
  relaunch starts the new process and clears it. While `ending` is set, `restart` and `reboot` are
  refused with `"the world is already stopping"`, and a closed connection is not treated as a
  disconnect to reconnect from.

## CLI `stop` behavior

The CLI `stop` command sends the request and waits for the process to exit, checking every 50 ms
until its timeout expires:

- An evaluation error that does not start with `runtime error: ` indicates that the call slept, so
  the wait continues.
- Any other answer is reported as `world.Del()` failing with a runtime error. The message notes that
  the process is still running and suggests fixing `world/Del()` or running `stop` again with
  `--kill`.
- Because the debugger can disconnect before the process exits, a lost connection is followed by a
  wait for the process to exit.

A world whose `Del()` sleeps forever is terminated by the debug adapter after 5 seconds, whereas the
CLI `stop` command leaves it running unless `--kill` is passed.

## Applies to

- `auxtools-protocol/src/messages.rs`: `Request::Eval` (`ending` field), `Response::ShuttingDown`
- `auxtools-cli/src/client/ops.rs`: `Ending`, `Ending::request`
- `auxtools-cli/src/debuggee.rs`: `Debuggee::end`
- `auxtools-cli/src/adapter/lifecycle.rs`: `Lifecycle`, `terminate`, `terminate_overdue`,
  `world_shutting_down`, `exit_overdue`, `outliving_its_world`, `reboot`, `reboot_overdue`,
  `restart`, `relaunched`, `connection_closed`
- `auxtools-cli/src/cli/stop.rs`: `ended_by`, `stop`
- `debug_server/src/server/sessions.rs`: `world_shutting_down`
- `debug_server/src/launched.rs`: `world_del`
- `debug_server/src/server/pause.rs`: ending path of `handle_breakpoint`
