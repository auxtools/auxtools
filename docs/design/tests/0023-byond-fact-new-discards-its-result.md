# Releasing hook results for `New()` calls that discard their result

BYOND invokes `New()` with the discard-result bit (`0x04` in the proc type,
`PROC_TYPE_DISCARD_RESULT`) set and never releases its result. Because a hook's return value holds a
reference, leaving it unreleased would leak one reference per hooked `New()` call.

When a hook returns, `call_proc_by_id_hook_trampoline` performs the same result handling as BYOND's
`exec_proc`: it calls the caller's callback if there is one, and if there is a callback or the
discard-result bit is set, it releases the result and returns `null` to BYOND.

`test_hooks` verifies this behavior by holding a string, running `auxtest_new_discard` (which calls
a hooked `New()` returning that string), and checking that the string's reference count is
unchanged.

## Applies to

- `auxtools/src/hooks.cpp`: `PROC_TYPE_DISCARD_RESULT`, `call_proc_by_id_hook_trampoline`
- `tests/auxtest/src/hooks.rs`: `test_hooks`
- `tests/auxtest_host/auxtest_host.dm`: `/datum/auxtest_discard`, `auxtest_new_discard`
