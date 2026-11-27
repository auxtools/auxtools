# Mapping and resolving resource files as DreamMaker does

To reload an edited icon or sound, the launcher must map the file on disk to the resource IDs used
by the running world. The resource watcher builds this mapping by joining two tables: the `.dmb`
maps resource IDs to CRCs (see
[reading `.dmb` tables in DreamMaker's order](0001-read-dmb-tables-in-dreammakers-order.md)), and
the `.rsc` maps CRCs to file names.

Files with identical bytes produce the same CRC and therefore share a single resource ID.
Conversely, a file referenced under two spellings (for example, with different casing) can have two
IDs. Because files with identical bytes share one ID, an edit to one of them cannot be shown on its
own. The watcher logs this situation and suggests a rebuild.

## Resolving resource names

DreamMaker looks up a resource name under the project directory first, then under each `FILE_DIR`
in order, ignoring case even on Linux. The resource watcher follows the same resolution rules:

- Backslashes in the resource name are converted to forward slashes.
- The path is checked under the project directory first, then under each `FILE_DIR` in order, and
  the first existing file wins.
- Each path component is matched exactly first. If no exact match exists, it is matched without
  regard to ASCII case. When several entries match case-insensitively, the first in sorted order is
  chosen.

## `.rsc` record layout

A `.rsc` file consists of a sequence of records with the following layout:

| Field | Size |
|---|---|
| length | u32 |
| in use | u8 |
| type | u8 |
| crc | u32 |
| time | u32 |
| time | u32 |
| data length | u32 |
| name | NUL-terminated |
| data | data length bytes |

The `length` field excludes both itself and the `in use` byte, so the next record starts
`5 + length` bytes after the start of the current record. A record whose `in use` byte is `0` is
free space and is skipped. Records with fewer than 19 bytes from the `in use` byte onward, as well
as records with an empty name, are also skipped.

`debug_server/src/world_resources.rs` parses the same `.rsc` record format, so the two readers must
stay in agreement. Sharing a single reader implementation would avoid maintaining a second copy.

## Applies to

- `auxtools-cli/src/resources.rs`: `rsc_names`, `resolve`, `existing`
- `auxtools-dm/src/dmb_tables.rs`: `resource_crcs`
