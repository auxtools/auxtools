# Skin and script field offsets, and proc layout test baselines

The world's skin and client script live in a world data block whose field offsets moved between
BYOND 515 and 516, so auxtools reads those offsets directly from BYOND's machine code. Separately,
the proc layout tests check Rust struct sizes and offsets against values measured from BYOND
`516.1688` binaries rather than reading BYOND directly.

## Skin and client script offsets

A global points to a holder, the holder points to the world data block at `+0x0C`, and the skin and
client script fields sit at offsets inside that block.

The skin field is read immediately after BYOND sends the skin message (`0xAF`) to a client, where a
value of `0xFFFF` means the world has no skin:

- **Linux:** `mov eax,[G]; mov eax,[eax+0x0C]; mov edx,[eax+F]`, followed by a comparison with
  `0xFFFF`.
- **Windows:** `mov eax,[G]; mov ecx,[eax+0x0C]; mov eax,[ecx+F]`, followed by a comparison with
  `0xFFFF`.

The function that sends the client script takes a `u16` connection and uses `cdecl`. Near its start,
it reads the script's file count, a `u16` at offset `N` in the data block:

- **Linux from `515.1644`:** `mov eax,[G]`, then `mov eax,[eax+0x0C]` and `cmp word [eax+N],0`.
- **Linux before `515.1644`:** `mov edx,[G]; mov eax,[edx+0x0C]; cmp word [eax+N],0`.
- **Windows:** `mov edx,[G]` (with `G` at byte 64 of the signature), then `mov ecx,[edx+0x0C]` and
  `cmp ax,[ecx+N]`.

The offsets in BYOND's binaries are identical on Linux and Windows across every build in both
ranges:

| Builds | Skin field F | Script file count N |
|---|---|---|
| 515.1637 to 515.1647 | 0x4C | 0x48 |
| 516.1648 to 516.1689 | 0x5C | 0x58 |

The signatures leave `F` and `N` as wildcards. The `world_skin_field` and `client_script_count`
recipes extract them from the matched instruction, just as `world_skin_data` and
`client_script_data` extract the `0x0C` offset. A single signature therefore covers both 515 and 516
for each code shape and continues to work if a later build moves the fields again. The client
script's text ID is read at `N - 8` and its list of files at `N - 4`.

Because a signature that matches the wrong code still yields some displacement, and no test compares
the extracted `F` and `N` against the table above, compare the extracted offsets by hand on both a
515 build and a 516 build whenever a recipe changes.

## Proc layout test baselines

The expected sizes and offsets in `layout_tests` in `auxtools/src/raw_types/procs.rs` were measured
on the `516.1688` binaries:

- `ExecutionContextPre1668` holds the `516.1688` Linux layout (148 bytes), and
  `ExecutionContextPost1668` holds the `516.1688` Windows layout (152 bytes).
- `ProcInstance`, `ProcInstanceInnerPost516`, and `ProcEntry` match `516.1688`.
- The single 515 value, `ProcInstanceInnerPre516`'s `args_count` at `0x28`, is one dword earlier
  than 516's offset.
- The tests do not record which builds the earlier Windows and 515 layouts were measured on.

Because these tests check the Rust structs against `516.1688`'s numbers without inspecting BYOND,
they do not guarantee that other BYOND builds share the same layout.

## Applies to

- `auxtools/src/registry/linux.rs`: `SKIN`, `SKIN_EARLY`, `SCRIPT`, `SCRIPT_EARLY`, and the
  `world_skin`, `world_skin_data`, `world_skin_field`, `client_script_send`, `client_script_data`,
  and `client_script_count` recipes
- `auxtools/src/registry/windows.rs`: `SKIN`, `SCRIPT`, and the `world_skin`, `world_skin_data`,
  `world_skin_field`, `client_script_send`, `client_script_data`, and `client_script_count` recipes
- `debug_server/src/skin.rs`: `field`
- `debug_server/src/client_script.rs`: `fields`
- `auxtools/src/raw_types/procs.rs`: `layout_tests`
