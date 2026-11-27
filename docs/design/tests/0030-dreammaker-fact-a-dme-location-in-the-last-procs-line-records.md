# Interleaved `.dme` locations in the last proc's line records

When a `.dme` file has a `#define` after its last include, DreamMaker can start the last proc of
that last included file with the `.dme`'s location between two records of the proc's line. A hot
reload must keep breakpoints and line moves accurate around this extra location, and a breakpoint
moved by a reload must land after both records, where a new breakpoint on that line goes.

Two host worlds reproduce this case deliberately:

- `hot_reload_test_host.dme` ends its includes with `resources.dm`, followed by `// END_INCLUDE`
  (the marker DreamMaker's editor uses for the end of its include list) and
  `#define AFTER_THE_CODE`. `last_in_file` is kept as the last proc of `resources.dm`, and
  `hot_reload_test` moves a breakpoint on its `return "last [...]"` line with a reload to check
  where it lands.
- `attach_host.dme` includes `attach_host.dm` followed by `#define AFTER_THE_CODE`. `last_in_file`
  is likewise kept as the last proc of `attach_host.dm`, and the launcher tests check reloads and
  breakpoints around it.

Removing the define or adding a proc after `last_in_file` stops the hosts from reproducing the case
without any test failing.

## Applies to

- `tests/hot_reload_test_host/hot_reload_test_host.dme`
- `tests/hot_reload_test_host/resources.dm`: `last_in_file`
- `tests/launcher_test_host/attach_host.dme`
- `tests/launcher_test_host/attach_host.dm`: `last_in_file`
- `tests/hot_reload_test/src/main.rs`: `/last_in_file` breakpoint check
