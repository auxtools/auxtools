# Staging hot reload changes and preserving running code

A hot reload modifies a running world whose active and sleeping frames continue executing the
bytecode they started with, and whose old call sites still refer to procs and variables by ID.
Because a reload comprises several steps that can fail, auxtools never frees or overwrites code
that may still run, and stages allocations and checks before writing changes to the running world.

## Keeping old code and staging preparation

- A reload never frees or rewrites code that may still run. Running and sleeping frames keep their
  old bytecode, and removed procs and variables stay allocated for both those frames and old call
  sites (see [removed definitions stay in the world](0028-removed-definitions-stay-in-the-world.md)).
- A reload allocates the tables it needs and checks its changes before writing to the running world.
  For parent changes, the new type tree and the new proc name maps are planned before any of them is
  applied.
- The world continues running its old code until the last preparation step succeeds: adding types
  (`add_types`), preparing code, preparing resources, appending new entries (`append`), and
  repointing resources.

## Unsettled types from a failed reload

New types are the exception to staged preparation. Types added by `add_types` before a failure stay
allocated but unused, and the error reports how many were added.

A type added by a failed reload is marked "unsettled". When created, it received its parent's
variable tables, declared variables, and init proc. A subsequent new variable reaches both the type
and its parent, whereas a changed default or init proc reaches only the parent. A later reload may
reuse the type as added only while all of the following hold:

- It was added with the same parent and built-in values.
- Its variable tables and init proc still equal the parent's.
- The parent's inherited record fields have not changed.

Otherwise, the reload is refused and asks the user to undo the change or rebuild.

## Partial failures during apply

Even after preparation succeeds, `apply` can still record failures, for example when a name map or
a built-in variable cannot be written. These failures are reported to the client, and steps already
applied are not rolled back.

## Applies to

- `debug_server/src/hot_reload.rs`: `prepare`, `add_types`, `append`, `apply`, `inherits_as_added`,
  `inherited`, `UNSETTLED_TYPES`
- `debug_server/src/server/reload.rs`: `prepare_hot_reload`
- `debug_server/src/reload_reparent.rs`: `prepare`
