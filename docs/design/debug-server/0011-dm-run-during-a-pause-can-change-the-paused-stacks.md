# DM run during a pause can change the paused stacks

While the world is paused, the debug server can run DM code for breakpoint conditions, evaluations,
hot reloads, skin and script changes, reconnects, and spawns. That code can end existing procs,
start new ones, or end a sleeping frame by deleting its `src`. In addition, if paused callers still
have active `try` handlers when an evaluation runs, BYOND delivers errors from the evaluation to the
paused caller's `catch` block once execution resumes.

The server protects paused state against these side effects in four ways:

- The call stacks fetched for the pause are invalidated after each DM run and read again when next
  needed. This happens after a breakpoint condition, after an evaluation on an object, and after any
  `HotReload`, `Skin`, `Script`, `Reconnect`, or `Spawn` request handled during the pause.
- A breakpoint's claim is looked up again after its condition runs, because the condition's code
  could have removed the breakpoint.
- Before writing results back into a selected frame, an evaluation checks with `frame_is_live` that
  the frame is still live. If not, it fails with "evaluation ended the selected frame".
- `hide_tries` clears every throw handler on the running stack before an evaluation, and
  `show_tries` restores them afterward.

## Applies to

- `debug_server/src/server/breakpoints.rs`: `invalidate_stacks` callers
- `debug_server/src/server/pause.rs`: `invalidate_stacks` callers
- `debug_server/src/server/eval.rs`: `call_with_locals`, `hide_tries`, `show_tries`,
  `frame_is_live`, `compile_and_run_expr`
