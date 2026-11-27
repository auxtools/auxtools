# Color folding in `rgb()` and `rgb2num()`

DreamMaker folds constant `rgb()` and `rgb2num()` calls into text or lists at compile time. The
resulting bytes depend on where DreamMaker rounds to `f32`, and its HCY color space uses custom
weights. Doing all arithmetic in `f64` or all in `f32` produces different bytes near
half-integers, and for color spaces other than RGB, `rgb2num()` folding precision varies across
builds and platforms.

## `rgb()`

- The compiler converts a color percentage to a fraction of `1` in `f32` and runs the rest of the
  color calculation in `f64`.
- Each channel is scaled to `255`, converted to `f32`, and rounded half up. For example,
  `178.49999` produces `179`, because as an `f32` it is `178.5`.
- In HCY, chroma is scaled to the hue's maximum at the given luminance. BYOND's weights are red
  `0.299`, green `0.557`, and blue `0.114` (the green weight is not the usual `0.587`). Chroma and
  luminance are not clamped.

## `rgb2num()`

- The color space argument is truncated to a whole number. Any value other than `1`, `2`, and `3`
  means RGB, including `null` and text.
- The compiler folds only RGB results and leaves HSV, HSL, and HCY results to run time, because
  their folded precision differs across builds and platforms and would match DreamMaker on only some
  builds.

## Applies to

- `dmasm/compiler/src/compiler/fold.rs`: `share`, `channel`, `hsv_text`, `hcy_text`
- `dmasm/compiler/src/compiler/builtin_procs.rs`: `rgb2num_space`, `rgb2num_const`
