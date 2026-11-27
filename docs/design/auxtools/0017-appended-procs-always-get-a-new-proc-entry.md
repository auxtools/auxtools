# Appended procs always get a new proc entry

BYOND's `procdef_add` returns an existing entry when one with the same path already exists, and
`add_proc` preserves that reuse behavior. To ensure a new entry is always allocated instead,
`append_proc` temporarily hides every existing entry with the target path by replacing its path word
with the bitwise NOT of the path, calls `add_proc`, and then restores the original path words. A
result of `NONE` (`0xFFFF`) indicates failure.

## Applies to

- `auxtools/src/runtime_tables.rs`: `append_proc`, `add_proc`
