# Binary compatibility with Ubuntu 22.04 and Windows 10

Released launcher and debug library binaries must run on the systems BYOND servers use, including
old ones. A binary built on a newer system links against newer versions of glibc and libstdc++. CI
enforces these compatibility floors with build environment choices and library allowlist checks.

## Linux

- Linux binaries are built on Ubuntu 22.04 so they require at most glibc 2.35 and libstdc++ from
  GCC 12 (`GLIBCXX` 3.4.30).
- `ci/check-linux.sh` checks every built file to verify that it is 32-bit x86, requires no newer
  glibc or `GLIBCXX` versions, and imports only libraries from an allowlist: `libc.so.6`,
  `libm.so.6`, `libgcc_s.so.1`, `libstdc++.so.6`, `ld-linux.so.2`, `libdl.so.2`, `libpthread.so.0`,
  and `librt.so.1`.
- The Linux tests also run on Ubuntu 22.04 (the test service's Linux image), the oldest environment
  the binaries support. A dependency on a newer system library shows up as a test failure.

## Windows

- Windows binaries link the C runtime statically and import only system DLLs present on Windows 10.
  An import from a redistributable DLL indicates a misconfigured build.
- `ci/check-windows.sh` checks every built file to verify that it is 32-bit x86, targets Windows 10
  or older, and imports only DLLs from an allowlist: `kernel32.dll`, `ntdll.dll`, `advapi32.dll`,
  `ws2_32.dll`, `userenv.dll`, `bcrypt.dll`, `bcryptprimitives.dll`, and
  `api-ms-win-core-synch-l1-2-0.dll`.
- The Windows test job (`test-windows`), which runs during a release, runs on `windows-2022`
  (Windows Server 2022, which is of the Windows 10 generation).

Any new shared-library dependency fails the check scripts until it is confirmed to exist on the
oldest supported operating system and added to the allowlist.

## Applies to

- `.github/workflows/ci.yml`: `build-linux` and `test-windows` jobs
- `ci/check-linux.sh`
- `ci/check-windows.sh`
