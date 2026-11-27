# Updating file modification times when tests edit or replace files

The launcher and source worker detect file changes by checking modification times, so tests
explicitly update a file's modification time whenever they copy or edit it.

Two timestamp pitfalls make implicit modification times unreliable in tests:

- On Windows, copying a file preserves the source file's modification time, so copying over an
  existing file does not register as a change. `resource_saved` in `dap_test` copies a version of
  `lamp.dmi` into place and then sets its modification time to now, matching an editor save. Without
  this step, the resource reload test waits for a reload that never comes on Windows.
- File modification times can be coarser than the time between a build and an edit, giving an edit
  made immediately after a build the same modification time as the `.dmb`. The rebuild check in
  `launcher_test` writes the edited source and then sets its modification time to two seconds after
  the `.dmb`'s. Without this step, the rebuild test can see the edit as older than the build.

## Applies to

- `tests/dap_test/src/groups/reloading.rs`: `resource_saved`
- `tests/launcher_test/src/main.rs`: `edit` closure in the detached rebuild test
