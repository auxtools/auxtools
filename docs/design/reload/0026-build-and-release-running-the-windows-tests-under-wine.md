# Running Windows tests under Wine

The test service runs the Windows test binaries on Linux under Wine. Because Wine differs from
native Windows in path mapping, panic exit codes, and headless window creation, `ci/run-tests.sh`
and the test service's Wine image handle the environment as follows:

- **Linux paths on drive `Z:`:** In a default Wine prefix, the entire Linux filesystem is exposed as
  drive `Z:`. `ci/run-tests.sh` passes BYOND's path to Windows binaries as `Z:` followed by the path
  with backslashes.
- **Zero exit codes on Rust panics:** A Rust binary that panics under Wine sometimes exits with
  status `0`. `ci/run-tests.sh` fails a test with exit status `0` if its log contains a panic line
  (`thread '...' panicked at`) at the start of a line. Because a world's own output is indented in
  the log, panics from the world do not trigger this check.
- **Headless graphics driver:** Without a display, Wine has no graphics driver and sometimes refuses
  to create a window, causing `dd.exe` to exit with code `1` before logging in. Wine's `null`
  graphics driver accepts every window, though the setting applies to the entire prefix (so other
  programs in that prefix get no windows either), and an already open desktop keeps the driver it
  started with. When `WINEPREFIX` is unset, `ci/run-tests.sh` uses a dedicated prefix in
  `target/wine-prefix`, sets the `null` graphics driver there, and waits for any running
  `wineserver` to exit so the setting takes effect.
- **Prefix creation and caller-supplied prefixes:** The registry query that checks the graphics
  driver also creates the prefix once, rather than in every parallel test. When the caller supplies
  `WINEPREFIX`, `ci/run-tests.sh` leaves that prefix untouched, printing only a warning and the
  command to configure the `null` driver.
- **Test service Wine image:** The service's Wine image pins WineHQ 11.0 on Debian trixie. Its job
  script creates the prefix when a container starts and gives each test a copy, so the prefix is a
  supplied one there and `ci/run-tests.sh` does not set the `null` driver in it.

## Applies to

- `ci/run-tests.sh`
