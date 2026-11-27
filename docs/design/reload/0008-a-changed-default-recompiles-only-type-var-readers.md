# Recompiling only `type::var` readers when a default changes

Only `type::var` embeds a variable default's value or source location into compiled code. All other
reads of a variable go through the instance at run time. A changed or moved default therefore
recompiles only code that reads it with `type::var`, leaving instance reads untouched. Searching for
every use of the variable's name would recompile nearly the entire codebase for common names such
as `name` or `icon`, so it is not done.

A variable default can itself fold `type::var` or `type::proc()`, which means its value can change
when what it reads changes even if its own expression is untouched. Defaults are therefore compared
by both their expression and their folded value. A default whose expression stays the same but
whose folded value changed counts as changed, so a default that depends on another type's default
or proc follows it.

## Applies to

- `dmasm/reload/src/reload.rs`: `const_users`, `compare_types`, `var_default`
