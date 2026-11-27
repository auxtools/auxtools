# Entering the instruction hook from BYOND's dispatcher

BYOND's instruction dispatcher (`execute_instruction`) keeps the running frame and opcode in
registers while leaving `EAX`, `ECX`, and `EDX` live across the dispatch jump. Because no calling
convention supported by Rust matches this register state, auxtools enters its instruction hook
through custom assembly stubs: `detour` first tries patching the dispatcher's opcode jump table and
falls back to a whole-function detour if the table is not found.

## Dispatcher registers and frame layout

The registers used by `execute_instruction` vary by platform and BYOND build:

| Build | Frame | Opcode |
|---|---|---|
| Windows | EDI | EDX |
| Linux before 515.1644 | EDI | ECX |
| Linux from 515.1644 (`LINUX_FRAMELESS`) | ESI | EBX |

The start of a frame has this layout (described by `FrameStart`):

| Offset | Field |
|---|---|
| 0x00 | proc instance pointer |
| 0x04 | parent frame |
| 0x08 | file name |
| 0x0C | line |
| 0x10 | bytecode pointer |
| 0x14 | bytecode offset (u16) |

BYOND 516 has `0x18D` opcodes, and BYOND 515 has `0x179`.

## Jump-table stubs

`take_table` locates the dispatcher's opcode jump table by scanning for a
`cmp REGISTER, last opcode`, a near or short `ja` to the unknown-opcode path, and a
`jmp [table + REGISTER * 4]`. The opcode count is the compared value plus one, up to `MOST_OPCODES`
(`0x200`). Every table entry must lie within `0x100000` bytes of the dispatcher so chained stubs
cannot dispatch one instruction forever. On every supported build, BYOND's entries are within
`0x1A000`. `take_table` saves BYOND's entries in `ORIGINAL_TABLE` and points every entry at a stub.
A detour on the unknown-opcode path handles opcodes past the end of the table.

- **Naked stubs:** Each platform and register pair has its own set of naked stubs: `table_stub_*`,
  `first_stub_*`, and `unknown_stub_*`.
- **Register and stack preservation:** Each stub pushes `EAX`, `ECX`, `EDX`, and the frame before
  calling into Rust, then restores them afterward because the dispatcher keeps `EAX`, `ECX`, and
  `EDX` live. The 16 bytes pushed also keep the stack 16-byte aligned at the call site on Linux.
- **Direct frame reads:** To avoid a function call on every instruction, the stubs and
  `about_to_run` read the bytecode pointer at `0x10` and the bytecode offset at `0x14` directly.
  Before installing anything, `take_table` verifies that byte 3 of the dispatcher's code is `0x14`
  and byte 6 is `0x10` (the displacements of its first two frame reads). Two of the registry's
  signature masks do not check them.
- **Re-dispatch on change:** After running the handler, `instruction_dispatched` returns `1` if the
  bytecode, offset, or opcode changed. The stub then jumps back to the start of the dispatcher to
  dispatch again. Otherwise it jumps to BYOND's original entry for the opcode.

## Whole-function detour

If `take_table` cannot find or use the jump table, auxtools detours the start of
`execute_instruction` with an assembly entry point, `execute_instruction_hook`. One assembly file
exists per platform and toolchain (`execute_instruction_hook.unix.S`, `.windows.S` for MinGW, and
`.windows.asm` for MSVC), and `build.rs` compiles the appropriate file with `cc`. Linux before
`515.1644` uses `execute_instruction_hook_edi`.

- The entry passes the frame register to `handle_instruction`. On Linux it saves `ECX` and `EDX` and
  pads the stack to 16 bytes. On Windows it saves `EAX` and `ECX`.
- After the call, the entry moves `EAX` back into the frame register. `handle_instruction` must
  therefore return the frame in `EAX`. It returns the frame it was given, which is
  `*CURRENT_EXECUTION_CONTEXT` when the dispatcher calls in.
- The entry then loads the original's trampoline address into `EAX` and jumps to it.

`execute_instruction_hook` has no standard calling convention. It is installed solely as BYOND's
dispatch target and must never be called from Rust.

## Caveats

- A new BYOND build with a different frame layout or register assignment requires new stubs and a
  new assembly entry. The `0x14` and `0x10` displacement check refuses the table stubs if a build's
  layout changes.
- `handle_instruction` does not follow a frame switch and returns the frame it was called with.
- The instruction handler runs without a panic guard, so any panic that escapes it aborts the world
  process.

## Applies to

- `auxtools/src/instruction_hook.rs`: `take_table`, `detour`, `handle_instruction`,
  `instruction_dispatched`, `about_to_run`, `FrameStart`, `MOST_OPCODES`, `table_stub!`,
  `unknown_stub!`, `execute_instruction_hook`
- `auxtools/src/instruction_hook/`: platform assembly entry files
- `auxtools/build.rs`: assembly compilation
