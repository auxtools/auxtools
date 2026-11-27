# Frame ordering and local variable names in call stacks

When auxtools reads BYOND call stacks, it normalizes frame order across running and sleeping procs
and resolves local and parameter names from per-code records rather than relying solely on the proc
table.

## Frame order

BYOND stores a sleeping proc's stack frames in the reverse order of a running stack's frames.
Walking a running stack from the current frame through parent links yields the innermost frame
first, so `CallStacks` reverses suspended stacks so both kinds list the innermost frame first.

## Local and parameter names

Bytecode does not name its local variables, which allows procs with different local names to share
bytecode. In addition, a reloaded proc or a frame still executing old code can need local and
parameter names that the proc table no longer holds.

- auxtools stores local and parameter names in `LOCAL_NAMES`, keyed by proc ID and bytecode address.
- `set_local_names` also records the proc's current parameter names, so frames running that code
  continue to show those names even if a reload renames the parameters. When no entry exists in
  `LOCAL_NAMES`, names fall back to the proc table.
- When the bytecode manager moves code to a new address, it copies the recorded names to the new
  address with `copy_local_names`. Before a proc takes over shared code, frames of other procs
  executing that code have their current names saved with `keep_frame_names`.

Recorded names are cleared at shutdown. Code that inspects frames must call `local_names_for` and
`parameter_names_for` rather than reading the proc table directly, so it retrieves the names the
frame's code was built with.

## Applies to

- `auxtools/src/debug.rs`: `CallStacks::from_context`, `LOCAL_NAMES`, `set_local_names`,
  `local_names_for`, `parameter_names_for`, `copy_local_names`, `clear_local_names`
- `auxtools/src/bytecode_manager.rs`: `keep_frame_names`, `give_own_entry`
