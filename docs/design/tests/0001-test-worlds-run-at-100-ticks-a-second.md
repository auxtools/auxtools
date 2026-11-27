# Running test worlds at 100 ticks per second

Every test host world sets `fps = 100` on `/world` and runs a per-tick loop so debug server requests
wait about 10 ms instead of BYOND's 100 ms default.

The debug server answers requests from a running world only while DM code is executing. An otherwise
idle world with a per-tick loop runs DM code once per tick, so at BYOND's default rate of 10 ticks
per second, a request can wait up to 100 ms for a response (which is also the latency a real world
at the default tick rate sees). The idle-world waker does not shorten this wait: it activates only
when no DM code has run for a full second while a request or client is waiting, covering worlds that
run nothing at all, such as a world asleep under `world.sleep_offline`. A world with a per-tick loop
never reaches that threshold.

Setting `fps = 100` shrinks each tick to 10 ms, and each host ensures DM code runs every tick:

- The debug, DAP, and hot reload hosts run `main_loop`, which sleeps `world.tick_lag` on each
  iteration.
- The launcher's attach host has no per-tick loop of its own (`run_until_stopped` sleeps a tenth of
  a second on each iteration). Because the launcher starts it, the world code the launcher adds
  includes `auxtools_keepalive`, which sleeps `world.tick_lag` on each iteration and serves as the
  per-tick loop.

Removing `fps = 100` or the per-tick loop leaves the test suites correct, but makes them much
slower.

## Applies to

- `tests/dap_test_host/main.dm`: `/world` `fps`, `main_loop`
- `tests/debug_test_host/debug_test_host.dm`: `/world` `fps`, `main_loop`
- `tests/hot_reload_test_host/main.dm`: `/world` `fps`, `main_loop`
- `tests/launcher_test_host/attach_host.dm`: `/world` `fps`, `run_until_stopped`
- `debug_server/src/launched.rs`: `auxtools_keepalive` in the launcher world code
- `debug_server/src/waker.rs`: idle-world waker
