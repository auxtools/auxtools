# Calling BYOND's original `call_proc_by_id` from the detour

auxtools detours BYOND's `call_proc_by_id` to run proc hooks and per-call work. Because this detour
executes on every DM proc call, its entry point uses a fast naked assembly path that avoids overhead
in the common case while working around a Linux stack overrun bug in BYOND.

## Stack layout and Linux register preservation

On Linux, `call_proc_by_id` is `cdecl` without `regparm`. All twelve dwords of its arguments sit on
the stack, with the hidden return buffer first. The function pops that 4-byte return buffer itself
with `retn 4` and leaves its address in `EAX`. On Windows there is no hidden buffer. Consequently,
the proc ID sits at `[esp+0x14]` on Linux and `[esp+0x10]` on Windows, following the return address,
`usr`, and the proc type (with Linux's return-value pointer right after the return address).

Linux BYOND's `call_proc_by_id` omits `PushCacheKey` from its stack count, so expressions such as
`L[k] ||= f(args.Copy())` overrun the stack and clobber the caller's saved `EBX`, `ESI`, `EDI`, and
`EBP`. Windows BYOND preserves its callee-saved registers.

## Fast path and trampoline

The detour entry, `call_proc_by_id_entry`, is a naked function:

1. It jumps to the C++ trampoline if per-call work is pending, a world load is pending,
   `runtime_intercepted` is set, or `HOOKED_LOW_WORDS` records a hook for the low 16 bits of the
   proc ID.
2. Otherwise, it increments `PROC_CALLS` and calls BYOND's original function.

How the original is called depends on the platform:

- **Linux:** The fast path saves `EBP`, `EBX`, `ESI`, and `EDI` around the call, subtracts 12 bytes
  so the stack is 16-byte aligned at the call site, and copies the twelve argument dwords. Because
  the original pops the 4-byte return buffer, the entry adds 56 bytes (not 60) afterward before
  restoring the registers and returning with `ret 4`. The C++ trampoline calls the original through
  `call_keeping_registers` in `hooks.cpp`, which performs the same sequence: saving the four
  registers, pushing 12 bytes of padding and the twelve dwords, and adding 56 bytes after the call.
- **Windows:** The fast path jumps directly to the original without saving registers, and the
  trampoline invokes the original with an ordinary C++ call.

Both the Rust entry and `call_keeping_registers` must keep the Linux register saves, because
dropping either reintroduces clobbered registers from the stack overrun. The 56-byte adjustment
relies on the original popping its hidden return buffer.

## Hook table storage

The proc hook table is a plain static wrapped in `GameThreadOnly` rather than a `thread_local!`. In
a Linux `.so`, accessing a `thread_local!` calls `__tls_get_addr`, which is too expensive for a
table checked on every DM proc call. Only BYOND's thread may touch these statics, and looking up
hooks from any other thread is not safe.

## Applies to

- `auxtools/src/hooks.rs`: `call_proc_by_id_entry`, `PROC_ID_ARGUMENT`, `HOOKED_LOW_WORDS`,
  `GameThreadOnly`, `PROC_HOOKS`
- `auxtools/src/hooks.cpp`: `call_keeping_registers`, `call_original`, `CallProcById_Ptr`
