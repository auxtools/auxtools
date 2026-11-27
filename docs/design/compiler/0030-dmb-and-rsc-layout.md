# Reading `.dmb` and `.rsc` tables in the comparison tool

The comparison tool pairs the procs of a DreamMaker-built `.dmb` with the procs in the source, and
loads the tables that dmasm's code is assembled against: strings, types, procs, variables, prefab
instances, and resource IDs. Because a `.dmb` has no index to its tables, the reader parses every
table sequentially from the start of the file, skipping fields it does not need, and verifies its
position using the string table's checksum and byte count.

The binary layout below describes how BYOND 515 and 516 write `.dmb` and `.rsc` files.

## File structure and IDs

A `.dmb` begins with two text lines: `world bin vMAJOR` and `min compatibility vLHS RHS`. Fields
added by later BYOND builds depend on `MAJOR` or on `LHS` (the first compatibility number).

After the text header, the tables follow as a single lengthless stream in a fixed order: header, map
grid, types, mob types, strings, misc entries (bytecode and lists of IDs), procs, variables, prefab
instances, the map's extra turfs, the world's settings, and the resources. The resources end the
file, and reaching any table requires reading every table before it.

- An ID or count is 4 bytes in a world with large IDs and 2 bytes otherwise. Large IDs are indicated
  by flag bit 30 of the header's first word. When bit 31 is set, another 4 bytes follow the flags.
- A missing object ID is written as `0xFFFF` in either ID width, and the reader also accepts
  `0xFFFFFFFF`.

## String table

The string table is XORed with keys derived from file offsets, so the reader tracks positions
relative to the start of the file rather than the start of the tables:

- Each string's 16-bit length is XORed with the low 16 bits of the length's file offset.
- Each byte of a string is XORed with a key that starts at the low byte of its file offset and
  increases by 9 for each byte.

The string table's stored checksum (present from major 468) and the byte count in the header verify
that every table before the string table was read with the right layout. If either value disagrees,
the reader reports an error naming the mismatch.

## Record layouts

A type record contains these fields in order:

| field | size | present when |
| --- | --- | --- |
| name | id | always |
| parent | object id | always |
| name, desc, icon, icon_state | 4 object ids | always |
| dir | 1 byte | always |
| a byte; if it is 0x0F, 4 more bytes | 1 or 5 bytes | major 307 or later |
| text | object id | always |
| maptext | object id | LHS 494 or later |
| maptext_width, maptext_height | 4 bytes | LHS 494 or later |
| maptext_x, maptext_y | 4 bytes | LHS 508 or later |
| suffix | object id | always |
| unnamed | 4 bytes, plus 4 more from LHS 514; 1 byte before major 306 | always |
| verbs list | object id | always |
| procs list | object id | always |
| initializer proc | object id | always |
| initialized vars list | object id | always |
| vars list | object id | always |
| unnamed | object id | LHS 514 or later |
| layer | 4 bytes | major 267 or later |
| a byte; if nonzero, the transform, 24 bytes | 1 or 25 bytes | LHS 500 or later |
| a byte; if nonzero, the color matrix, 80 bytes | 1 or 81 bytes | LHS 509 or later |
| overridden built-in vars list | object id | major 306 or later |

The remaining records and trailing tables follow these layouts:

- **Mob type entry:** The type ID, the key (an object ID), and a byte. When that byte is `0x80` or
  greater, 6 bytes follow for the sight flags, `see_in_dark`, and `see_invisible`.
- **Proc record:** The path, name, desc, and category (all object IDs), 2 bytes for the verb's `src`
  range and kind, a byte followed by 5 more bytes when its bit `0x80` is set, the bytecode's misc
  ID, and the locals and params lists (object IDs).
- **Variable record:** 5 bytes for the initial value's tag and data, followed by the name's string
  ID. From major and LHS 512, 4 additional bytes follow the variable table.
- **Trailing tables:** After the prefab instances come the map's extra turfs, the world's settings
  in the layout BYOND 515 and 516 write, and finally a `(crc, type)` pair for each resource ID,
  which ends the file.

## Resource (`.rsc`) entries

A `.rsc` entry consists of a 4-byte length followed by the entry body. The body starts with a valid
byte, a type byte, a CRC (bytes 2–5), two timestamps, and the data length, with the NUL-terminated
name at offset 18. The reader pairs resource IDs with names by matching the CRCs in the `.dmb`
against the CRCs in the `.rsc`.

## Caveats

- Reading any field with the wrong size shifts every subsequent table. The string table check
  catches mistakes in tables before the string table, but not in tables after it.
- Strings of 65535 bytes or more are not read.
- The reader is only as complete as the comparison tool requires. When a newer BYOND build adds a
  field, `read_type` or `read_resources` needs a new version condition.

## Applies to

- `dmasm/compiler/examples/compare_dreammaker/dmb.rs`: `Reader`, `Dmb::load`, `header_lines`,
  `read_type`, `read_strings`, `read_resources`, `Dmb::read_rsc`
