# Connecting to worlds over local sockets

The debug library in a running world listens on a local socket (a named pipe on Windows) named after
the world's PID: `auxtools-debug-{pid}`. Because worlds do not accept connections while loading or
rebooting, can stall inside a single DM instruction, and can accept and immediately drop connections
while exiting, clients retry while a world loads and enforce deadlines on every stage.

## Retrying, handshakes, and unavailable worlds

- **Retrying while not listening:** A world does not accept debugger connections while it loads or
  reboots, so a loading world looks the same as a world that has not started yet. `when_listening`
  retries the connection every 100 ms (`LISTEN_INTERVAL`) until it succeeds, the process exits, or
  the caller's deadline passes.
- **Bounded named pipe waits:** Because a shutting-down world may never open another named pipe
  instance, opening a connection waits at most 1 second for a pipe instance rather than blocking
  indefinitely.
- **Hello deadline:** The debug library's listener sends the hello as soon as it accepts a
  connection, regardless of what the world is doing. Only a stopped process takes longer, so
  `read_hello` requires the hello within 5 seconds (`HELLO_PATIENCE`) and otherwise fails with
  `"the debug server did not send its handshake within 5 seconds"`.
- **Unavailable worlds:** A world whose debugger cannot serve sends
  `Response::Unavailable { reason }` after the hello and then `Disconnect`. Clients report the
  reason and leave the world untouched, while the world continues running without the debugger.

## Request deadlines, reconnects, and process liveness

- **Request deadlines:** The debug server answers requests only between DM instructions, so a world
  stuck inside one instruction never answers. Probes run with a 5-second deadline
  (`PROBE_PATIENCE`), and debug adapter requests time out after 60 seconds (`RESPONSE_TIMEOUT`). A
  world runs at least one instruction per tick, so a responsive world answers well within those
  limits.
- **Quiet reconnects:** A reboot disconnects every client, and a connection made during a reboot
  waits until the next world has loaded. After an unrequested disconnect, the debug adapter reports
  once that it is waiting to reconnect (saying nothing when the world is shutting down or
  rebooting). It then keeps retrying without reporting individual failed attempts until it connects
  or the process exits. While a process is exiting, its socket can still accept connections and
  close them immediately, which counts as a failed attempt.
- **Process liveness:** On Linux, `kill(pid, 0)` succeeds for a zombie process, and the launcher
  reaps only its own children. `alive` therefore also reads `/proc/PID/stat` and treats state `Z` as
  exited. On Windows, `alive` opens the process and checks whether it has ended.
- **Message receive deadlines:** Because std's `Receiver::recv_timeout` returns a queued message
  even after its timeout has passed, `Client::next` checks the deadline itself before each receive.
  Otherwise, a world that keeps sending events would keep a caller reading past its deadline.

## Applies to

- `auxtools-protocol/src/hello.rs`: `world_socket_name`
- `auxtools-protocol/src/messages.rs`: `Response::Unavailable`
- `auxtools-cli/src/local.rs`: `connect_to`, `connect`, `open`, `read_hello`, `HELLO_PATIENCE`,
  `PROBE_PATIENCE`, `probe`, `alive`
- `auxtools-cli/src/client.rs`: `when_listening`, `LISTEN_INTERVAL`, `Client::next`
- `auxtools-cli/src/debuggee.rs`: `RESPONSE_TIMEOUT`
- `auxtools-cli/src/adapter/lifecycle.rs`: `connect`, `connection_closed`
- `debug_server/src/server/sessions.rs`: `Acceptor`, `refuse_on`
