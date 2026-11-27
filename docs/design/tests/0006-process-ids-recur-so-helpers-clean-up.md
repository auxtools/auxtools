# Cleaning up PID-named resources in test helpers

Windows and Wine reuse process IDs quickly, so a test or world process can inherit the PID of an
earlier run. Because several test helpers name temporary directories and stand-in debugger listeners
after a PID, they clean up before use and on drop so tests never pick up leftovers or stale
responses from another process.

- `TempDir::new` names its directory `NAME-PID` after the test process and removes any existing
  directory with that name before creating it, clearing anything left behind by an earlier process
  with the same PID.
- `attach_host_copy` in `launcher_test` also removes, one by one, the control files an earlier run
  may have left in its directory and fails if any cannot be removed: `stop`, `fail`, `reboot`,
  `call`, `refuse_del`, `fail_del`, `slow_del`, `slow_reboot`, and `refuse_reboot`. These files
  carry instructions a run sent to its world. A stale `refuse_del`, for example, would make the new
  world refuse to stop.
- The `StandIn` listener in `launcher_test`, which answers as a world's debugger, closes on drop so
  it cannot send stale responses to a later world that Wine assigns the same PID.
- `KilledOnDrop` terminates and reaps its child process when dropped, including after a failed
  assertion. This remains safe if the child has already been waited for, because
  `std::process::Child::kill` does nothing to an already reaped child even when its PID has been
  reused.

Any new test helper that names a resource after a process ID must clean up on drop or before use to
avoid reintroducing collisions across reused PIDs.

## Applies to

- `tests/support/src/temp.rs`: `TempDir::new`
- `tests/launcher_test/src/main.rs`: `attach_host_copy`, `StandIn`, `KilledOnDrop`
