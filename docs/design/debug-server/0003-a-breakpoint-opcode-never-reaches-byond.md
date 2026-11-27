# A breakpoint opcode never reaches BYOND

A breakpoint replaces the first word of an instruction with `OPCODE_DEBUG_BREAK` (`0x1337`) and each
of its operand words with `OPCODE_DEBUG_OPERAND` (`0x1338`), storing the original words by address
in `ORIGINAL_BYTECODE`. BYOND does not recognize these opcodes and terminates a proc with a
bad-instruction error if it executes one. Because DM code can also continue running after
`world/Del` while the debugger is shutting down, every code path must ensure BYOND never sees a
breakpoint opcode.

- When the instruction hook encounters a breakpoint, it copies the original instruction back into
  place before BYOND runs it, stores the breakpoint words in `DEFERRED_INSTRUCTION_REPLACE`, and
  reinstalls them at the start of the next hook call.
- After handling any pause earlier in the same hook call, the hook re-reads the opcode because the
  pause may have placed a breakpoint on the instruction about to execute. If so, it restores the
  original instruction in the same way.
- At shutdown, `instruction_hooking_shutdown` removes all breakpoints so code running after
  `world/Del` cannot hit one that nothing handles.
- When the debugger gives up after repeated panics (see
  [stopping panics and world shutdowns at `guard`](0016-panics-and-world-ends-stop-at-guard.md)), it
  removes all breakpoints before it stops handling instructions.

Any code path that disables the instruction hook must remove all breakpoints first. In addition,
`passes` (the hook's fast path) must never skip an instruction while a deferred replacement is
pending, or the breakpoint would not be restored.

## Applies to

- `debug_server/src/instruction_hooking.rs`: `OPCODE_DEBUG_BREAK`, `OPCODE_DEBUG_OPERAND`,
  `ORIGINAL_BYTECODE`, `DEFERRED_INSTRUCTION_REPLACE`, `hook_instruction`,
  `instruction_hooking_shutdown`, `passes`, `Server::handle_instruction`
- `debug_server/src/lib.rs`: panic limit in `settle`
