# Copying BYOND's shared scratch list after `range()` and `orange()`

Some BYOND instructions fill and push a single shared scratch list rather than allocating a new
list, so the next call of the same kind overwrites it. `Range` (`0x59`) and `ORange` (`0xAD`)
always behave this way. So do the Target forms of `View`, `OView`, and `Block` (`ViewTarget`
`0x1D`, `OViewTarget` `0x1E`, and `BlockTarget` `0x20`).

DreamMaker follows `range()` and `orange()` with `CopyList` so the caller holds its own list,
omitting the copy only when the call is on the left side of `<<`, where the list is consumed
immediately. DreamMaker emits the Target forms of `view()`, `oview()`, and `block()` only on the
left side of `<<` (as in `view() << "hi"`), pushing the shared scratch list without copying it.

dmasm always follows `range()` and `orange()` with `CopyList`, including on the left side of `<<`,
and never emits the Target forms (though the bytecode crate still models them so it can read and
write DreamMaker's output). Omitting `CopyList` would allow a later `range()` to overwrite a list
that code still holds. The extra copy on the left of `<<` only costs time, though the bytecode
there differs from DreamMaker's.

## Applies to

- `dmasm/compiler/src/compiler/builtin_procs.rs`: `range` and `orange` case in `emit`
- `dmasm/bytecode/src/instructions.rs`: `View`, `ViewTarget`, `OViewTarget`, `BlockTarget`,
  `Range`, `ORange`
