# Compiling `&` and `*` pointers like DreamMaker 516.1668

DM pointers use `&x` to take the address of a variable and `*p` to read or write through a pointer.
DreamMaker treats parentheses around these operators specially, and builds before 516.1668 compile
`*(&x)` wrongly. dmasm compiles `&` and `*` as DreamMaker 516.1668 does for every target build.

## Parentheses

`&(x)` is the same as `&x`, and `*(p)` is the same as `*p`. The compiler moves the operator inside
the parentheses before emitting code.

`*(&x)` reads through the pointer it creates. Starting in BYOND build 516.1668, DreamMaker compiles
this to read `x`. Earlier builds instead read `*x`, which dereferences `x`'s value and produces a
`"bad pointer"` error unless `x` holds a pointer. Rather than reproducing that bug, dmasm reads `x`
for every build and emits a `ByondDerefOfReference` warning on earlier builds.

## Fields and non-writable values

For `&t.v`, DreamMaker evaluates the object as usual into the cache and then takes the address of
the field on it, giving `cache = t; &cache["v"]`. The compiler keeps the cache chain and wraps only
the final field in a pointer.

Dereferencing a value on the stack, or a variable that cannot be written (such as the `&x` in
`*(&x)`), first stores that value in the cache key. The read then goes through the cache key.

## List elements

An evaluated list element (`EvalKind::ListRef`) means `L[K]` has been evaluated with the index on
top of the stack and the list under it. Code that consumes it depends on this order. For `&L[K]`,
`*L[K]`, and the increments, the compiler stores the top of the stack into the cache key and the
next value into the cache, and then uses the element `cache[key]`.

Starting in BYOND build 516.1668, DreamMaker leaves out the index of a `*p[i]` read, just as it does
in a write. dmasm keeps the index and emits a `ByondPointerIndexDropped` warning.

## Applies to

- `dmasm/compiler/src/compiler.rs`: `EvalKind::ListRef` and the parenthesis case in
  `Compiler::emit_expr`
- `dmasm/compiler/src/compiler/unary.rs`: `emit_single`, `reference`, `warn_made_pointer`
- `dmasm/compiler/src/target.rs`: `derefs_made_pointer`
