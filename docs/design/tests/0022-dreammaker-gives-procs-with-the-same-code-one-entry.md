# Shared code entries for identical procs and twin test macros

DreamMaker assigns a single shared code entry to procs whose bytecode is identical. Because locals
and parameters are numbered rather than named in bytecode, procs differing only in local or
parameter names produce the same code. In a world built with `DEBUG`, line numbers are embedded in
the bytecode, so two procs share a code entry only if they are defined on the same line. Profiling,
breakpoints, and hot reloads must therefore distinguish such procs by proc rather than by code
entry.

The test host worlds construct twin procs using macros that define both procs on one line:

- `PROFILED_TWINS` in the debug host defines `profiled_twin_a` and `profiled_twin_b` for the
  profiler.
- `TWINS` in the hot reload host defines `twin_a` and `twin_b`.
- `NAMED_TWINS` defines `twin_c` and `twin_d`, differing only in the name of a local variable.
- `PARAM_TWINS` defines `twin_e` and `twin_f`, differing only in the name of a parameter. It sits at
  the top of `reload.dm` so no reload shifts its line and the two procs keep the same code.

The reload tests check that reloading one twin does not change the other.

## Placement caveats

- Moving a twin macro where a reload shifts its line gives the twins different code after the
  reload, so the test no longer covers shared code.
- `TWINS` sits immediately after `breakpoint_target` in `reload.dm`. `hot_reload_test` checks that,
  from one line before `breakpoint_target`'s `x += 1` through five lines after it, the only lines
  with code are the three lines of `breakpoint_target`. Shortening the lines between them brings
  `var/twin_runs = 0` and `var/twin_waiting = 0` into that window, which is safe only if a constant
  global initializer records no line (which has not been verified).

## Applies to

- `tests/debug_test_host/debug_test_host.dm`: `PROFILED_TWINS`
- `tests/hot_reload_test_host/reload.dm`: `PARAM_TWINS`, `TWINS`, `NAMED_TWINS`
- `tests/hot_reload_test/src/main.rs`: "lines are located where a reload moved them" check
