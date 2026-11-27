# Reaching globals from evaluated expressions

An evaluated expression runs in the evaluation stub with the frame's names passed as parameters, and
also needs access to the world's globals under a name that no user variable can shadow. The debug
server passes the globals object as a parameter named `global vars` (`GLOBALS`). Because DM code
cannot write an identifier containing a space, it can never shadow a user's variable.

Globals added by a hot reload are inserted into BYOND's table behind `global.vars`, which stores
`(variable ID, flags)` pairs sorted by name and is binary-searched by `global.vars`. auxtools
locates that table by scanning BYOND's code. On a BYOND build where the table is not found, newly
added globals are missing from `global.vars`. A global name therefore resolves through `global.vars`
when present there, and otherwise falls back to the reload's own record of added globals
(`ADDED_GLOBALS`, queried by `added_global`) by ID.

## Applies to

- `debug_server/src/server/eval.rs`: `GLOBALS`, `global` scope callback in `compile_and_run_expr`
- `debug_server/src/hot_reload.rs`: `ADDED_GLOBALS`, `added_global`
- `auxtools/src/runtime_tables.rs`: `global_vars`, `add_global_vars`
