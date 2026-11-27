# The profiler does not count engine work between instructions

The profiler measures the time between consecutive DM instructions. Because running DM frames live
on the native stack, BYOND moves sleeping frames to heap blocks, and BYOND can run its own engine
work between instructions (such as the scheduler resuming another proc), the profiler counts a time
gap only when the next instruction's frame directly follows from the previous one:

- it is the same frame, under
  [the offset `0` rule](0001-identify-frames-by-proc-instance-id.md)
- it is called by the previous frame
- it is a caller recorded on the profiler's call trail (a return)
- it is the previous frame's parent

Any other frame lies outside the previous frame's call chain. The profiler treats the preceding gap
as engine work, does not count it, and clears the call trail. A gap after a sleep instruction is
never counted, even when execution resumes immediately.

## Applies to

- `debug_server/src/profiler.rs`: `Profiler::enter`, the `followed` test
