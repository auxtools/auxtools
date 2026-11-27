# DAP and VS Code display rules for frames, variables, and disassembly

DAP clients, and VS Code in particular, rely on display rules that are not obvious from the
protocol specification and that raw DM data does not satisfy on its own. The debug adapter adapts
stack traces, variable lists, and disassembly responses to meet three constraints:

- **Frame columns and the exception widget:** VS Code displays the exception widget for a stopped
  frame only when the frame's column is greater than 0. The adapter assigns column `1` to the top
  stack frame and column `0` to every other frame so the widget appears on the frame that stopped.
- **Unique variable names:** Within a single DAP variables container, VS Code merges or hides
  entries that share a name, even though DM objects can hold multiple entries with the same name.
  The adapter appends ` #N` to repeated names, where `N` counts earlier entries with that name, so
  the second `x` appears as `x #1` and the third as `x #2`.
- **Exact disassembly counts:** A DAP disassemble response must contain exactly the number of
  instructions requested. When the requested range extends outside the proc's code, the adapter
  returns invalid instructions for those addresses instead of omitting them.

## Applies to

- `auxtools-cli/src/adapter.rs`: `stack_trace`, `variables`, `disassemble`
