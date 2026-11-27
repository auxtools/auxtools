# Proc names, references, and definition order

A proc can be reached by a dynamic call through its runtime name, by a path such as
`/type/proc/name`, by `.proc/name`, or by `type::name()`. Which definition each form resolves to
depends on DreamMaker's ordering of a type's definitions for that proc, which differs from the
order the parser produces. DreamMaker also rejects some of these forms in cases the parser accepts.

## Runtime names and parameters

- A dynamic call finds a proc by its runtime name. That name comes from `set name`, or otherwise
  from the proc's name with underscores replaced by spaces.
- BYOND omits `...` and absolute-path parameters from a proc's parameter list, so the compiler
  gives them no argument slots.

## Definition order

- An override keeps its position in the source.
- A declaration takes the position of the type's first declaration of the same kind (`proc` or
  `verb`).
- Because of this, an override can precede the declaration. SpacemanDMM always places the
  declaration first, so the compiler reorders them.
- In `stddef.dm`, the `proc` blocks of `/sound`, `/icon`, `/matrix`, `/database`,
  `/database/query`, `/regex`, and `/generator` precede all user code (`STDDEF_PROC_BLOCKS`), which
  affects the definition order DreamMaker sees for those types.

Definition order decides which override `type::name()` reaches. Hot reload moves definitions and
recomputes the order with `definition_order_at`.

## Proc references

- DreamMaker requires `/type/proc/name` to name the type that declares the proc and refuses it
  otherwise. dmasm compiles it and emits a `ProcPathNotDeclared` warning.
- `.proc/name` searches ancestor types.
- `type::name()` resolves, on the nearest type that defines the proc, to the last of that type's
  definitions in DreamMaker's order. The declaration itself is reached with `.proc/name` rather
  than `type::name()`, and a path names only the first override.
- DreamMaker never resolves `type::name()` to a verb's declaration and refuses `type::verb()` when
  there is no override. dmasm refuses it as well.

## Applies to

- `dmasm/compiler/src/compiler/procs.rs`: `call_var`, `compile_proc_as`, `STDDEF_PROC_BLOCKS`,
  `definition_order`, `definition_order_at`, `last_definition`, `type_proc_path`, proc path lookup
