# BYOND calling conventions and single-byte `Value` tag masking

BYOND is 32-bit x86, and its internal functions do not use a single calling convention across
platforms and builds. In addition, BYOND leaves the three bytes above a `Value`'s tag byte undefined
in values it returns (for example, the length returned as a `Value` by Linux `get_length`), so code
that compares a tag masks it to a single byte first, and the shim's `clean` masks a `Value`'s tag
before passing it to BYOND.

## Calling conventions

On Windows, the internal BYOND functions that auxtools calls through the C++ shim are `cdecl`. On
Linux, some functions take their first arguments in registers (GCC's `regparm`) while others are
plain `cdecl`. Because nothing in the binary names the convention, it is determined from the
function's first instructions:

- A function that reads `EAX` or `EDX` before touching its stack arguments takes them in registers
  and is declared with `LINUX_REGPARM2` or `LINUX_REGPARM3`.
- A function that reads `[esp+arg_N]` is `cdecl`.
- A function that returns a `Value` uses `EAX` for the address of the return buffer.

The `LINUX_REGPARM*` macros expand to nothing on Windows. Using the wrong convention passes garbage
arguments.

Build-dependent calling convention differences are selected at run time using flags populated in
`symbols::resolve_full`:

- `remove_from_list` takes its arguments in registers on Linux through `516.1673` and on the stack
  from `516.1674` onward (`LINUX_REMOVE_FROM_LIST_ON_STACK`).
- Linux BYOND 515 before `515.1644` returns the count from `get_length` in `EAX`, matching Windows.
  From `515.1644` onward (`LINUX_FRAMELESS`), Linux returns a `Value` through a buffer, which the
  shim unpacks.

## Masking `Value` tags

Because the upper three bytes of a tag word can hold garbage, comparing a whole tag word can fail
even when the value is a `Number`. Any code that inspects a tag masks it to a single byte first:

- The C++ shim's `clean` masks a `Value`'s tag before passing it to BYOND.
- `length_value_to_count` masks the length's tag before checking for `Number` (tag `0x2A`). A length
  that is not a `Number`, or is not greater than zero, counts as `0`.

## Applies to

- `auxtools/src/hooks.h`: `LINUX_REGPARM2`, `LINUX_REGPARM3`, `clean`
- `auxtools/src/raw_types/funcs.cpp`: the `DEFINE_byond*` macros, `remove_from_list`, `get_length`,
  `length_value_to_count`
- `auxtools/src/symbols.rs`: `remove_from_list_in_registers`, `get_length_returns_count`
- `auxtools/src/registry/builds.rs`: `LINUX_FRAMELESS`, `LINUX_REMOVE_FROM_LIST_ON_STACK`
