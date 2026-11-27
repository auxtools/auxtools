# BYOND reallocates the proc table as it grows

Constructing a verb with a new name using `new` (such as
`new /mob/verb/auxtest_verb(M, "auxtest clone 1")`) appends an entry to BYOND's proc table. Because
BYOND reallocates the table in memory as it grows, any pointer into the table remains valid only
until DM code that can add verbs runs again. Caching a `ProcEntry` pointer across DM execution reads
freed memory once the table moves.

auxtools reads a proc's table entry fresh each time it is needed, and `Proc::entry` documents that
the returned pointer is valid only until DM code runs again.

`test_procs` verifies this directly: it reads every proc's bytecode, calls `auxtest_grow_proc_table`
to add 4096 verbs, and checks that the proc table moved (failing if it did not move, since the test
would otherwise check nothing). It then verifies that every proc still reads the same bytecode as
before.

## Applies to

- `tests/auxtest_host/auxtest_host.dm`: `auxtest_grow_proc_table`
- `tests/auxtest/src/procs.rs`: `test_procs`
- `auxtools/src/proc.rs`: `Proc::entry`
