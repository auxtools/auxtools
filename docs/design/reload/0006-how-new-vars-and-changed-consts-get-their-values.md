# Initializing new variables and updating global `const` values

A hot reload can add variables with constant defaults, computed defaults, or global or static
initializers, and can also change a global `const`.

## New variables and computed defaults

BYOND's usual placeholder for a computed default (the value `initial()` returns) would also become
the value of the variable on instances that already exist, so a hot reload never applies a computed
value to existing instances:

- **New global or static variable with an initializer:** Gets an init proc in `NewVar::init` that
  is evaluated once. Until it runs, the variable holds `null` as a placeholder `value`.
- **New variable with a computed default:** Receives `null`. On a new type, `initial()` returns
  `null` until the world restarts. On an existing type, both existing instances and `initial()`
  stay `null` until then. Both cases emit a `NewVarComputedDefault` warning noting that `initial()`
  is wrong until a restart.
- **Changed computed default:** Not applied to existing instances, which keep `null`, and emits a
  `ComputedDefaultNotApplied` warning.

## Global `const` values and `static` or `tmp` initializers

- **Changed global `const`:** Because no compiled code writes a global `const` (its value is folded
  into readers), a changed global `const` is set directly as a `Redeclared` entry with its new
  value, and the code it was folded into is compiled again (see
  [recompiling code affected by changed `const` values](0007-code-that-reads-a-changed-const-is-compiled-again.md)).
- **`const` initializers refused by DreamMaker:** SpacemanDMM folds some `const` initializers that
  DreamMaker refuses as not constant, namely those that read `type` or `parent_type` and those
  containing a comparison (`==`, `!=`, `<`, `<=`, `>`, `>=`, `~=`, `~!`), including inside
  parentheses and in any part of `? :`. Rather than being refused, they are accepted with a
  `ConstRefusedByDreamMaker` warning stating that a rebuild will fail.
- **`static` and `tmp` initializers:** SpacemanDMM does not fold `static` or `tmp` variable
  initializers. Their value comes from SpacemanDMM's folded constant when one is available, or else
  from simple expression evaluation. If neither produces a value, the variable has no literal value
  and is treated as computed.

## Applies to

- `dmasm/reload/src/reload.rs`: `NewVar`, `Redeclared`, `new_var`, `var_default`, `var_constant`,
  `dm_refused_const`, `refused_const_warning`
