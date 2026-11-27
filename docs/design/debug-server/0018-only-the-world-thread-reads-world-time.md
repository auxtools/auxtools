# Only the world's thread reads `world.time`

Journal entries record `world.time`, which is read through BYOND's value functions. Those functions
are safe to call only on the world's thread, whereas the debug server also runs its own threads:
the acceptor, connection threads, writer threads, and the waker.

`journal::world_time`, and `journal::add` which calls it, must therefore be called only from the
world's thread.

## Applies to

- `debug_server/src/journal.rs`: `world_time`, `add`
