# Adapter main-thread loop, prompt disconnects, and launched-world shutdown

The debug adapter runs its request loop on the main thread, lets an incoming disconnect cut short
any in-flight wait on a stuck world, and always stops a world that the session itself launched.

## Main-thread loop

On Linux, `Game::start` sets `PR_SET_PDEATHSIG` to `SIGKILL` in `pre_exec`. Linux delivers that
signal when the thread that spawned the child exits, not when the parent process exits. Running the
adapter loop (`serve`) on a worker thread would therefore kill the launched world whenever that
thread ended, so the loop runs on the main thread.

## Cutting waits short on disconnect

The adapter loop handles one request at a time, and the debug server reads no requests while
evaluating DM code, so a pause request cannot break into a long evaluation. Without a mechanism to
abandon an in-flight wait, a stuck world or long evaluation would hold up the client's disconnect.

The stdin reader thread sets the `leaving` flag as soon as it reads a `disconnect` request (via
`asks_to_disconnect`), before the request reaches the loop, and also sets `leaving` when stdin
closes. Setting the flag inside the loop would be too late while the loop is blocked waiting on the
world.

`Debuggee::request` normally waits up to `RESPONSE_TIMEOUT` (60 seconds), checking `leaving` every
50 ms (`LEAVING_LOOK`). Once `leaving` is set, it waits only until `LEAVING_TIMEOUT` (2 seconds
after the request was sent), then fails with `"stopped waiting for the debug server: the client
asked to disconnect"`. The loop then reaches the disconnect, limiting how long a stuck world delays
disconnection to roughly 2 seconds. Giving up the wait is the only way out of a long evaluation,
because a pause request would not be read until the evaluation ends.

## Stopping launched worlds on disconnect

A world launched by the debug adapter depends on the adapter to read its output. On disconnect, a
world launched by this session is stopped even if the client asked not to terminate it, and the
adapter prints `"Terminating the world launched by this session. Attach to an existing world to
leave it running on disconnect."` An attached world is stopped only when the client requests it, so
users who want a world to keep running attach to an existing world instead of launching one.

## Applies to

- `auxtools-cli/src/adapter.rs`: `serve`, stdin reader thread before `serve`, `asks_to_disconnect`
- `auxtools-cli/src/debuggee.rs`: `Debuggee::request`, `RESPONSE_TIMEOUT`, `LEAVING_TIMEOUT`,
  `LEAVING_LOOK`
- `auxtools-cli/src/adapter/lifecycle.rs`: `disconnect`
- `auxtools-cli/src/game.rs`: `pre_exec` in `Game::start`
