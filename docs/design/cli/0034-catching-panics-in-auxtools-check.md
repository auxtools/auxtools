# Catching and reporting panics in `auxtools check`

`auxtools check` parses a project and compiles every proc with dmasm to list known BYOND bugs.
During compilation, `Snapshot::compile` wraps each proc in `std::panic::catch_unwind` and returns
the diagnostic "the compiler panicked" (`Id::CompilerPanicked`) when a panic occurs, allowing
`check` to list that proc as "not checked" and continue. Because a large codebase can produce many
such panics, Rust's default panic hook would print a message and a location for each one, so
`check` keeps the hook quiet while it runs.

Installing an empty panic hook for the rest of the process would also silence panics that nothing
catches, such as a panic in the parser, ending the command with an exit status and no output.
Instead, `catching_panics` wraps both the parse and the compile loop in `catch_unwind` with a
scoped panic hook:

- On the calling thread, the hook stays quiet and records the place (`file:line:column`) and
  message of the last panic.
- On every other thread, the hook passes panics to the hook that was installed before, so they
  still print normally and cannot be mistaken for the panic that ended the check.
- After `catch_unwind` finishes, `catching_panics` puts the earlier hook back.

A panic caught by the compiler never reaches `catching_panics` and leaves nothing to report. A
panic that reaches `catching_panics` is returned as `Panicked { place, message }`, and `check`
prints "error: internal error in auxtools. Include this in a bug report: PLACE: MESSAGE" and exits
with code 1.

## Applies to

- `auxtools-cli/src/cli/check.rs`: `catching_panics`, `Panicked`, `check`
- `dmasm/reload/src/reload.rs`: `Snapshot::compile`, `Snapshot::known_bugs`
