# Matching DreamMaker's constant folding and edge cases

DreamMaker folds many constant expressions at compile time and stores the resulting values. To match
DreamMaker's bytecode, dmasm folds the same expressions to the same results, including cases where
DreamMaker's compile-time value differs from what the operator produces at run time.

## Regrouping

DreamMaker regroups chains of `+`, `*`, `&`, `|`, and `^` recursively, turning `a + (b + c) + d`
into `a + ((b + c) + d)`. Regrouping determines which constants sit next to each other and fold
together, so dmasm performs the same transformation. Otherwise, expressions such as `a + 1 + 2` and
`a + (1 + 2)` would fold differently from DreamMaker and produce different bytecode.

## Hashes and text

- DreamMaker folds `md5()` and `sha1()` of constant text into the resulting hash string. The
  compiler computes these hashes itself and depends on the `md-5` and `sha1` crates for that
  purpose. Removing those crates would prevent folding constant `md5()` and `sha1()` calls and make
  the bytecode differ.
- `text2ascii("a")` folds to `97` and `text2ascii("")` folds to `0`, and `text2ascii_char()` behaves
  the same way. Both return the code of the whole first character rather than its first byte.

## Edge-case operands

- Shifting text or `null`, or shifting by text or `null`, folds as if the text or `null` were `0`.
  A shift of text or `null` therefore yields `0`, and a shift of a number by text or `null` yields
  the number's 24 bits unshifted.
- A number divided by text folds to the number itself. A number divided by `null` does the same
  where DreamMaker allows it, while BYOND 516 treats that case as division by zero.
- DreamMaker folds `0` raised to a negative power or a NaN power to `0`, except for `-1`, which
  yields `1/0` (infinity).

## Applies to

- `dmasm/compiler/src/compiler/binary_ops.rs`: `regroup`
- `dmasm/compiler/src/compiler/fold.rs`: `constant`, `numbers`
- `dmasm/compiler/Cargo.toml`: `md-5` and `sha1` dependencies
