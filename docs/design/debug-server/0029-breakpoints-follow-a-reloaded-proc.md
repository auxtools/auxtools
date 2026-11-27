# Breakpoints follow a reloaded proc

When a hot reload changes a proc, its breakpoints must move to the new code. A breakpoint can sit
on a source line or on the proc's entry with no line, and the `Claimant` trait lets the reload ask
which kind each breakpoint is:

- A breakpoint on a proc's entry stays on the new code's entry.
- Line breakpoints move according to line matching (see
  [matching lines between old and new code](0030-matching-lines-between-old-and-new-code.md)).

Several breakpoints can move onto the same instruction. When that happens, they share the
instruction, except that a single client's second breakpoint on that instruction is dropped with a
warning so only the first keeps its condition and hit count.

## Applies to

- `debug_server/src/hot_reload.rs`: `Claimant`, `restore_breakpoints`, `keep_claims`
