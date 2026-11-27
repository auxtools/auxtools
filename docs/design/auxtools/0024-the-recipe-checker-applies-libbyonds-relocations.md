# Applying `libbyond.so` relocations in the recipe checker

`libbyond.so` calls its own exported functions through relocations, so those call sites hold
unrelocated addresses on disk. The `check_recipes` example maps the shared object the same way the
loader does and applies `R_386_32` (type 1) and `R_386_PC32` (type 2) relocations whose symbol value
is nonzero, skipping all other relocation types. Without this step, recipes that follow calls to
exports would land on unrelocated addresses and fail.

## Applies to

- `auxtools/examples/check_recipes.rs`
