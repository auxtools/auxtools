# Linux BYOND overruns its stack and clobbers saved registers

On Linux, BYOND's `call_proc_by_id` omits `PushCacheKey` from its stack count. Code such as
`L[k] ||= f(args.Copy())` then overruns a buffer in BYOND's own frame and clobbers the saved `ebx`,
`esi`, `edi`, and `ebp` registers, causing BYOND to lose its caller's registers. This alone crashes
DreamDaemon without auxtools. On Windows, BYOND checks its stack after the proc and terminates the
world itself with exception `c0000409`.

On Linux, auxtools saves those registers around the call: `call_keeping_registers` in `hooks.cpp`
calls BYOND's original `call_proc_by_id`, and `call_proc_by_id_entry` in `hooks.rs` does the same
in the instruction-hook path.

The debug test host reproduces the overrun: `overrun()` calls `/datum/overrun/proc/calls`, which
calls `stored()`, where `src.items["b"] ||= overrun_args((args.Copy() + null), ...)` overruns.
`debug_test` evaluates `overrun()` and expects 7 with the world still running. This check runs only
on Unix, because on Windows BYOND terminates the world itself. Avoid simplifying `stored()`, as
doing so can stop it from reproducing the overrun and let the test pass without checking anything.

## Applies to

- `auxtools/src/hooks.cpp`: `call_keeping_registers`, `call_original`
- `auxtools/src/hooks.rs`: `call_proc_by_id_entry`
- `tests/debug_test_host/debug_test_host.dm`: `/datum/overrun`, `overrun`
- `tests/debug_test/src/main.rs`: `overrun()` check
