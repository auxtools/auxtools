# Debug server client connections and writer threads

Clients connect to the debug server over a local socket (a named pipe on Windows) named by process
ID. The connection layer uses a blocking acceptor thread, immediate protocol greetings, and
per-client writer threads to handle BYOND, Windows, and Wine constraints:

- The world's thread can be busy for a long time or in the middle of a reboot, and during a reboot
  there is a brief period with no server.
- On Windows, flushing a named pipe blocks until the client reads.
- Under Wine, a named pipe listener in nonblocking mode never accepts a connection, and a thread
  that starts or ends while the process exits can crash the process.

To work within those constraints, the debug server handles connections as follows:

- The listener blocks in `accept` on its own thread so it works under Wine.
- Each connection's thread sends the protocol hello itself before any server knows about the
  client. A busy or rebooting world still greets connecting clients immediately.
- The acceptor outlives the server. Clients that connect while no server is running are queued and
  handed to the next world's server when it starts.
- Each client's outgoing messages are queued to a dedicated writer thread with room for `OUTBOX`
  (1024) messages, so a slow client never blocks the world's thread.

Because ending or starting threads during process exit can crash under Wine, the test that runs
last joins the writer threads and keeps its client connections open with `mem::forget`. Closing
those connections would end their reader threads and start the thread that the `interprocess` crate
uses to close pipes, both during exit.

## Applies to

- `debug_server/src/connection.rs`: `Listener`, `listen`
- `debug_server/src/server/sessions.rs`: `Acceptor`, `ACCEPTOR`, `OUTBOX`, `serve`, `post`, `writer`
- `debug_server/src/server.rs`: the last test
