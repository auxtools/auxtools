# Joining threads and skipping the pipe linger thread under Wine

Under Wine, a process can hang or crash while exiting if one of its threads is starting or ending at
the same moment. The launcher's connection code avoids this race across the three places where it
uses threads, so a command that exits immediately after talking to a world finishes cleanly under
Wine.

## Connection and probe threads

- **Client reader thread:** A client's reader thread ends when the world replies to `Disconnect`.
  `Client::close` hangs up the connection and joins the reader thread before returning.
- **Named pipe linger thread:** Dropping a named pipe that was written to normally starts the
  `interprocess` crate's linger thread, which waits for the other end to read what was sent. Hanging
  up a named pipe marks it as flushed (`skip_linger`), so dropping it starts no linger thread. The
  world can still read what was sent after this end closes.
- **World-list probe workers:** Listing worlds probes each world on a worker thread. Probe workers
  are joined once they answer (`Probe::finished_by`). A worker that does not answer by the deadline
  is left running, and its world is listed as not responding. Leaving that worker alive at exit is
  accepted because it happens only for an unresponsive world.

## Verifying thread exit in tests

Tests verify that hello and probe threads have exited by counting threads by name in
`/proc/self/task`. Only Linux exposes this listing, and on other platforms the count is `0`.

A thread that has already been joined can still appear in `/proc/self/task` for a short time,
because the kernel wakes the joining thread before finishing the joined thread's exit and removing
it from the list. Such a thread has `PF_EXITING` (`0x4`) set in the flags of its `stat` file (the
ninth field), so the counter excludes entries with that flag. Counting `/proc/self/task` entries
alone would report a joined thread as still running and cause tests such as
`a_listing_returns_after_its_probes_end` (which check immediately after a listing) to fail
intermittently.

## Applies to

- `auxtools-cli/src/client.rs`: `Client::close`
- `auxtools-cli/src/local.rs`: `skip_linger`, `Shared::hang_up`, `Probe::finished_by`, and test
  helpers `threads_named` and `released`
