# Holding closed socket handles in DreamDaemon on Windows

On Windows, DreamDaemon keeps callbacks for links whose sockets were closed. When Windows reuses a
closed socket's handle, hub traffic can go to a client link, or the hub's 60-second timeout can
close another client's socket.

In a launched world on Windows, the debugger replaces `byondcore.dll`'s imports of `wsock32.dll`'s
`closesocket` with its own function. That replacement function:

1. Shuts the socket down immediately with `SD_BOTH`, ending the connection without releasing the
   handle.
2. Queues the handle and returns `0`. A second close of a held handle also returns `0`.
3. Closes queued handles that have been held for `HOLD` (90 seconds), performing this cleanup on
   later close calls.

Setting `AUXTOOLS_HOLD_SOCKETS=0` disables this workaround. The 90-second hold is longer than the
hub's 60-second timeout, though the size of the margin has no recorded reason.

Two consequences follow from how the workaround is installed and drained:

- A held handle is closed only when a later `closesocket` call arrives after its hold has passed.
- If `byondcore.dll` or `wsock32.dll` is missing, or if no import is found, the workaround is not
  installed and the reason is printed.

## Applies to

- `debug_server/src/socket_hold_windows.rs`: `install`, `held_close`, `HOLD`, `HOLD_VAR`
- `debug_server/src/launched.rs`: `auxtools_debug_attach`
