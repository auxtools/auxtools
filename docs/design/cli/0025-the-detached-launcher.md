# The detached launcher reports one status line and outlives its parent

A world launched with `auxtools launch` needs a launcher process for its entire lifetime, because
the launcher reads the world's output and the world is tied to it so that it exits when the
launcher exits. When `auxtools launch` leaves a world running in the background, it spawns a
detached launcher and waits only until the world is up.

## Startup handshake

- **Single status line:** The detached launcher writes exactly one line to stdout (`ready PID`,
  `unavailable PID WHY`, or `failed WHY`) and sends all other output to its log file. The parent
  waits up to `COMING_UP` (120 seconds) for that line.
- **Continuous stdout draining:** After reading the status line, the parent keeps reading and
  discarding the child's stdout so the detached launcher cannot block if the pipe buffer fills.
- **Startup timeout kills the launcher:** A startup timeout terminates the launcher. Because the
  launcher terminates the world when it exits, killing it is enough to stop the world. The timeout
  error reports that the world was stopped and names the log file.

## Process isolation and pipe safety

Both the detached launcher and the world must survive the pipes and the process group closing
around them:

- **Separate session or process group:** On Unix, the detached launcher calls `setsid` in
  `pre_exec`. Calling `setsid` between `fork` and `exec` is sound because `setsid` is
  async-signal-safe, as `pre_exec` requires. On Windows, the parent creates the detached launcher
  with `CREATE_NEW_PROCESS_GROUP`, `CREATE_NO_WINDOW`, and `CREATE_BREAKAWAY_FROM_JOB` when
  allowed, retrying without `CREATE_BREAKAWAY_FROM_JOB` if breaking away is refused. This keeps the
  launcher alive when its parent terminates.
- **Non-inheritable standard handles on Windows:** Before spawning the detached launcher, the
  parent marks its own stdin, stdout, and stderr non-inheritable. Otherwise, an inherited stdout
  pipe would keep the parent's readers from ever reaching end of file.
- **Ignoring `SIGPIPE`:** The detached launcher ignores `SIGPIPE` so that a reader of its output
  closing cannot terminate the launcher and take the world down with it.

## Applies to

- `auxtools-cli/src/cli/launch.rs`: `DETACHED`, `COMING_UP`, both `spawn` functions, `catch`, and
  detached launcher startup
