# BYOND build predicates in `target.rs`

DreamMaker's output changes across BYOND builds: some builds fold more expressions, some fix bugs,
and some change the order of global initializers. dmasm compiles for a chosen `ByondVersion` and
must change its output at the same builds, keeping every such threshold as a named predicate on
`ByondVersion` in `target.rs`. Build numbers appear nowhere else in the compiler, so finding
everything that varies by build requires reading only one file.

BYOND build numbers increase monotonically across major versions (515.1647 is followed by 516.1648).
Each predicate compares only the build number and never the major version, because the build number
alone orders the releases. For example, `is_516` is `at_least(1648)`.

## Predicates not covered in other design notes

- **Overflowing integer literals (`clamps_integer_literals`):** Before 515.1644, DreamMaker turns
  an overflowing integer literal into `2147483647`, and the parser is configured to do the same.
- **Two-argument `log()` (`folds_log_in_single`):** From 515.1644 onward, DreamMaker folds
  `log(b, x)` in single precision. Earlier builds divide the logarithms in `f64` and round once.
- **`isnan()` and `isinf()` on constants (`folds_nan_tests_cleanly`, `folds_within_nan_tests`):**
  - BYOND 515 builds do not fold them at all.
  - Builds 516.1670, and 516.1672 onward, fold them directly to the constant.
  - Other BYOND 516 builds emit `PushInt result; GetFlag`. `GetFlag` returns the flag from the
    preceding test rather than the result. dmasm emits the same sequence and warns with
    `ByondFoldedTestFlag`.
  - Build 516.1671 folds them only inside another expression. Standing alone, it compiles them as a
    test using the `PushInt result; GetFlag` sequence above.
- **Compare operator in `list.Find()` and `^` (`find_calls_compare_operator`):** From 516.1674
  onward, `list.Find(x)` and `list ^ x` call `x.operator<=>()` for each item. dmasm warns about
  such an overload with `ByondCompareOperatorInFind`.
- **Global initializer order:**
  - From 515.1643 onward, global initializers visit `/mutable_appearance` right after `/image`.
    Earlier builds visit it with the late built-in types (`visits_mutable_appearance_at_root`).
  - From 516.1685 onward, a deferred `/type::var` read through unnamed ancestors runs last among
    global initializers (`runs_scoped_reads_last`).

Other predicates are documented alongside the behavior they control:
[pointers](0028-pointers.md), [weighted `pick()`](0018-weighted-pick.md),
[implicit types for `new`, `locate()`, and `astype()`](0007-implicit-type-of-new-locate-astype.md),
[safe navigation chains](0003-safe-navigation-chains.md),
[trigonometry and NaN signs](0012-trigonometry-and-nan-signs.md), and
[inverse trigonometry and glibc](0013-inverse-trig-depends-on-glibc.md).

## Maintenance rules

Supporting a new BYOND build means adding or moving predicates in `target.rs` and using them where
the behavior is compiled. Writing a build number anywhere else in the compiler is a mistake.
Because predicates compare only the build number, this rule holds only while BYOND continues
numbering builds upward across major versions.

## Applies to

- `dmasm/compiler/src/target.rs`: `ByondVersion` and its predicates
- `dmasm/compiler/src/compiler/builtin_procs.rs`: `emit_tests` (for the `isnan()` and `isinf()`
  cases)
- `dmasm/compiler/src/compiler/fold.rs`: `constant` (for the `isnan()` and `isinf()` cases)
