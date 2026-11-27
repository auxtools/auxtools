# Hidden `(init)` procs for non-constant variable initializers

DreamMaker compiles a type's non-constant variable initializers, such as
`var/list/stuff = list(1, 2)`, into a hidden proc on that type without a path. BYOND assigns the
path `/TYPE/proc/(init)` only after startup.

auxtools handles these generated procs in two places:

- **Stepping:** Stepping treats an `(init)` proc as generated code rather than user code. Step In
  on `new /datum/made` skips `(init)` and stops in `New()`.
- **Lookup by name:** The debug server's `unnamed` module finds and names these procs. When the
  path read at startup was empty, it reads the path again by proc ID. Tools that look up an
  `(init)` proc by path must do so after the world has started. `launcher_test` creates a
  `/datum/inits`, queries `/datum/inits/(init)` with `auxtools proc`, and expects one definition.

## Applies to

- `tests/dap_test_host/stepping.dm`: `/datum/made`
- `tests/launcher_test_host/attach_host.dm`: `/datum/inits`
- `tests/dap_test/src/groups/stepping.rs`: "Step Into New skips initializer" check
- `tests/launcher_test/src/inspect.rs`: `procs`
- `debug_server/src/instruction_hooking.rs`: `is_generated_proc`
- `debug_server/src/unnamed.rs`: `find`, `proc_ref`
