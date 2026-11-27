# Locating the launcher binary when Cargo's path is gone

Cargo passes the launcher binary's path to tests in the launcher's package through
`CARGO_BIN_EXE_auxtools`, but CI builds the test binaries in one job and copies them elsewhere to
run, leaving that recorded path nonexistent.

`use_launcher` therefore accepts `CARGO_BIN_EXE_auxtools` only when the file exists at that path.
Otherwise, `launcher_path` looks for `auxtools` (`auxtools.exe` on Windows) one and three
directories above the test binary, and panics with `"build it first"` if the binary is in neither
location.

## Applies to

- `tests/support/src/paths.rs`: `use_launcher`, `launcher_path`
