# BYOND encodings for variable flags and verb `in` clauses

Because the world applies a reload plan using BYOND's own values, the plan stores variable
declaration flags and built-in verb `in` clauses in BYOND's encodings.

## Variable flags

BYOND encodes a declared variable's kind as flag bits: `1` for `static` or `global`, `2` for
`const`, and `4` for `tmp`.

`NewVar::flags` and `Redeclared::flags` carry these bits directly. A `const` always receives bit `1`
as well, producing `3`, and a global variable always receives bit `1`.

## Verb `in` clauses

BYOND encodes a verb argument's built-in `in` clause as a single word. The low byte identifies the
kind:

| Kind      | Value |
|-----------|-------|
| `view`    | 1     |
| `oview`   | 2     |
| `usr.loc` | 3     |
| `range`   | 5     |
| `orange`  | 6     |
| `usr`     | 8     |
| `world`   | 0x10  |

The byte above holds the range: `0x7D` when no range is specified, and `0x7F` for kinds that take no
range. An argument with no `in` clause is encoded as `0x7D01`.

`InList::Builtin` carries this encoded word. Both `usr` and `usr.contents` encode as kind `8`. The
`view`, `oview`, `range`, and `orange` clauses take an optional integer range below `0x7D` and may
also name `usr`. Any other clause is not built in and is represented either as `InList::Kept`
(retaining the running world's list) or as `InList::New` with compiled code when the list changed.

## Applies to

- `dmasm/reload/src/reload.rs`: `NewVar`, `Redeclared`, `byond_flags`, `InList`, `builtin_in_list`
