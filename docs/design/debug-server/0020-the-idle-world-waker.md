# Keeping a sleeping world awake with an idle TCP connection

A world with `world.sleep_offline` set runs no instructions while it has no clients. Because the
debug server answers requests only between instructions, requests to an idle world would go
unanswered. Opening an idle TCP connection to the world's own port keeps BYOND awake.

Two behaviors of that idle connection shape how the waker and client management work:

- BYOND hangs up a connection that never sends anything.
- The connection appears in the world as a client without a key, and calling `winset()` on that
  client raises a runtime error.

To keep the world responsive without disturbing real clients:

- A waker thread checks every 250 ms. When requests are waiting or clients are connected and the
  world has shown no activity for one second, it opens a connection to `127.0.0.1` on `world.port`
  with a one-second timeout.
- Activity is the instruction counter plus BYOND's proc call count. A DM loop that makes no proc
  calls advances the instruction counter only when the hook hears every instruction, so the waker
  turns full hearing on before deciding that the world is idle.
- On each round, the waker drains its connection with nonblocking reads to see whether BYOND closed
  it, and opens a new one when it did. A peek would keep returning the same unread data and never
  report the close.
- Code that counts or messages real clients skips clients without a key. The skin reload counts
  only clients with a key, and reconnect sends `.reconnect` only to them.

## Applies to

- `debug_server/src/waker.rs`: `watch`, `open`, `activity`, `PATIENCE`, `CHECK_EVERY`
- `debug_server/src/skin.rs`: `clients`, `reconnect`
