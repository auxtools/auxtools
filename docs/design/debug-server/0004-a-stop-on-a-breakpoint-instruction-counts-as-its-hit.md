# Stops on a breakpoint instruction always count as hits

Execution can stop on an instruction that holds a breakpoint for reasons other than the breakpoint
itself, including a step, a pause request, a runtime error or throw, or a data breakpoint. To keep
hit counts and logpoints accurate regardless of how execution reached the instruction, the debug
server still counts the breakpoint's hit and sends its log message whenever any of those stops lands
on a breakpoint instruction.

Because a breakpoint overwrites the first word of its instruction, line stepping (which watches for
`DbgLine` opcodes to detect a new line) sees the breakpoint opcode instead when a breakpoint sits on
a `DbgLine`. When the original instruction underneath a breakpoint is a `DbgLine`, the instruction
hook reports that line to line stepping itself. If Step Over or Step Into is stepping that frame,
the step becomes Break On Next so it does not skip the line after the breakpoint.

## Applies to

- `debug_server/src/server/pause.rs`: `Server::handle_breakpoint`
- `debug_server/src/instruction_hooking.rs`: `OPCODE_DEBUG_BREAK` branch of
  `Server::handle_instruction`
