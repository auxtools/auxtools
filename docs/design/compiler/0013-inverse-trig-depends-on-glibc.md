# Inverse trigonometry folding and glibc rounding differences

DreamMaker converts radians to degrees in `f64`, but the precision of the intermediate radians
depends on the platform, BYOND build, and glibc version where DreamMaker ran. Because the compiler
cannot know which glibc built a running world, dmasm always folds `arcsin()`, `arccos()`, and
`arctan()` with `f64` radians and warns on builds from 515.1644 onward when rounding radians to
`f32` would yield a different value.

## Platform and glibc behavior

- Windows DreamMaker, and DreamMaker 515.1643 and earlier, compute the radians in `f64`.
- Linux DreamMaker from 515.1644 onward calls the `f32` inverse trigonometry functions.
- glibc before 2.41 returns the x87 result of those functions unrounded, producing the same value
  as `f64`.
- glibc 2.41 and later round the result to `f32` before converting to degrees, so `arctan(2)` folds
  to `63.43495` instead of `63.434948`.

Because of this, identical DM source folds to different constants depending on the glibc on the
machine that ran DreamMaker.

## Compiler folding and warnings

The compiler folds `arcsin()`, `arccos()`, and `arctan()` with `f64` radians (`degrees`), matching
DreamMaker on Windows and on Linux with glibc before 2.41.

For builds from 515.1644 onward (`has_single_inverse_trig`), the compiler also evaluates the result
with the radians rounded to `f32` (`degrees_of_single_radians`). When the two values differ, it
emits a `FoldedAngle` warning naming both values, without using the rounded result. A proc reloaded
into a world built by Linux DreamMaker with glibc 2.41 or later can therefore hold an angle one
`f32` step away from what that DreamMaker folded, and the warning identifies where.

## Comparison tool and tests

The comparison tool probes the installed DreamMaker once by building a small world and checking
whether `arctan(2)` folds to `63.43495` (`rounds_radians`). If it does, the tool accepts the cases
listed as rounded-angle differences after substituting DreamMaker's rounded angles for dmasm's.

The unit test `inverse_trig_folds_to_dreammakers_bits` expects DreamMaker's results on Windows and
with glibc 2.35, whereas glibc 2.41 differs from them by one `f32` step.

## Applies to

- `dmasm/compiler/src/compiler/versions.rs`: `degrees`, `degrees_of_single_radians`
- `dmasm/compiler/src/compiler/fold.rs`: `warn_angle`, `inverse_trig_folds_to_dreammakers_bits`
- `dmasm/compiler/src/target.rs`: `has_single_inverse_trig`
- `dmasm/compiler/examples/compare_dreammaker/main.rs`: `rounds_radians` and its use in `main`
- `dmasm/compiler/examples/compare_dreammaker/compare.rs`: `take_rounded_angles`
