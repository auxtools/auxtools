# The Windows crash handler must not wait or panic

On Windows, the debugger installs an unhandled exception filter that prints a crash report and
reports the exception to clients as a runtime stop, `"native exception"`. The filter returns
`EXCEPTION_EXECUTE_HANDLER` so the process takes the exit the exception leads to.

Because the filter runs on the faulting thread, that thread may already hold the code index lock or
the journal's frame cache lock. After a native exception, the DM frame may also be corrupted, and
reading it can fault again.

- The report runs inside `auxtools::guard`. A panic escaping the filter would abort the process
  instead of letting it take the exit the exception leads to.
- The report prints the native exception first: its code, the module and address, and for an access
  violation whether it was reading, writing, or executing and at which address. It reads the
  current DM context last and prints the proc and line.
- The code index accessor uses `try_lock` and returns `None` when the lock is held, so callers read
  code without the cache.
- The journal's stack reader also uses `try_lock` and skips its frame cache when the lock is held.

## Applies to

- `debug_server/src/crash_handler_windows.rs`: `exception_filter`, `report`
- `debug_server/src/code_index.rs`: `index`
- `debug_server/src/server/journaling.rs`: `journal_stack`
