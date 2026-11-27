# Checking for known BYOND bugs

`auxtools check` compiles every proc in a DM project with `dmasm-compiler` and reports known BYOND
compiler and runtime bugs affecting the target build. It writes no `.dmb` and does not require BYOND
to be installed, making it straightforward to run in CI (see [README.md](../README.md) for
installation).

## Usage

```sh
auxtools check tgstation.dme -DCIBUILDING --byond-version 516.1689
```

If `DME` is omitted, `auxtools check` uses the sole `.dme` in the current directory. Pass the
project's build defines with `-D` and the target BYOND build with `--byond-version` (defaults to
`516.1689`).

## What it checks

Examples of issues `auxtools check` catches include:

- Discarded return values in `operator<<=` and `operator>>=` (and in `operator~=` and `operator~!`
  on `516.1674–516.1688`)
- `operator**` calling `operator*`
- Ignored return values from `area/Uncross()` starting in `516.1683`
- `continue LABEL` and nested-loop `try` crashes
- `||=` and `&&=` bugs on `?.` chains
- `pick()` ending in a zero weight before `516.1689`

See [dmasm/COMPILER.md](../dmasm/COMPILER.md#warnings) for the full list. Static checking cannot
catch bugs that appear only at runtime.

## Output and exit codes

Text output prints each finding as `FILE:LINE: PROC: diagnostic` (or `FILE: PROC: diagnostic` when
no line number is available). `--json` emits `{byond, procs, found, not_checked, parse_errors}`,
where each entry in `found` carries a stable diagnostic `id`.

Exit code `0` means no bugs were found, `5` means one or more bugs were found, and `1` means the
project could not be read. Parse errors and procs that dmasm cannot compile are listed as unchecked
on `stderr` (and in JSON) without changing the exit code.
