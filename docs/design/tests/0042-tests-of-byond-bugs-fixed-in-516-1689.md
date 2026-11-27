# Testing BYOND bugs fixed in 516.1689

BYOND 516.1689 fixed two bugs that auxtools reports: discarding the result of an `operator~=`
overload, and compiling a weighted `pick()` whose last entry has a zero weight into code that jumps
out of the proc. Tests therefore expect each report only on older BYOND builds:

- **`operator~=` overload:** `auxtools check` reports the `operator~=` bug for build `516.1688` and
  not for `516.1689`. The test runs `auxtools check` with `--byond-version 516.1689` and with
  `516.1688` and compares the results.
- **Zero-weight `pick()`:** Evaluating `pick(1; a, 0; a)` warns "`pick()` ends with a zero-weight
  entry, for which DreamMaker generates a jump out of the proc, whereas dmasm selects the last
  entry without a constant zero weight" only when `world.byond_build` is below `1689`. The launcher
  test reads the running build, branches on it, and looks for "warning: line 2: `pick()` ends with
  a zero-weight entry" on builds before `1689`.

## Applies to

- `tests/launcher_test/src/check.rs`: `checks`
- `tests/launcher_test/src/inspect.rs`: evaluation warning check
