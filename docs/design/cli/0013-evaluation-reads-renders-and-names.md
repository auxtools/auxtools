# Read-only hovers, render depth, and frame parameter order

VS Code evaluates the expression under the mouse automatically on hover, where running DM code could
call procs with side effects or sleep. Hover requests therefore inspect state without executing DM
code, while Debug Console and watch expressions can run code. Expressions compiled in a stack frame
also depend on a fixed parameter order shared between the debug server and external tools.

## `Read` and `Eval`

- Hovers are sent as `Request::Read`, which evaluates without running DM code by reading names,
  variables, and list entries, and responds with `Response::Eval` just like a full evaluation.
  Expressions that require running DM code are left to the Debug Console and watches.
- Debug Console and watch evaluations are sent as `Request::Eval` and may execute DM code.
- DM code run by an evaluation can sleep, or BYOND can move it to the background, leaving no result
  to return while execution continues. An `EvalError` with `still_running` set reports that the code
  slept or was moved to the background and continues running without a result.

## Render depth

`Request::Render`, `RenderVariables`, and `EvalOn` take a depth limit. Depth `1` includes list
entries, and depth `2` includes object variables, with each nested child consuming one level. At the
depth limit, a list reports only its length and an object reports only its metadata (reference,
type, and name).

## Parameter order for frame expressions

An expression in a stack frame is compiled with the frame's names as parameters. Tools that compile
expressions the way the debug server does must pass those names in the same order, or names will map
to different parameter slots and produce output that does not match the debug server's:

- **Condition and log message checks:** To check that a breakpoint condition or a log message
  expression compiles, the debug server passes `.`, `usr`, `src`, then the proc's arguments, and
  then its locals. The `dmasm_compile` tool takes its parameters in this order.
- **Frame evaluations:** To run an evaluation in a frame, the debug server uses a longer list: any
  names supplied by the request come first, followed by `args`, `.`, `usr`, `src`, the named
  arguments, the visible locals, and finally a name for the global variables.

## Applies to

- `auxtools-protocol/src/messages.rs`: `Request::Read`, `Request::Eval`, `Request::Render`,
  `Rendered`, `EvalError`
- `auxtools-cli/src/adapter.rs`: `Adapter::evaluate`, `hover_refuses`
- `debug_server/src/server/breakpoints.rs`: `expression_error`
- `debug_server/src/server/eval.rs`: evaluation path
- `tools/debug_client/src/bin/dmasm_compile.rs`
