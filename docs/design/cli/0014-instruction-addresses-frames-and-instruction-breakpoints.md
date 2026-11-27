# Instruction addresses, frame IDs, and instruction breakpoints

DAP refers to instructions by string memory references and resends the full list of instruction
breakpoints on every change, while hot reload replaces a proc's bytecode and keeps old code alive
for sleeping procs until the world reboots. auxtools packs each instruction's proc ID, code ID, and
word offset into a single `u64` address and uses that address to preserve instruction breakpoints
across reloads.

## Instruction addresses

An instruction address is a `u64` laid out as `0xPPPPPPPPCCCCOOOO`:

- High 32 bits: the proc ID.
- Next 16 bits: the code ID.
- Low 16 bits: the instruction offset, counted in 4-byte words.

Code ID `0` is the proc's current code. Non-zero code IDs are older versions of the code kept until
reboot, so addresses of old code remain valid only until the world reboots. Code ID `65,535` is old
code that ran after all other IDs were used, and no request can name it. DAP memory references and
instruction pointer references use this encoding.

A hot reload never changes a proc's ID, so only the code ID and the offset of an instruction address
can change.

## Frame IDs

Running frames are numbered by stack depth from `0`, innermost first. Sleeping frames receive IDs
that stay valid for the entire pause.

## Instruction breakpoints

- An instruction breakpoint is accepted only at the start of an instruction. Placing a breakpoint
  inside an instruction would overwrite part of its operands.
- To find where a reload left a breakpoint, the debug adapter stores the address originally
  requested and replaces the low 16 bits with the claim's current offset. This works because the
  proc ID never changes.
- DAP's full breakpoint list is matched against the breakpoints set previously. An incoming entry
  matches an existing breakpoint when it requests either the same address or the address that a
  reload moved that breakpoint to. Because one breakpoint's moved address can equal another
  breakpoint's originally requested address, the list is paired twice: once matching by requested
  address first and then by moved address, and once in the reverse order. The pairing that matches
  more breakpoints wins, with ties broken in favor of matching by requested address first.
- An instruction breakpoint keeps its ID, and therefore its hit count, across a reload that moves
  it.
- A breakpoint whose code was removed by a reload stays unverified, because its address points into
  code that no longer exists.

## Applies to

- `auxtools-protocol/src/messages.rs`: `address`, `from_address`, `CodeRef`, `StackFrame::id`
- `auxtools-cli/src/adapter.rs`: `live_address`, `set_instruction_breakpoints`,
  `apply_instruction_breakpoints`
