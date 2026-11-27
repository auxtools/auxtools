# Identifying proc statics by `__PROC__` path and index

BYOND records a proc's `var/static` under its local name only, so the name alone cannot distinguish
two statics in the same proc that share a name. dmasm identifies each proc-scoped static by the
proc's `__PROC__` path and its source-order index, and tracks every global operand with its slot.

## Global variable operands and scopes

A global variable operand (`GlobalVar`) carries the name BYOND records along with its slot and
declaration scope:

- `id` is the index into BYOND's `global_values`. It is `None` for code built from source, and the
  assembler resolves the global by name.
- `scope` records where the variable was declared. A proc-scoped static (`StaticScope::Proc`) is
  identified by two values: the path that `__PROC__` gives, and its index among the proc's static
  locals with the same name, counting from 0 in source order. Local consts count in this index as
  well because a local const is also a global.

`proc_path` builds the path used for `__PROC__` and for these scopes: `/type/proc/name` for a proc
declaration and `/type/name` for an override.

Code that compares globals by name alone conflates multiple statics in the same proc, so the
comparison tool identifies each global by its slot, as described in
[the comparison tool's normal form](0031-comparison-tool-normal-form.md). Changing the path format
or index counting breaks the match between a reloaded proc's statics and the running world's
globals.

## Static initializers and `__PROC__`

A static variable's initializer runs in an initializer proc, but `__PROC__` inside it names the proc
in which the static is declared. When compiling a static variable's initializer, the caller passes
the path of the declaring proc so `__PROC__` resolves to that proc rather than the initializer
proc. The caller also passes the visible statics (by name, declaration index, and declared type)
and the declared type of the variable whose initializer the body returns, allowing an `astype()` or
a bare `new` in the initializer to take that type just as in the declaration.

Before BYOND build 515.1644, DreamMaker rejects `__PROC__` with one exception: the second argument
of `call(src, __PROC__)`. For builds before 515.1644, the compiler allows `__PROC__` only as the
second of two targets in `call(a, b)`, without checking that the first target is `src`, and rejects
`__PROC__` everywhere else.

## Applies to

- `dmasm/bytecode/src/operands.rs`: `GlobalVar`, `StaticScope`
- `dmasm/compiler/src/compiler/procs.rs`: `proc_path`, `compile_proc_as`
- `dmasm/compiler/src/compiler/term.rs`: the `call()` term and the `__PROC__` term
- `dmasm/compiler/src/target.rs`: `has_proc_macro`
