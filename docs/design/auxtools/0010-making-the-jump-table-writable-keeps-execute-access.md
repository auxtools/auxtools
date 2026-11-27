# Preserving execute access when making the jump table writable

A jump table can share memory pages with executable code (for example, in Windows BYOND), so
removing execute access from those pages crashes the process. `writable` makes the table's pages
writable while preserving execute access on any page that already has it:

- **Linux:** Reads `/proc/self/maps` to determine the current protection of the pages covering the
  table, rounded out to 4096-byte page boundaries. If any overlapping mapping is executable, it
  calls `mprotect` with read, write, and execute permissions. Otherwise, it requests read and write.
  If `/proc/self/maps` cannot be read or no mapping covers the range, it changes nothing and fails,
  because a wrong guess could remove execute access from code.
- **Windows:** Calls `VirtualProtect` with `PAGE_EXECUTE_READWRITE`. If the previous protection
  was data-only (`PAGE_READONLY`, `PAGE_READWRITE`, or `PAGE_WRITECOPY`), it then sets the pages to
  `PAGE_READWRITE`.

When `writable` fails, `take_table` refuses, and auxtools falls back to the whole-function detour
described in
[how the instruction hook enters from BYOND's dispatcher](0008-how-the-instruction-hook-enters-from-byonds-dispatcher.md).

## Applies to

- `auxtools/src/instruction_hook.rs`: `writable`, `take_table`
