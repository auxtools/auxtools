# Reloading the client script

BYOND's world data holds the client script's text as a string ID (or `0xFFFF` when there is none)
and a list of resource IDs containing the included `.dms` files followed by the `client/script`
file. A client script reload modifies only the text field and leaves the resource list untouched.

auxtools locates three values by scanning BYOND's code: the global that holds the world data holder
(`client_script_send`), the offset of the world data within the holder (`client_script_data`), and
the offset of the file count within the world data (`client_script_count`). The other two fields
are derived from the count's offset:

| Field | Offset in the world data | Type |
|---|---|---|
| text | count − 8 | `u32` string id, `0xFFFF` for none |
| file list | count − 4 | pointer to `count` resource ids |
| file count | count | `u16` |

Because the text field's offset is derived, the debugger verifies before its first write that the
field currently holds the text from the build (identical text shares the same interned string ID).
If the field does not match, either the layout guess is wrong or the world is out of date, so the
debugger refuses the reload instead of writing to the wrong address.

The debugger tracks what it has installed in `OUR_TEXT`, an `Option<Option<u32>>`: `None` means the
field still holds the build's value, `Some(None)` means the debugger removed the text, and
`Some(Some(id))` means the field holds a string the debugger installed. The field holds a counted
reference to an installed string, and that reference belongs to the debugger. It is released only
when the debugger replaces its own string, never for the build's string. `OUR_TEXT` is reset when
the world ends.

## Applies to

- `debug_server/src/client_script.rs`: `NO_TEXT`, `Place`, `place`, `fields`, `OUR_TEXT`,
  `check_built`, `write_text`
