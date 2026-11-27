# One-way requests and in-order server responses

Most requests to the debug server produce exactly one response, while a few only update server
state and return nothing. The server reads requests one at a time and sends responses strictly in
the order the requests arrive.

## Requests without a response

Three requests receive no response from the debug server:

- **`Defines`:** Sets the macros the server uses to expand expressions. They apply to all clients
  until the world process exits, persist across reboots, and mark the code as changed. A later
  `Defines` request from any client replaces them, while macros built into the world are kept
  alongside them.
- **`JournalAdd`:** Appends an entry to the journal. The server sends the entry as `Journaled` to
  every client following the journal with a matching kind, except the client that added it (see
  [the journal](0012-the-journal.md)).
- **`CatchRuntimes`:** Sets the requesting client's filters for uncaught and caught runtime errors.

A client must never wait for a response to `Defines`, `JournalAdd`, or `CatchRuntimes`, because
doing so blocks until the client's deadline expires.

## Relying on response order

Because the debug server answers requests in order, a client can send a request whose answer it
does not need without waiting for it. For example, the world probe sends `Watcher` using
`Client::tell` (which expects no answer), then sends `WorldInfo` and waits only for that response.
Because a watcher holds no pause, nothing depends on the first request having been processed before
the second is sent.

Code that relies on this ordering will break if the server ever answers requests out of order.

## Applies to

- `auxtools-protocol/src/messages.rs`: `Request::Defines`, `Request::JournalAdd`,
  `Request::CatchRuntimes`
- `debug_server/src/server.rs`: `Server::handle_request`
- `debug_server/src/server/journaling.rs`: `journal`
- `auxtools-cli/src/local.rs`: `probed`
- `auxtools-cli/src/client.rs`: `Client::tell`
