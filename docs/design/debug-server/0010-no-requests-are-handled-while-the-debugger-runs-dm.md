# Blocking client requests while the debugger runs DM

The debug server handles client requests from the instruction hook between DM instructions, and the
same hook also sees DM instructions executed by the debugger itself. To prevent a request from
starting inside another request (such as a pause request entering a nested pause loop while a pause
is waiting for an evaluation), the server sets `in_eval` whenever it runs its own DM code.

The debugger runs DM code of its own:

- evaluating an expression while the world is paused and the pause loop waits for the result
- assembling an expression, which can call procs such as `text2path()` and `fcopy_rsc()`
- reading a type path for a hover through the evaluation stub
- assembling a hot reload, which may evaluate DM to resolve paths and resources

Before running any of this DM code, the server sets `in_eval` and configures the instruction hook to
receive every instruction, clearing `in_eval` afterward. This covers expression evaluation including
assembly (`eval_expr_plain`), direct reads (`read_expr`), hot reloads (`apply_hot_reload`), and data
breakpoint checks. While `in_eval` is set, the hook processes no requests, takes no step actions,
and does not pause. Any new code that runs DM from a request handler must set `in_eval` in the same
way. Because of this guard, a pause cannot interrupt a long-running evaluation, and a client that
needs to leave waits only a short time for the answer instead.

Outside an evaluation, `Server::process` handles every pending request in a single pass and pauses
afterward if any request asked for a pause. The pause flag is accumulated with `|=` rather than
`||`, because short-circuiting with `||` would dequeue and drop every remaining request once an
earlier request asked for a pause.

## Applies to

- `debug_server/src/instruction_hooking.rs`: `Server::handle_instruction`
- `debug_server/src/server/eval.rs`: `eval_expr_plain`
- `debug_server/src/server/reading.rs`: `read_expr`
- `debug_server/src/server/reload.rs`: `apply_hot_reload`
- `debug_server/src/server.rs`: `Server::process`
