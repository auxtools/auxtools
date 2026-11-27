# Folding trigonometry and NaN signs with BYOND's algorithms

BYOND computes trigonometry in degrees with its own argument reduction, and BYOND 515 and 516
reduce angles differently. Invalid floating-point operations on x86 produce the default NaN with
its sign bit set (`0xFFF8000000000000`). BYOND carries this NaN through results, and some of its
functions change its sign, whereas Rust's NaN is positive and host math libraries can handle NaN
signs differently. Following the target build, the compiler therefore folds these functions using
its own algorithms and the negative x86 NaN (`DEFAULT_NAN`), because even a single-bit difference or
a flipped NaN sign makes the bytecode differ from DreamMaker's and can change program output.

## NaN signs

- `abs()` preserves the sign of a NaN. Because Rust's `f64::abs` clears the sign bit, the compiler
  negates only values below zero.
- `arcsin()`, `arccos()`, and `arctan()` leave a NaN unchanged, except that BYOND 515's `arccos()`
  clears its sign. The compiler does not pass NaNs to the host's math functions, because a 32-bit
  build's functions produce a different sign.

## `sin()` and `cos()`

- BYOND 515 converts the angle to radians directly, and an infinite angle yields the negative NaN.
- BYOND 516 normalizes an angle to `[0, 360)` and reduces it by quadrants. This produces exact
  values on the axes, so `cos(90)` is `-0`.
- BYOND 516 strips a negative angle's sign first and restores it for `sin()`.
- In BYOND 516, an infinite angle yields x86's negative NaN, and `cos()` clears the sign of a NaN.
  With the sign restored, `sin(-1.#INF)` is a positive NaN.

## `tan()`

- BYOND 515 converts the angle to radians directly.
- BYOND 516 normalizes the angle to `(-180, 180]` and reduces it to at most 45 degrees. It computes
  in `f64` and rounds to `f32` before taking the reciprocal and applying the sign.
- The operation order matters: `tan(90)` is `-inf`, `tan(270)` is `inf`, and `tan(60)` is the `f32`
  reciprocal of `tan(30)`.
- `tan()` of an infinity yields the negative x86 NaN. In BYOND 516, `tan()` removes the angle's
  sign and restores it on the result, so `tan(-1.#INF)` is a positive NaN. A NaN input remains a
  NaN.

## `arctan(x, y)` and inverse functions

In `arctan(x, y)`, a `y` of `-0` counts as `0`, so `arctan(-1, -0)` is `180` rather than `-180`.
The inverse trigonometric functions also depend on the platform DreamMaker ran on, as described in
[inverse trigonometry folding and glibc rounding](0013-inverse-trig-depends-on-glibc.md).

## Applies to

- `dmasm/compiler/src/compiler/versions.rs`: `DEFAULT_NAN`, `sin_cos`, `tan`
- `dmasm/compiler/src/compiler/fold.rs`: `math1`, `arctan2_radians`
