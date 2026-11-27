# Running proc hooks from the instruction hook when calls bypass the detour

On Linux BYOND builds before `516.1664`, GCC inlined the interpreter's proc calls so they never
reach the `call_proc_by_id` detour. On those builds (`calls_bypass_detour` is `true`), auxtools
installs the instruction hook at initialization and dispatches proc hooks when a frame starts
(whenever its bytecode offset is zero). Either `handle_instruction` or the proc-entry table stub
(`first_stub_*`, which checks offset `0x14` of the frame for zero) calls `proc_starting`.

`proc_starting` runs the following steps:

1. Returns immediately if the frame is already executing `RETURN_DOT`, preventing a hook from
   running twice for the same frame.
2. Calls `on_proc_call`. If that requests new code to run at the start of this proc
   (`start_runs_new_code`), points the frame's bytecode at the proc's current code.
3. Looks up a hook for the proc and returns if none is registered.
4. Runs the hook with the frame's `src`, `usr`, and arguments. A failed hook yields `null` and is
   reported through `/proc/auxtools_stack_trace`.
5. Stores the hook's result in the frame's `.`, points the frame's bytecode at `RETURN_DOT`, and
   sets the bytecode offset to zero.

`RETURN_DOT` is a one-word buffer containing the `End` opcode (`0x00`), so the frame returns `.`
without ever executing its own bytecode. For how `src`, `usr`, arguments, and `.` are
reference-counted in this path, see
[reference ownership across auxtools and BYOND calls](0006-who-holds-the-references-when-auxtools-and-byond-call-each-other.md).

## Applies to

- `auxtools/src/hooks.rs`: `calls_bypass_detour`, `RETURN_DOT`, `proc_starting`
- `auxtools/src/instruction_hook.rs`: `init`, `handle_instruction`, `instruction_first`, the
  `first_stub_*` stubs
- `auxtools/src/lib.rs`: `start_runs_new_code`
