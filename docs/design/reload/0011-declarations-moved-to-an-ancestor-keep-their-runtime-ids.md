# Preserving runtime IDs when declarations move to an ancestor

An edit can move a variable or proc declaration from a subtype up to one of its ancestors. Because
existing instances and compiled code in the running world already use the subtype's variable IDs,
values, and proc IDs for the old declaration, existing types keep those runtime IDs and values
while the rest of the type tree receives the ancestor's declaration and the subtype's existing proc
becomes an override.

- A new variable on the ancestor lists, in `NewVar::moved_from`, the subtypes whose declaration
  moved up to it. Those subtypes and their descendants keep their runtime declaration, whereas any
  other redeclaration of the name is a duplicate.
- A moved variable keeps its runtime declaration with the ancestor's flags and the default that
  applies. When its `static`, `const`, or `tmp` flags change, it is redeclared.
- Moving a variable to an ancestor that declares it `static` or `const` is refused
  (`MovedVarConflicts`), because it would conflict with the ancestor's global slot.
- `__PROC__` evaluates to an override's path without `/proc`. When a proc's declaration moves to an
  ancestor, the definition left on the subtype becomes an override, so `__PROC__` produces
  different text there. A moved proc declaration whose compiled code changes because of `__PROC__`
  is compiled again even though its source did not change.

## Applies to

- `dmasm/reload/src/reload_moved.rs`: `var_moved_up`, `var_moved`, `moved_from`, `proc_moved_up`,
  `moved_users`
- `dmasm/reload/src/reload.rs`: `NewVar::moved_from`
