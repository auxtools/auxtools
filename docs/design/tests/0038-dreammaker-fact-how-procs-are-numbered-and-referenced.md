# How DreamMaker numbers procs and resolves proc references

DreamMaker numbers procs in the order it encounters their declarations and keeps all definitions
when a type defines the same proc more than once, making the second definition override `#1`. In a
proc reference, a path containing `/proc/` selects the declaration, whereas a path without `/proc/`
selects the last definition.

The debugger's proc references and name-based breakpoints address each definition by its override
number, and the evaluator matches DreamMaker's proc reference rules. The test host worlds reproduce
these cases:

- **Duplicate definitions:** `/datum/thing/twice/bump` is defined twice in the DAP test host's
  `stepping.dm`.
- **Override before declaration:** The hot reload host's `main.dm` starts with `/early_global()`,
  an override of `/proc/early_global`, before every proc declaration so DreamMaker numbers it ahead
  of `/proc/early_global` in `reload.dm`. `hot_reload_test` verifies that calling `early_global()`
  returns the declaration's result, "early declared", and that reloading the declaration changes it.
- **Proc reference paths:** `launcher_test` evaluates `/datum/twice/proc/act` and
  `/datum/twice/act`, expecting each to print as written, and verifies that
  `/client == client_type()` is true.

## Applies to

- `tests/dap_test_host/stepping.dm`: `/datum/thing/twice`
- `tests/hot_reload_test_host/main.dm`: `/early_global`
- `tests/hot_reload_test_host/reload.dm`: `/proc/early_global`
- `tests/hot_reload_test/src/main.rs`: `early_global` checks
- `tests/launcher_test/src/inspect.rs`: proc reference checks
