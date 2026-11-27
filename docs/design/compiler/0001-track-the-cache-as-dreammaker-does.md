# Tracking the cache register as DreamMaker does

BYOND reads a field chain such as `a.b.c` through a cache register, and DreamMaker omits a load
when it knows the cache already holds the base of the chain. dmasm must reuse the cache in the
exact same places. Reusing the cache where DreamMaker reloads the chain can read a different object
if the chain changed between the two reads, and reloading where DreamMaker reuses the cache causes
the reverse mismatch.

DreamMaker loses track of the cache at many points, some of them surprising. The rules below were
found by comparing dmasm's output against DreamMaker's. The compiler tracks what it believes the
cache holds in `Compiler::cache` and clears it wherever DreamMaker forgets it.

## Labels and the ternary exception

- DreamMaker knows what the cache holds at a label only when exactly one jump targets that label
  and the preceding code does not fall through to it. At any other label, it forgets the cache.
- The knowledge carried to the label is the cache state at the jump. When several jumps carry
  different states, the state is unknown.
- Preceding code falls through unless its last instruction is `Jmp`, `JmpLoop`, `Switch`,
  `SwitchRange`, `PickSwitch`, or `PickProb`. The comparison tool also counts `Ret` and `End`.
- A jump from `Spawn` to the label after the spawned block carries no knowledge. After a spawned
  block, DreamMaker does not know what the cache holds.

The ternary operator is an exception to the label rule. The label of the right-hand branch has
exactly one incoming jump (the `Jz` of the condition) and no fall-through (because the left-hand
branch ends in `Jmp`), yet DreamMaker still forgets the cache there. The ternary compiler clears
the cache explicitly after that label. Keeping the knowledge would be valid, but the resulting
bytecode would differ from DreamMaker's.

## Loops, branches, and assignments

- At the start of every `for` loop (`for (init; test; step)`, `for ()`, `for (x in L)`,
  `for (x = a to b)`, and `for (k, v in L)`), DreamMaker does not know what the cache holds.
- At the end of every loop, including `while` and `do ... while`, DreamMaker does not know what the
  cache holds either.
- A `for (x in L)` loop reads the list again instead of reusing what the cache held before the loop.
- When every arm of an `if` chain folds away, DreamMaker still forgets the cache, both where no arm
  is left and after the arm it keeps.
- At the end of an assignment statement, DreamMaker forgets a field chain (a cache with more than
  one step, such as `a.b.c`) in the cache. This covers `x = y`, `x += y`, and `x++`, even when `x`
  is a local variable, but does not cover an assignment to a list element or a variable declaration.

## Folded logical chains and calls

- Where DreamMaker folds `&&` or `||`, it forgets what the cache holds.
- In a chain of `||` or `&&`, a constant that decides the chain folds wherever it appears. When
  DreamMaker drops an operand this way, it also forgets the cache.
- When the folded chain leaves only a number, the cache is forgotten as for any folded number. When
  it leaves text, `null`, or a path (as in `"a" || x` or `null && x`), the compiler must forget the
  cache explicitly.
- Calls outside the cache invalidate a cached field chain. These are global calls, calls by path,
  calls by name, calls to the proc itself, `Call` and `CallStatement` whose proc is not read through
  the cache, `Del`, `Sleep`, increments and decrements, and `PushEval`.
- A call through `a.b.f()` keeps the old receiver in the cache even if `f` modifies the chain,
  matching DreamMaker.
- For a method call, a receiver that is a variable chain goes into the call operand after the
  arguments are pushed. A computed receiver, such as a value on the stack or a list element, is
  placed in the cache and saved with `PushCache` and `PopCache` across the arguments because the
  arguments may change the cache.

## Comparison and correctness

If the compiler believes the cache holds something it does not, the compiled proc reads the wrong
object at runtime. Forgetting the cache too often only costs an extra load, but it makes the
bytecode differ from DreamMaker's.

The comparison tool treats any difference in cache reuse as a real difference, even when the two
programs are otherwise identical after normalizing, so changes to these rules show up there.
It expands the loads that DreamMaker omits using the same label rule. It does not model the ternary
exception, which causes no harm because DreamMaker reloads the chain on the right-hand side anyway,
leaving no omitted load to expand.

## Applies to

- `dmasm/compiler/src/compiler.rs`: `Compiler::emit_ins`, `emit_label`, `emit_loop_label`,
  `end_assignment_statement`, and the folded-logic check in `Compiler::emit_expr`
- `dmasm/compiler/src/compiler/ternary.rs`: `emit_branches_after_test`
- `dmasm/compiler/src/compiler/binary_ops.rs`: the `&&` and `||` chain in `emit`
- `dmasm/compiler/src/compiler/statements.rs`: `emit_statement`, the folded `if`, and the `ForList`
  arm
- `dmasm/compiler/src/compiler/procs.rs`: `track_cache`
- `dmasm/compiler/src/compiler/follow.rs`: the method call arms of the chain
- `dmasm/compiler/examples/compare_dreammaker/compare.rs`: `normalize`, `compare`
