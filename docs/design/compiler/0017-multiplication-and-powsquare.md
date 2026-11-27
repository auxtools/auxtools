# Emitting `PowSquare` for `x ** 2` and `x * x`

BYOND provides `PowSquare` (`0x173`) to square the value on top of the stack and `PowNegativeOne`
(`0x174`) for `x ** -1`. DreamMaker emits `PowSquare` for `x ** 2` and also for multiplications
whose two operands are the same plain variable (`x * x`), using specific rules to decide what
counts as a repeated plain variable.

dmasm emits `PowSquare` for `x ** 2` and for `x * x` under DreamMaker's exact conditions:

- Both operands must be the same plain variable: a name, an integer, text, `null`, or a type path,
  optionally in parentheses, followed only by field accesses or negation.
- A fraction never counts as a plain variable, because DreamMaker does not treat two equal
  fractions as the same value when deciding whether an operand repeats.
- When a multiplication is the left operand of another multiplication, DreamMaker does not emit
  `PowSquare` for the inner multiplication, and parentheses do not change this. Thus `x * x * y`
  and `(x * x) * y` do not square `x`, whereas `f(x * x) * y` and `-(x * x) * y` do. The compiler
  tracks whether the left operand of a multiplication is itself a multiplication with `mul_lhs`.

Because `PowSquare` and `Mul` produce the same value, these rules only affect whether the bytecode
matches DreamMaker's. A plain variable read twice is read once with `PowSquare`, which would matter
only if the read had side effects, and a plain variable has none.

## Applies to

- `dmasm/compiler/src/compiler/binary_ops.rs`: `Pow` and `Mul` arms of `emit`, `same_variable`,
  `is_plain_variable`
- `dmasm/compiler/src/compiler.rs`: `mul_lhs` field of `Compiler`
- `dmasm/bytecode/src/instructions.rs`: `PowSquare`, `PowNegativeOne`
