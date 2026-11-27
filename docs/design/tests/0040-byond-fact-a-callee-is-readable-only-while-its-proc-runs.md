# A `/callee` value is readable only while its proc runs

A `/callee` value, available starting in BYOND 516, can be read only while the proc it represents
is running.

To test debugger inspection of `/callee` values, the debug test host stores `callee` in a local
variable named `this_proc` at the start of `main_loop`, which stays active for the lifetime of the
world. `debug_test` evaluates `this_proc` in `main_loop`'s frame and verifies the fields the
debugger reports for it. On BYOND builds before 516, `this_proc` does not exist and the check is
skipped.

## Applies to

- `tests/debug_test_host/debug_test_host.dm`: `main_loop`
- `tests/debug_test/src/main.rs`: "a callee shows its proc" check
