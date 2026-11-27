# Breakpoints as separate, client-owned instruction claims

Multiple clients can set breakpoints in the same world, and a single client can place several
breakpoints on the same instruction with distinct conditions, hit conditions, and log messages. Hot
reload moves breakpoints into new code, and editors resend a file's entire breakpoint list whenever
any breakpoint in that file changes. Each breakpoint is therefore tracked as an independent claim
owned by one client.

## Server-side claims

- **Isolated per breakpoint:** Every breakpoint is distinct, even from the same client's other
  breakpoints at the same instruction. Conditions, hit conditions, hit counts, and condition-error
  counts belong to the individual breakpoint, so two clients never see each other's conditions or
  hit counts.
- **Client-chosen IDs:** The client selects each breakpoint ID. Sending `BreakpointSet` with an ID
  that the client already uses silently replaces that breakpoint, even if the new request targets a
  different location.
- **Entry-point placement:** With `exact` set, the breakpoint goes on the exact instruction given.
  Without `exact`, offset `0` denotes the proc's entry: the server moves the breakpoint to the
  proc's entry offset, and hot reload keeps it at the entry of the new code. All other offsets are
  used as given.
- **Updating triggers in place:** `BreakpointTrigger` replaces a breakpoint's condition, hit
  condition, and log message at the breakpoint's current location, which hot reload may have moved.
  A new condition or hit condition resets the hit count. The server replies with `BreakpointSet`:
  `Failed` for an unknown ID, and `Refused` when the new condition or a log expression does not
  compile.
- **Unsetting and disconnect cleanup:** `BreakpointUnset` returns how many hits the breakpoint
  skipped because its condition failed. When a client disconnects, the world removes that client's
  breakpoints and releases any pause held only by that client. When the last client disconnects,
  all breakpoints are removed.

## Client-side handling

- A client sends `Disconnect` only to a world that sent a hello, because worlds without a hello
  predate that request.
- When an editor resends a file's breakpoints, any breakpoint the debug adapter already tracks
  keeps its existing instruction claim and hit count in the world. A changed condition or hit
  condition is sent with `BreakpointTrigger` so it updates the claim at its current location rather
  than at the source line that hot reload may have moved it away from.
- `trace` relies on disconnect cleanup if it fails partway through, requiring no manual removal of
  breakpoints already set. On normal exit, however, it unsets its breakpoints before disconnecting
  so it can collect each breakpoint's condition-error count from `BreakpointUnset`. Disconnecting
  first would drop the breakpoints without returning those counts.

## Applies to

- `auxtools-protocol/src/messages.rs`: `Request::BreakpointSet`, `Request::BreakpointTrigger`,
  `Request::BreakpointUnset`
- `debug_server/src/server/breakpoints.rs`: `Claim`, `Claimant`, `handle_breakpoint_set`,
  `handle_breakpoint_trigger`, `handle_breakpoint_unset`, `breakpoint_offset`, `forget_client`
- `debug_server/src/server/sessions.rs`: `remove_client`
- `auxtools-cli/src/client.rs`: `Client::leave`
- `auxtools-cli/src/adapter.rs`: `apply_breakpoints`
- `auxtools-cli/src/cli/trace.rs`: `trace`
