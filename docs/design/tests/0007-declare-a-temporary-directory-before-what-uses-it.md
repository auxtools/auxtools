# Ordering `TempDir` declarations so worlds drop first

Because `TempDir` deletes its directory on drop, anything that uses that directory must be dropped
before the `TempDir` itself. Otherwise, a still-running world process has its files removed
underneath it.

Rust drops struct fields in declaration order and function locals in reverse declaration order, so
every `TempDir` must obey two ordering rules:

- Declare local variables that use a `TempDir` after the `TempDir`.
- Declare struct fields that use a `TempDir` before the field holding the `TempDir`.

For example, `hot_reload_test`'s `Built` struct declares `world` before `source`, where `Source`
owns the `TempDir`. This guarantees the world exits before its source directory is deleted.
Swapping the fields of `Built`, or declaring a world local before its `TempDir`, removes files
under a running world.

When a test check fails, the test process exits without running drops and leaves its directory
behind. The next process assigned the same PID cleans it up, as described in
[cleaning up PID-named resources in test helpers](0006-process-ids-recur-so-helpers-clean-up.md).

## Applies to

- `tests/support/src/temp.rs`: `TempDir`
- `tests/hot_reload_test/src/main.rs`: `Built`
