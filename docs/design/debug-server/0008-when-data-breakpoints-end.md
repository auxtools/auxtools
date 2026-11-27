# Data breakpoint lifetimes and detecting DM throws

A data breakpoint monitors either a pointer target or a frame slot (a local variable, an argument,
or `.`). While any data breakpoints are active, the debug server checks them before each
instruction. Because BYOND raises runtime errors on deleted pointer targets, unwinds frames without
running `Ret` or `End`, and does not pass DM `throw` statements through the runtime error hook, the
server tracks unwinding frames and `throw` opcodes explicitly to end data breakpoints at the right
time.

## Relevant BYOND behavior

- Reading a pointer whose target was deleted raises a runtime error.
- Frames unwound by a runtime error or a `throw` execute neither `Ret` nor `End`.
- `world/Error` runs above the frame of an uncaught error, so the unwound frames still appear live
  while `world/Error` executes.
- BYOND does not pass a DM `throw` through the runtime error hook.
- A pointer to an evaluated expression's own variables expires when the evaluation returns.

## Ending and validating data breakpoints

- **Deleted targets:** Data breakpoint checks run as debugger evaluations. Runtime errors from
  reading deleted targets are handled by the data breakpoint check, which ends the breakpoint with
  "target no longer exists" and never triggers the world's runtime error breakpoints.
- **Detecting `throw`:** The instruction hook always listens for the `throw` opcode (`0x12D`), even
  when it is listening for nothing else. It reports thrown exceptions directly and marks the frames
  that start unwinding.
- **Unwound frames:** When a runtime error or a `throw` begins, `note_unwinding` records the frames
  from the current frame outward. `end_unwound_watches` ends data breakpoints on those frames only
  once execution runs in a frame the exception left or in another stack. Frames that remain live or
  asleep keep their data breakpoints.
- **Returning frames:** A data breakpoint on a frame ends when the frame returns, but only after any
  stop on its `Ret` or `End` instruction. A stop there still shows the frame's data breakpoints, and
  a data breakpoint set during that stop ends when the frame returns as well.
- **Target validation:** A new data breakpoint accepts only a pointer that names something and can
  still be read. Otherwise, it is refused with "expression does not refer to a persistent target for
  a data breakpoint. Select an object variable, a list entry, or a global".

## Applies to

- `debug_server/src/server/watches.rs`: `watched_changes`, `note_unwinding`, `end_unwound_watches`,
  `end_returning_watches`, pointer check before `watched_place`
- `debug_server/src/instruction_hooking.rs`: `OPCODE_THROW` branch of `Server::handle_instruction`,
  `rest`
