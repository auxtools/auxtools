# Resources before the world has loaded them

When the launcher starts a world, the debugger attaches before BYOND loads the world's resources.
Calling `fcopy_rsc()` during that window allocates cache IDs that BYOND later reuses for other
files from the build. Any other world loads the debug library after its resources are already
loaded.

## Built resources in `.dmb` and `.rsc`

The `.dmb`'s resource table lists each built resource's CRC by resource ID, using BYOND's own CRC
computed by `byond_crc` in `auxtools-dm/src/dmb_tables.rs`. The `.rsc` file next to the `.dmb` is a
sequence of records:

| Offset in the record | Size | Field |
|---|---|---|
| 0 | `u32` | length: the number of bytes after the in-use flag |
| 4 | `u8` | in-use flag |
| 5 | `u8` | type |
| 6 | `u32` | CRC |
| 10 | `u32` | time |
| 14 | `u32` | time |
| 18 | `u32` | data length |
| 22 | NUL-terminated | name, with backslashes |
| after the name | | data |

The next record begins `5 + length` bytes after the start of the current record. `built_file` reads
the names of in-use `.rsc` records to tell which files came with the build, skipping unused records
and converting backslashes in names to slashes.

## Waiting for resources to load

- Resources count as loaded unless the launch path says otherwise. The launcher's attach marks them
  as loading, and the helper `/world/New` marks them as loaded. A world without that hook is marked
  as loaded immediately.
- `world/New` runs once the world has read its data and before any client logs in. The helper
  `/world/New` reports to the debugger before calling `..()`, and therefore before the world's own
  `New`, so reloads that waited for resources are applied before any client can see the world.
- Until resources are loaded, the assembler does not call `fcopy_rsc()`. Instead, it resolves a
  file's ID by content by looking up the file's BYOND CRC in the `.dmb`'s resource table.
- A reload that names a file not in the build waits until resources are loaded. Once one reload
  waits, every later reload waits as well so reloads are always applied in the order they were
  sent.

## Applies to

- `debug_server/src/world_resources.rs`: `LOADED`, `loading`, `loaded`, `built_id`, `not_built`,
  `built_file`, `read_names`
- `debug_server/src/assemble_env.rs`: `get_resource_index`
- `debug_server/src/server/reload.rs`: `waiting_reloads`, `world_data_loaded`
- `debug_server/src/launched.rs`: `start`, `world_new`
