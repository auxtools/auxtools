# Test time limits derived from Wine measurements

Test timeouts are sized from waits measured under Wine, which is the slowest environment where the
suites run alongside Linux and Windows, often several at once. Each limit must be long enough that a
slow but working world passes, yet short enough that a dead world fails the run soon.

| Limit | Value | Reason |
| --- | --- | --- |
| `test_support::dap::TIMEOUT` | 15 s | The longest wait measured for a DAP response or event, with five suites at once under Wine, was 1.2 s. |
| `test_support::dap::START_TIMEOUT` | 90 s | For a world to start and connect. Under Wine, about one `dd.exe` in 500 had not connected after 15 s. |
| `TIMEOUT` in `test_support::debug` | 15 s | A request takes about a second under Wine with five suites at once. The limit is short enough that a dead world fails the run soon. |
| `START_TIMEOUT` in `test_support::debug` | 30 s | For connecting to a starting world's debug server. No measurement is recorded for it. |
| `test_support::cli::TIMEOUT` | 20 s | Longer than the `--timeout 15` that the tests pass to commands. A command that times out then reports what it waited for before the helper gives up. |
| `test_support::cli::COMMAND_LIMIT` | 60 s | Longer than any command's own time limit. A command still running at this limit is stopped, and the check fails with the command and what it printed. |

Keeping the CLI helper timeouts above the command timeouts ensures that when a world stops
answering, the test reports the command's own timeout message rather than the helper's. Lowering the
startup limits reintroduces rare startup failures under Wine.

## Applies to

- `tests/support/src/dap.rs`: `TIMEOUT`, `START_TIMEOUT`
- `tests/support/src/debug.rs`: `TIMEOUT`, `START_TIMEOUT`
- `tests/support/src/cli.rs`: `TIMEOUT`, `COMMAND_LIMIT`
