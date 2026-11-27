# Chain lookup of variables and procs

In a chain such as `a.b.c`, `a:b()`, or `x::name`, DreamMaker decides which variable or proc each
step names from the type of the preceding value. It uses the declared type only in certain
positions and falls back elsewhere to an untyped search across the entire type tree. dmasm must
reach the same result because the variable found determines whether the read is a static global, a
const, or a plain field (see
[reading static and const variables from global slots](0004-statics-and-consts-are-read-from-global-slots.md)),
and the proc found determines the call's name and return type.

## Value typing along a chain

What a value comes from is tracked in `After`:

- A value with a declared type keeps that type.
- After an index, DreamMaker ignores the list's declared element type. A field of a list element is
  resolved as an untyped access.
- After a proc call, only a `.` access uses the proc's declared return type (from `as /type`). A
  `:` access after a call is untyped.

For call return types, a proc defined in the code returns the type declared with `as /type`, while a
built-in proc's result keeps the typing found before it, which is untyped.

## Lookup rules

- **Field of a chain (`field_var`):** Looked up on the type of the preceding value. With no type,
  after an index, or through `:` after a call, DreamMaker resolves the variable as an untyped
  `x:name`.
- **`x:name` (`colon_var`):** Names `x`'s own variable if `x`'s type declares one. Otherwise, it
  names the last declaration among the subtypes of `x`'s type in DreamMaker's visiting order.
  Otherwise, and always when `x` has no type, it names the last declaration among all types.
  Because `/list` has no subtypes that declare variables, it ends up searching every type.
- **`x:name()` (`colon_proc_type`):** Calls the proc of `x`'s type if that type defines it, or else
  the last declaration among its subtypes, matching how `x:name` finds a variable. With no type or
  with `/list`, every type is searched. A global proc is never found this way.
- **`x::name` (`scope_var`):** Names a variable of `x`'s declared type. When `x` has no type (such
  as after a call that declares no return type), it names the variable an untyped `x:name` finds.
- **Type path at the head of a `::` chain:** A `.` in the type path searches upward. For example,
  `/datum/config.settings` resolves to `/datum/config/settings`, or else `/datum/settings`, or else
  `/settings`.

In these rules, "last" means last in the order DreamMaker visits types, where a type's subtypes
follow it. Because a new type or variable declaration elsewhere in the codebase can change what an
untyped access resolves to, hot reload must recompile procs whose untyped names may now find a
different declaration.

## Applies to

- `dmasm/compiler/src/compiler/procs.rs`: `After`, `field_var`, `call_result`, `start_type`,
  `colon_var`, `colon_proc_type`, `callee_type`, `scope_var`, `last_declaring`, `scope_head_type`
