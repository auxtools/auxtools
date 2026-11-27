# Compiling `null` literals as variable reads

DreamMaker compiles a `null` literal as a read of the `null` variable (`GetVar null`) wherever a
variable is allowed, and emits `PushVal null` in other places. The two forms behave the same.

To match DreamMaker, the compiler returns `null` as a variable (`Variable::Null`) rather than as a
value pushed onto the stack. Code that needs a value then reads it with `GetVar null`. As a result,
`null` can be used anywhere DreamMaker allows a variable, such as the operand of an instruction that
takes a variable.

When the comparison tool reports where dmasm's code first differs from DreamMaker's,
`first_difference` skips any place where dmasm has `PushVal null` and DreamMaker has `GetVar null`,
and points at the next difference instead. This only affects the message. The two forms still count
as a difference when the tool decides whether the code matches.

## Applies to

- `dmasm/compiler/src/compiler/term.rs`: the `Null` term
- `dmasm/compiler/examples/compare_dreammaker/compare.rs`: `first_difference`
