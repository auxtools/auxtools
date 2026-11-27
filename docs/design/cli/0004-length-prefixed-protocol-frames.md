# Length-prefixed protocol frames

The launcher and the debug library communicate over a local socket (a named pipe on Windows) using
serde types encoded with bincode. Because bincode does not delimit message boundaries, each message
is framed with its byte length as a little-endian `u32`, followed immediately by the bincode
payload.

- **Length cap:** Any message length exceeding `MAX_MESSAGE_LEN` (256 MiB) is rejected as an error
  on both sending and reading.
- **Incremental allocation:** The read buffer grows as bytes arrive instead of pre-allocating the
  announced length, so a corrupt or hostile length header without payload data allocates nothing.
- **Close on error:** After any framing or decoding error, the next message boundary is unknown, so
  the stream cannot be reused and must be closed. Callers never attempt to skip a bad message and
  continue: every read error terminates the connection.

## Applies to

- `auxtools-protocol/src/framing.rs`: `frame`, `write`, `read`, `MAX_MESSAGE_LEN`
