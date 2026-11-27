# Hit conditions and failing breakpoint conditions

Breakpoints and data breakpoints can carry a hit condition (passed as free text by DAP) and a DM
condition expression that may fail at run time, for example on a `null` reference. Hit conditions
count from 1, and a failing condition skips the hit and is reported only once so a repeatedly
failing expression does not flood the client.

## Hit conditions

A hit condition is `N`, or one of the following operators followed by `N`:

- `==` (a single `=` is also accepted): matches the `N`th hit only.
- `>`, `>=`, `<`, `<=`: compares the hit count against `N`.
- `%`: matches every `N`th hit. `%0` is refused with `"hit interval must be greater than zero"`.

A bare `N` matches the `N`th hit and every subsequent hit, equivalent to `>= N`. Hits count from 1,
and whitespace around the text and the number is ignored. Any other input is refused with a message
listing the accepted forms.

## Failing conditions

When a breakpoint's condition expression fails to evaluate, that hit is skipped: the world neither
pauses nor logs. The client that set the breakpoint is notified once with `ConditionFailed`, whose
message names the location and the condition, states that such hits are skipped, and notes that the
error is shown once.

The breakpoint counts the skipped hits, and `BreakpointUnset` returns that count (see
[breakpoints as separate, client-owned instruction claims](0008-breakpoints-are-separate-claims-owned-by-a-client.md)).
Updating the condition with `BreakpointTrigger` allows a subsequent failure to be reported again.

## Applies to

- `auxtools-protocol/src/messages.rs`: `HitCondition::parse`, `HitCondition::takes`,
  `Response::ConditionFailed`
- `debug_server/src/server/breakpoints.rs`: `condition_failed`
