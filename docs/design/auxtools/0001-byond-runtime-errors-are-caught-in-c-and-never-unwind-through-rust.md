# Catching BYOND runtime errors in C++

BYOND raises DM runtime errors as C++ exceptions. Because Rust cannot catch a C++ exception, an
exception that unwinds into a Rust stack frame terminates the process. Rust therefore never directly
calls a BYOND function that can raise a runtime error. Every such function goes through a C++
wrapper in `funcs.cpp` that catches exceptions and returns a status code (`1` for success, `0` when
an error was caught).

Many BYOND functions called by auxtools raise runtime errors to report their own failures. For
example, `call_datum_proc_by_name` raises one for an unknown proc or a `null` receiver, and some
BYOND builds raise one when `Byond_ReadPointer` reads a variable of a deleted object.

## Runtime interception

auxtools detours BYOND's `runtime` function with `runtime_hook`. The `runtime_intercepted` flag
controls how the detour handles an error:

- When `runtime_intercepted` is set, the detour throws an `AuxtoolsException` carrying the error
  message. On MinGW builds (`USE_SJLJ`), it calls `longjmp` to the current `setjmp` buffer instead.
- When `runtime_intercepted` is clear, the detour invokes each registered `RuntimeErrorHook` under a
  panic guard and then calls BYOND's original `runtime` function so the error follows BYOND's normal
  path.

`RuntimeContext` sets `runtime_intercepted` for a C++ scope and restores the previous value when the
scope exits.

## C++ wrappers and the proc-call trampoline

The BYOND function pointers (`call_proc_by_id_byond`, `get_variable_byond`, and the other `*_byond`
statics in `raw_types/funcs.rs`) are populated from Rust in `symbols::resolve_full`, but only the
C++ wrappers in `funcs.cpp` call them. Each wrapper opens a `RuntimeContext` and invokes BYOND
inside `BYOND_TRY`:

- Every wrapper except `call_proc_by_id` opens `RuntimeContext(true)`, converting BYOND runtime
  errors into exceptions that the wrapper catches. This turns failures such as an unknown proc or
  `null` receiver in `call_datum_proc_by_name`, or a deleted-object read in `read_pointer` (which
  is reported as a failed read), into failed calls.
- The `call_proc_by_id` wrapper opens `RuntimeContext(false)` so runtime errors raised by the called
  DM proc follow BYOND's normal path. The wrapper still keeps `BYOND_TRY`, though the detour does
  not throw under `RuntimeContext(false)` and the catch branch is not expected to run.

DM procs invoked from a hook must report their runtime errors through BYOND rather than unwinding
into an outer wrapper. Whenever `runtime_intercepted` is set, the proc-call detour routes calls
through `call_proc_by_id_hook_trampoline`, which opens `RuntimeContext(false)` before running the
hook or the original proc. Without this reset, a DM runtime error inside a proc called beneath an
outer `RuntimeContext(true)` wrapper would unwind into that wrapper instead of being reported by
BYOND.

When a hook replaces a proc, `call_proc_by_id_hook_trampoline` also performs `exec_proc`'s result
handling in C++, after masking the result's tag to a single byte with `clean`:

- If the caller supplied a callback, the trampoline invokes it with the result and the callback
  value.
- If the caller supplied a callback, or ignores the result (`proc_type` bit `0x04`,
  `PROC_TYPE_DISCARD_RESULT`), the trampoline releases the result and returns `null`.
- Otherwise, it returns the result.

This result handling stays in C++ because a callback can raise a BYOND runtime error as a C++
exception, which must not unwind through Rust frames.

## Rules for new wrappers

- Never call a `*_byond` function pointer directly from Rust, because an uncaught runtime error will
  unwind into Rust and terminate the process. Add a C++ wrapper in `funcs.cpp` for any new BYOND
  function.
- Choose `RuntimeContext(true)` when a BYOND error should be returned to Rust as a failed call, and
  `RuntimeContext(false)` only when BYOND should handle the error itself.

## Applies to

- `auxtools/src/hooks.cpp`: `runtime_hook`, `call_proc_by_id_hook_trampoline`,
  `PROC_TYPE_DISCARD_RESULT`
- `auxtools/src/hooks.h`: `RuntimeContext`, `runtime_intercepted`, `clean`
- `auxtools/src/raw_types/funcs.cpp`: `BYOND_TRY`, `BYOND_CATCH`, and the BYOND wrappers (in
  particular `call_proc_by_id`, `call_datum_proc_by_name`, and `read_pointer`)
- `auxtools/src/raw_types/funcs.rs`: the `*_byond` statics
- `auxtools/src/hooks.rs`: `call_proc_by_id_entry`
