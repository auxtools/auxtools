# Reading static and const variables from global slots

BYOND stores every static variable and every user-defined const variable in a slot of
`global_values`, including local consts (which are stored as globals like static locals). Built-in
constants of each object, such as `type`, are declared const but are not globals because their
values differ per object. DreamMaker reads global statics and consts directly even when the source
reaches them through an object or a chain, and it declines to fold certain const reads that look
foldable.

## Which variables are globals

- Static variables and user-defined const variables are globals.
- Built-in variables of a type, such as `type`, are declared const but vary per object. They are
  never treated as globals and never folded.
- A local const is a global, like a static local. A number or text const is also folded into
  constant expressions that use it.

## Reading statics alone

- When code reads a static variable through an object, as in `obj.static_var`, DreamMaker reads the
  global directly without evaluating the object it was reached through.
- In a chain, the last `::` before any safe access that reads a static or const variable is read
  alone, and DreamMaker omits the code before it.
- Similarly, `/type::a::b` (where `b` is a static variable) reads `b`'s global slot alone without
  evaluating `/type::a`.
- A static variable of `src`'s type named alone is reached through `src` when a field or call
  follows it. DreamMaker emits `cache = src; cache = global(x); ...`.

Evaluating the object before a static read would run code that DreamMaker skips (such as a call
with side effects) and produce different bytecode.

## Unfolded const reads and variable defaults

- `&&`, `||`, and `? :` return whichever operand they choose. If that operand is a bare const read,
  DreamMaker reads the const's global instead of folding it, so `7 && CONST` reads `CONST`'s
  global.
- A const at the end of a chain, read alone, is read from its global rather than folded. Folding a
  const that DreamMaker reads from its global would produce different bytecode even though the
  value is the same.
- SpacemanDMM folds a variable default of the form `type::name()` before dmasm sees it. dmasm
  corrects it, and a default that reads such a variable with `type::var` takes the corrected value.
- A read of a non-const static with `type::var` does not take that corrected value. DreamMaker reads
  a static's current value at runtime.

## Hot reload tracking

`chain_targets_in_proc` reports, for each field, index, and call at the start of a chain, what the
code reading it resolves to: the type whose static or const variable a field reads, or the proc a
call invokes. It returns none for an index, for a field of the object itself, and for any target
that is not found. Hot reload compares its results for the old and new source to find procs whose
chains now read a different global or call a different proc and therefore need recompiling.

## Applies to

- `dmasm/compiler/src/compiler/procs.rs`: `is_global_member`, `last_scoped_static`, `const_lookup`,
  `resolve_proc_references`
- `dmasm/compiler/src/compiler/follow.rs`: the `Field` arm of the chain
- `dmasm/compiler/src/compiler.rs`: static and scope handling in `Compiler::emit_expr`,
  `chain_targets_in_proc`
- `dmasm/reload/src/reload.rs`: caller of `chain_targets_in_proc`
- `dmasm/compiler/src/compiler/fold.rs`: `constant`
- `dmasm/compiler/src/compiler/statements.rs`: the `var` statement
