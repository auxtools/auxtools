# Reading `.dmb` tables in DreamMaker's order

Hot reload of resource files requires the `.dmb` resource table, which maps resource IDs to CRCs.
Because the resource table is the last table in the file and a `.dmb` stores no table lengths,
`resource_crcs` parses every preceding table field by field in DreamMaker's output order, skipping
fields it does not need.

Two validation checks catch any layout mismatch: the string table's total size (and, from `major`
468, its checksum) must match, and the resource table must end at the exact end of the file. Any
mismatch returns an error naming the problem along with `"unsupported .dmb layout"`. The reader
never guesses or silently returns wrong CRCs if a newer `.dmb` format adds or moves a field before
the resource table.

## Header versions and field widths

A `.dmb` file may begin with a `#!` line, followed by two header lines and then the binary body.
Many fields exist only from a specific format version onward, governed by two header numbers:

- `major`: parsed from the first header line (`world bin vMAJOR`).
- `lhs`: the first number on the second header line (`min compatibility vLHS ...`). All `lhs`
  thresholds below (494, 500, 508, 509, 512, and 514) are compared against this value, not against
  the BYOND build that compiled the world.

In the table below, an "id" represents an ID or a count. It is 4 bytes wide in a world with large
IDs and 2 bytes wide otherwise. All numbers are little-endian.

## Table layout

The tables appear in the order below. A field marked "unnamed" is skipped by the reader without a
name in the code.

| Part | Field | Size | Present when |
|---|---|---|---|
| Flags | flags | u32 | always. Bit `0x40000000` means large ids. |
| | unnamed | u32 | flags bit `0x80000000` is set |
| Map | x size, y size, z size | u16 each | always |
| | runs: turf, area, extra turfs | 3 ids | repeated until the copy counts cover x × y × z cells |
| | copies | u8 | after each run. A count of 0 is an error. |
| | string bytes | u32 | always: the total length of all strings, each counted with one extra byte |
| Types | count | id | always |
| | path, parent, name, desc, icon, icon_state | 6 ids | each type |
| | dir | u8 | each type |
| | unnamed | u8, then a u32 if the u8 is `0x0F` | `major` ≥ 307 |
| | text | id | each type |
| | maptext | id | `lhs` ≥ 494 |
| | maptext width and height | 4 bytes | `lhs` ≥ 494 |
| | maptext_x, maptext_y | 4 bytes | `lhs` ≥ 508 |
| | suffix | id | each type |
| | unnamed | u32, and another u32 when `lhs` ≥ 514 | `major` ≥ 306 |
| | unnamed | u8 | `major` < 306 |
| | verbs, procs, init proc, initialized vars, vars | 5 ids | each type |
| | unnamed | id | `lhs` ≥ 514 |
| | layer | u32 | `major` ≥ 267 |
| | transform flag | u8 | `lhs` ≥ 500 |
| | transform | 24 bytes (6 floats) | the transform flag is not 0 |
| | color matrix flag | u8 | `lhs` ≥ 509 |
| | color matrix | 80 bytes (20 floats) | the color matrix flag is not 0 |
| | overriding vars | id | `major` ≥ 306 |
| Mobs | count | id | always |
| | path, key | 2 ids | each mob |
| | flag | u8 | each mob |
| | sight flags, see_in_dark, see_invisible | 6 bytes | the flag is `0x80` or more |
| Strings | count | id | always |
| | length parts | u16 each | each string, see below |
| | text | the length | each string, see below |
| | checksum | u32 | `major` ≥ 468 |
| Lists | count | id | always |
| | n, then n ids | u16, then n ids | each list |
| Procs | count | id | always |
| | path | id | large ids, or `major` ≥ 224 |
| | name, desc, category | 3 ids | each proc |
| | verb src | 2 bytes | each proc |
| | flag | u8 | each proc |
| | unnamed | 5 bytes | flag bit `0x80` is set |
| | bytecode, locals, parameters | 3 ids | each proc |
| Vars | count | id | always |
| | tag and data | 5 bytes | each var |
| | name | id | each var |
| | unnamed | u32 | `lhs` ≥ 512 |
| World init procs | count, then that many ids | id each | always |
| Instances | count | id | always |
| | tag and data | 5 bytes | each instance |
| | init proc | id | each instance |
| Map extra turfs | count | u32 | always |
| | cells since the last | u16 | each entry |
| | turf | id | each entry |
| World settings | 7 ids; u32; 2 ids | | always |
| | 4 bytes | | `lhs` > 514 (strictly greater) |
| | 4 bytes; u8; 1 id; u16 n and n ids; u16; 2 ids; 8 bytes; u16; 5 ids; 6 bytes | | always |
| Resources | count | id | always |
| | crc | u32 | each resource |
| | type | u8 | each resource |

The world settings hold the world's `mob`, `turf`, `area`, `name`, and other settings. They precede
the skin in DreamMaker's output, except `icon_size` and `map_format`, which follow the skin. The
reader skips the world settings without naming their fields. Which skipped field corresponds to
which setting, and where the skin falls among them, is not recorded in the code and is unverified.

Because the reader skips fields without naming them in code, this table is the record of what each
field represents. Naming the fields in the reader itself (for example, with a skip helper that takes
field names) would let the code document them directly. The world settings, lists, and other
preceding tables are parsed only to reach the resource table, and nothing else in auxtools consumes
their contents from this reader.

## Strings and BYOND's CRC

String lengths and text are obfuscated:

- Each 16-bit length part is XORed with its own offset, truncated to 16 bits. Offsets are measured
  from the start of the `world bin` line, so any leading `#!` line is excluded.
- A length part of `0xFFFF` is followed by another part that is added to it, making the total length
  the sum of the parts.
- Each byte of text is XORed with a key that starts as the offset of the first text byte (truncated
  to 8 bits) and increments by 9 for each subsequent byte, wrapping at 256.

The string table checksum uses BYOND's CRC over the decoded bytes of every string in order, with a
`0` byte appended after each string. The sum of each string's length plus one must equal the
`string bytes` field from the map section.

BYOND uses a single 32-bit CRC algorithm for both resource UIDs and the `.dmb` string checksum. It
is processed MSB first with polynomial `0xAF`, starts at `0xFFFFFFFF` (all ones), and has no final
inversion. The CRC of an empty byte slice is therefore `0xFFFFFFFF`, and entry 1 of its lookup
table is `0xAF`.

## Applies to

- `auxtools-dm/src/dmb_tables.rs`: `resource_crcs`, `header`, `Reader`, `byond_crc`, `crc_byte`,
  `crc_table`
- `auxtools-cli/src/resources.rs`: resource watcher (caller)
