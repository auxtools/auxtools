# Skipping BYOND's `spawn(-1)` frame copy on Step Over

In BYOND, `spawn(-1)` executes its block immediately in a copy of the current proc's frame before
advancing to the next statement.

Step Over skips that frame copy rather than stopping inside the spawned block. To verify this
behavior, `spawns_now` in the DAP test host sets `n` to `1`, spawns a block that sets `n` to `2`,
and then sets `n` to `3`, while the stepping tests step over it.

## Applies to

- `tests/dap_test_host/stepping.dm`: `spawns_now`
- `tests/dap_test/src/groups/stepping.rs`: "Step Over a spawn (-1)" check
