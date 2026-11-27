# Built-in variable defaults and removed built-in overrides

Most built-in variables are fields of a BYOND type record, whereas a few built-in types store
theirs as defaults in the variable table instead: `/sound`, `/particles`, `/exception`, `/regex`,
and `/icon`. Built-in variables of these variable-table types go into `Plan::var_defaults` like
other defaults, and the world copies an inherited default as a variable-table entry.

SpacemanDMM lacks some of BYOND's built-in defaults, such as `mob` `density`, `layer`, and
`see_in_dark`, so the reload cannot compute the value a built-in variable should revert to when an
override is removed. Furthermore, once a built-in type overrides a built-in variable, its original
default is gone at run time, and its parent's value can differ (for example, `mob` `layer` is `4`
while `movable` `layer` is `3`).

Removing a built-in default override is therefore handled as follows:

- When a type stops overriding a built-in variable's default, the running parent's value is copied
  to the type and to the subtypes that inherit it (`Plan::builtin_inherited`), preserving BYOND's
  own built-in defaults. `name` and `text` are excluded from this.
- For a type-record variable, the copied value applies only to new instances, and the reload emits
  a `BuiltinDefaultNewInstancesOnly` warning.
- Removing a type-record override from a built-in type itself is refused as not restorable
  (`BuiltinDefaultNotRestorable`).

## Applies to

- `dmasm/reload/src/reload.rs`: `VAR_TABLE_BUILTINS`, `builtin_in_var_table`, `BuiltinInherited`,
  `inheriting`, `var_default`
