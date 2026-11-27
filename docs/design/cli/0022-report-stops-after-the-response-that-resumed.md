# Ordering stop events after resume responses

VS Code treats the response to a step or continue request as a signal that the world is running. If
a `stopped` event arrives before that response, the response overwrites the paused state and the
client displays the world as running while it is actually paused.

To prevent VS Code from losing stops, the debug client hands a stop to the debug adapter as
`Input::Stopped` only after the response to the request that resumed the world, so `stopped` events
always follow that response. Reordering these messages, for example by emitting a `stopped` event
as soon as the world reports it, causes VS Code to miss the stop.

When a step finishes its stack without pausing because the stack has no caller left to stop in, the
world sends `StepEnded` and the debug adapter emits no event. The client already shows the world as
running after receiving the step response.

## Applies to

- `auxtools-cli/src/debuggee.rs`: `ToAdapter::input`, `Debuggee`
- `auxtools-cli/src/adapter.rs`: `Input::Stopped` arm of the adapter loop
- `auxtools-protocol/src/messages.rs`: `Response::StepEnded`
