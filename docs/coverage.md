# Line coverage

auxtools records source line coverage for a whole world run through `auxtools launch`, counting how
many times execution reaches each executable source line (without branch coverage or per-proc
figures). It requires no changes to your DM source and no extra library, replacing the removed
`auxcov` crate (`start_code_coverage()` and `stop_code_coverage()`) with built-in support in the
debug library and launcher. Your world must be compiled with debug information (`#define DEBUG` in
the `.dme`) so DreamMaker writes line numbers into the `.dmb`. See [README.md](../README.md) for
installation.

## Recording a whole run

```sh
auxtools launch tgstation.dmb --lcov coverage.info
auxtools launch tgstation.dmb --no-hot-reload --lcov coverage.info --cobertura coverage.xml
```

Pass `--lcov FILE`, `--cobertura FILE`, or both to `auxtools launch` to record coverage from the
world's first line (including global variable initializers) until it exits. This mode is suited to
CI runs where a unit-test world executes its tests and shuts itself down. You can add
`--no-hot-reload` in CI where source files are not edited, though coverage works with or without it.

Across a `world.Reboot()`, the launcher records each world load from its first line and combines
them into one report by summing hit counts for each line. The debug library sends its coverage data
before each reboot and when the world shuts down (`del(world)`, `world.Del()`, `auxtools stop`, or
`Ctrl+C` in the launcher, including when `world/Del()` raises a runtime error). When the world
process exits, the launcher writes the requested files and prints a summary line such as:

```text
Coverage: 412 of 9131 lines covered (4.5%) in 37 of 652 files
```

With `--detach`, the background launcher writes the report files when the world exits (for example,
after `auxtools stop`).

If the world process crashes or is terminated instead of shutting down, it sends no coverage data
and no report file is written. The launcher keeps its normal exit codes (`0` when the world exited
with code `0`, `4` when it exited with another code or was terminated), except that when the world
exits with code `0` without reporting coverage, the launcher prints `error: no coverage report
written: world exited without reporting coverage` and exits with code `1`.

If the world was compiled without debug information, the launcher prints `Coverage disabled: the
world has no line numbers because it was compiled without debug information. Add #define DEBUG to
the .dme and rebuild` and lets the world run normally.

## Report formats and what is counted

Both `--lcov` and `--cobertura` list every executable line in every file of the world with its hit
count, including lines with `0` hits. File paths are the names DreamMaker recorded in the world's
debug information, relative to the `.dme` directory (such as `code/game/atoms.dm`), with backslashes
written as forward slashes.

- **LCOV (`--lcov FILE`):** Writes one record per file containing `SF:`, a `DA:line,hits` entry for
  each executable line, `LF:`, and `LH:`. Supported by `genhtml`, Codecov, Coveralls, and editor
  extensions that show coverage in the gutter.
- **Cobertura XML (`--cobertura FILE`):** Writes one `<package>` per directory, one `<class>` per
  file, and one `<line>` (`number` and `hits`) per executable line, with branch figures set to `0`
  and `<source>` set to `.` so tools resolve file paths from their working directory. Supported by
  GitLab, Jenkins, Azure Pipelines, and Codecov.

Helper procs added by the debugger and expressions evaluated with `auxtools eval` or the Debug
Console are not included in coverage reports, though world procs called by those expressions are
counted. After a hot reload during a recording, the report reflects the code in the world at the end
of the recording.

## Overhead and limits

From the command line, coverage is recorded for a whole run only through `auxtools launch`, and
cannot be started on a world that is already running. The debug server supports one active coverage
recording per world at a time, owned by the client that started it, and discards the recording if
that client disconnects.

While recording, the debug library is called once per executed source line rather than once per
bytecode instruction, so the slowdown is much smaller than instruction profiling. When the world
shuts down or reboots, the debug library reads every proc in the world once to list all executable
lines.
