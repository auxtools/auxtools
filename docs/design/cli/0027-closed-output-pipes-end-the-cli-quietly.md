# Exiting quietly when an output pipe closes

When CLI output is piped to a consumer such as `| head` and the reader closes early, writes to
stdout fail. Rust ignores `SIGPIPE` by default so writes return errors rather than stopping the
process, and `print!` panics on write failure. On Unix, restoring the default `SIGPIPE` action lets
a closed pipe terminate the process quietly, whereas Windows has no equivalent signal.

The CLI handles closed output pipes across both platforms in three ways:

- **Unix `SIGPIPE` handling:** The CLI restores the default `SIGPIPE` action at startup so writing
  to a closed pipe terminates the process quietly.
- **`say!` and `say_part!` macros:** Instead of `print!`, the CLI writes through `say!` and
  `say_part!`, which exit quietly on a failed write rather than panicking. This covers Windows.
- **Streaming `trace` output:** `trace` checks each line write as it streams and stops once stdout
  closes.

The detached launcher is the one exception. It ignores `SIGPIPE` because the running world depends
on it, as described in [the detached launcher](0025-the-detached-launcher.md).

## Applies to

- `auxtools-cli/src/cli.rs`: `say!`, `say_part!`, `say`, `run`
- `auxtools-cli/src/cli/trace.rs`: `Out::line`
