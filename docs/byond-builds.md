# Supported BYOND builds

This page lists the BYOND builds supported by auxtools, explains how newer and unsupported builds
are handled, and shows how to check symbol resolution against a new BYOND binary. For installation
and usage, see [README.md](../README.md).

auxtools supports BYOND `515.1637–515.1647` and `516.1648–516.1689` on Windows and Linux (see
[ci/supported-builds.txt](../ci/supported-builds.txt) for the exact list of tested builds). Older
builds and other major versions are refused with a version error.

Builds newer than `516.1689` are attempted using the latest signature recipes, with resource reload
and `text2path()` negative-cache updates disabled until the build has been validated. If a required
BYOND symbol cannot be located, the debugger refuses to start while the world continues running
normally. Builds known to be broken are listed in
[ci/known-broken-builds.txt](../ci/known-broken-builds.txt).

## Checking a new BYOND build

Use the `check_recipes` example to verify that auxtools resolves every required BYOND symbol in a
`libbyond.so` or `byondcore.dll` binary without running a world:

```sh
# Linux
cargo run -p auxtools --target i686-unknown-linux-gnu --example check_recipes -- linux 516.1689 BYOND/bin/libbyond.so

# Windows
cargo run -p auxtools --target i686-pc-windows-msvc --example check_recipes -- windows 516.1689 BYOND/bin/byondcore.dll
```

`check_recipes` exits with `0` if every required symbol resolves, `1` if any lookup fails, `2` if
auxtools refuses the build outright, and `64` if its arguments are invalid or name an unknown
recipe. Symbol patterns live in [auxtools/src/registry](../auxtools/src/registry).
