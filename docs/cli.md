# Command-line workflow

The `auxtools` CLI launches, inspects, steps, traces, profiles, and manages local BYOND worlds from
a terminal or script. For installation and basic launch options, see [README.md](../README.md).

## Commands

Run `auxtools help COMMAND` (or `auxtools COMMAND --help`) for any command's full options, JSON
schema, behavior, and exit codes. Editors integrate with auxtools by running `auxtools dap` over
`stdin` and `stdout`.

| Group | Commands |
| --- | --- |
| World management | `launch`, `list`, `status`, `stop`, `reboot`, `reconnect`, `log` |
| Inspection | `stack`, `vars`, `eval`, `proc`, `disassemble` |
| Execution | `pause`, `continue`, `step-over`, `step-in`, `step-out`, `wait` (`pause`, `reload`, `reboot`), `trace`, `profile` |
| Source code | `check` |
| Integration | `dap`, `help`, `version` (`--version`, `-V`) |

For details on `profile` and `check`, see [Instruction profiling](profiling.md) and
[Checking for known BYOND bugs](bug-checker.md).

## Example session

```sh
auxtools launch tgstation.dmb --detach
auxtools wait pause --break code/game/foo.dm:120
auxtools vars
auxtools eval 'src.health'
auxtools step-over
auxtools continue
auxtools trace /mob/proc/Life --by caller --for 5
auxtools profile --for 10
auxtools log --runtimes --follow
auxtools wait reload
auxtools reboot
auxtools stop
```

## Selecting a world

`auxtools list` displays every local world with the debug library loaded, including worlds running
an incompatible debug protocol version. A world's debug library and the CLI must speak the same
protocol version, which they do when both come from the same auxtools release. `auxtools list` shows
each world's protocol version and whether it is compatible, and `auxtools version` shows the CLI's.
When only one compatible world is running, commands target it automatically. Otherwise, pass
`--pid PID`.

## Scripting and timeouts

Commands write results to `stdout` and diagnostics to `stderr`. Pass `--json` for machine-readable
output: `log` and `trace` stream JSON Lines (one event per line), while other commands print a
single JSON value.

Per-request responses time out after 30 seconds by default (exit code `2`). Override this with
`--response-timeout SECONDS` on any world-targeting command except `stop`. Separately, `--timeout
SECONDS` limits how long an event or operation waits in `stop` (default 10 seconds), `reboot` and
`reconnect` (default 60 seconds), and `step-*`, `wait`, and `eval --wait` (unlimited by default).
`trace` and `profile` use `--for SECONDS`.

Commands that leave the world paused (`pause`, `step-*`, `wait pause`) accept `--hold-max SECONDS`
(or the `AUXTOOLS_HOLD_MAX` environment variable) to resume automatically if no debugger reconnects
within that window.

## Exit codes

| Exit code | Meaning |
| --- | --- |
| `0` | Success (for `trace` and `profile`, duration or hit limit reached or interrupted with `Ctrl+C`; for `log --follow`, the world exited) |
| `1` | Command failed (for `wait reload`, the reload was rejected or partially applied) |
| `2` | Operation timed out (request response, wait/step/stop/reboot timeout, or detached `launch` 120-second startup limit) |
| `3` | World exited or rebooted before the command finished |
| `4` | Foreground `launch`: the world exited with a nonzero code or was terminated |
| `5` | `check`: known BYOND bugs found |
