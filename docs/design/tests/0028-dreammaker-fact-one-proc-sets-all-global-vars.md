# Global variable initializers across files compile into one proc

DreamMaker compiles all global variable initializers into a single proc in DreamMaker's order. Its
code starts in one file that declares a global and continues through each of the other files, so its
debug information names several files. A breakpoint on a global initializer in any file must
therefore be found in that proc.

The DAP test host includes global initializers in more than one file:
`var/early_global = early_value()` in `main.dm` and `var/tooled = sub_tool(20)` in `sub/tool.dm`
(a file other than the world's first). `before_the_world_runs` sets breakpoints on `early_value`,
`tooled`, and `sub_tool` and expects to stop in each within that single proc.

## Applies to

- `tests/dap_test/src/groups/breakpoints.rs`: `before_the_world_runs`
- `tests/dap_test_host/sub/tool.dm`: `tooled`
- `tests/dap_test_host/main.dm`: `early_global`
