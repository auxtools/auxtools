# Type paths where `text2path()` falls short

The assembler turns type paths into values by evaluating `text2path()` in the running world, but
`text2path()` fails or disagrees with DreamMaker in several cases:

- `text2path()` returns `null` for `/list` and `/savefile`, although both are valid type paths.
- `text2path("/client")` returns a value whose data is `/client`'s path ID, whereas code compiled by
  DreamMaker always uses `0` as the data of a `/client` type value.
- Before a rebooted world starts running, `text2path("/image")` can return `null`.
- `text2path()` caches failed lookups, so a type added by a hot reload after a failed lookup of the
  same path still resolves to `null`.
- A proc declaration removed by a reload stays in the running world, so `text2path()` still finds
  it.

To handle these cases, `type_value` resolves a path in four steps:

1. `/list`, `/savefile`, and `/client` are built directly with the `ListTypepath`,
   `SaveFileTypepath`, and `ClientTypepath` tags and data `0`.
2. `text2path()` is queried.
3. Image types are recognized from the type table by checking whether `/image` is among the type's
   ancestors, receiving the `ImageTypepath` tag and the type ID.
4. Types added since are found in the runtime type table, taking their tag from the nearest
   ancestor that `text2path()` knows.

When a reload removes every proc that a path can name, the path resolves only to a non-proc type
value, and any proc value that `text2path()` still finds for the removed declaration is ignored. If
no type value exists, the reload refuses the code that names the path.

## Applies to

- `debug_server/src/assemble_env.rs`: `type_value`, `added_type`
- `debug_server/src/hot_reload.rs`: `image_type`, `removed_path_type`, `removed_proc`
