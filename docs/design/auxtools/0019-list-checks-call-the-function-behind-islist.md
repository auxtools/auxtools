# List checks call the function behind `islist()`

A `filters` list and a single filter share the same value tag, so checking the tag alone cannot tell
whether a value is a list. BYOND's exported list check also rejects a `filters` list on `516.1659`
(observed on Windows `516.1659`). `List::is_list` therefore calls BYOND's internal `value_is_list`
function, the same function behind DM's `islist()`, rather than the exported check.

`value_is_list` takes a second argument controlling whether a pointer to a list counts as a list.
`List::is_list` passes `0` as that last argument, matching the `islist()` opcode, so a pointer to a
list does not count as a list.

The `value_is_list` recipes locate the function by signature rather than through an export. Each
signature is distinguished by the highest list tag the function accepts, since BYOND 515 has one
fewer list tag than 516:

| Build | Signature bytes | Highest tag |
|---|---|---|
| Linux 516 | `80 FA 55` (`cmp dl, 0x55`) | 0x55 |
| Linux late 515 | `80 FA 54` | 0x54 |
| Windows 516 | `83 C0 F1 83 F8 46` (tag minus 0x0F, compared with 0x46) | 0x55 |
| Windows 515 | `83 C0 F1 83 F8 45` | 0x54 |

If a later BYOND build adds a new list tag, the upper bound changes and the signature must be
updated to match.

## Applies to

- `auxtools/src/list.rs`: `List::is_list`
- `auxtools/src/raw_types/funcs.cpp`: `value_is_list`
- `auxtools/src/registry/linux.rs`: `value_is_list` recipes
- `auxtools/src/registry/windows.rs`: `value_is_list` recipes
