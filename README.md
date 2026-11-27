# auxtools

auxtools is a debugger, instruction profiler, experimental hot reloader, static bug checker, and
command-line launcher for BYOND worlds on Windows and Linux. Use it through the
[VS Code extension](editors/vscode/README.md) or the [`auxtools` CLI](docs/cli.md), build native
BYOND extensions against the [`auxtools` Rust library](docs/rust-library.md), or work with DM
bytecode directly through [dmasm](docs/dmasm.md).

If you want to build or test auxtools from source, see [CONTRIBUTING.md](CONTRIBUTING.md).

## Installation

- **VS Code extension (`sohpea.auxtools`):** Install **auxtools Debugger** from the Visual Studio
  Marketplace or Open VSX, or install `auxtools-VERSION.vsix` from
  [GitHub Releases](https://github.com/auxtools/auxtools/releases). The extension requires VS Code
  1.80 or later, bundles both Linux and Windows binaries, and adds the launcher directory to `PATH`
  in VS Code terminals by default (`auxtools.addToTerminalPath`).
- **Command-line tools:** Download `auxtools-linux.tar.gz` (32-bit x86, glibc 2.35 or later) or
  `auxtools-windows.zip` (Windows 10 or later) from
  [GitHub Releases](https://github.com/auxtools/auxtools/releases) and extract both files into the
  same directory. The `auxtools` launcher expects `libdebug_server.so` (Linux) or
  `debug_server.dll` (Windows) beside it. Matching symbol archives (`auxtools-linux-symbols.tar.gz`
  and `auxtools-windows-symbols.zip`) are published alongside each release.
- **Rust library:** The `auxtools` crate is not published on crates.io. Depend on this repository
  by Git release tag (see [The `auxtools` Rust library](docs/rust-library.md)).
- **From source:** See [CONTRIBUTING.md](CONTRIBUTING.md) for toolchain setup and build commands.

## Quick start

Compile your world with DreamMaker as usual. The launcher does not compile `.dmb` files. To use line
breakpoints, make sure `DEBUG` is defined in your `.dme` (`#define DEBUG`) before compiling so
DreamMaker writes line numbers into the `.dmb`. Many codebases define it already. Without it, line
breakpoints stay unverified, though breakpoints on procs still work.

- **In VS Code:** Open a workspace containing a compiled `.dmb` and press `F5` (or run **auxtools:
  Debug in DreamDaemon**). See the [VS Code extension README](editors/vscode/README.md) for launch
  configurations, settings, and editor-specific features.
- **From the command line:** Start the world with:

```sh
auxtools launch WORLD.dmb
```

The launcher loads the debug library into BYOND automatically without requiring any changes to your
world's source. By default, worlds run in console DreamDaemon (`dd.exe` on Windows, `DreamDaemon`
on Linux) with `-trusted` passed to BYOND. On Windows, `--server` also supports windowed
DreamDaemon (`dreamdaemon-gui`, which adds `-close` by default) and `dreamseeker`. Passing
`-- ARGS...` replaces the default BYOND arguments.

The launcher locates BYOND automatically from `--byond PATH`, the `BYOND_SYSTEM` environment
variable, and standard install locations (the Windows registry and `Program Files` on Windows;
`~/byond`, `/usr/local/byond`, and `/usr/share/byond` on Linux). Across a `world.Reboot()`, the
launcher reconnects automatically and restores breakpoints and hot reloads. If the debug library
cannot initialize (for example, on an unsupported BYOND build), the world keeps running without
debugging, `auxtools launch` and `auxtools list` report the reason, and `auxtools stop` can still
shut it down.

## Debugger features

- Line, function (proc), instruction, and data breakpoints with DM conditions and hit counts
- Logpoints on source lines with interpolated DM expressions
- Pausing on uncaught or caught runtime errors, with comma-separated include/exclude message filters
- Stepping by source line or bytecode instruction (`step-over`, `step-in`, `step-out`)
- Call stacks for both active and sleeping procs
- Viewing and modifying local, argument, object, and global variables
- Evaluating DM expressions and multi-statement proc bodies, plus spawned evaluations
  (`eval --spawn`) that can sleep
- Live proc disassembly
- Persistent event journal tracking runtime errors, hot reloads, console output, world loads, and
  spawned evaluations across reboots

Multiple debugger clients (VS Code windows and CLI commands) can connect to the same world at once.
Each session manages its own breakpoints, data breakpoints, and runtime error filters. Pausing
stops the world for every session, and any session can resume it.

## Command-line workflow

The `auxtools` CLI launches, inspects, steps, traces, profiles, and manages local worlds from a
terminal or script, with `--json` output and configurable timeouts. Editors integrate through
`auxtools dap`. See [Command-line workflow](docs/cli.md) for the command list, scripting options,
example session, and exit codes.

## Instruction profiling

`auxtools profile` measures DM execution time by proc, source line, and bytecode instruction, and
can produce text, JSON, or a standalone interactive HTML report. See
[Instruction profiling](docs/profiling.md) for CLI usage, timing methodology, and overhead limits,
or [Instruction profiling](editors/vscode/README.md#instruction-profiling) in the VS Code extension
README.

## Line coverage

`auxtools launch --lcov` and `--cobertura` record source line hit counts across a world run for
worlds compiled with `#define DEBUG`, printing a terminal summary and writing LCOV tracefiles or
Cobertura XML reports with no changes to DM source. See [Line coverage](docs/coverage.md) for usage,
report formats, and overhead.

## Hot reload (experimental)

Hot reload is enabled by default for launched worlds and applies supported changes to procs, types,
variables, resources, `.dmf` skins, and inline `client/script` text on save. See
[Hot reload (experimental)](docs/hot-reload.md) for supported edits, reboot and rebuild rules, and
runtime behavior.

## Checking for known BYOND bugs

`auxtools check` compiles every proc in a project with `dmasm-compiler` and reports known BYOND
compiler and runtime bugs for a target build without writing a `.dmb` or requiring BYOND to be
installed. See [Checking for known BYOND bugs](docs/bug-checker.md) for options, output formats, and
exit codes.

## The `auxtools` Rust library

The `auxtools` crate lets 32-bit native Rust `cdylib` libraries hook DM procs, read and mutate BYOND
values, lists, and strings, call DM procs, and subscribe to runtime errors. See
[The `auxtools` Rust library](docs/rust-library.md) for Cargo setup, Rust and DM examples, and
lifecycle and thread-safety rules.

## Supported BYOND builds

auxtools supports BYOND `515.1637–515.1647` and `516.1648–516.1689` on Windows and Linux, and
attempts newer builds with resource reload and `text2path()` negative-cache updates disabled. See
[Supported BYOND builds](docs/byond-builds.md) for tested and known-broken build lists and how to
check a new BYOND build with `check_recipes`.

## Documentation

- [VS Code extension README](editors/vscode/README.md): Launch configurations, extension settings,
  UI commands, evaluation details, and hot reload limits
- [Command-line workflow](docs/cli.md): CLI command list, world selection, scripting, timeouts, and
  exit codes
- [Instruction profiling](docs/profiling.md): Profiling with `auxtools profile`, timing measurement,
  and overhead limits
- [Line coverage](docs/coverage.md): Recording line coverage with `auxtools launch`, LCOV and
  Cobertura XML reports, and overhead
- [Hot reload (experimental)](docs/hot-reload.md): Supported live edits, reboot and rebuild rules,
  skins and client scripts, and runtime behavior
- [Checking for known BYOND bugs](docs/bug-checker.md): Static checking with `auxtools check`,
  options, output formats, and exit codes
- [The `auxtools` Rust library](docs/rust-library.md): Writing native Rust `cdylib` extensions with
  `#[hook]`, DM setup, and thread safety
- [Supported BYOND builds](docs/byond-builds.md): Tested and known-broken builds, newer build
  behavior, and checking recipes on a new build
- [dmasm](docs/dmasm.md): `dmasm-bytecode`, `dmasm-compiler`, and `dmasm-reload` crates, along with
  the [proc compiler reference](docs/dmasm-compiler.md) and
  [DM bytecode specification](docs/dmasm-specification.md)
- [CONTRIBUTING.md](CONTRIBUTING.md): Building from source, running unit and integration tests, CI
  checks, and releasing
- [Design notes](docs/design/README.md): Design decisions, internal mechanisms, and facts about
  BYOND that the code depends on

## License

This repository, including the `auxtools` and `auxtools-impl` crates, is licensed under the MIT
License. See [LICENSE](LICENSE).

The following components are licensed under GPL-3.0:

- `auxtools-cli` ([auxtools-cli/LICENSE](auxtools-cli/LICENSE))
- `debug_server` ([debug_server/LICENSE](debug_server/LICENSE))
- `auxtools-protocol` ([auxtools-protocol/LICENSE](auxtools-protocol/LICENSE))
- `auxtools-dm` ([auxtools-dm/LICENSE](auxtools-dm/LICENSE))
- `instruction_hooking` ([instruction_hooking/LICENSE](instruction_hooking/LICENSE))
- `dmasm`, `dmasm-bytecode`, `dmasm-compiler`, and `dmasm-reload` ([dmasm/LICENSE](dmasm/LICENSE))
- VS Code extension in `editors/vscode` ([editors/vscode/LICENSE](editors/vscode/LICENSE))
