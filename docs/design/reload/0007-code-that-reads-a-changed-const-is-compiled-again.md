# Recompiling code affected by changed `const` values

DreamMaker folds a `const` into compiled code everywhere except standalone read positions, where
it reads the `const`'s variable directly. Hot reload tracks both folded reads and rebinding chains
so any proc whose compiled bytecode or local `const` values are affected by a change is compiled
again.

## Standalone read positions

When code reads a `const` on its own rather than as part of a larger constant expression,
DreamMaker reads the `const`'s variable and does not fold the value. `read_alone` finds these
positions and identifies each expression by its address:

- a return value
- a local's initializer
- an assignment's value
- a parameter's default
- an argument of a call that is not a built-in proc's, including `..()` and `.()`
- a list element
- an index
- an embedded expression in text
- both sides of an output statement
- the source of an input statement
- either value of a `? :` that is itself read on its own

This list must match DreamMaker exactly: a missing position would fold a `const` that DreamMaker
reads from its variable, and an extra position would do the reverse.

## Recompilation triggers

A chain of `.`, `:`, or `::` ending in a `const` is folded as DreamMaker folds it, from the
declared types of the variables along the chain. Although a hot reload updates a `const`'s
variable, compiled code still reads the variable of the declaration it was compiled against. BYOND
also keeps a proc's local `const` in its own variable, which standalone reads access and which the
world updates whenever the proc is reloaded.

A proc is compiled again in four situations:

- A `const` folded into the proc changes value.
- A variable in a chain ending in a `const` is retyped in a way that changes the chain's folded
  value.
- A chain resolves to another type's `static` or `const` variable after the reload, because the
  existing bytecode still names the old declaration's variable.
- A local `const` in the proc takes a new value, even when the proc's code is unchanged. Its value
  can change through a changed `const` or default, through a variable it reads taking a different
  declared type, or through an `x:name` lookup resolving to another declaration.
  `StaticVar::constant` marks a local `const` so the world sets its variable to the new value on
  each reload of the proc.

## Applies to

- `dmasm/reload/src/reload.rs`: `read_alone`, `folds_changed`, `chain_rebinds`,
  `local_const_users`, `StaticVar::constant`
