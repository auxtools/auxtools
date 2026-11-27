# Cache use and typing in `?.` and `?:` chains

A safe access (`a?.b` or `a?:b`) tests its object for `null` and jumps past the rest of the
expression when it is `null`. DreamMaker compiles each safe access as a new chain starting from the
cache, following several cache and typing rules that are not obvious from the source. dmasm
replicates each of them:

- Each `?.` starts a new chain from the cache.
- `initial(a?.b)` reads `a.b` whether or not the `null` check applies.
- In `initial()` on a chain with multiple `?.` operators, DreamMaker saves the cache around each
  `?.` after the first, so the variable is read from the object that the first `?.` found.
- DreamMaker ignores the declared type of the value before `?:`. The field or call after `?:` is
  looked up untyped, just as `x:name` is. The same holds for every segment of a safe write that
  starts with `?:`.
- When a safe chain reaches a static variable through `::`, DreamMaker reads that static alone and
  omits the code between the `?.` and the `::`.
- Starting in BYOND build 516.1685, `a?.L[k] += v` restores the cache after the right-hand side.
  Earlier builds read the list from whatever the right-hand side left in the cache. dmasm follows
  the target build.

Because of the `?:` rule, a typed variable before `?:` gives no type information to what follows.
The field after it is resolved the same way as an untyped `x:name` lookup, as described in
[chain lookup of variables and procs](0006-chain-lookup-of-vars-and-procs.md).

## Applies to

- `dmasm/compiler/src/compiler/builtin_procs.rs`: `initial_safe` and the `initial()` arm of
  `emit_code`
- `dmasm/compiler/src/compiler/follow.rs`: `emit_safe_inner`
- `dmasm/compiler/src/compiler/statements.rs`: the `untyped` field of the safe write segments
- `dmasm/compiler/src/target.rs`: `restores_cache_for_safe_element`
