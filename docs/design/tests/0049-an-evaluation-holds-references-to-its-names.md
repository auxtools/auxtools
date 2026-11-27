# An evaluation holds references to the strings named in its code

An evaluation holds references to the strings named in its code, so measuring a string's reference
count with an evaluation that names it counts the evaluation's own references.

Because of this, `hot_reload_test` measures the reference counts of `hr_own` and `step_x` with
`#string-refs NAME` before any evaluation names them, then measures them again after a reload and
compares the counts.

## Applies to

- `tests/hot_reload_test/src/main.rs`: reparenting checks in `declarations`
