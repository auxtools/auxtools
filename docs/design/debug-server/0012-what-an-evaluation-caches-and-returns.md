# What an evaluation caches and returns

Evaluated expressions are compiled, assembled, and run in the evaluation stub. Three aspects of
compilation, execution, and result inspection determine what the debug server caches and returns:

- **Compilation cache:** Compiled evaluation bytecode is cached by `EvalCodeKey`, which combines the
  code generation (incremented by every hot reload), the command, the parameter names, and the type
  of `src`. Compile errors are cached. Assembly failures (`CodeError::assembly`) are not cached
  because assembly looks up the world's resources, procs, and paths, which can change without a
  reload (for example, when a resource file is created for the debugger), so the next evaluation
  tries again. The cache holds at most `EVAL_CODE_KEPT` (256) entries and is cleared when full.
- **Sleeping and backgrounded evaluations:** Background procs sleep periodically.
  `world.loop_checks` moves a proc to the background after about a million loop iterations, and
  `set background = 1` does so as well. Code that sleeps or moves to the background keeps running
  but has no result to return, so the evaluation replies with `still_running` set. On BYOND 516 and
  later, where BYOND's background flag can be read (`background_known`), the message states which
  happened: "BYOND moved the code to the background" or "the code slept". On older BYOND builds, it
  reports that the code slept or was moved to the background. These messages point to
  `eval --spawn --wait` to wait for a result.
- **Expansion paths:** Expanding a result in the Variables view re-evaluates a path built from the
  original expression. Because an evaluated statement body has side effects, only plain expressions
  receive an expansion path. Statement bodies receive none, so expanding their result never runs the
  body again.

## Applies to

- `debug_server/src/server/eval.rs`: `EvalCodeKey`, `CodeError::assembly`, `compile_and_run_expr`,
  `background_known`, `handle_eval`
