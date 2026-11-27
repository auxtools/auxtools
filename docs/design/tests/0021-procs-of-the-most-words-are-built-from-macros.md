# Building maximum-size 65,535-word test procs from macros

A BYOND proc can hold at most 65,535 bytecode words, and both the debugger and the reload compiler
must handle a proc of exactly that size. The host worlds construct such procs from nested macros:
`BIG1` is `big_count += 1;`, and each of `BIG4`, `BIG16`, `BIG64`, `BIG256`, `BIG1K`, and `BIG4K`
expands to four copies of the preceding macro. A proc body consisting of
`BIG4K BIG4K BIG4K BIG256 BIG256 BIG256 BIG16 BIG16 BIG16 BIG1 BIG1` compiles to 65,535 words.

- In the `auxtest` host, `BIG_TWINS` defines two such procs, `auxtest_big_a` and `auxtest_big_b`, on
  one line. Because line numbers are part of the bytecode, placing both on one line gives them
  identical code (see
  [shared code entries for identical procs](0022-dreammaker-gives-procs-with-the-same-code-one-entry.md)).
- In the hot reload host, `BIG_PROC(name, param)` defines one such proc, and nothing in the host
  uses it. The test that needs it adds a use during a reload so the other reloads do not parse the
  proc.

Changing `BIG1` or the macro sums changes the compiled size, and a proc one word larger does not
compile.

## Applies to

- `tests/auxtest_host/auxtest_host.dm`: `BIG1` to `BIG_MAX`, `BIG_TWINS`
- `tests/hot_reload_test_host/reload.dm`: `BIG1` to `BIG4K`, `BIG_PROC`
