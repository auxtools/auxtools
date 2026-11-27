# Bytecode operand encoding and decoding

BYOND bytecode encodes several operands with rules that go beyond plain numeric IDs: field names
share an operand slot with access modifiers, some operands have fixed shapes that DreamMaker pads,
and certain values carry type information that a path would lose. Inspecting what a proc reads
likewise requires full operand decoding rather than scanning raw bytecode words.

## Field IDs and access modifiers

A field name in bytecode is stored as a string ID, using the same operand slot as access modifiers
such as `usr`, `src`, and `callee`, whose values run from `0xFFE5` (`usr`) to `0xFFF1` (`callee`).
Because a string ID can fall in that range, BYOND sets bit `0x10000000` (`STRING_NOT_MODIFIER_BIT`)
on any field string ID that collides with an access modifier.

The assembler sets `0x10000000` on field string IDs in that range, and the disassembler clears it.
Writing a colliding string ID without that bit would cause BYOND to read `usr`, `src`, or another
access modifier in place of the field.

## `in` operands

The `in` operator has five forms identified by an operand byte: `0x0B` for a range, `0x0C` for a
range with a step, `0x05` for a value, `0x06` for block corners, and `0x13` for block coordinates.

- The coordinate form of `x in block(...)` always takes six coordinates, and DreamMaker pads missing
  coordinates with `PushVal null`.
- DreamMaker emits the step form (`0x0C`) for `x in a to b step c`, even though DreamDaemon always
  returns false for it and leaves an extra value on the stack. The bytecode crate still models the
  step form so it can read and write DreamMaker's output.

## Prefab and datum type values

A prefab value's variables hold initializer code such as `src.a = 1`. The assembler orders those
variables by the string ID of each variable's name so that source order does not determine the
output.

The disassembler keeps a datum type value (tag `0x20`) as its raw tag and data rather than
converting it to a path. This allows consumers to rebuild the exact type reference, whereas going
through `value_to_string_data` would lose it and fails for types the environment cannot name.

## Detecting reads of `caller`

`reads_caller` determines whether a proc reads `caller` (`0xFFF0`) by decoding its operands.
Searching raw bytecode words produces false positives because `0xFFF0` can also appear as a valid
proc ID, a type ID, or half of a float.

## Applies to

- `dmasm/bytecode/src/access_modifiers.rs`: `STRING_NOT_MODIFIER_BIT`, `is_access_modifier`
- `dmasm/bytecode/src/operands.rs`: the `Field` case of `Variable`'s assembly, `IsInParams`,
  `Value::Prefab`, and the `0x20` case of `Value`'s disassembly
- `dmasm/bytecode/src/bytecode.rs`: `reads_caller`
