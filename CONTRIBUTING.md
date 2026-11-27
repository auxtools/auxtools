# Contributing to auxtools

This guide covers building auxtools from source, running the unit, extension, and BYOND integration
test suites, packaging the VS Code extension, and cutting releases.

The design notes in [docs/design/](docs/design/README.md) record what the code does that is not
obvious and why. When you change code that a note describes, update the note in the same change.

## Repository layout

| Path | License | Purpose |
| --- | --- | --- |
| `auxtools/` | MIT | Core Rust library for hooking DM procs and working with BYOND values |
| `auxtools-impl/` | MIT | Procedural macros (`#[hook]`, `#[init]`, `#[shutdown]`, `#[runtime_handler]`) |
| `auxtools-cli/` | GPL-3.0 | `auxtools` launcher, CLI commands, hot reload watcher, and DAP adapter |
| `debug_server/` | GPL-3.0 | In-process debug library (`libdebug_server.so` / `debug_server.dll`) loaded into BYOND |
| `auxtools-protocol/` | GPL-3.0 | IPC protocol framing and message types (`PROTOCOL_VERSION` in `auxtools-protocol/src/lib.rs`) |
| `auxtools-dm/` | GPL-3.0 | DM token formatting and `.dmb` resource checksum reader |
| `instruction_hooking/` | GPL-3.0 | Bytecode instruction hook interface used by `debug_server` |
| `dmasm/` | GPL-3.0 | DM bytecode assembler/disassembler, compiler, and hot reload planner ([dmasm/README.md](dmasm/README.md)) |
| `editors/vscode/` | GPL-3.0 | VS Code extension (`sohpea.auxtools`), JS tests, and `.vsix` packager |
| `tests/` | — | Integration test suites (`test_runner` with the `auxtest` `cdylib`, `debug_test`, `hot_reload_test`, `launcher_test`, `dap_test`) and host worlds |
| `tools/debug_client/` | — | Standalone debug protocol test client |
| `ci/` | — | CI helper scripts, Windows cross-compilation setup, and BYOND build lists |

## Prerequisites

### Rust toolchain

