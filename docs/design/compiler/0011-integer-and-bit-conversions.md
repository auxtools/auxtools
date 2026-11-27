# Integer and bit conversions during constant folding

DM numbers are 32-bit floats. Integer and bit operations convert them before operating, and BYOND's
conversions follow x86 instructions rather than Rust's `as` operator. To fold constants to the same
values as DreamMaker, the compiler reproduces each conversion explicitly. Using Rust's `as`
anywhere in these paths produces the wrong value for `NaN` and out-of-range inputs.

## Conversion rules

- **Integer conversion (`int`):** BYOND converts numbers to integers by truncation. Matching x86
  conversion semantics, an out-of-range number or `NaN` becomes `i32::MIN`. Because Rust's `as`
  saturates instead, the compiler performs this conversion explicitly.
- **Bit operations (`bits`):** Bit operations work on 24 bits, so `-1` is `0xFFFFFF`, and a folded
  bit operation never produces a value above `0xFFFFFF`. Outside the `i32` range, a number becomes
  the nearest `i32` and then its low 24 bits, so `2**33` becomes `0x7FFFFFFF` and then `0xFFFFFF`.
- **Bitwise NOT (`~x`, `not_bits`):** Unlike the other bit operations, `~x` takes `x` as a `u32`.
  Negative numbers and `NaN` become `0`, numbers above `2**32` become `0xFFFFFFFF`, and the rest
  wrap, so `2**32` becomes `0`. The result is the low 24 bits of the complement.
- **Shifts:** A shift uses the low five bits of its count (matching x86), so `1 << 33` is `2`. The
  count is capped at `65535` before taking the low five bits, and a negative count shifts nothing.
  A negative number shifted left produces `0` (as with `~`), whereas shifted right it shifts its 24
  bits. The result of a left shift is cut to 24 bits.
- **Modulo (`mod_int`):** A modulo operand is converted to `i32` with saturation, matching
  DreamMaker: numbers above `2**31` become `2147483647`, and numbers below `-2**31` become
  `-2147483647`. Exactly `2**31` becomes `i32::MIN` (as in plain integer conversion), and so does
  `NaN`.

## Out-of-range and unfolded operands

For a bit-operation constant outside the 32-bit range, DreamMaker produces erratic bits that differ
by build. The compiler does not reproduce them and warns with `BitOperandOutOfRange`.

When only the left operand of `&`, `|`, or `^` is constant, DreamMaker stores that constant as its
low 24 bits. A constant right operand, and a shift's constant operand, are stored as written. In
debugger expressions, a left operand outside the `i32` range is stored as written.

## Applies to

- `dmasm/compiler/src/compiler/fold.rs`: `int`, `bits`, `not_bits`, `wide`, `warn_bits`, `mod_int`,
  `numbers`
- `dmasm/compiler/src/compiler/binary_ops.rs`: bit operator arm of `emit`
