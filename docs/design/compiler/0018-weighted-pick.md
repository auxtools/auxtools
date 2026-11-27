# Compiling weighted `pick()` into `PickSwitch` thresholds

`pick(w1; a, w2; b, ...)` selects an entry with a probability proportional to its weight. When all
weights are constant, DreamMaker compiles `pick()` into a `PickSwitch`: one random draw compared
against cumulative thresholds out of `65535`. The thresholds must match DreamMaker exactly, or the
same random state selects a different entry in a proc compiled by dmasm than in one compiled by
DreamMaker.

## Constant weights

- An entry without an explicit weight has weight `100`, and `prob(n)` as a weight counts as `n`.
- A text weight counts as `0` (typically from writing `"a";5` in reverse order).
- Each entry's threshold is the previous threshold plus `floor(weight / total * 65535)`. The last
  entry is the `PickSwitch` default and has no threshold.
- The debugger's expression code uses the same threshold calculation so the same random state picks
  the same entry.

## Zero weights

DreamMaker's handling of zero and text weights changed in BYOND build 516.1689, and older builds
emit broken bytecode for a trailing zero weight:

- From 516.1689 onward (`keeps_zero_pick_weight`), a zero weight keeps its case at the previous
  threshold.
- Before 516.1689, DreamMaker omits the case for a numeric zero weight while keeping the entry's
  value code. Text weights always keep their case.
- Before 516.1689, when the last entry has a zero weight, DreamMaker's default jumps outside the
  proc. dmasm does not reproduce that jump: instead, it makes the last entry whose weight is not a
  constant zero the default and emits a `ByondZeroWeightPick` warning.

## Applies to

- `dmasm/compiler/src/compiler/term.rs`: weighted `pick()` arm, `pick_weight`
- `dmasm/compiler/src/target.rs`: `keeps_zero_pick_weight`
