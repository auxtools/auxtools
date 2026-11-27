# Resolving breakpoint lines under hot reload

Under hot reload, a running world can execute code that is newer than the built `.dmb` but older
than the editor's buffer. A breakpoint line sent by the editor can therefore correspond to three
different texts: the unsaved buffer, the saved file on disk, or the code currently running in the
world.

## Tracking unsaved buffers

DAP provides no standard way for a client to report that an editor buffer has unsaved edits. After
a file is saved, VS Code sends the file's breakpoints again with `sourceModified` set, which means
the lines refer to the saved file on disk.

The debug adapter distinguishes unsaved buffers from saved files with three rules:

- **Extension notifications:** The VS Code extension sends a custom `auxtoolsUnsaved` request
  containing the file's path and whether it has unsaved changes.
- **Other DAP clients:** Clients that never send `auxtoolsUnsaved` are never treated as having
  unsaved text, so their breakpoint lines always refer to the saved file on disk.
- **Saves clear unsaved state:** Breakpoints received with `sourceModified` set always refer to the
  file on disk, even if the file is still marked unsaved.

## Applying and deferring breakpoints

- **Hot reload disabled:** Breakpoints are applied to the running code immediately. If a file has
  been saved since the `.dmb` was built, the adapter prints a console note stating that its
  breakpoints follow the built code and may fall on different lines.
- **Pending edits:** While a file has edits that the world does not yet run, existing breakpoints
  in the world stay in place until the reload applies those edits, and the new lines are held and
  applied after the reload. An empty breakpoint list is the sole exception, removing the file's
  breakpoints immediately.
- **Updating before the next request:** Once a reload is applied, the adapter updates breakpoints
  to the reloaded text before handling the next request. Otherwise, a save notification arriving
  next would compare the file against stale world text and treat the world as behind its file.

## Applies to

- `auxtools-cli/src/adapter.rs`: `UnsavedArguments`, `unsaved_changed`, `set_breakpoints`,
  `code_changed`, and the `code::Event::Reload` arm of `code_event`
