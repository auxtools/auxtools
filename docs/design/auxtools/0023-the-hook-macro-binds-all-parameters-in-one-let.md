# Binding `#[hook]` parameters in a single `let`

The `#[hook]` attribute macro converts a hook's parameters into local bindings read from the
argument vector `args`, after first padding `args` with `null` values up to the parameter count.

The generated code binds every parameter in one tuple `let` statement. Binding each parameter in its
own `let` would allow a parameter named `args` to shadow the argument vector before later parameter
initializers read from it. Keep all parameter bindings in a single statement when modifying the
macro's generated code.

## Applies to

- `auxtools-impl/src/lib.rs`: `hook` attribute macro
