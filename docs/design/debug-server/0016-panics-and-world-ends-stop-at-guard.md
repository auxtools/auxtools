# Stopping panics and world shutdowns at `guard`

BYOND calls into the debug server through `extern "C"` functions, and a Rust panic unwinding into
BYOND's caller aborts DreamDaemon. Evaluated DM code can also shut down or reboot the world,
destroying the server while Rust frames that reference it are still on the stack. By the time the
call returns, the next world already has its own procs. Every entry point from BYOND therefore runs
inside `guard`, which stops both panics and world-end unwinds before they cross the FFI boundary.

- When `guard` catches a panic, `Server::recover_from_panic` drops connected clients, clears pause
  and evaluation state, and stops the profiler.
- After three consecutive panics (`LOST`), or a panic during recovery itself, the debugger gives up:
  it removes all breakpoints, stops handling instructions, and leaves the world running without a
  debugger. Any call that returns normally resets the panic count.
- An evaluation compares the world-end counter (`WORLD_ENDS`) before and after calling DM. If the
  counter changed, the evaluation does not restore `try` handlers or the stub's locals table because
  those IDs and frames no longer belong to it. Instead, it leaks its result and arguments rather
  than dropping them, and unwinds with the `WorldEnded` payload.
- `guard` catches a `WorldEnded` unwind before it reaches code that would touch the destroyed
  server, reports "the world shut down", and does not count it as a panic.

## Applies to

- `debug_server/src/lib.rs`: `WorldEnded`, `WORLD_ENDS`, `LOST`, `guard`, `guard_server`, `settle`
- `debug_server/src/server.rs`: `Server::recover_from_panic`
- `debug_server/src/server/eval.rs`: `call_with_locals`, `compile_and_run_expr`
