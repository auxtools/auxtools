# Failed `auxtools_init` is final for the process

Once `auxtools_init` fails, auxtools records the failure for the lifetime of the process, so
`auxtest` runs its failed-initialization check last.

When `auxtools_init` fails, it clears its hooks and procs and records the failure. Every subsequent
`auxtools_init` call in the same process returns the same `FAILED (...)` text without trying again,
because a retry would build on whatever the failed attempt left behind.

The `auxtest` world runs its tests, shuts auxtools down, initializes it again, and reboots. On the
eighth run, after the last normal shutdown, it asks the test library to force the next
initialization to fail (`auxtest_fail_next_init`) and runs `auxtest_failed_init`, which checks:

- Initialization fails, and a second `auxtools_init` call returns the same failure text.
- `auxtest_hooks` is not hooked, so calling it executes its own DM code (which crashes).

Calling `auxtools_shutdown` after the failed initialization must still return `SUCCESS`.

The world then reports `SUCCESS: Finished` and shuts down. Because nothing can initialize auxtools
again in that process afterward, the failed-initialization check must remain last.

## Applies to

- `tests/auxtest_host/auxtest_host.dm`: `do_tests`, `auxtest_failed_init`
- `tests/auxtest/src/lib.rs`: `auxtest_fail_next_init`, `fail_init_on_request`
- `auxtools/src/lib.rs`: `INIT_FAILURE`, `auxtools_init`
