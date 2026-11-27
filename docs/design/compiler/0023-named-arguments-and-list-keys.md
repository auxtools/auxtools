# Named arguments and list keys from source shape

`f(a = 1)` passes a named argument, and `list(a = 1)` creates an associative entry. DreamMaker
decides from the exact source shape of the argument whether it is named and how its key is
interpreted:

- A proc argument is a named argument only when it is an assignment written without parentheses and
  its left-hand side is a plain name rather than a nested expression (tested with
  `can_short_circuit(unroll(lhs))`). Anything else, such as `f(a + 1 = 2)`, is an argument with a
  computed key.
- In a `list()` call, a key that matches a built-in constant's name stands for the constant's value
  rather than its name (for example, `list(MALE = 1)` uses the key `"male"`).
- In a proc call, the name is used as written.

A named argument in a proc call goes into an associative argument list, so the call passes a single
list instead of separate values.

## Applies to

- `dmasm/compiler/src/compiler/args.rs`: `emit` (the plain-name test is
  `can_short_circuit(unroll(lhs))`)
