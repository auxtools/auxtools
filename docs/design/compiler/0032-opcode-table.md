# Opcode table omissions and stack effects

BYOND's interpreter handles more opcodes than DreamMaker 516 emits, including opcodes for old
statement forms. The instruction table in the bytecode crate defines what the disassembler can read
and what the assembler can write, omitting opcodes that DreamMaker 516 never emits. Several included
instructions also have unusual stack effects that code generation relies on.

## Omitted opcodes

These opcodes have handlers in BYOND, but no known DM code makes DreamMaker 516 emit them, so they
are left out of the instruction table:

| opcode | meaning |
| --- | --- |
| 0x06 | old `target << sound(file, repeat)` |
| 0x26 | old `spawn` that names a proc instead of having a body; takes an arg_count operand |
| 0x4F | flips the test flag |
| 0x9C | does nothing |
| 0x9D | a third entry into the `input()` code; has the same three operand words as `Input` |
| 0xBD | replaces the value on top of the stack with null |
| 0xCE | 0x26 with `arglist()` |
| 0xD5 | old `target << sound(...)` with 1 to 5 arguments; takes an arg_count operand |
| 0xD6 | old `target << S` for a /sound datum |
| 0xD8 | runs as an invalid opcode |

The disassembler fails on any proc containing an omitted opcode. This can only happen with bytecode
that DreamMaker 516 does not produce, such as code compiled by much older compilers. Supporting one
of these opcodes later requires giving it the operands documented above.

## Notable instructions

A few instructions that are in the table have non-obvious meanings:

| instruction | meaning |
| --- | --- |
| `WaitforBegin` (0xA3), `WaitforEnd` (0xA4) | the old `waitfor` block statement |
| `IconBlendSimple` (0xE0) | `_dm_icon_blend(icon, other, function)` without x and y |
| `RgbEx` (0x161) | `rgb()` when the colour space isn't known to be `COLORSPACE_RGB` at compile time |
| `GradientIndex` (0x163) | `gradient(Gradient, index)`: the gradient is already one value, and the index is pushed separately |

## Stack effects

- `SetCacheJmpIfNull` (`0x13D`) pops the value only if it does not jump.
- `SetCachePopJmpIfNull` (`0x13E`) pops the value whether or not it jumps.
- `PushCache` (`0x142`) and `PopCache` (`0x143`) use a separate stack that lives in the same
  allocation as the value stack.

Because `SetCacheJmpIfNull` and `SetCachePopJmpIfNull` leave different stack depths on the jumping
path, code that uses them must account for the difference.

## Applies to

- `dmasm/bytecode/src/instructions.rs`: `Instruction` and its opcode list
