# Adding and editing types in BYOND's type table

BYOND's type table entries own heap-allocated matrices, lazily populated appearance cache slots, and
(starting in BYOND build `515.1641`) typepath values. When auxtools adds or edits a type at run
time, it clones owned memory and invalidates caches to avoid double frees and stale appearances.

## Type table conventions

- **Type appearance cache:** A type's slot in BYOND's type appearance cache starts as `NONE`
  (`0xFFFF`) until BYOND first needs it, at which point BYOND builds the appearance from the type's
  current variables.
- **Matrices:** BYOND frees a type's transform matrix (6 floats) and color matrix (20 floats) when
  the type is destroyed.
- **Typepaths:** From `515.1641` onward (`TYPEPATHS`), each type entry stores the type's typepath
  value (adding 8 bytes), and new types must be registered for `text2path`. Builds before `515.1641`
  do not store typepath values and look up mob types by type ID.

## Adding a type

`add_type` copies the parent's type table entry and then updates the new entry:

- Sets the new type's appearance cache entry to `NONE`.
- Allocates separate copies of the parent's transform and color matrices with
  [BYOND's allocator](0014-byond-tables-are-grown-with-byonds-allocator-and-read-through-their-globals.md).
  A type never shares a matrix pointer with another type, because BYOND would double-free a shared
  pointer.
- Registers a typepath only on builds that support it, registering the parent first if the parent
  does not yet have one. A type is a mob if its parent's typepath tag byte is the mob tag, or on
  builds before `515.1641` if its parent is in the mob table. Each new mob type receives a copy of
  its parent's mob table entry.

## Editing matrices and appearances

`copy_type_matrix` gives a type its own copy of a matrix and leaves the previous copy allocated
because readers may still access it. Leaking old matrix copies is the cost of not knowing who still
reads them.

`forget_type_appearance` writes `NONE` to a type's appearance cache entry so BYOND rebuilds the
appearance from the type's current variables. Existing instances keep their previous appearance.

## Applies to

- `auxtools/src/runtime_tables.rs`: `add_type`, `copy_type_matrix`, `forget_type_appearance`,
  `add_mob_type`, `NONE`
- `auxtools/src/registry/builds.rs`: `TYPEPATHS`
