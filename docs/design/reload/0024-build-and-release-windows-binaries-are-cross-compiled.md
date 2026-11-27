# Cross-compiling Windows binaries on Linux

CI builds Windows binaries for `i686-pc-windows-msvc` on a Linux runner rather than on Windows.
Because Microsoft's C++ headers require clang 19 or newer, the `build-windows` job runs on
`ubuntu-26.04`, which ships a sufficiently recent clang.

## Toolchain and linker setup

- `ci/msvc/fetch-sdk.sh` downloads Microsoft's libraries and headers. Sourcing `ci/msvc/env.sh` from
  bash with `MSVC_SDK` pointing to that directory configures `clang-cl`, `lld-link`, `llvm-lib`, and
  the SDK library paths so `cargo build --target i686-pc-windows-msvc` works.
- The build links the C runtime statically (`+crt-static`) and passes `/PDBALTPATH:%_PDB%`, ensuring
  the PDB path embedded in each binary contains no build-machine directory.
- Because MASM ships only with Visual Studio, `env.sh` places a shell-script stand-in
  (`ci/msvc/bin/ml.exe`) on `PATH`. When the build asks to assemble `FILE.asm`, the script invokes
  clang to assemble `FILE.S` instead (which holds the same assembly in GNU syntax) and marks the
  resulting object with `@feat.00` as having no exception handlers, matching MASM's `/safeseh` as
  required by the linker for every 32-bit object.
- `env.sh` sets `CARGO_TARGET_I686_PC_WINDOWS_MSVC_RUSTFLAGS`, which replaces rather than appends to
  the target's `rustflags` in `.cargo/config.toml`. For this reason `env.sh` repeats `+crt-static`.

## Maintenance rules

- Any flag added to the Windows target in `.cargo/config.toml` does not take effect in CI builds
  unless it is also added to `ci/msvc/env.sh`.
- CI's Windows binaries assemble `execute_instruction_hook.windows.S` rather than
  `execute_instruction_hook.windows.asm`, which only a native MSVC toolchain assembles. Any change
  to one file must be mirrored in the other.

## Applies to

- `ci/msvc/env.sh`
- `ci/msvc/fetch-sdk.sh`
- `ci/msvc/bin/ml.exe`
- `.cargo/config.toml`
- `.github/workflows/ci.yml`: `build-windows` job
- `auxtools/src/instruction_hook/`: instruction hook sources
