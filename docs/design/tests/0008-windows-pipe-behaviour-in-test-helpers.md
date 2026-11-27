# Handling Windows named pipe behavior in test helpers

Test helpers that communicate over Windows named pipes work around three behaviors specific to
named pipes on Windows: a pipe server can miss a client that disconnects before being accepted,
switching a pipe to nonblocking mode can fail with `ERROR_PIPE_BUSY` after data has been written to
it, and a nonblocking pipe returns zero bytes both when empty and when closed.

Each helper handles the corresponding quirk directly:

- **Waking the listener on drop.** `StandIn`'s `Drop` implementation wakes its listener thread by
  connecting to it so the thread observes the stop flag. Because one connection can be missed if the
  client disconnects before the server accepts it, `Drop` retries connecting every 20 ms for up to
  10 seconds until the thread exits.
- **Enabling nonblocking mode before writing.** The stand-in world in the support crate calls
  `set_nonblocking` on its stream before writing its hello message, allowing the test to close the
  stream during a nonblocking read. Writing the hello before `set_nonblocking` causes intermittent
  failures on Windows.
- **Handling zero-byte reads.** On Windows, the stand-in's reader thread treats a zero-byte read as
  "nothing yet", sleeps briefly, and reads again until the test closes the connection. Treating a
  zero-byte read as the end of the stream on Windows ends the stand-in's connection while the
  launcher is still talking to it. On other platforms, a zero-byte read ends the loop.

## Applies to

- `tests/launcher_test/src/main.rs`: `Drop` for `StandIn`
- `tests/support/src/standin.rs`: connection setup and reader thread
