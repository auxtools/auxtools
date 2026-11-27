# Pause lifetime with multiple clients, watchers, and holds

Several clients can connect to a world at once, such as an editor session, CLI commands, and a
launcher that only reloads. A pause must end when no client remains to resume it (otherwise the
world hangs), yet a short-lived CLI command must not release a pause that an editor is inspecting,
and a CLI command that pauses the world may need the pause to outlive the command itself.

## Debugging clients and watchers

Every new connection starts as a debugging client. Sending `Watcher` turns the connection into a
watcher, which does not keep a pause alive:

- When the last debugging client disconnects, the world resumes even if watchers remain connected.
- A breakpoint pauses the world only while a debugging client is connected or a pause is held.
  Otherwise, any log message on the breakpoint is written and execution continues.
- A runtime error pauses the world whenever any connected client's runtime error filter matches it,
  whether that client is a watcher or not.

Clients that never debug register as watchers so they do not interfere with pause lifetimes:

- A session launched without debugging only launches and reloads, so it sends `Watcher` to avoid
  releasing pauses held by other clients when it disconnects.
- The CLI commands `status`, `log`, `proc`, `disassemble`, `trace`, and `profile` never stop the
  world and never need a pause to persist. They send `Watcher` immediately after connecting so a
  paused world does not wait for them. The waits for a reload and for a reboot send `Watcher`
  themselves within their own timeout.

A command that forgets to send `Watcher` keeps a paused world paused until it disconnects.

## Holding a pause

`HoldPause` keeps the current pause, or the next pause, active after the requesting client
disconnects. If the client disconnects before any pause occurs, the request is dropped. Only one
client holds a pause at a time: a later `HoldPause` from another client takes over, and the hold
ends when the pause ends.

`HoldMax { seconds }` limits how long a held pause lasts without a debugging client:

- `None`, a negative value, or a non-finite value means no time limit.
- The limit is stored per client and copied when that client sends `HoldPause`, so `HoldMax` must
  be sent before `HoldPause`.
- The timer starts at the first moment during the held pause when no debugging client is connected,
  and it does not reset if a debugging client connects later.
- Once the limit expires, the world resumes as soon as no debugging client is connected, journals
  `HoldEnded { seconds }`, and sends `Resumed` to every client.

The CLI's pause, step, and pause-wait commands send `HoldPause` so the world stays paused after
they exit. Right after connecting (before any `HoldPause`), they send `HoldMax` using `--hold-max`
or `AUXTOOLS_HOLD_MAX` when the option is omitted. The CLI accepts only a finite, non-negative
number of seconds for this limit.

## Pause ownership

- **Breakpoint stops:** `Stop::paused_by` lists the receiving client's own breakpoints at that
  instruction that requested the pause, excluding its logpoints there. A client uses this field to
  tell its own stop apart from another client's.
- **Runtime errors:** Because the world pauses whenever any client's filter matches, a CLI command
  checks the runtime error against its own filter before treating the pause as its own.
- **Non-debugging sessions:** A session launched without debugging receives no stop events, so it
  could neither observe nor resume a pause it triggered. The debug adapter refuses `pause` in that
  mode with `"debugging is disabled"`.

## Applies to

- `auxtools-protocol/src/messages.rs`: `Request::HoldPause`, `Request::HoldMax`,
  `Request::Watcher`, `Stop::paused_by`
- `debug_server/src/server/pause.rs`: `handle_breakpoint`, `let_hold_go`
- `debug_server/src/server/sessions.rs`: `recv_request`, `remove_client`, `has_debuggers`
- `debug_server/src/server.rs`: `HoldPause`, `HoldMax`, and `Watcher` arms
- `auxtools-cli/src/cli.rs`: `only_looks`, `watch_only`, runtime arm of the pause wait
- `auxtools-cli/src/cli/hold.rs`: `hold_max`
- `auxtools-cli/src/adapter/lifecycle.rs`: `Watcher` request in the launch path
- `auxtools-cli/src/adapter.rs`: `Adapter::pause`
