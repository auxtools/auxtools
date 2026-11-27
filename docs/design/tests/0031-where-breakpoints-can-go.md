# Valid instruction offsets for instruction and proc breakpoints

A breakpoint replaces an instruction's first word. Placing one on a later word of a multi-word
instruction (such as on the value of `return 2`) would turn the remaining words into separate
instructions.

Breakpoint placement enforces two rules:

- **Instruction breakpoints:** The debug server refuses an offset inside an instruction or past the
  end of the code. When an instruction breakpoint falls inside an instruction, the debug adapter
  reports it as unverified with the message
  "ADDRESS is inside an instruction in PROC (the instruction starts at ADDRESS)".
- **Proc breakpoints:** A breakpoint on a proc by name is placed on the instruction after the proc's
  first `DbgLine` rather than at offset `0`.

`dap_test` and `debug_test` check both refusals. `launcher_test` relies on the proc breakpoint's
placement: after `step-over --instruction` from a proc breakpoint, it expects the instruction after
the one after the first `DbgLine`.

## Applies to

- `tests/dap_test/src/groups/bytecode.rs`: instruction breakpoint checks
- `tests/debug_test/src/main.rs`: breakpoint offset checks
- `tests/launcher_test/src/inspect.rs`: instruction Step Over check
