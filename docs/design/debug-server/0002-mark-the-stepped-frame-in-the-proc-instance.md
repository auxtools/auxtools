# Marking the stepped frame in its proc instance

Step Over, Step Into, Step Out, and instruction steps need to track which frame they are stepping.
The debugger marks that frame by writing a 16-bit value into a proc instance field that auxtools
calls `mega_hack`. Because BYOND does not initialize this field and reuses proc instance memory, a
new call can begin with a stale mark left over from an earlier step. In addition, `spawn(-1)` runs
its block in a new execution context above the current one, carrying a copy of the same proc
instance with the same mark.

To keep step tracking accurate across reused instances and `spawn(-1)` copies:

- Each mark is new: the stepper increments marks from `0x8001` up to `u16::MAX` and then wraps back
  to `0x8001`.
- While a step is active, the stepper clears `mega_hack` at the start of every new call, as reported
  by `starts_call` (see
  [identifying frames by BYOND proc instance ID](0001-identify-frames-by-proc-instance-id.md)).
  Without clearing at call entry, a reused proc instance could carry the active mark and be mistaken
  for the stepped frame.
- A context counts as the stepped frame only if its proc instance carries the mark and no parent
  context carries the same mark. A context with the mark below it is a `spawn(-1)` copy, so Step
  Over skips it instead of stopping inside the `spawn(-1)` block as if it were the next line of the
  stepped proc. `forget_copy` clears the mark on such a copy.

## Applies to

- `debug_server/src/instruction_hooking.rs`: `ProcInstanceRef::new`, `is`, `is_frame`, `is_copy`,
  `forget_copy`, step handling in `Server::handle_instruction`
