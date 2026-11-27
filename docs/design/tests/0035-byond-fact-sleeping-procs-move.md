# Frame and thread identity when BYOND moves sleeping procs

BYOND moves a proc's frames to a new memory location the first time the proc sleeps. Deleting the
`src` of a sleeping proc also moves sleeping procs in memory, even though deletion does not end the
proc immediately (BYOND ends it only when it wakes, if at all).

The debugger identifies frames and threads in a way that survives these moves. Two tests cover
these cases:

- Stepping over a `sleep` keeps the same thread ID for the proc.
- With three `/datum/dozer` procs asleep, each with its own `src`, evaluating an expression in one
  frame to delete its `src` leaves the frames of the remaining procs identified as the same frames.

## Applies to

- `tests/dap_test/src/groups/frames.rs`: sleeping dozers check
- `tests/dap_test/src/groups/stepping.rs`: Step Over sleep check