Install Rust via [rustup](https://rustup.rs). [rust-toolchain.toml](rust-toolchain.toml) pins the
toolchain and requests both 32-bit targets (`i686-unknown-linux-gnu` and `i686-pc-windows-msvc`),
because BYOND's engine is 32-bit x86. On the first build, rustup installs the pinned toolchain
automatically.

Minimum supported Rust versions (verified in CI):

- **`1.95`** for the full workspace (`Cargo.toml`)
- **`1.88`** for `auxtools`, `auxtools-impl`, and `dmasm-bytecode`

Cargo fetches three crates from two Git repositories: `dreammaker` and `dap-types` follow the
`master` branch of [auxtools/SpacemanDMM](https://github.com/auxtools/SpacemanDMM), and `byond-scan`
comes from Absolucy/byond-scan at a revision named in `Cargo.toml`. `Cargo.lock` records the exact
commit of each.

### Platform tools

- **Linux:** Install 32-bit C/C++ multilib toolchains to compile (`gcc-multilib` and `g++-multilib`
  on Ubuntu/Debian). To run 32-bit binaries and Linux BYOND on a 64-bit host, also install 32-bit
  runtime libraries:

```sh
sudo dpkg --add-architecture i386
sudo apt-get update
sudo apt-get install build-essential gcc-multilib g++-multilib libc6-i386 libstdc++6:i386
```

- **Windows (native):** Install Visual Studio Build Tools with the C++ build tools workload so
  `cl.exe`, `ml.exe`, and the Windows SDK are available for `i686-pc-windows-msvc`.
- **Cross-compiling Windows from Linux:** Install `clang`, `lld`, and `llvm`, then fetch the pinned
  Windows SDK and CRT with `ci/msvc/fetch-sdk.sh` (see below).
- **VS Code extension packaging and tests:** Python 3 and Node.js.

## Building

### Linux

```sh
cargo build --workspace --release --target i686-unknown-linux-gnu
```

The launcher and debug library are written to `target/i686-unknown-linux-gnu/release/auxtools` and
`target/i686-unknown-linux-gnu/release/libdebug_server.so`.

### Windows (native)

```sh
cargo build --workspace --release --target i686-pc-windows-msvc
```

The launcher and debug library are written to `target/i686-pc-windows-msvc/release/auxtools.exe`
and `target/i686-pc-windows-msvc/release/debug_server.dll`.

### Windows (cross-compiling on Linux)

Fetch the pinned Windows SDK and CRT (about 800 MB via `xwin`, and running the script accepts
Microsoft's SDK license), load the cross-compilation environment, and build:

```sh
ci/msvc/fetch-sdk.sh ~/.cache/msvc-sdk
export MSVC_SDK=~/.cache/msvc-sdk
. ci/msvc/env.sh
cargo build --workspace --release --target i686-pc-windows-msvc
```

`ci/msvc/env.sh` links the CRT statically (`-C target-feature=+crt-static`) and wires `clang-cl`,
`lld-link`, `llvm-lib`, and the `ci/msvc/bin/ml.exe` assembler shim.

### Packaging the VS Code extension

After building the Linux target, the Windows target, or both, package the `.vsix` from the
repository root:

```sh
python3 editors/vscode/package.py
```

The script bundles whichever platform binaries exist under `target/<target>/release/` into
`auxtools-VERSION.vsix` (stripping debug sections from Linux ELF binaries when `strip` is
available). Pass `--profile NAME` (default `release`) or `--out FILE` to customize the build
profile or output path.

## Formatting and linting

CI checks formatting with a pinned nightly rustfmt toolchain:

```sh
cargo +nightly-2026-10-24 fmt --all
```

Run Clippy across the workspace with the CI script:

```sh
ci/clippy.sh --locked --target i686-unknown-linux-gnu --workspace
```

## Testing

### Rust unit tests

These tests do not require BYOND:

```sh
cargo test --release --locked --target i686-unknown-linux-gnu \
  -p auxtools-cli -p auxtools -p auxtools-protocol -p auxtools-dm \
  -p debug_server -p dmasm -p dmasm-bytecode -p dmasm-compiler \
  -p dmasm-reload -p test_support
```

### VS Code extension tests

Run the JavaScript test files with Node.js (and verify a packaged `.vsix` with `package.test.js`):

```sh
node editors/vscode/expression.test.js
node editors/vscode/unsaved.test.js
node editors/vscode/buildflags.test.js
node editors/vscode/report.test.js
node editors/vscode/messages.test.js

python3 editors/vscode/package.py --out /tmp/auxtools.vsix
python3 -m zipfile -e /tmp/auxtools.vsix /tmp/vsix
node editors/vscode/package.test.js /tmp/vsix
```

### BYOND integration tests

The integration test suites (`test_runner`, `debug_test`, `hot_reload_test`, `launcher_test`, and
`dap_test`) require a BYOND installation and a `debug_server` library built with the
`debug_server/test-commands` feature enabled (release builds never include this feature, and CI
verifies its absence).

Build with test commands enabled into a separate target directory, then invoke `ci/run-tests.sh
PROGRAMS BYOND [TEST...]`:

```sh
cargo build --workspace --release --target i686-unknown-linux-gnu \
  --features debug_server/test-commands --target-dir target/test-commands
ci/run-tests.sh target/test-commands/i686-unknown-linux-gnu/release /path/to/byond
```

Useful environment variables for `ci/run-tests.sh`:

- `WINE=wine` — Run Windows test binaries on Linux under Wine (uses `WINEPREFIX`, or creates
  `target/wine-prefix` with Wine's null graphics driver if unset).
- `SERIAL=1` — Run suites sequentially from the repository root with no per-suite timeout instead of
  in parallel under `target/run-tests/`.
- `TIMEOUT=SECONDS` — Per-suite timeout in parallel mode (default `900`).

### BYOND recipes and DreamMaker comparison

- To check BYOND symbol recipes against a `libbyond.so` or `byondcore.dll` without running a world,
  see [Checking a new BYOND build](docs/byond-builds.md#checking-a-new-byond-build).
- To compare `dmasm-compiler` bytecode output against DreamMaker across the regression corpus, see
  [Comparing with DreamMaker](dmasm/README.md#comparing-with-dreammaker).

## Continuous integration and artifacts

[.github/workflows/ci.yml](.github/workflows/ci.yml) runs formatting, Clippy, MSRV builds (`1.95`
and `1.88`), Linux and Windows builds, binary compatibility checks, unit and extension tests, BYOND
integration tests, and `.vsix` packaging.

- **Binary compatibility checks:** `ci/check-linux.sh` enforces 32-bit x86 ELF output, glibc
  `<= 2.35`, libstdc++ (`GLIBCXX`) `<= 3.4.30`, and a fixed list of system shared libraries.
  `ci/check-windows.sh` enforces 32-bit x86 PE output, Windows 10 (`<= 10`) subsystem version, and a
  fixed list of Windows 10 system DLL imports.
- **BYOND test coverage:** Pushes to `master` and pull requests from this repository run a
  representative set of BYOND builds on Linux and Wine, listed in `BUILDS` in
  `.github/workflows/ci.yml`. Pull requests from forks run builds, unit and extension tests, and
  packaging without the BYOND integration tests. Release runs test every build in
  [ci/supported-builds.txt](ci/supported-builds.txt) on Linux and Wine (with the expected failures
  listed in [ci/known-broken-builds.txt](ci/known-broken-builds.txt)) and the representative set on
  native Windows.

| Artifact | Contents |
| --- | --- |
| `auxtools-linux.tar.gz` | `auxtools` and `libdebug_server.so` (debug sections stripped) |
| `auxtools-linux-symbols.tar.gz` | `auxtools.debug` and `libdebug_server.so.debug` (place beside the binaries for GDB) |
| `auxtools-windows.zip` | `auxtools.exe` and `debug_server.dll` |
| `auxtools-windows-symbols.zip` | `auxtools.pdb` and `debug_server.pdb` |
| `auxtools-VERSION.vsix` | VS Code extension bundling both Linux and Windows binaries |

## Releasing

Trigger **Actions > Release > Run workflow** on `master` and provide a `MAJOR.MINOR.PATCH` version
number (for example, `4.0.0`). [.github/workflows/release.yml](.github/workflows/release.yml)
handles the entire release pipeline:

1. Validates that the workflow is running on `master` and that `VERSION` is valid and newer than
   existing tags and `master`'s current version.
2. Uses `ci/set-version.py` to commit `VERSION`, creates an annotated `vVERSION` tag, and commits
   the next patch version with a `-dev` suffix.
3. Calls [.github/workflows/ci.yml](.github/workflows/ci.yml) against `vVERSION` to build, test all
   supported BYOND builds, and package all artifacts.
4. From the `release` environment, pushes `vVERSION` and the updated `master` branch, creates the
   GitHub Release with the packaged artifacts, and publishes `auxtools-VERSION.vsix`
   (`sohpea.auxtools`) to the Visual Studio Marketplace (`vsce --oidc`) and Open VSX
   (`ovsx --trusted-publishing`).
