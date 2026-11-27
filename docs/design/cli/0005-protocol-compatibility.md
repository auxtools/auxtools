# Protocol versioning and wire compatibility

A launcher and a debug library from different auxtools releases frequently meet (such as an older
running world with a newer launcher, or vice versa) and must recognize each other so they can
refuse incompatible sessions cleanly. Because bincode encodes enum variants by index, inserting a
variant renumbers every subsequent variant. In addition, dmasm's reload plan types and instructions
travel over the wire directly inside `Request::HotReload`, so changing them changes the wire
protocol even if no type in `auxtools-protocol` changes.

## Compatibility rules

- **Fixed `Hello` prefix:** `Hello` is the server's first message on every connection, and clients
  of all versions read it. Its fields `protocol: u32`, `auxtools: String`, and `pid: u32` never
  change or move. Later versions may append fields after them. An incompatible launcher can
  therefore still read `Hello` and report which protocol version the world speaks.
- **Append-only `Request` variants:** New variants are always added at the end of `Request` so
  existing variants keep their bincode indices for older peers.
- **Bump `PROTOCOL_VERSION` on breaking changes:** Raise `PROTOCOL_VERSION` for any incompatible
  wire change, including changes to dmasm's plan types or instructions. `PROTOCOL_VERSION` is 15 at
  the time of writing. A compile-time assertion keeps it above `LAST_NAMED_PROTOCOL` (12), ensuring
  a current world can never be mistaken for an auxtools 3 world.

Reordering `Request` variants or modifying a plan type without bumping `PROTOCOL_VERSION` breaks
older peers in ways that may not surface as a decode error. The plan-format test guards the plan
side, while `Request` variant ordering relies on code review.

## Catching reload-plan format changes

The test `a_plan_of_another_shape_is_another_protocol_version` in `plan_format.rs` detects wire
changes in reload plans and instructions. It serializes a sample plan with bincode, appends the
name and operand list of every dmasm instruction shape, and hashes the resulting bytes with FNV-1a
(offset basis `0xCBF29CE484222325`, prime `0x100000001B3`). FNV-1a is used instead of `std`'s
hasher because it produces identical results across machines and toolchains.

`RECORDED` stores the expected `(protocol version, hash)` pair. When the hash changes, a launcher
and a debug library sharing the same protocol version are no longer wire-compatible: raise
`PROTOCOL_VERSION` and update `RECORDED` with the new hash printed by the test.

## Detecting auxtools 3 worlds

Worlds running auxtools 3 do not send `Hello`. Their protocol version is encoded only in their
socket name, `auxtools-debug-v{protocol}-{pid}`, up through version 12 (`LAST_NAMED_PROTOCOL`).
Later worlds use `auxtools-debug-{pid}` and send `Hello`.

When connecting to the current socket name fails, the launcher probes the legacy socket names from
version 12 down to 1. A match identifies the world's protocol version so the launcher can report
the world as incompatible rather than not listening.

## Applies to

- `auxtools-protocol/src/hello.rs`: `Hello`, `world_socket_name`, `LAST_NAMED_PROTOCOL`,
  `old_socket_name`
- `auxtools-protocol/src/lib.rs`: `PROTOCOL_VERSION`
- `auxtools-protocol/src/messages.rs`: `Request`
- `auxtools-protocol/src/plan_format.rs`: `RECORDED`, `hash`, `shape`,
  `a_plan_of_another_shape_is_another_protocol_version`
- `auxtools-cli/src/local.rs`: `open`, `auxtools3_protocol`
