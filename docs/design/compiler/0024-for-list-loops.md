# Filtering `for (x in L)` loops by `as` types and declared types

A `for (var/T/x in L)` loop visits only items of the matching kind. BYOND's iterator accepts a type
filter (`mob`, `obj`, `turf`, `area`), and DreamMaker inserts an `istype()` check inside the loop
when the declared type cannot be expressed by the filter alone. A loop with no list, such as
`for (var/datum/T/x)`, iterates over all datums of that type.

## Type filters

The compiler chooses the iterator filter in this order:

1. The `as` types written on the loop itself.
2. Otherwise, the `as` types on the variable's declaration.
3. Otherwise, the kind of the declared type: `/obj` gives `obj`, `/mob` gives `mob`, `/turf` gives
   `turf`, `/area` gives `area`, `/atom/movable` gives `mob` and `obj`, `/atom` gives all four, and
   all other types give no filter.

When the filter comes from the kind of the declared type, an `istype()` check also tests the
declared type exactly. That check is omitted for exactly `/mob`, `/turf`, and `/area`, where the
iterator filter is sufficient.

## List types

- A variable whose type path contains `list` anywhere is treated as a list (for example,
  `var/datum/d/list/x` is a list too).
- A loop variable declared `var/list/T/x` is checked as a `T`, or as a list if `T` is not a type.

## Loops without a list

- A loop over all datums of a type, as in `for (var/datum/T/x)`, passes the type to the iterator.
- `in world` iterates over the world's atoms instead.

## Applies to

- `dmasm/compiler/src/compiler/statements.rs`: `ForList` arm of `emit_statement_inner`,
  `loop_filter`, list type check
