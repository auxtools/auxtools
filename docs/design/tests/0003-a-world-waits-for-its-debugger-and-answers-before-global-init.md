# Launched worlds wait for a debugger and answer before global initialization

Tests rely on two startup behaviors of the debug library: a world runs no DM code until its
debugger connects and sends breakpoints, and the debug server begins answering requests before
global initialization finishes.

Because no DM code runs until the debugger is configured, breakpoints in load-time code such as
global variable initializers and `world/New` are hit. Because the debug server answers requests
earlier than that, an early evaluation can observe a global variable as `null`, including an object
created by a global initializer.

The test suites rely on and account for both behaviors:

- `dap_test`'s `before_the_world_runs` sets breakpoints on global initializers and on `world/New`
  before `configurationDone`, expecting each to be hit.
- `debug_test` evaluates `helper.greet("you")` only after the first breakpoint stop, because
  `helper` is a global (`var/mob/helper/helper = new`) that is not initialized yet when the world
  first answers.
- `hot_reload_test`'s `wait_for_world_new` waits until the sleeper started by `world/New` reports
  `"sleeping"`, rather than reading a global variable as soon as it connects.
- `hot_reload_test` builds and starts each section's world while the previous section is running.
  Because the early world executes nothing until its section connects, pre-starting it is safe and
  saves startup time.

A test that evaluates a global variable immediately after connecting can see `null` and fail
intermittently. Conversely, if a world ever began executing DM code before its debugger connected,
the pre-started worlds in `hot_reload_test` would run ahead of their section and breakpoints in
load-time code would be missed.

## Applies to

- `tests/dap_test/src/groups/breakpoints.rs`: `before_the_world_runs`
- `tests/debug_test/src/main.rs`: verb call check after the first breakpoint
- `tests/hot_reload_test/src/main.rs`: `wait_for_world_new`, section loop in `main`
