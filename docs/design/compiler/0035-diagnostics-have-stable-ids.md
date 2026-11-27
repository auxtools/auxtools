# Stable IDs for compiler diagnostics

Every compile error, warning, BYOND bug, and hot reload refusal emitted by the compiler carries a
stable `Id`. Callers such as the debugger, the CLI, and hot reload decide by the ID rather than by
the message text, so messages can be reworded freely. Matching on message text breaks when a message
is reworded, and renaming or removing an `Id` is a breaking change for callers.

Each `Diagnostic` also carries a `Span`:

- `file` is relative to the `.dme`'s directory. It is `None` for code compiled from text, such as a
  debugger expression, in which case the caller names the file.
- `column` counts from 1. It is kept for tools, but it is not shown in the message text.

## Applies to

- `dmasm/compiler/src/diagnostic.rs`: `Id`, `Diagnostic`, `Span`
