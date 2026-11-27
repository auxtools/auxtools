# Ordering DAP responses and stopped events

- **Stepping:** VS Code interprets the response to a step request to mean that the world is running.
  If a step's `stopped` event arrived before the response, VS Code would display the world as
  running while it is paused. The debug adapter therefore sends a step's `stopped` event only after
  the step's response. `dap_test` steps in 100 times and verifies after each response that no
  `stopped` event has arrived yet.
- **Pause followed by continue:** A client can send pause and continue requests back to back. A stop
  reported before the continue response is stale because that continue ended it, whereas a stop
  reported after the response is a pause that took effect later, so the world must actually be
  paused then. `dap_test` sends pause and continue together three times, drops any `stopped` events
  received before the continue response, and checks that any subsequent stop corresponds to a
  genuinely paused world whose stopped thread has frames.
- **Scenario completion:** Because the launcher delivers messages in order, any `stopped` event that
  occurs before a scenario ends arrives before the scenario's end. `run_collecting` in `dap_test`
  relies on this ordering to collect every stop, resuming each stop it sees and returning once the
  scenario's end arrives.

## Applies to

- `tests/dap_test/src/groups/stepping.rs`: Step In ordering check
- `tests/dap_test/src/groups/pausing.rs`: pause and continue check
- `tests/dap_test/src/world.rs`: `run_collecting`
