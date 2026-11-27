# Identifying frames by BYOND proc instance ID

The debug server must recognize the same call frame across pauses, across steps of a step command,
between data breakpoint checks, and between profiler samples. Memory addresses are not stable enough
for this, so auxtools identifies frames using BYOND's proc instance IDs, dedicated sleeping-frame
records, and an offset `0` continuation check.

## Why addresses are unstable

Several BYOND behaviors invalidate raw context and proc instance pointers across execution:

- BYOND reuses freed execution context addresses and proc instance memory for later calls, so a new
  call can receive the exact address of a previous one.
- Running frames live on the native stack. When a proc sleeps, BYOND moves its frame to a new heap
  block, changing its context address. The proc instance stays the same call, though the instance's
  address changes on its first sleep.
- BYOND stores a proc instance's ID inside the instance itself, in the field auxtools names
  `argslist_idx`. That ID stays the same after the first sleep, while a `spawn` copy of a call
  receives a different ID.
- Debugger evaluations can end sleeping procs and create new ones, shifting the positions of other
  procs in BYOND's sleep queue.
- On the running stack, an execution context's `parent` field points to its caller. On a sleeping
  stack, BYOND stores frames in reverse order, so the same field points to the callee. auxtools
  exposes this field as `parent_context`, a name that is only accurate for the running stack.

## Frame IDs, threads, and sleeping stacks

Because the protocol uses integer frame IDs, the frame kind is encoded in the numeric range. IDs
below `SLEEPING_FRAMES` (`1 << 30`) represent depths in the running stack, innermost first. IDs at
or above `SLEEPING_FRAMES` refer to sleeping frames.

Each sleeping frame is stored as a `SleepingFrame` holding its stack's thread ID, its depth, and its
proc instance address, never its position in the sleep queue. A `SleepingFrame` receives the next
free ID the first time it is shown, and that ID remains valid while the frame sleeps even if
evaluations end or create other sleeping procs.

A stack's thread ID is the BYOND ID of its outermost proc instance plus one, excluding frames from
the launcher's spawn helpers when finding that outermost frame. Thread `0` represents the running
stack.

To find a sleeping stack's innermost frame and its caller, `get_proc_ctx` starts at the instance's
context and follows `parent_context` until it reaches `null`. The last context is the innermost
frame, and the one before it is the caller. `CallStacks::from_context` reverses sleeping stacks for
the same reason. Any code walking `parent_context` must know which kind of stack it is on, and
renaming the accessor would make this distinction visible in the code.

## Data breakpoints and the offset 0 rule

A data breakpoint on a local variable, an argument, or `.` records BYOND's proc instance ID and the
proc ID rather than a memory address. Because a watched frame can end between checks and its ID can
then belong to another call, the data breakpoint ends if the instance's proc differs from the
recorded one or if the context is at the start of a new call.

A context at offset `0` can be either a new call that reused the previous call's context address or
the same call after a jump back to its start. The same context continues the same call only if the
last instruction it ran could jump. `starts_call` checks this using the stepper's last frame and
last opcode, which the stepper records on every instruction it receives. Because `starts_call` is
valid only until the stepper records the current instruction, callers in the instruction hook must
query it before that point.

The opcodes that can jump, listed in `can_jump`, are `Jmp`, `Jnz`, `Jz`, `Spawn`, `Switch`,
`PickSwitch`, `SwitchRange`, `JmpOr`, `JmpAnd`, `JmpLoop`, `JnzLoop`, `JzLoop`, `ForRange`,
`ForRangeStep`, `Try`, `Catch`, `TryJmp`, `SetCacheJmpIfNull`, and `SetCachePopJmpIfNull`.

## Profiler frames

A profiler `Frame` pairs the context address with the proc instance address, so a later call that
reuses a freed context address does not match the earlier frame. At offset `0`, the profiler applies
the same rule: the same frame continues the same call only if the last instruction could jump, and
otherwise starts a new call.

## Applies to

- `debug_server/src/server/pause.rs`: `SLEEPING_FRAMES`, `SleepingFrame`,
  `State::sleeping_frame_id`, `thread_of`, `shown`
- `debug_server/src/instruction_hooking.rs`: `get_proc_ctx`, `starts_call`, `Stepper::last_frame`
- `debug_server/src/server/watches.rs`: `Place::Slot`, `instance_id`, `watched_changes`
- `debug_server/src/profiler.rs`: `Frame`, `can_jump`, the `followed` test
- `auxtools/src/debug.rs`: `CallStacks::from_context`
