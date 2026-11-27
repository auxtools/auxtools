# Reporting `auxtest` results through stderr

The `auxtest` world reports its test results through stderr rather than `world.log`, because BYOND
redirects or decorates `world.log` output while leaving stderr mostly untouched.

The world writes results via the test library's `auxtest_out`, which prints lines to stderr
starting with `SUCCESS: ` or `FAILED: `. `test_runner` reads stderr, fails with every `FAILED: `
line it finds, and requires both a successful process exit and exactly one `SUCCESS: ` line.

## Applies to

- `tests/auxtest/src/lib.rs`: `auxtest_out`
- `tests/auxtest_host/auxtest_host.dm`: `auxtest_out`
- `tests/test_runner/src/main.rs`: `main`
