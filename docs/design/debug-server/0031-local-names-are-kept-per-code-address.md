# Local names are kept per code address

Bytecode does not name its locals or parameters, so auxtools stores local and parameter names per
code address, and a frame running code at an address shows that address's names.

A hot reload can hand back bytecode that old frames are already running. This happens when a reload
renames only locals or parameters (so the new code matches the current code) or when it restores
code from before an earlier reload. Those old frames must keep their original names.

- When the address of the installed code already has other names, auxtools places the new names on
  a copy of the code at a new address created by `auxtools::copy_code`.
- A copy's code entry needs one more word than the code itself: a serial number that keeps BYOND
  from handing back an existing entry with the same words.
- A reload refuses such a rename for code of 65,535 words or more because there is no room for that
  extra word. This restriction applies both to renaming locals or parameters and to a reload that
  only moves lines.

## Applies to

- `debug_server/src/hot_reload.rs`: `names_need_copy`, `moved_names_need_copy`, names step of
  `apply_code`
- `auxtools/src/bytecode_manager.rs`: `copy_code`, `give_own_entry`, `new_entry`
