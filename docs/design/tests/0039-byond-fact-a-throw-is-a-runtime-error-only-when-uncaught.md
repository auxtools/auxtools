# Stopping on uncaught `throw` instructions in BYOND

In BYOND, a `throw` produces a runtime error only when nothing catches it.

When an uncaught `throw` occurs, the debugger pauses at the `throw` instruction before it executes,
preserving the active frame and its local variables, and reports the thrown value in the stop
details. `dap_test` verifies this for both a thrown exception and thrown text, and confirms that
the runtime error is still logged once after the world resumes.

## Applies to

- `tests/dap_test/src/groups/runtimes.rs`: "uncaught throw stops with its value" check
