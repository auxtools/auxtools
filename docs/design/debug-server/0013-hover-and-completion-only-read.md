# Read-only evaluation for hover and completion

Clients evaluate hover and completion expressions automatically without explicit user action, so
those lookups must not run DM code. Indexing anything other than a list could invoke `operator[]`,
and any operator could call an overload.

Hover and completion therefore use `Read` instead of `Eval`. `read_expr` parses the expression and
reads its value directly, accepting only names, variables, and list entries whose keys are constants
or are read the same way. Any other expression is refused with "expression may run code, so it must
be evaluated explicitly" (`RUNS_CODE`).

Looking up a type path for a read still executes the evaluation stub, so the server marks itself as
evaluating while it runs (see
[blocking client requests while the debugger runs DM](0010-no-requests-are-handled-while-the-debugger-runs-dm.md)).

## Applies to

- `debug_server/src/server/reading.rs`: `read_expr`, `read`, `RUNS_CODE`
