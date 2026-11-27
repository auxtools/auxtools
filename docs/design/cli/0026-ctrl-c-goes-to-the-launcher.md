# Routing Ctrl+C to the launcher instead of the world

Pressing Ctrl+C in a terminal normally signals every process in the foreground process group, which
would terminate the world without running its shutdown code. To let the launcher intercept Ctrl+C
and request a clean shutdown instead, auxtools isolates the world process from terminal interrupts
and manages console control events in the launcher.

## Process isolation

- **Linux:** The world calls `setpgid(0, 0)` in `pre_exec` so it runs in its own process group and
  Ctrl+C reaches only the launcher. The same `pre_exec` hook calls `prctl` to set
  `PR_SET_PDEATHSIG` to `SIGKILL` so the world terminates if the thread that launched it exits.
  Both calls are async-signal-safe, making it sound to run them between `fork` and `exec`.
- **Windows:** The world is created with `CREATE_NEW_PROCESS_GROUP` (along with `CREATE_SUSPENDED`
  for library injection, as described in
  [how the launcher starts a world](0028-how-the-launcher-starts-a-world.md)), so it ignores
  Ctrl+C.

## Windows console control handlers

On Windows, a console control handler governs process termination. For close, logoff, and shutdown
events, Windows terminates the process when the handler returns or when the handler's deadline
expires, and a closed console terminates the process in any case.

- **Launcher handler:** Counts the event and returns `1`. For close, logoff, and shutdown events
  (event `2` and above), it first waits up to 10 seconds, polling every 100 ms, for the world to
  finish shutting down.
- **`trace` handler:** Claims every console event and only records the interrupt. Because a closed
  console still terminates the process, claiming that event cannot keep the process alive.

## Applies to

- `auxtools-cli/src/game.rs`: `Game::start`
- `auxtools-cli/src/cli/launch.rs`: `catch`
- `auxtools-cli/src/cli/trace.rs`: `interruptible`
