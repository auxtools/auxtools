# Using long path names for copied test worlds on Windows

On Windows, `dap_test` converts the directory of any copied world that includes a skin or script to
its long path name with `GetLongPathNameW` before building it.

Windows can name the temporary directory in 8.3 short form, such as `RUNNER~1` on CI runners. When a
world sits under an 8.3 path, DreamMaker rejects included `.dmf` or `.dms` files as outside the
world, causing compilation to fail. Calling `long_name` avoids that failure on Windows and returns
the directory unchanged on other platforms.

## Applies to

- `tests/dap_test/src/groups/reloading.rs`: `long_name`
- `tests/dap_test/src/groups/skins.rs`: caller of `long_name`
- `tests/dap_test/src/groups/scripts.rs`: caller of `long_name`
