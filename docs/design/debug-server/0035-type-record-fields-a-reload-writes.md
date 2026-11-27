# Type record fields written by a reload

When a reload sets built-in variable defaults or adds a type, it writes directly to BYOND's type
record.

## Record, value list, and mob table layout

A movable type's record keeps its computed bound and step variables in a block from offset `0x40`
to `0x54` (`bound_x` and `bound_y` can be negative):

| Offset | Bits | Var |
|---|---|---|
| `0x40` | low 16 (`0xFFFF`) | bound_x |
| `0x40` | high 16 (`0xFFFF0000`) | bound_y |
| `0x44` | low 16 | bound_width |
| `0x44` | high 16 | bound_height |
| `0x48` | 32-bit float | step_x |
| `0x4C` | 32-bit float | step_y |
| `0x50` | 32-bit float | step_size |

Other fields and tables follow these layouts:

- `mouse_opacity` is stored in two bits of the type flags (`0x3000`) as the value minus one, so the
  default of `1` is stored as `0`.
- A type's built-in value list is a sequence of entries. Each entry consists of a name string ID
  followed by the value encoded as for `PushVal`. A number (tag `0x2A`) uses its tag word followed
  by the two 16-bit halves of its bits, high half first, for four words in total. Any other value
  stores bits 16 to 23 of its data in the tag word's second byte, followed by the low 16 bits of
  the data, for three words in total. A string has tag `0x06`.
- Only mob types have an entry in BYOND's mob table, which holds the key text, `sight`,
  `see_in_dark`, and `see_invisible`.

## Reload writes and inheritance

- Bound, step, and other list-held built-ins are written as built-in value list entries using the
  encoding above, replacing any existing entry with the same name.
- A new type copies these words from its parent's record: `name`, `desc`, `icon`, `icon_state`,
  `text`, `maptext`, the words at the `MAPTEXT_X` and `MAPTEXT_WIDTH` offsets, the flags, the high
  flags, the aligned word holding `dir`, the bound and step block from `0x40` to `0x54`, `suffix`,
  and `layer`. It also copies the parent's built-in value list and mob entry.
- When a type inherits a bound or step variable again, its list entry is copied from the parent
  along with the matching part of the `0x40` block. When a type changes parent, the block is
  rebuilt from the new parent's block and the type's own values. A `bounds` text value is parsed
  into the bound fields.
- Copying inherited built-ins skips the mob table entry when the parent has none instead of
  failing.

## Caveats

- A layout change in a newer BYOND build breaks reloads of built-in defaults and new types.
- When a reload changes a bound or step default on an existing type, it writes only the built-in
  value list entry, not the `0x40` block. Whether BYOND then reads the stale block is unverified.
- The number tag `0x2A` is written as a bare literal in the code.

## Applies to

- `debug_server/src/hot_reload.rs`: `builtin_writes`, `write_builtins`, `inherited`, `add_type`
- `debug_server/src/reload_inherit.rs`: `bounds_place`, `listed_entries`, `copy_builtin`
- `debug_server/src/reload_reparent.rs`: `inherit_builtins`, `bounds_block`
