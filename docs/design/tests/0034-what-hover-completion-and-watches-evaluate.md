# Evaluation limits for hover, completion, and watch expressions

Hover and completion run automatically without explicit user action, so they must never execute
code that could have side effects or loop forever. A watch or evaluation with a body can be
expanded, and VS Code evaluates an expanded child's `evaluateName` to add it as a watch.

- **Single expressions in hover:** Hover accepts only a single expression. A body, such as
  `again:\ngoto again`, `var/x = n\nx`, or `return n`, is refused with "hover requires a single
  expression".
- **Macros that call procs:** Because a macro can expand to a proc call, hover and completion treat
  expressions containing such a macro as potentially running code. Hover refuses them with
  "expression may run code, so it must be evaluated explicitly", and completion offers no items.
  `dap_test` checks that `CURRENT`, a macro that reads through a call, does not run during three
  hovers and three completions, and runs once when evaluated.
- **Expanded body results:** A watch expression or evaluation with a body omits `evaluateName` on
  its expanded children, because evaluating an `evaluateName` would run the body again.
- **Empty-prefix completion cap:** Completions for an empty prefix return only the first 200 names
  (`MOST_COMPLETIONS` in the debug server). Tests looking for a specific name supply a prefix.

## Applies to

- `auxtools-cli/src/adapter.rs`: `hover_refuses`
- `debug_server/src/server/reading.rs`: `RUNS_CODE`
- `debug_server/src/server/values.rs`: `MOST_COMPLETIONS`
- `tests/dap_test/src/groups/expressions.rs`: completion and body checks
- `tests/dap_test/src/groups/reloading.rs`: macro hover check
