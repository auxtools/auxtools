# Polling until a paused world resumes after debugging clients leave

When a test expects a paused world to resume after its last debugging client disconnects, it polls
for up to 10 seconds instead of checking once.

A paused world stays paused while any connected client has not declared itself a watcher. A client
from another process can briefly count as a non-watching client, such as an older `auxtools list`
that connects to every world without declaring that it only watches. A single check would fail
intermittently whenever another process happened to hold a connection at that moment, for example
another test suite running `auxtools list` on the same machine.

Two helpers implement the 10-second polling loop:

- `debug_test`'s `expect_carries_on` connects as a watcher and queries the status until the world
  is no longer paused.
- `launcher_test`'s `carries_on` runs `auxtools status` until it prints `pid N: running`.

## Applies to

- `tests/debug_test/src/main.rs`: `expect_carries_on`
- `tests/launcher_test/src/main.rs`: `carries_on`
