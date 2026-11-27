# Reading and canceling Windows named pipe I/O

On Windows, the launcher communicates with worlds over named pipes, which have two constraints: a
read from a named pipe cannot time out, and `CancelIoEx` cannot cancel a read that has not started
yet (a read starting just after the cancel call would block forever).

To enforce a handshake timeout, `read_hello` reads the hello on a dedicated thread while the caller
waits on a channel with a 5-second timeout. If the timeout expires, the caller hangs up the
connection, which ends the reading thread.

To prevent races between hanging up and starting new reads, each read is counted starting before its
hung-up check. Hanging up cancels active reads in a loop (`cancel_reads`), retrying every
millisecond until the count drops to zero. Any read that begins after the hang-up observes the
hung-up flag and returns without blocking. On Unix, hanging up shuts down the socket, which ends any
pending read.

## Applies to

- `auxtools-cli/src/local.rs`: `read_hello`, `Shared`, `Connection`, `cancel_reads`
