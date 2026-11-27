# Compiling `&&` and `||` chains and conditions around the test flag

BYOND maintains a test flag that comparisons and `Test` set and conditional jumps read. Some
instructions leave their result in the test flag, while others leave a value that must be fetched
with `GetFlag` or checked with `Test`. Because DreamMaker chooses between these forms based on the
source shape, the same runtime value can be tested in two different ways.

DreamMaker flattens `&&` and `||` chains such as `a || b || c` so that every operand jumps directly
to the end of the chain. Flattening looks through parentheses, so `(a || b) || (c || d)` compiles
as a single chain.

Comparisons with `==` and `in`, as well as `is*()` checks, set the test flag directly, so
DreamMaker does not fetch the flag into a value and test it again when they appear in a condition.
A logic chain that folds down to a comparison, however, still fetches and tests its value again,
whereas a lone comparison leaves its result in the flag. `is_logic` distinguishes the two cases,
with or without surrounding parentheses, so ternary and `if` conditions know whether to omit
`GetFlag` and test the flag directly. Folding a logic chain therefore changes the bytecode shape
without changing the value.

## Applies to

- `dmasm/compiler/src/compiler/binary_ops.rs`: `&&` and `||` arm of `emit`, `flatten`
- `dmasm/compiler/src/compiler/fold.rs`: `is_logic`
- `dmasm/compiler/src/compiler/statements.rs`: `emit_condition`
- `dmasm/compiler/src/compiler/ternary.rs`: `emit`
