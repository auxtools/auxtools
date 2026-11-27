# Testing worlds built without `DEBUG` for loops jumping to offset 0

A world built without `DEBUG` has no file or line instructions, so a loop at the start of a proc
jumps back to offset `0`. With `DEBUG` enabled, a proc starts with its file and line instructions
and the loop jumps back to a later offset instead.

To cover jumps to offset `0`, `debug_test_plain.dme` includes `debug_test_host.dm` without defining
`DEBUG`, and `debug_test` builds it as well to check that breakpoints and pauses handle a jump to
offset `0`.

## Applies to

- `tests/debug_test_host/debug_test_plain.dme`
- `tests/debug_test/src/main.rs`: plain-world checks
