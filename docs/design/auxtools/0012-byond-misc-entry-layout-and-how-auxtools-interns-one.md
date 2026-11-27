# BYOND misc entry layout and interning

A BYOND misc entry is a 12-byte structure with this layout:

| Offset | Field |
|---|---|
| 0x0 | word count (u16) |
| 0x4 | reference count (u32) |
| 0x8 | pointer to the words |

`misc_words` and the bytecode manager read the word count at offset `0`, the reference count at
offset `4`, and the words pointer at offset `8`.

## Interning misc entries

On Linux BYOND builds before `515.1644`, misc interning is inlined. The callable `misc_intern`
accepts a 12-byte entry in the layout above and takes ownership of its words buffer. Other builds
provide `misc_intern_copy`, which copies the words it is given.

`intern_misc` calls `misc_intern_copy` wherever it exists. On Linux builds before `515.1644`, it
builds the 12-byte entry itself:

- It allocates both the entry and the words buffer with
  [BYOND's allocator](0014-byond-tables-are-grown-with-byonds-allocator-and-read-through-their-globals.md)
  so BYOND can free them. A words buffer passed to `misc_intern` must never come from Rust's
  allocator, because BYOND takes over the buffer and will free it.
- It writes the word count at offset `0`, `0` at offset `2`, a reference count of `0` at offset `4`,
  and the words pointer at offset `8`.

In both paths, `intern_misc` then takes a permanent reference to the returned entry. BYOND shares
entries that contain identical words. Inputs exceeding `u16::MAX` words are refused, and a result of
`NONE` (`0xFFFF`) indicates failure.

## Applies to

- `auxtools/src/runtime_tables.rs`: `MiscIntern`, `misc_intern`, `intern_misc`, `misc_words`
- `auxtools/src/bytecode_manager.rs`: `misc_entry`, `references`, `code_at`
- `auxtools/src/raw_types/misc.rs`: `Bytecode_V2`
