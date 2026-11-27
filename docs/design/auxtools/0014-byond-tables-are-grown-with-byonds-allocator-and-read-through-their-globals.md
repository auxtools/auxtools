# Allocating BYOND tables and reading them through globals

On Windows, BYOND allocates its internal tables from `ucrtbase.dll`'s heap, whereas auxtools links
its C runtime statically and allocates from a separate heap. Because BYOND frees and reallocates
these tables itself, memory allocated by Rust's allocator would be freed on the wrong heap on
Windows. BYOND also reallocates tables (such as the alist table and the proc table) as they grow, so
any DM code that runs can leave a cached pointer into a table dangling.

## Using BYOND's allocator

To allocate, grow, or free any memory that BYOND will later free or reallocate, auxtools calls the
C runtime's `realloc` and `free` through `crt_realloc` and `crt_free`, never its own allocator. On
Windows, these functions are resolved from `ucrtbase.dll` via `GetModuleHandleA` and
`GetProcAddress`. On Linux, they are the C library's `realloc` and `free`.

This requirement covers:

- The type, preorder, proc name map, and type appearance cache tables
- The mob, instance, and `in` list tables
- Proc name buckets, variable names and values, and type matrices
- The 12-byte misc entries described in
  [BYOND misc entry layout and interning](0012-byond-misc-entry-layout-and-how-auxtools-interns-one.md)

## Reading tables through globals

auxtools reads growable BYOND tables through their global pointers on each access and never holds a
pointer into a table across DM execution. For example, `List::alist_pairs` reads `ALIST_TABLE` and
checks the ID against `ALIST_TABLE_COUNT` on every call.

## Applies to

- `auxtools/src/runtime_tables.rs`: `ucrtbase`, `crt_realloc`, `crt_free`, and every caller of
  `functions.crt_realloc`
- `auxtools/src/list.rs`: `List::alist_pairs`
- `auxtools/src/raw_types/funcs.rs`: `ALIST_TABLE`, `ALIST_TABLE_COUNT`
