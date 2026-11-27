# Breakpoints in shared proc bytecode

Several procs can share a single bytecode entry, so one bytecode address can belong to multiple
procs. After a hot reload, procs whose new bytecode is identical also share one entry. Writing a
breakpoint directly into shared bytecode would stop every proc that executes it, so the debugger
calls `give_own_entry` to give a proc its own code entry before setting a breakpoint there.

Frames that were already running before that split continue executing the old shared bytecode, which
still carries the breakpoints of the other procs. The debug server handles shared code in three
places:

- When a hot reload restores breakpoints, it gives each proc its own code entry first so other procs
  sharing the new code do not stop at those breakpoints.
- When the instruction hook encounters a breakpoint opcode in a frame whose proc now points to
  different bytecode, `runs_another_procs_code` is true and the hook skips reporting the breakpoint.
  It still restores the original instruction, as required for every breakpoint (see
  [a breakpoint opcode never reaches BYOND](0003-a-breakpoint-opcode-never-reaches-byond.md)).
- The profiler keys its per-code counters (`Profiler::codes`) by proc ID and code address together
  so shared bytecode is accounted to each proc separately.

## Applies to

- `debug_server/src/hot_reload.rs`: `restore_breakpoints` loop in `apply`
- `debug_server/src/instruction_hooking.rs`: `runs_another_procs_code`
- `debug_server/src/profiler.rs`: `Profiler::codes`
- `auxtools/src/bytecode_manager.rs`: `give_own_entry`
