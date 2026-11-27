# Recompiling code when proc paths or bare names retarget

Compiled code stores a proc's ID rather than its path. Unchanged code is therefore compiled again
when a proc path in its compiled form resolves to a different definition after a hot reload, or
when bare names switch between binding to a member variable or proc added or removed by the reload
and binding to a global.

## Resolution rules

The reload's resolution rules must match DreamMaker, because any mismatch would leave reloaded code
calling a different definition than a fresh build would call:

- `/T/f` resolves to the first override of `f` with that exact path.
- `/T/proc/f` resolves to the declaration.
- `/T::f` resolves to the last definition on `T`, and never resolves to a verb's declaration.
- A `type::name()` call resolves to the last definition on the nearest type that defines the name
  (the type itself or an ancestor). When that type is the root, it resolves to the global proc's
  declaration.
- `/T.proc/name` resolves to the declaration on `T` or its nearest ancestor with one. `.proc/name`
  searches from the type of the code that contains it.

Inside code belonging to a type or its subtypes, a bare name binds to a member variable of that
name while one exists, and to the global variable otherwise. Unqualified calls bind to member procs
and global procs the same way.

## Source scanning and local scopes

To locate affected code, a source scan looks for anything that might name an affected path or bare
name. False positives cost only an extra compile: the newly compiled code is compared against the
old code (`same_code`), and unchanged results are dropped.

In both dmasm and DreamMaker, a local variable is in scope from its declaration to the end of its
block. The scan tracks which bare names read a parameter or a local, and treats a local's own
initializer and a loop's list as outside the local's scope so names there still count as possible
uses of a member or a global.

## Applies to

- `dmasm/reload/src/reload_retargeted.rs`: `Retargeted`, `resolve`, `last_definition`, `type_call`,
  `declared_from`, `proc_might_name`, `same_code`, `local_reads`, `walk_scopes`
